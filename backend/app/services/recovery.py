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
SKIP_IDS = {
    "file.save", "fabric.export", "library.add", "library.remove", "library.set_roots", "component.save",
    "anim.record", "anim.stop", "anim.play", "anim.pause", "job.state", "job.harvest", "cache.frame",
}
SKIP_PREFIXES = ("link.",)
REPLAY_PREFIXES = {
    "pattern", "panel", "styleline", "style", "print", "component", "sew", "tack", "button", "buttonhole",
    "hardware", "zipper", "stitch", "fray", "finish", "quilt", "trim", "fabric", "colorway", "body", "history",
    "view", "motion", "key", "morph", "measure3d", "stage", "light", "fur",
    "arrange", "pin", "place", "wind", "fold", "fitshell", "uv", "thickness", "quad", "retopo", "topology",
}
REPLAY_IDS = {
    "sample.load", "library.insert", "snapshot.save", "snapshot.restore", "snapshot.remove",
    "place.volume_set", "place.point_set", "place.defaults", "anim.set", "anim.auto_key",
}
LIVE_IDS = {"pin.box", "pin.lasso", "sim.apply", "refit"}
LIVE_REASON = "it took the positions of the running simulation, which the crash journal does not keep"
REPLAY_DENY = {"fabric.export", "component.save"} | LIVE_IDS
NOT_REPEATED = {
    "cache": "changes to recorded simulation caches are not repeated, because the recordings themselves are not kept in the crash journal",
    "map": "a fit map is a display choice of the 3D View, not part of the garment",
    "compare": "a comparison is a display choice of the 3D View, not part of the garment",
    "exchange": "it wrote or read a file and is not repeated",
    "file": "it opened, saved or replaced the document and is not repeated",
}
NOT_REPEATED_DEFAULT = "this command is not repeated after a crash"
HISTORY_IDS = {"history.undo", "history.redo"}
NO_UNDO_STEP = {"map", "compare", "exchange.export_dxf"}
REPORT_ROWS = 20
KEEP_PENDING = 5
SECOND_SERVICE = "second_service"
JOURNAL_ERROR = "journal_error"
CLOSING = "closing"
_lock = threading.Lock()
_restoring = threading.Lock()
_owner = None
_log_bytes = 0
_count = 0
_done = 0
_undone = 0
_closed = True
_truncated = False
_off_reason = JOURNAL_ERROR


def replayable(name: str) -> bool:
    if not isinstance(name, str) or name in REPLAY_DENY or skipped(name, None):
        return False
    return name in REPLAY_IDS or name.split(".", 1)[0] in REPLAY_PREFIXES


def not_repeated_reason(name: str) -> str:
    if name in LIVE_IDS:
        return LIVE_REASON
    return NOT_REPEATED.get(str(name).split(".", 1)[0], NOT_REPEATED_DEFAULT)


def protection() -> dict:
    return {"autosave_active": not _closed, "autosave_off_reason": _off_reason if _closed else None}


def foreign() -> bool:
    return _owner is None and _closed


def _acquire() -> bool:
    global _owner
    if _owner is not None:
        return True
    try:
        handle = (_folder() / "backend.lock").open("a+b")
    except OSError:
        return True
    try:
        if os.name == "nt":
            import msvcrt
            handle.seek(0)
            msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        handle.close()
        return False
    _owner = handle
    return True


def _rotate_pending() -> None:
    pending = _pending_folder()
    if not pending.is_dir():
        return
    kept = _rotated()
    n = int(kept[-1].name.split(".", 1)[1]) + 1 if kept else 1
    os.replace(pending, pending.parent / f"recovery.{n}")
    for old in (kept + [pending.parent / f"recovery.{n}"])[:-KEEP_PENDING]:
        shutil.rmtree(old, ignore_errors=True)


def _rotated() -> list[Path]:
    found = [p for p in _folder().glob("recovery.*") if p.name.split(".", 1)[1].isdigit() and p.is_dir()]
    return sorted(found, key=lambda p: int(p.name.split(".", 1)[1]))


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
    global _closed, _count, _log_bytes, _truncated, _off_reason
    with _lock:
        if not _acquire():
            _closed = True
            _off_reason = SECOND_SERVICE
            return pending_info()
        previous = _read_json(_state_file())
        log = _log_file()
        unclean = bool(previous) and previous.get("clean") is False
        had_base = bool(previous.get("autosave_at")) and autosave_file().is_file()
        commands = _line_count(log)
        if unclean and (had_base or commands):
            pending = _pending_folder()
            _rotate_pending()
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
                "truncated": previous.get("truncated") is True,
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
        _truncated = False
        return pending_info()


def close() -> None:
    global _closed, _off_reason
    with _lock:
        if _closed:
            return
        _closed = True
        _off_reason = CLOSING
        try:
            _state(clean=True, closed=_now())
        except OSError:
            pass


def active() -> bool:
    return not _closed


def pending_info() -> dict | None:
    info = _read_json(_pending_folder() / "info.json")
    if not info:
        return None
    try:
        commands = int(info.get("commands", 0))
    except (TypeError, ValueError):
        commands = 0
    return {
        "pending": True,
        "autosave_time": info.get("autosave_time"),
        "commands": commands,
        "truncated": info.get("truncated") is True,
        "base": info.get("base", "empty"),
        "session_started": info.get("session_started"),
    }


def _clean_args(args: dict | None) -> tuple[dict, list[str]]:
    kept: dict = {}
    dropped: list[str] = []
    for key, value in (args or {}).items():
        if isinstance(value, (str, list, dict)) and len(json.dumps(value, separators=(",", ":"))) > MAX_STRING:
            dropped.append(key)
        else:
            kept[key] = value
    return kept, dropped


