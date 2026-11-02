import json
import os
import shutil
import threading
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

from app.core import config

MAX_STRING = 8192
MAX_LOG_BYTES = 64 * 1024 * 1024
SKIP_GROUPS = {"sim", "export"}
SKIP_IDS = {
    "file.save", "fabric.export", "library.add", "library.remove", "library.set_roots", "component.save",
    "anim.record", "anim.stop", "anim.play", "anim.pause", "job.state", "job.harvest", "cache.frame",
}
SKIP_PREFIXES = ("link.",)
_lock = threading.Lock()
_log_bytes = 0
_count = 0
_closed = True


def _folder() -> Path:
    folder = config.data_dir() / "autosave"
    folder.mkdir(parents=True, exist_ok=True)
    return folder


def _state_file() -> Path:
    return _folder() / "journal.json"


def _log_file() -> Path:
    return _folder() / "journal.jsonl"


def _pending_folder() -> Path:
    return _folder() / "recovery"


def autosave_file() -> Path:
    return _folder() / "autosave.ffbrc"


def _now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def _write_json(path: Path, value: Any) -> None:
    temp = path.with_name(path.name + ".tmp")
    temp.write_text(json.dumps(value), encoding="utf-8")
    os.replace(temp, path)


def _read_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return data if isinstance(data, dict) else {}


def _line_count(path: Path) -> int:
    try:
        with path.open("rb") as handle:
            return sum(1 for line in handle if line.strip())
    except OSError:
        return 0


def _state(**changes: Any) -> None:
    state = _read_json(_state_file())
    state.update(changes)
    _write_json(_state_file(), state)


def start() -> dict | None:
    global _closed, _count, _log_bytes
    with _lock:
        previous = _read_json(_state_file())
        log = _log_file()
        unclean = bool(previous) and previous.get("clean") is False
        had_base = bool(previous.get("autosave_at")) and autosave_file().is_file()
        commands = _line_count(log)
        if unclean and (had_base or commands):
            pending = _pending_folder()
            shutil.rmtree(pending, ignore_errors=True)
            pending.mkdir(parents=True, exist_ok=True)
            if had_base:
                shutil.copy2(autosave_file(), pending / "autosave.ffbrc")
            if log.is_file():
                os.replace(log, pending / "journal.jsonl")
            _write_json(pending / "info.json", {
                "autosave_time": previous.get("autosave_at") if had_base else None,
                "commands": commands,
                "base": "autosave" if had_base else "empty",
                "session_started": previous.get("started"),
                "found": _now(),
            })
        try:
            log.write_text("", encoding="utf-8")
        except OSError:
            pass
        _write_json(_state_file(), {"clean": False, "started": _now(), "autosave_at": None, "pid": os.getpid()})
        _closed = False
        _count = 0
        _log_bytes = 0
        return pending_info()


def close() -> None:
    global _closed
    with _lock:
        if _closed:
            return
        _closed = True
        try:
            _state(clean=True, closed=_now())
        except OSError:
            pass


def pending_info() -> dict | None:
    info = _read_json(_pending_folder() / "info.json")
    if not info:
        return None
    return {
        "pending": True,
        "autosave_time": info.get("autosave_time"),
        "commands": int(info.get("commands", 0)),
        "base": info.get("base", "empty"),
        "session_started": info.get("session_started"),
    }


def _clean_args(args: dict | None) -> tuple[dict, list[str]]:
    kept: dict = {}
    dropped: list[str] = []
    for key, value in (args or {}).items():
        if isinstance(value, str) and len(value) > MAX_STRING:
            dropped.append(key)
        else:
            kept[key] = value
    return kept, dropped


def skipped(name: str, group: str | None) -> bool:
    return name in SKIP_IDS or name.startswith(SKIP_PREFIXES) or group in SKIP_GROUPS


def record(name: str, args: dict | None, group: str | None) -> None:
    global _count, _log_bytes
    if _closed or skipped(name, group):
        return
    kept, dropped = _clean_args(args)
    entry = {"n": _count + 1, "t": _now(), "id": name, "args": kept}
    if dropped:
        entry["dropped"] = dropped
    line = json.dumps(entry, separators=(",", ":")) + "\n"
    with _lock:
        if _log_bytes + len(line) > MAX_LOG_BYTES:
            return
        try:
            with _log_file().open("a", encoding="utf-8") as handle:
                handle.write(line)
        except OSError:
            return
        _count += 1
        _log_bytes += len(line)


def checkpoint() -> None:
    global _count, _log_bytes
    with _lock:
        if _closed:
            return
        try:
            _log_file().write_text("", encoding="utf-8")
            _state(autosave_at=_now(), clean=False)
        except OSError:
            return
        _count = 0
        _log_bytes = 0


def discard() -> bool:
    with _lock:
        found = _pending_folder().is_dir()
        shutil.rmtree(_pending_folder(), ignore_errors=True)
        return found


def restore(dispatch: Callable[[str, dict], dict]) -> dict:
    info = pending_info()
    if info is None:
        raise LookupError("there is nothing to recover")
    folder = _pending_folder()
    base = folder / "autosave.ffbrc"
    failures: list[dict] = []
    if info["base"] == "autosave" and base.is_file():
        reply = dispatch("file.open", {"path": str(base)})
        if not reply.get("ok"):
            raise RuntimeError(f"the autosave could not be opened: {reply.get('error')}")
    else:
        dispatch("file.new", {})
    replayed = 0
    log = folder / "journal.jsonl"
    try:
        lines = log.read_text(encoding="utf-8").splitlines() if log.is_file() else []
    except OSError:
        lines = []
    for line in lines:
        try:
            entry = json.loads(line)
            name, args = entry["id"], entry.get("args") or {}
        except (ValueError, KeyError, TypeError):
            failures.append({"line": line[:80], "error": "unreadable log line"})
            continue
        reply = dispatch(name, args)
        if reply.get("ok"):
            replayed += 1
        else:
            failures.append({"id": name, "error": (reply.get("error") or {}).get("message", "failed")})
    discard()
    return {"restored": True, "base": info["base"], "autosave_time": info["autosave_time"], "logged": info["commands"], "replayed": replayed, "failed": failures[:20]}
