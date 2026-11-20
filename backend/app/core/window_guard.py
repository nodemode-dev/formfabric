import asyncio
import json
import logging
import re
import secrets

from fastapi import HTTPException, Request, WebSocket, WebSocketDisconnect

from app.core import config

_ORIGIN = re.compile(r"^([a-z][a-z0-9+.\-]*)://(\[[0-9a-f:]+\]|[a-z0-9._\-]+)(?::([0-9]{1,5}))?$")
_DEFAULT_PORT = {"http": 80, "https": 443}
REFUSAL = "Only the formfabric window can do this"
WINDOW_HEADER = "x-ffb-window"
READS = ("GET", "HEAD", "OPTIONS")
OPEN = ("/api/mcp", "/api/window/key")
CLAIM = {"claimed": False, "secondClaim": False}
_log = logging.getLogger("formfabric.window_guard")
_warned_short = False
HANDSHAKE_SECONDS = 5


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


def _window_token(given: str, expected: str) -> bool:
    return secrets.compare_digest(given.encode("utf-8"), expected.encode("utf-8"))


def _warn_short() -> None:
    global _warned_short
    if not _warned_short:
        _warned_short = True
        _log.error("FFB_WINDOW_TOKEN is shorter than %d characters: every window route is refused until a longer token is set", config.TOKEN_MIN)


def claim() -> str | None:
    if config.window_key_given():
        return None
    if CLAIM["claimed"]:
        CLAIM["secondClaim"] = True
        return None
    CLAIM["claimed"] = True
    return config.window_token()


def from_window(headers, same_origin_read: bool = False) -> bool:
    if not host_ok(headers):
        return False
    if config.window_token_rejected():
        _warn_short()
        return False
    expected = config.window_token()
    return expected is not None and _window_token(headers.get(WINDOW_HEADER, ""), expected)


class WindowKeyGuard:
    def __init__(self, app) -> None:
        self.app = app

    async def __call__(self, scope, receive, send) -> None:
        if scope["type"] != "http" or scope.get("method") in READS or str(scope.get("path", "")) in OPEN:
            await self.app(scope, receive, send)
            return
        headers = {k.decode("latin-1").lower(): v.decode("latin-1") for k, v in scope.get("headers", [])}
        if from_window(headers):
            await self.app(scope, receive, send)
            return
        body = json.dumps({"detail": REFUSAL}).encode("utf-8")
        await send({"type": "http.response.start", "status": 403, "headers": [(b"content-type", b"application/json"), (b"content-length", str(len(body)).encode())]})
        await send({"type": "http.response.body", "body": body})


def require_window(request: Request) -> None:
    if not from_window(request.headers):
        raise HTTPException(status_code=403, detail=REFUSAL)


def require_window_read(request: Request) -> None:
    if not from_window(request.headers, same_origin_read=True):
        raise HTTPException(status_code=403, detail=REFUSAL)


async def socket_from_window(ws: WebSocket) -> bool:
    if from_window(ws.headers):
        return True
    expected = config.window_token()
    if expected is None or not host_ok(ws.headers):
        return False
    try:
        given = await asyncio.wait_for(ws.receive_text(), timeout=HANDSHAKE_SECONDS)
    except (asyncio.TimeoutError, WebSocketDisconnect, RuntimeError):
        return False
    return _window_token(given, expected)