def skipped(name: str, group: str | None) -> bool:
    return name in SKIP_IDS or name.startswith(SKIP_PREFIXES)


def _behind(name: str) -> bool:
    global _done, _undone
    if name in HISTORY_IDS:
        if (_done if name == "history.undo" else _undone) == 0:
            return True
        _done, _undone = (_done - 1, _undone + 1) if name == "history.undo" else (_done + 1, _undone - 1)
    elif replayable(name) or not (name in NO_UNDO_STEP or name.split(".", 1)[0] in NO_UNDO_STEP):
        _done, _undone = _done + 1, 0
    return False


def record(name: str, args: dict | None, group: str | None) -> bool:
    global _count, _log_bytes, _truncated
    if _closed or skipped(name, group):
        return False
    kept, dropped = _clean_args(args)
    entry = {"n": _count + 1, "t": _now(), "id": name, "args": kept}
    if dropped:
        entry["dropped"] = dropped
    line = json.dumps(entry, separators=(",", ":")) + "\n"
    with _lock:
        if _behind(name):
            return True
        if _log_bytes + len(line) > MAX_LOG_BYTES:
            if not _truncated:
                _truncated = True
                print("[formfabric] the crash journal reached 64 MiB and stopped recording; work after this point is only covered by the next autosave", flush=True)
                try:
                    _state(truncated=True)
                except OSError:
                    pass
            return False
        try:
            with _log_file().open("a", encoding="utf-8") as handle:
                handle.write(line)
        except OSError:
            return False
        _count += 1
        _log_bytes += len(line)
    return False


def checkpoint() -> None:
    global _count, _log_bytes, _truncated, _done, _undone
    with _lock:
        if _closed:
            return
        try:
            _log_file().write_text("", encoding="utf-8")
            _state(autosave_at=_now(), clean=False, truncated=False)
        except OSError:
            return
        _done = _undone = 0
        _count = 0
        _log_bytes = 0
        _truncated = False


def discard() -> bool:
    with _lock:
        if foreign():
            return False
        found = _pending_folder().is_dir()
        shutil.rmtree(_pending_folder(), ignore_errors=True)
        return found


def _names(rows: list[dict]) -> str:
    seen = list(dict.fromkeys(str(row.get("id", "an unreadable line")) for row in rows))
    return ", ".join(seen[:6]) + (" and others" if len(seen) > 6 else "")


def _summary(total: int, replayed: int, skipped: list[dict], failures: list[dict], truncated: bool) -> str:
    parts = [f"{replayed} of {total} logged commands were repeated on top of the last autosave."]
    if skipped:
        parts.append(f"{len(skipped)} were left out on purpose ({_names(skipped)}).")
    if failures:
        parts.append(f"{len(failures)} could not be repeated ({_names(failures)}).")
    if truncated:
        parts.append("The journal was full before the crash, so the last work before it is missing.")
    return " ".join(parts)


def restore(dispatch: Callable[[str, dict], dict]) -> dict:
    if not _restoring.acquire(blocking=False):
        raise RuntimeError("a restore is already running")
    try:
        return _restore(dispatch)
    finally:
        _restoring.release()


def _restore(dispatch: Callable[[str, dict], dict]) -> dict:
    info = pending_info()
    if info is None:
        raise LookupError("there is nothing to recover")
    if foreign():
        raise RuntimeError("another FormFabric service is running on this data folder")
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
    skipped: list[dict] = []
    done: list[bool] = []
    undone: list[bool] = []
    log = folder / "journal.jsonl"
    try:
        lines = [line for line in log.read_text(encoding="utf-8").splitlines() if line.strip()] if log.is_file() else []
    except OSError:
        lines = []
    for line in lines:
        try:
            entry = json.loads(line)
            name, args = entry["id"], entry.get("args") or {}
        except (ValueError, KeyError, TypeError):
            failures.append({"line": line[:80], "error": "this line of the journal could not be read"})
            done.append(False)
            continue
        if name in HISTORY_IDS:
            source, target = (done, undone) if name == "history.undo" else (undone, done)
            if source and not source.pop():
                skipped.append({"id": name, "reason": "it undid or redid a step that was not repeated"})
                target.append(False)
                continue
            reply = dispatch(name, {})
            if reply.get("ok"):
                replayed += 1
                target.append(True)
            else:
                failures.append({"id": name, "error": (reply.get("error") or {}).get("message", "failed")})
            continue
        if not replayable(name) and (str(name) in NO_UNDO_STEP or str(name).split(".", 1)[0] in NO_UNDO_STEP):
            skipped.append({"id": str(name)[:60], "reason": not_repeated_reason(name)})
            continue
        undone.clear()
        if not replayable(name):
            skipped.append({"id": str(name)[:60], "reason": not_repeated_reason(name)})
            done.append(False)
            continue
        if entry.get("dropped") or not isinstance(args, dict):
            failures.append({"id": name, "error": "its arguments were too large to keep in the journal"})
            done.append(False)
            continue
        reply = dispatch(name, args)
        done.append(bool(reply.get("ok")))
        if reply.get("ok"):
            replayed += 1
        else:
            failures.append({"id": name, "error": (reply.get("error") or {}).get("message", "failed")})
    discard()
    summary = _summary(len(lines), replayed, skipped, failures, info["truncated"])
    print(f"[formfabric] recovery: {summary}", flush=True)
    return {
        "restored": True, "base": info["base"], "autosave_time": info["autosave_time"], "logged": info["commands"], "replayed": replayed,
        "skipped": skipped[:REPORT_ROWS], "skipped_count": len(skipped), "failed": failures[:REPORT_ROWS], "failed_count": len(failures),
        "truncated": info["truncated"], "summary": summary,
    }
