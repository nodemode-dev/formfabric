import hashlib
import json
import os
import re
import shutil
import sys
import tempfile
import threading
import time
import zipfile
from datetime import datetime, timezone
from pathlib import Path

from app.core import config
from app.services import recent

SUFFIX = ".ffbrc"
GARMENT_ID = re.compile(r"^g_[0-9a-f]{20}$")
TRASH_ENTRY = re.compile(r"^(g_[0-9a-f]{20})-(\d{1,12})$")
TEMPLATE_ID = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")
THUMB_LIMIT = 2_000_000
PROJECT_LIMIT = 512 * 1024 * 1024
DXF_LIMIT = 64 * 1024 * 1024
JSON_LIMIT = 64 * 1024 * 1024
NAME_LIMIT = 80
LIST_LIMIT = 500
_BAD_NAME = re.compile(r'[<>:"/\\|?*\x00-\x1f]')
_RESERVED = {"con", "prn", "aux", "nul", *(f"com{i}" for i in range(1, 10)), *(f"lpt{i}" for i in range(1, 10))}
_IMAGE_HEADS = ((b"\x89PNG", "image/png"), (b"\xff\xd8\xff", "image/jpeg"))
_lock = threading.RLock()
_panels: dict[str, tuple[int, int, int | None]] = {}

TEMPLATES = (
    {"id": "dress", "file": "dress.ffbrc", "name": "Sleeveless dress", "category": "Dresses", "tags": ["dress", "sleeveless", "female"]},
    {"id": "skirt", "file": "skirt.ffbrc", "name": "A-line skirt", "category": "Skirts", "tags": ["skirt", "a-line", "female"]},
    {"id": "tshirt-female", "file": "tshirt-female.ffbrc", "name": "T-shirt, female body", "category": "Tops", "tags": ["t-shirt", "top", "female"]},
    {"id": "tshirt-male", "file": "tshirt-male.ffbrc", "name": "T-shirt, male body", "category": "Tops", "tags": ["t-shirt", "top", "male"]},
)
CATEGORIES = ("Dresses", "Skirts", "Tops")


class NotInLibrary(Exception):
    pass


def library_dir() -> Path:
    path = config.data_dir() / "Garments"
    path.mkdir(parents=True, exist_ok=True)
    return path


def trash_dir() -> Path:
    path = library_dir() / ".trash"
    path.mkdir(parents=True, exist_ok=True)
    return path


def _thumbs_dir() -> Path:
    path = config.data_dir() / "thumbs"
    path.mkdir(parents=True, exist_ok=True)
    return path


def _imports_dir() -> Path:
    path = config.data_dir() / "Imports"
    path.mkdir(parents=True, exist_ok=True)
    return path


def templates_dir() -> Path:
    configured = os.environ.get("FFB_TEMPLATES", "").strip()
    if configured:
        return Path(configured)
    bundled = getattr(sys, "_MEIPASS", None)
    if bundled:
        return Path(bundled) / "assets" / "samples"
    return config.REPO / "assets" / "samples"


def garment_id(path: Path) -> str:
    key = os.path.normcase(str(path.resolve()))
    return "g_" + hashlib.sha1(key.encode("utf-8")).hexdigest()[:20]


def in_library(path: Path) -> bool:
    try:
        if path.is_symlink() or not path.is_file() or path.suffix.lower() != SUFFIX or path.name.startswith("."):
            return False
        return path.resolve().parent == library_dir().resolve()
    except OSError:
        return False


def _umask() -> int:
    current = os.umask(0)
    os.umask(current)
    return current


FILE_MODE = 0o666 & ~_umask()


def place(tmp: str, target: Path) -> None:
    try:
        os.chmod(tmp, FILE_MODE)
    except OSError:
        pass
    os.replace(tmp, target)


def _write_atomic(target: Path, data: bytes) -> None:
    fd, tmp = tempfile.mkstemp(prefix=".tmp-", dir=str(target.parent))
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        place(tmp, target)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _copy_atomic(source: Path, target: Path) -> None:
    fd, tmp = tempfile.mkstemp(prefix=".tmp-", dir=str(target.parent))
    try:
        with os.fdopen(fd, "wb") as out, source.open("rb") as src:
            shutil.copyfileobj(src, out, 1024 * 1024)
            out.flush()
            os.fsync(out.fileno())
        place(tmp, target)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def clean_name(raw: str, fallback: str = "Untitled") -> str:
    name = _BAD_NAME.sub(" ", str(raw))
    if name.lower().endswith(SUFFIX):
        name = name[: -len(SUFFIX)]
    name = " ".join(name.split()).strip(" .")[:NAME_LIMIT].strip(" .")
    if not name or name.lower() in _RESERVED:
        name = fallback
    return name


