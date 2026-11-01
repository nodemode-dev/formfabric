import os
from pathlib import Path

from app import __version__

APP_NAME = "formfabric"
VERSION = __version__
FRONTEND_PORT = 5280
BACKEND_PORT = 8280
REPO = Path(__file__).resolve().parents[3]


def dist_dir() -> Path | None:
    configured = os.environ.get("FFB_DIST")
    if configured:
        path = Path(configured)
        return path if (path / "index.html").is_file() else None
    path = REPO / "frontend" / "dist"
    return path if (path / "index.html").is_file() else None


def engine_port() -> int:
    raw = os.environ.get("FFB_PORT", "").strip()
    return int(raw) if raw.isdigit() and int(raw) > 0 else BACKEND_PORT


def engine_url() -> str:
    return f"http://127.0.0.1:{engine_port()}"


def data_dir() -> Path:
    raw = os.environ.get("FORMFABRIC_DATA_DIR", "").strip()
    path = Path(raw).expanduser() if raw else Path.home() / "Formfabric"
    path.mkdir(parents=True, exist_ok=True)
    return path


def is_desktop() -> bool:
    return os.environ.get("FFB_DESKTOP") == "1"


def cors_origins() -> list[str]:
    configured = os.environ.get("FFB_CORS_ORIGINS")
    if configured:
        return [o.strip() for o in configured.split(",") if o.strip()]
    return [f"http://localhost:{FRONTEND_PORT}", f"http://127.0.0.1:{FRONTEND_PORT}"]


def window_origins() -> list[str]:
    preview = FRONTEND_PORT + 1
    return [*cors_origins(), f"http://localhost:{preview}", f"http://127.0.0.1:{preview}", "tauri://localhost", "http://tauri.localhost"]
