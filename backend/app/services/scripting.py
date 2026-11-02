from __future__ import annotations

"""Plugins are trusted code: a plugin is arbitrary Python that runs in the service process with the
user's rights. Install only plugins you wrote or reviewed. A plugin that runs past FFB_PLUGIN_TIMEOUT
seconds (default 300) is reported as timed out and refused further commands, but Python cannot stop
its thread, so a plugin stuck outside ffb calls keeps running until the service exits."""
import ast
import io
import os
import runpy
import sys
import threading
import time
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parents[2]


class ScriptError(RuntimeError):
    def __init__(self, message: str, code: str = "failed", command: str | None = None):
        super().__init__(message)
        self.code = code
        self.command = command


class Ffb:
    def __init__(self, dispatch: Callable[[str, dict], dict]):
        self._dispatch = dispatch
        self.deadline: float | None = None

    def _check(self) -> None:
        if self.deadline is not None and time.monotonic() > self.deadline:
            raise ScriptError("the plugin ran past its time limit", "timeout")

    def call(self, command: str, given: dict | None = None, /, **args: Any) -> Any:
        self._check()
        reply = self._dispatch(command, {**(given or {}), **args})
        if not reply.get("ok"):
            error = reply.get("error") or {}
            raise ScriptError(f"{command}: {error.get('message', 'failed')}", str(error.get("code", "failed")), command)
        return reply.get("result")

    def try_call(self, command: str, given: dict | None = None, /, **args: Any) -> dict:
        self._check()
        return self._dispatch(command, {**(given or {}), **args})

    def commands(self) -> list[str]:
        from app.services import engine

        return [c["id"] for c in engine.catalogue()["commands"]]


def plugin_dirs() -> list[Path]:
    dirs = [ROOT / "plugins"]
    extra = os.environ.get("FFB_PLUGIN_DIR")
    dirs.append(Path(extra) if extra else Path.home() / ".formfabric" / "plugins")
    return [d for d in dirs if d.is_dir()]


def _describe(path: Path) -> dict | None:
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"))
    except (OSError, SyntaxError, UnicodeDecodeError):
        return None
    if not any(isinstance(n, ast.FunctionDef) and n.name == "run" for n in tree.body):
        return None
    doc = (ast.get_docstring(tree) or "").strip()
    title, _, rest = doc.partition("\n")
    params, click = {}, False
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            try:
                value = ast.literal_eval(node.value)
            except (ValueError, SyntaxError):
                continue
            if node.targets[0].id == "PARAMS" and isinstance(value, dict):
                params = {k: _param(v) for k, v in value.items() if isinstance(k, str) and isinstance(v, dict)}
            elif node.targets[0].id == "CLICK":
                click = value is True
    return {"name": path.stem, "title": title or path.stem.replace("_", " ").capitalize(), "description": rest.strip(), "path": str(path), "params": params, "click": click}


def _param(spec: dict) -> dict:
    kind = spec.get("type") if spec.get("type") in ("number", "integer", "boolean", "choice", "text") else "number"
    out = {"type": kind, "label": str(spec.get("label", ""))[:40]}
    for key in ("default", "min", "max", "step"):
        if isinstance(spec.get(key), (int, float, bool, str)):
            out[key] = spec[key]
    if kind == "choice":
        out["options"] = [str(o) for o in spec.get("options", []) if isinstance(o, (str, int, float))][:32]
    return out


def list_plugins() -> list[dict]:
    found: dict[str, dict] = {}
    for d in plugin_dirs():
        for path in sorted(d.glob("*.py")):
            info = _describe(path)
            if info:
                found[info["name"]] = info
    return sorted(found.values(), key=lambda p: p["title"].lower())


def resolve_args(params: dict, given: dict | None) -> dict:
    out = {k: v for k, v in (given or {}).items() if k in params}
    for key, spec in params.items():
        if key not in out:
            if "default" in spec:
                out[key] = spec["default"]
            continue
        value, kind = out[key], spec["type"]
        if kind == "boolean":
            if isinstance(value, str) and value.lower() in ("true", "false"):
                value = value.lower() == "true"
            if not isinstance(value, bool):
                raise ScriptError(f"{key} must be true or false", "bad_args")
        elif kind in ("number", "integer"):
            try:
                value = float(value) if not isinstance(value, bool) else None
            except (TypeError, ValueError):
                value = None
            if value is None or value != value or value in (float("inf"), float("-inf")):
                raise ScriptError(f"{key} must be a number", "bad_args")
            if kind == "integer":
                if value != int(value):
                    raise ScriptError(f"{key} must be a whole number", "bad_args")
                value = int(value)
            lo, hi = spec.get("min"), spec.get("max")
            if isinstance(lo, (int, float)) and not isinstance(lo, bool) and value < lo:
                raise ScriptError(f"{key} must be at least {spec['min']}", "bad_args")
            if isinstance(hi, (int, float)) and not isinstance(hi, bool) and value > hi:
                raise ScriptError(f"{key} must be at most {spec['max']}", "bad_args")
        elif kind == "choice":
            if str(value) not in spec.get("options", []):
                raise ScriptError(f"{key} must be one of {', '.join(spec.get('options', []))}", "bad_args")
            value = str(value)
        else:
            if not isinstance(value, str):
                raise ScriptError(f"{key} must be text", "bad_args")
        out[key] = value
    return out


class _Capture(io.TextIOBase):
    def __init__(self, fallback: Any):
        self.fallback = fallback
        self.local = threading.local()

    def write(self, text: str) -> int:
        target = getattr(self.local, "buffer", None)
        return (target or self.fallback).write(text)

    def flush(self) -> None:
        target = getattr(self.local, "buffer", None)
        (target or self.fallback).flush()


_capture_lock = threading.Lock()


def _capture() -> _Capture:
    with _capture_lock:
        if not isinstance(sys.stdout, _Capture):
            sys.stdout = _Capture(sys.stdout)
        return sys.stdout


def plugin_timeout() -> float:
    try:
        return max(1.0, float(os.environ.get("FFB_PLUGIN_TIMEOUT", "300")))
    except ValueError:
        return 300.0


def run_file(path: str | Path, ffb: Ffb, args: dict | None = None, timeout: float | None = None) -> dict:
    limit = timeout if timeout else plugin_timeout()
    ffb.deadline = time.monotonic() + limit
    capture = _capture()
    out = io.StringIO()
    box: dict[str, Any] = {}

    def work() -> None:
        capture.local.buffer = out
        try:
            module = runpy.run_path(str(path), init_globals={"ffb": ffb, "args": args or {}}, run_name="__formfabric__")
            box["result"] = module["run"](ffb, args or {}) if callable(module.get("run")) else None
        except BaseException as err:
            box["error"] = err
        finally:
            capture.local.buffer = None

    worker = threading.Thread(target=work, name="ffb-plugin", daemon=True)
    worker.start()
    worker.join(limit)
    if worker.is_alive():
        raise ScriptError(f"the plugin did not finish within {int(limit)} seconds; it can no longer call commands", "timeout")
    if "error" in box:
        raise box["error"]
    return {"result": box.get("result"), "output": out.getvalue()}
