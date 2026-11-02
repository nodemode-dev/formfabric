from __future__ import annotations

import json
import os
import re
import tempfile
import time
from pathlib import Path
from typing import Any

from .client import Client, default_client

FORMATS = {"blender": "glb", "unreal": "glb", "formsurface": "glb", "unity": "fbx", "any": "glb"}
UNITS = {"glb": ("m", 0.001), "fbx": ("cm", 0.1)}
ID_BASE = 2_000_000


def default_folder() -> Path:
    given = os.environ.get("FORMSCULPT_GOF", "").strip()
    return Path(given) if given else Path.home() / "Formsculpt" / "GoF"


def stem(name: str) -> str:
    return re.sub(r"[^0-9A-Za-z]", "_", name) or "garment"


def _read(manifest: Path) -> dict:
    try:
        data = json.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return data if isinstance(data, dict) else {}


def _write_atomic(path: Path, text: str) -> None:
    handle, temp = tempfile.mkstemp(dir=path.parent, prefix=".gof.", suffix=".tmp")
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as out:
            out.write(text)
        os.replace(temp, path)
    except BaseException:
        try:
            os.unlink(temp)
        except OSError:
            pass
        raise


def send(name: str = "formfabric_garment", target: str = "any", folder: str | os.PathLike | None = None, client: Client | None = None, version: str = "") -> dict:
    if target not in FORMATS:
        raise ValueError(f"target must be one of {', '.join(FORMATS)}")
    client = client or default_client()
    kind = FORMATS[target]
    units, scale = UNITS[kind]
    directory = Path(folder) if folder else default_folder()
    directory.mkdir(parents=True, exist_ok=True)
    base = stem(name)
    file = f"{base}.{kind}"
    path = directory / file
    options: dict[str, Any] = {"units": units, "axis": "y_up"}
    written = client.call("export.run", format=kind, path=str(path), **options)
    manifest = directory / "gof.json"
    current = _read(manifest)
    objects = [o for o in current.get("objects", []) if isinstance(o, dict) and o.get("file") != file]
    used = {o.get("id") for o in objects}
    ident = ID_BASE
    while ident in used:
        ident += 1
    previous = [o for o in current.get("objects", []) if isinstance(o, dict) and o.get("file") == file]
    if previous and isinstance(previous[0].get("id"), int) and previous[0]["id"] >= ID_BASE:
        ident = previous[0]["id"]
    entry = {
        "id": ident,
        "name": name,
        "file": file,
        "format": kind,
        "target": target,
        "revision": int(previous[0].get("revision", 0)) + 1 if previous else 1,
        "vertices": int(written.get("vertices", 0)),
        "triangles": int(written.get("triangles", 0)),
        "sent": int(time.time()),
        "returns": False,
        "units": units,
        "scale": scale,
        "app": "formfabric",
    }
    objects.append(entry)
    out = {
        "gof": 1,
        "app": current.get("app", "formfabric"),
        "version": current.get("version", version),
        "up_axis": "y",
        "mm_per_unit": current.get("mm_per_unit", 1.0),
        "objects": objects,
    }
    _write_atomic(manifest, json.dumps(out, indent=2))
    return {"folder": str(directory), "path": str(path), "manifest": str(manifest), "entry": entry, "written": written}
