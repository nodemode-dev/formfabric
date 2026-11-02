import json
import os
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


def dispatch(name: str, args: dict | None, notify: bool = True) -> dict:
    _require()
    mutating = not read_only(name)
    with _lock:
        before = _engine.epoch()
        reply = json.loads(_engine.dispatch(name, json.dumps(args or {})))
        changed = _engine.epoch() != before
        if reply.get("ok"):
            command, group = _meta().get(name, (name, None))
            if command == "file.save" and _is_autosave((args or {}).get("path")):
                recovery.checkpoint()
            elif changed and mutating:
                recovery.record(command, args, group)
    if reply.get("ok") and name in _RECENT_NAMES and isinstance((args or {}).get("path"), str):
        recent.note(args["path"])
    if reply.get("ok") and changed and mutating:
        _bump()
        if notify:
            events.publish("engine.changed", epoch=_epoch, name=name)
    return reply


def _is_autosave(path) -> bool:
    return isinstance(path, str) and os.path.abspath(path) == os.path.abspath(str(recovery.autosave_file()))


def stream_info() -> dict:
    _require()
    with _lock:
        return json.loads(_engine.stream_info())
