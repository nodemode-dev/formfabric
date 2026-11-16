import json
import os
import stat
import threading

from app.services import events, recent, recovery

try:
    import ffb_engine
except ImportError:
    ffb_engine = None

UNAVAILABLE = "the engine module is not built; run ./formfabric engine build"
_lock = threading.Lock()
_engine = ffb_engine.Engine() if ffb_engine else None
_epoch = 0
_read_only: set[str] | None = None
_info: dict = {}
_RECENT_NAMES = {"file.save", "file.open", "file_save", "file_open"}
_REBASE = {"file.new", "file.open", "exchange.import_dxf"} | recovery.LIVE_IDS
_FORGET = {"file.new", "sample.load"}
_RESOLVED = {"pin.add": ("target", "pin", "target"), "body.tape": ("body_point", None, "body_point")}
_MIB = 1024 * 1024
_READ_LIMITS = {"file.open": 1024 * _MIB, "exchange.import_dxf": 256 * _MIB, "fabric.import": 256 * _MIB, "motion.import": 1024 * _MIB, "body.import": 1024 * _MIB, "trim.import": 1024 * _MIB}


class EngineUnavailable(RuntimeError):
    def __init__(self) -> None:
        super().__init__(UNAVAILABLE)


def available() -> bool:
    return _engine is not None


def _require() -> None:
    if _engine is None:
        raise EngineUnavailable()


def catalogue() -> dict:
    _require()
    return json.loads(ffb_engine.catalogue())


def status() -> dict:
    if ffb_engine is None:
        return {"native": False, "version": None, "commands": []}
    return {"native": True, "version": ffb_engine.version(), "commands": [c["name"] for c in catalogue()["commands"]]}


def epoch() -> int:
    return _epoch


def _bump() -> None:
    global _epoch
    _epoch += 1


def _meta() -> dict:
    global _read_only, _info
    if _read_only is None:
        _read_only = set()
        _info = {}
        for c in catalogue()["commands"]:
            _info[c["name"]] = _info[c["id"]] = (c["id"], c["group"])
            if c["annotations"].get("readOnlyHint"):
                _read_only.update({c["name"], c["id"]})
    return _info


def read_only(name: str) -> bool:
    _meta()
    return name in _read_only


def _unreadable(name: str, args: dict | None) -> dict | None:
    limit = _READ_LIMITS.get(_meta().get(name, (name, None))[0])
    path = (args or {}).get("path")
    if limit is None or not isinstance(path, str) or not path:
        return None
    try:
        found = os.stat(path)
    except (OSError, ValueError):
        return None
    if not stat.S_ISREG(found.st_mode):
        message = f"{path} is not a regular file (it is a folder, a device or a pipe), so it was not read"
    elif found.st_size > limit:
        message = f"{path} is {found.st_size // _MIB} MiB, larger than the {limit // _MIB} MiB this command reads"
    else:
        return None
    return {"ok": False, "error": {"code": "bad_args", "message": message}}


def dispatch(name: str, args: dict | None, notify: bool = True) -> dict:
    _require()
    mutating = not read_only(name)
    refusal = _unreadable(name, args)
    if refusal is not None:
        return refusal
    with _lock:
        before = _engine.epoch()
        reply = json.loads(_engine.dispatch(name, json.dumps(args or {})))
        changed = _engine.epoch() != before
        if reply.get("ok"):
            command, group = _meta().get(name, (name, None))
            place = (args or {}).get("path")
            if command in ("file.open", "file.save") and isinstance(place, str) and place and not _in_autosave_folder(place):
                recovery.note_document(os.path.abspath(place))
            elif command in _FORGET or (command == "exchange.import_dxf" and (args or {}).get("mode") == "as_new"):
                recovery.note_document(None)
            if command == "file.save" and _is_autosave((args or {}).get("path")):
                recovery.checkpoint()
            elif changed and mutating:
                behind = recovery.record(command, _resolved(command, args, reply.get("result")), group)
                if (behind or command in _REBASE) and recovery.active():
                    saved = json.loads(_engine.dispatch("file.save", json.dumps({"path": str(recovery.autosave_file())})))
                    if saved.get("ok"):
                        recovery.checkpoint()
    if reply.get("ok") and name in _RECENT_NAMES and isinstance((args or {}).get("path"), str) and not _in_autosave_folder(args["path"]):
        recent.note(args["path"])
    if reply.get("ok") and changed and mutating:
        _bump()
        if notify:
            events.publish("engine.changed", epoch=_epoch, name=name)
    return reply


def _resolved(command: str, args: dict | None, result) -> dict | None:
    if command not in _RESOLVED or not isinstance(result, dict):
        return args
    key, inner, field = _RESOLVED[command]
    source = result.get(inner) if inner else result
    if not isinstance(source, dict) or field not in source:
        return args
    return {**(args or {}), key: source[field]}


def _is_autosave(path) -> bool:
    return isinstance(path, str) and os.path.abspath(path) == os.path.abspath(str(recovery.autosave_file()))


def _in_autosave_folder(path: str) -> bool:
    return os.path.abspath(path).startswith(os.path.abspath(str(recovery.autosave_file().parent)) + os.sep)


def stream_info() -> dict:
    _require()
    with _lock:
        return json.loads(_engine.stream_info())
