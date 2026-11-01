import json
import threading
from datetime import datetime, timezone
from pathlib import Path

from app.core import config

LIMIT = 20
_lock = threading.Lock()


def _store() -> Path:
    return config.data_dir() / "recent.json"


def _read() -> list[dict]:
    try:
        data = json.loads(_store().read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return []
    return [e for e in data if isinstance(e, dict) and isinstance(e.get("path"), str)] if isinstance(data, list) else []


def note(path: str) -> None:
    full = str(Path(path).expanduser().resolve())
    stamp = datetime.now(timezone.utc).isoformat(timespec="seconds")
    with _lock:
        entries = [e for e in _read() if e["path"] != full]
        entries.insert(0, {"path": full, "modified": stamp})
        try:
            _store().write_text(json.dumps(entries[:LIMIT]), encoding="utf-8")
        except OSError:
            return


def listing() -> list[dict]:
    with _lock:
        entries = _read()
        kept = [e for e in entries if Path(e["path"]).is_file()][:LIMIT]
        if len(kept) != len(entries):
            try:
                _store().write_text(json.dumps(kept), encoding="utf-8")
            except OSError:
                pass
    out = []
    for e in kept:
        p = Path(e["path"])
        try:
            modified = datetime.fromtimestamp(p.stat().st_mtime, timezone.utc).isoformat(timespec="seconds")
        except OSError:
            modified = e.get("modified", "")
        out.append({"path": e["path"], "name": p.name, "modified": modified})
    return out