def _free(folder: Path, stem: str, suffix: str, first: str = "") -> Path:
    candidate = folder / f"{stem}{first}{suffix}"
    count = 2
    while candidate.exists() or candidate.is_symlink():
        candidate = folder / f"{stem}{first} {count}{suffix}"
        count += 1
    return candidate


def _thumb_file(gid: str) -> Path:
    return _thumbs_dir() / f"{gid}.img"


def _iso(seconds: float) -> str:
    return datetime.fromtimestamp(seconds, timezone.utc).isoformat(timespec="seconds")


def _stamp(text: str) -> float:
    try:
        return datetime.fromisoformat(text).timestamp()
    except (TypeError, ValueError):
        return 0.0


def panel_count(path: Path) -> int | None:
    try:
        stat = path.stat()
    except OSError:
        return None
    key = str(path)
    held = _panels.get(key)
    if held is not None and held[0] == stat.st_mtime_ns and held[1] == stat.st_size:
        return held[2]
    count: int | None = None
    try:
        with zipfile.ZipFile(path) as archive:
            info = archive.getinfo("garment.json")
            if info.file_size <= JSON_LIMIT:
                panels = json.loads(archive.read(info)).get("panels")
                count = len(panels) if isinstance(panels, list) else None
    except (OSError, KeyError, ValueError, AttributeError, zipfile.BadZipFile):
        count = None
    if len(_panels) > 2000:
        _panels.clear()
    _panels[key] = (stat.st_mtime_ns, stat.st_size, count)
    return count


def _summary(path: Path, library: bool, used: float = 0.0) -> dict:
    stat = path.stat()
    gid = garment_id(path)
    return {
        "id": gid,
        "path": str(path),
        "name": path.name,
        "title": path.stem,
        "bytes": stat.st_size,
        "modified": _iso(max(stat.st_mtime, used)),
        "panels": panel_count(path),
        "thumb": _thumb_file(gid).is_file(),
        "library": library,
    }


def garments() -> list[dict]:
    used = {os.path.normcase(e["path"]): _stamp(e.get("modified", "")) for e in recent.entries()}
    out: list[dict] = []
    seen: set[str] = set()
    folder = library_dir()
    try:
        names = sorted(os.listdir(folder))
    except OSError:
        names = []
    for name in names:
        path = folder / name
        if not in_library(path):
            continue
        try:
            item = _summary(path, True, used.get(os.path.normcase(str(path.resolve())), 0.0))
        except OSError:
            continue
        seen.add(item["id"])
        out.append(item)
    for entry in recent.entries():
        path = Path(entry["path"])
        try:
            if path.suffix.lower() != SUFFIX or not path.is_file() or in_library(path):
                continue
            item = _summary(path, False, _stamp(entry.get("modified", "")))
        except OSError:
            continue
        if item["id"] not in seen:
            seen.add(item["id"])
            out.append(item)
    out.sort(key=lambda g: g["modified"], reverse=True)
    return out


def find(gid: str) -> dict:
    if not GARMENT_ID.match(gid):
        raise LookupError("No such garment.")
    for item in garments():
        if item["id"] == gid:
            return item
    raise LookupError("That garment is no longer in the list.")


def untitled_path() -> str:
    return str(_free(library_dir(), "Untitled", SUFFIX))


def _settings_file() -> Path:
    return config.data_dir() / "home.json"


def _id_list(value) -> list[str]:
    if not isinstance(value, list):
        return []
    out: list[str] = []
    for item in value:
        if isinstance(item, str) and GARMENT_ID.match(item) and item not in out:
            out.append(item)
    return out[:LIST_LIMIT]


def settings() -> dict:
    try:
        data = json.loads(_settings_file().read_text(encoding="utf-8"))
    except (OSError, ValueError):
        data = {}
    if not isinstance(data, dict):
        data = {}
    return {"pinned": _id_list(data.get("pinned")), "recentHidden": _id_list(data.get("recentHidden"))}


def update_settings(patch: dict) -> dict:
    with _lock:
        current = settings()
        for key in ("pinned", "recentHidden"):
            if key in patch:
                if not isinstance(patch[key], list):
                    raise ValueError(f"{key} must be a list of garment ids")
                current[key] = _id_list(patch[key])
        _write_atomic(_settings_file(), json.dumps(current).encode("utf-8"))
        return current


