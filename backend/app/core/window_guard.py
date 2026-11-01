import re
import secrets

from fastapi import HTTPException, Request, WebSocket

from app.core import config
from app.services import agents

_ORIGIN = re.compile(r"^([a-z][a-z0-9+.\-]*)://(\[[0-9a-f:]+\]|[a-z0-9._\-]+)(?::([0-9]{1,5}))?$")
_DEFAULT_PORT = {"http": 80, "https": 443}
REFUSAL = "this route answers the formfabric window only"


def exact(value: str) -> str | None:
    found = _ORIGIN.match(value.strip().lower())
    if found is None:
        return None
    scheme, host, port = found.groups()
    number = int(port) if port else None
    if number == _DEFAULT_PORT.get(scheme):
        number = None
    return f"{scheme}://{host}" + (f":{number}" if number is not None else "")


def window_origins() -> set[str]:
    return {key for key in (exact(o) for o in config.window_origins()) if key is not None}


def _token(headers) -> bool:
    given = headers.get(agents.INAPP_HEADER, "")
    return secrets.compare_digest(given.encode("utf-8"), agents.INAPP_TOKEN.encode("utf-8"))


def from_window(headers, same_origin_read: bool = False) -> bool:
    if _token(headers):
        return True
    origin = headers.get("origin")
    if origin is not None:
        key = exact(origin)
        return key is not None and key in window_origins()
    return same_origin_read and headers.get("sec-fetch-site", "").strip().lower() == "same-origin"


def require_window(request: Request) -> None:
    if not from_window(request.headers):
        raise HTTPException(status_code=403, detail=REFUSAL)


def require_window_read(request: Request) -> None:
    if not from_window(request.headers, same_origin_read=True):
        raise HTTPException(status_code=403, detail=REFUSAL)


def socket_from_window(ws: WebSocket) -> bool:
    return from_window(ws.headers)
