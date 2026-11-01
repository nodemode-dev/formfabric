import platform
import sys
from pathlib import Path

from fastapi import APIRouter, Depends
from fastapi.responses import FileResponse, JSONResponse

from app.core import config
from app.core.window_guard import require_window_read
from app.services import engine, recent

router = APIRouter()
IMAGE_TYPES = {".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".webp": "image/webp", ".gif": "image/gif"}
IMAGE_LIMIT = 20 * 1024 * 1024


@router.get("/health")
def health() -> dict:
    return {"ok": True, "app": config.APP_NAME, "version": config.VERSION}


@router.get("/version")
def version() -> dict:
    return {"app": config.APP_NAME, "version": config.VERSION}


@router.get("/recent", dependencies=[Depends(require_window_read)])
def recent_files() -> dict:
    return {"files": recent.listing()}


@router.get("/files/image", dependencies=[Depends(require_window_read)])
def file_image(path: str):
    target = Path(path)
    kind = IMAGE_TYPES.get(target.suffix.lower())
    if kind is None or not target.is_absolute():
        return JSONResponse({"ok": False, "error": "Not an absolute image path"}, status_code=400)
    try:
        resolved = target.resolve(strict=True)
        if not resolved.is_file() or resolved.stat().st_size > IMAGE_LIMIT:
            raise OSError
    except OSError:
        return JSONResponse({"ok": False, "error": "No such image, or larger than 20 MB"}, status_code=404)
    return FileResponse(resolved, media_type=kind)


@router.get("/capabilities")
def capabilities() -> dict:
    return {
        "app": config.APP_NAME,
        "version": config.VERSION,
        "desktop": config.is_desktop(),
        "python": sys.version.split()[0],
        "platform": platform.system().lower(),
        "engine": engine.status(),
        "frontend_built": config.dist_dir() is not None,
    }
