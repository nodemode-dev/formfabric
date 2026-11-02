from __future__ import annotations

import ast
import contextlib
import io
import os
import runpy
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

    def call(self, command: str, given: dict | None = None, /, **args: Any) -> Any:
        reply = self._dispatch(command, {**(given or {}), **args})
        if not reply.get("ok"):
            error = reply.get("error") or {}
            raise ScriptError(f"{command}: {error.get('message', 'failed')}", str(error.get("code", "failed")), command)
        return reply.get("result")

    def try_call(self, command: str, given: dict | None = None, /, **args: Any) -> dict:
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
    out = dict(given or {})
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
            if "min" in spec and value < spec["min"]:
                raise ScriptError(f"{key} must be at least {spec['min']}", "bad_args")
            if "max" in spec and value > spec["max"]:
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


def run_file(path: str | Path, ffb: Ffb, args: dict | None = None) -> dict:
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        module = runpy.run_path(str(path), init_globals={"ffb": ffb, "args": args or {}}, run_name="__formfabric__")
        result = module["run"](ffb, args or {}) if callable(module.get("run")) else None
    return {"result": result, "output": out.getvalue()}
