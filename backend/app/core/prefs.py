import json
import threading
from typing import Any

from app.core import config, seal

DEFAULTS: dict = {
    "claudeFolder": "",
    "claudeBin": "",
    "claudeFirstMessage": "",
    "agentsAllowed": True,
}
_lock = threading.Lock()


def _path():
    return config.data_dir() / "preferences.json"


def _stored() -> dict:
    try:
        data = json.loads(_path().read_text())
    except (OSError, ValueError):
        data = {}
    return data if isinstance(data, dict) else {}


def _write(values: dict) -> None:
    tmp = _path().with_suffix(".json.tmp")
    tmp.write_text(json.dumps(values, indent=1))
    tmp.replace(_path())


def _merged(data: dict) -> dict:
    out = json.loads(json.dumps(DEFAULTS))
    out.update(data)
    return out


def load() -> dict:
    with _lock:
        data = _stored()
        state = seal.check(_merged(data))
        if state == "adopt":
            try:
                _write(seal.sign(_merged(data)))
            except OSError:
                pass
    return seal.effective(_merged(data), state)


def update(values: dict[str, Any]) -> dict:
    for key, default in DEFAULTS.items():
        if key in values and values[key] is not None and type(values[key]) is not type(default):
            raise ValueError(f"preference {key} must be {type(default).__name__}")
    unknown = sorted(set(values) - set(DEFAULTS))
    if unknown:
        raise ValueError(f"unknown preference {', '.join(unknown)}")
    current = load()
    with _lock:
        stored = _stored()
        sealed = seal.sign({**stored, **{k: current.get(k) for k in seal.SEALED}, **values})
        _write(sealed)
    return seal.effective(_merged(sealed), seal.check(_merged(sealed)))
