from __future__ import annotations

import json
import os
import urllib.error
import urllib.parse
import urllib.request
from typing import Any

from .errors import BackendError, CommandError

DEFAULT_PORT = 8280
WINDOW_HEADER = "x-ffb-window"
INAPP_HEADER = "x-formfabric-inapp"
TOKEN_MIN = 32


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


_OPENER = urllib.request.build_opener(_NoRedirect, urllib.request.ProxyHandler({}))


def is_loopback(url: str) -> bool:
    parts = urllib.parse.urlsplit(url)
    return parts.scheme == "http" and parts.hostname in ("127.0.0.1", "localhost")


class Backend:
    name = "backend"

    def dispatch(self, name: str, args: dict | None = None) -> dict:
        raise NotImplementedError

    def catalogue(self) -> dict:
        raise NotImplementedError

    def close(self) -> None:
        return None


class InProcessBackend(Backend):
    name = "engine"

    def __init__(self) -> None:
        try:
            import ffb_engine
        except ImportError as exc:
            raise BackendError("the ffb_engine module is not importable; run ./formfabric engine build, or use the http backend") from exc
        self._module = ffb_engine
        self._engine = ffb_engine.Engine()

    @property
    def engine(self) -> Any:
        return self._engine

    def dispatch(self, name: str, args: dict | None = None) -> dict:
        return json.loads(self._engine.dispatch(name, json.dumps(args or {})))

    def catalogue(self) -> dict:
        return json.loads(self._module.catalogue())

    def version(self) -> str:
        return self._module.version()


def resolve_token(token: str | None = None) -> tuple[str | None, str | None]:
    if token:
        return (WINDOW_HEADER, token)
    for env, header in (("FFB_WINDOW_TOKEN", WINDOW_HEADER), ("FFB_INAPP_TOKEN", INAPP_HEADER)):
        value = os.environ.get(env, "").strip()
        if value:
            return (header, value)
    return (None, None)


def resolve_url(url: str | None = None) -> str:
    if url:
        return url.rstrip("/")
    given = os.environ.get("FFB_ENGINE_URL", "").strip()
    if given:
        return given.rstrip("/")
    port = os.environ.get("FFB_PORT", "").strip()
    return f"http://127.0.0.1:{port if port.isdigit() and int(port) > 0 else DEFAULT_PORT}"


class HttpBackend(Backend):
    name = "http"

    def __init__(self, url: str | None = None, token: str | None = None, timeout: float = 600.0) -> None:
        self.url = resolve_url(url)
        self.timeout = timeout
        self._header, self._token = resolve_token(token)
        if self._header == WINDOW_HEADER and self._token is not None and len(self._token) < TOKEN_MIN:
            raise BackendError(f"the window token must be at least {TOKEN_MIN} characters; the service refuses a shorter one")
        if self._token and not is_loopback(self.url):
            raise BackendError("refusing to send the window token to a URL that is not http on 127.0.0.1 or localhost; unset the token or use the local service")
        self._catalogue: dict | None = None

    def _request(self, path: str, body: dict | None = None) -> Any:
        headers = {"Accept": "application/json"}
        data = None
        if body is not None:
            data = json.dumps(body).encode("utf-8")
            headers["Content-Type"] = "application/json"
        if self._header and self._token:
            headers[self._header] = self._token
        request = urllib.request.Request(self.url + path, data=data, headers=headers, method="POST" if body is not None else "GET")
        try:
            with _OPENER.open(request, timeout=self.timeout) as response:
                return json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            detail = ""
            try:
                detail = json.loads(exc.read().decode("utf-8")).get("detail", "")
            except (ValueError, OSError):
                pass
            if exc.code == 403:
                raise BackendError(f"the service refused the call ({detail or 'forbidden'}); set FFB_WINDOW_TOKEN to the token the service was started with, or FFB_INAPP_TOKEN inside the app's Claude Code") from exc
            if exc.code == 503:
                raise BackendError(detail or "the engine module is not built in the service") from exc
            raise BackendError(f"{path}: HTTP {exc.code} {detail}".strip()) from exc
        except urllib.error.URLError as exc:
            raise BackendError(f"cannot reach the formfabric service at {self.url}: {exc.reason}") from exc
        except (OSError, ValueError) as exc:
            raise BackendError(f"the formfabric service at {self.url} gave no usable reply: {exc.__class__.__name__}") from exc

    def dispatch(self, name: str, args: dict | None = None) -> dict:
        return self._request("/api/engine/dispatch", {"name": name, "args": args or {}})

    def catalogue(self) -> dict:
        if self._catalogue is None:
            self._catalogue = self._request("/api/engine/catalogue")
        return self._catalogue

    def health(self) -> dict:
        return self._request("/api/health")


def make(kind: str = "auto", url: str | None = None, token: str | None = None) -> Backend:
    if kind == "engine":
        return InProcessBackend()
    if kind == "http":
        return HttpBackend(url, token)
    if kind != "auto":
        raise BackendError(f"backend must be auto, engine or http, got {kind}")
    if url or token or os.environ.get("FFB_ENGINE_URL"):
        return HttpBackend(url, token)
    try:
        return InProcessBackend()
    except BackendError:
        return HttpBackend(url, token)


def unwrap(command: str, reply: dict) -> Any:
    if reply.get("ok"):
        return reply.get("result")
    error = reply.get("error") or {}
    if isinstance(error, str):
        raise CommandError(command, "failed", error)
    raise CommandError(command, str(error.get("code", "failed")), str(error.get("message", "the command failed")))