def _drop_from_settings(gid: str) -> None:
    current = settings()
    if gid in current["pinned"] or gid in current["recentHidden"]:
        update_settings({k: [x for x in v if x != gid] for k, v in current.items()})


def templates() -> dict:
    folder = templates_dir()
    listed = []
    for spec in TEMPLATES:
        source = folder / spec["file"]
        if not source.is_file():
            continue
        listed.append({
            "id": spec["id"],
            "name": spec["name"],
            "category": spec["category"],
            "tags": list(spec["tags"]),
            "description": "",
            "source": "builtin",
            "panels": panel_count(source),
            "thumb": (folder / f"{spec['id']}.webp").is_file(),
        })
    present = {t["category"] for t in listed}
    return {"categories": [c for c in CATEGORIES if c in present], "templates": listed, "default": listed[0]["id"] if listed else ""}


def _template(tid: str) -> tuple[dict, Path]:
    if not isinstance(tid, str) or not TEMPLATE_ID.match(tid):
        raise LookupError("No such template.")
    for spec in TEMPLATES:
        if spec["id"] == tid:
            source = templates_dir() / spec["file"]
            if source.is_file():
                return spec, source
    raise LookupError("No such template.")


def template_thumb(tid: str) -> Path:
    spec, source = _template(tid)
    picture = source.parent / f"{spec['id']}.webp"
    if not picture.is_file():
        raise LookupError("That template has no preview.")
    return picture


def create(template: str, title: str | None = None) -> dict:
    spec, source = _template(template)
    with _lock:
        target = _free(library_dir(), clean_name(title or spec["name"], spec["name"]), SUFFIX)
        _copy_atomic(source, target)
        picture = source.parent / f"{spec['id']}.webp"
        if picture.is_file():
            try:
                _copy_atomic(picture, _thumb_file(garment_id(target)))
            except OSError:
                pass
        return _summary(target, True, time.time())


def duplicate(gid: str) -> dict:
    item = find(gid)
    source = Path(item["path"])
    with _lock:
        target = _free(library_dir(), clean_name(source.stem), SUFFIX, " copy")
        _copy_atomic(source, target)
        thumb = _thumb_file(gid)
        if thumb.is_file():
            try:
                _copy_atomic(thumb, _thumb_file(garment_id(target)))
            except OSError:
                pass
        return _summary(target, True, time.time())


def forget(gid: str) -> None:
    item = find(gid)
    recent.forget(item["path"])
    if not item["library"]:
        try:
            _thumb_file(gid).unlink()
        except OSError:
            pass
    _drop_from_settings(gid)


def move_to_trash(gid: str) -> str:
    item = find(gid)
    source = Path(item["path"])
    if not item["library"] or not in_library(source):
        raise NotInLibrary("This garment is outside the library, so it is never moved or deleted here. Remove it from the recent list instead.")
    with _lock:
        stamp = int(time.time())
        bin_dir = trash_dir()
        entry = bin_dir / f"{gid}-{stamp}"
        while entry.exists():
            stamp += 1
            entry = bin_dir / f"{gid}-{stamp}"
        entry.mkdir()
        meta = {"id": gid, "name": source.name, "title": source.stem, "panels": item["panels"], "deletedAt": stamp}
        _write_atomic(entry / "meta.json", json.dumps(meta).encode("utf-8"))
        os.replace(source, entry / source.name)
        thumb = _thumb_file(gid)
        if thumb.is_file():
            try:
                os.replace(thumb, entry / "thumb.img")
            except OSError:
                pass
    recent.forget(str(source))
    _drop_from_settings(gid)
    return entry.name


def _entry(name: str) -> Path:
    if not isinstance(name, str) or not TRASH_ENTRY.match(name):
        raise LookupError("That garment is no longer in the trash.")
    bin_dir = trash_dir()
    entry = bin_dir / name
    if entry.is_symlink() or not entry.is_dir() or entry.resolve().parent != bin_dir.resolve():
        raise LookupError("That garment is no longer in the trash.")
    return entry


def _entry_file(entry: Path) -> Path | None:
    try:
        files = [p for p in entry.iterdir() if p.is_file() and not p.is_symlink() and p.suffix.lower() == SUFFIX]
    except OSError:
        return None
    return files[0] if len(files) == 1 else None


