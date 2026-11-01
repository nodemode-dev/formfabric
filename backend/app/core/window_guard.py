import logging
import re
import secrets

from fastapi import HTTPException, Request, WebSocket

from app.core import config
from app.services import agents

_ORIGIN = re.compile(r"^([a-z][a-z0-9+.\-]*)://(\[[0-9a-f:]+\]|[a-z0-9._\-]+)(?::([0-9]{1,5}))?$")
_DEFAULT_PORT = {"http": 80, "https": 443}
REFUSAL = "this route answers the formfabric window only"
WINDOW_HEADER = "x-ffb-window"
WINDOW_QUERY = "ffb_window"
_log = logging.getLogger("formfabric.window_guard")
_warned = False


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


def allowed_hosts() -> set[str]:
    hosts = set()
    for origin in window_origins():
        found = _ORIGIN.match(origin)
        if found is not None and found.group(2) in ("127.0.0.1", "localhost"):
            hosts.add(found.group(2) + (f":{found.group(3)}" if found.group(3) else ""))
    return hosts


def host_ok(headers) -> bool:
    return headers.get("host", "").strip().lower() in allowed_hosts()


def _token(headers) -> bool:
    given = headers.get(agents.INAPP_HEADER, "")
    return secrets.compare_digest(given.encode("utf-8"), agents.INAPP_TOKEN.encode("utf-8"))


def _window_token(given: str, expected: str) -> bool:
    return secrets.compare_digest(given.encode("utf-8"), expected.encode("utf-8"))


def _warn_open() -> None:
    global _warned
    if not _warned:
        _warned = True
        _log.warning("FFB_WINDOW_TOKEN is not set: window routes fall back to the exact-origin rule, which a local program can forge")


def from_window(headers, same_origin_read: bool = False, query=None) -> bool:
    if not host_ok(headers):
        return False
    if _token(headers):
        return True
    expected = config.window_token()
    if expected is not None:
        given = headers.get(WINDOW_HEADER) or (query.get(WINDOW_QUERY, "") if query is not None else "")
        return _window_token(given, expected)
    _warn_open()
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
    return from_window(ws.headers, query=ws.query_params)
