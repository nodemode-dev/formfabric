import os
from pathlib import Path

from app import __version__

APP_NAME = "formfabric"
VERSION = __version__
_frontend_raw = os.environ.get("FFB_FRONTEND_PORT", "").strip()
FRONTEND_PORT = int(_frontend_raw) if _frontend_raw.isdigit() and 0 < int(_frontend_raw) < 65534 else 5280
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
    preview = FRONTEND_PORT + 2
    own = engine_port()
    return [*cors_origins(), f"http://localhost:{preview}", f"http://127.0.0.1:{preview}", f"http://127.0.0.1:{own}", f"http://localhost:{own}", "tauri://localhost", "http://tauri.localhost"]


TOKEN_MIN = 32
_TOKEN = os.environ.pop("FFB_WINDOW_TOKEN", "").strip()


def window_token() -> str | None:
    return _TOKEN if len(_TOKEN) >= TOKEN_MIN else None


def window_token_rejected() -> bool:
    return 0 < len(_TOKEN) < TOKEN_MIN