def _trashed(entry: Path) -> dict | None:
    found = TRASH_ENTRY.match(entry.name)
    held = _entry_file(entry)
    if found is None or held is None:
        return None
    try:
        meta = json.loads((entry / "meta.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        meta = {}
    if not isinstance(meta, dict):
        meta = {}
    panels = meta.get("panels")
    return {
        "entry": entry.name,
        "id": found.group(1),
        "title": held.stem,
        "name": held.name,
        "panels": panels if isinstance(panels, int) else None,
        "deletedAt": int(found.group(2)),
        "thumb": (entry / "thumb.img").is_file(),
    }


def trash() -> list[dict]:
    bin_dir = trash_dir()
    out = []
    try:
        names = os.listdir(bin_dir)
    except OSError:
        names = []
    for name in names:
        entry = bin_dir / name
        if not TRASH_ENTRY.match(name) or entry.is_symlink() or not entry.is_dir():
            continue
        item = _trashed(entry)
        if item is not None:
            out.append(item)
    out.sort(key=lambda g: (g["deletedAt"], g["entry"]), reverse=True)
    return out


def restore(name: str) -> dict:
    entry = _entry(name)
    held = _entry_file(entry)
    if held is None:
        raise LookupError("That garment is no longer in the trash.")
    with _lock:
        folder = library_dir()
        target = folder / held.name
        if target.exists() or target.is_symlink():
            target = _free(folder, clean_name(held.stem), SUFFIX, " (restored)")
        os.replace(held, target)
        thumb = entry / "thumb.img"
        if thumb.is_file():
            try:
                os.replace(thumb, _thumb_file(garment_id(target)))
            except OSError:
                pass
        shutil.rmtree(entry, ignore_errors=True)
        return _summary(target, True, time.time())


def purge(name: str) -> None:
    entry = _entry(name)
    with _lock:
        shutil.rmtree(entry)


def empty_trash() -> int:
    removed = 0
    for item in trash():
        try:
            purge(item["entry"])
            removed += 1
        except (LookupError, OSError):
            continue
    return removed


def image_type(head: bytes) -> str | None:
    for magic, kind in _IMAGE_HEADS:
        if head.startswith(magic):
            return kind
    if head[:4] == b"RIFF" and head[8:12] == b"WEBP":
        return "image/webp"
    return None


def put_thumb(path: str, data: bytes) -> str:
    if not data or len(data) > THUMB_LIMIT or image_type(data[:12]) is None:
        raise ValueError("A preview is a PNG, JPEG or WebP picture of at most 2 MB.")
    target = Path(path)
    if not target.is_absolute() or target.suffix.lower() != SUFFIX:
        raise LookupError("No such garment.")
    gid = garment_id(target)
    find(gid)
    _write_atomic(_thumb_file(gid), data)
    return gid


def thumb(gid: str) -> tuple[Path, str]:
    if not GARMENT_ID.match(gid):
        raise LookupError("No preview.")
    picture = _thumb_file(gid)
    try:
        if picture.is_symlink() or not picture.is_file():
            raise OSError
        with picture.open("rb") as handle:
            kind = image_type(handle.read(12))
    except OSError:
        kind = None
    if kind is None:
        raise LookupError("No preview.")
    return picture, kind


def import_kind(name: str) -> str:
    suffix = Path(str(name)).suffix.lower()
    if suffix == SUFFIX:
        return "garment"
    if suffix == ".dxf":
        return "dxf"
    raise ValueError("Only .ffbrc garment projects and .dxf patterns can be brought in here.")


def import_limit(kind: str) -> int:
    return PROJECT_LIMIT if kind == "garment" else DXF_LIMIT


def import_temp(kind: str) -> tuple[int, str]:
    folder = library_dir() if kind == "garment" else _imports_dir()
    return tempfile.mkstemp(prefix=".tmp-", dir=str(folder))


def finish_import(kind: str, name: str, tmp: str) -> dict:
    try:
        if kind == "garment":
            try:
                with zipfile.ZipFile(tmp) as archive:
                    archive.getinfo("manifest.json")
            except (OSError, KeyError, zipfile.BadZipFile):
                raise ValueError("That file is not a FormFabric project.")
            with _lock:
                target = _free(library_dir(), clean_name(Path(name).stem), SUFFIX)
                place(tmp, target)
            return {"kind": "garment", "path": str(target), "garment": _summary(target, True, time.time())}
        with _lock:
            target = _free(_imports_dir(), clean_name(Path(name).stem, "Pattern"), ".dxf")
            place(tmp, target)
        return {"kind": "dxf", "path": str(target)}
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise
