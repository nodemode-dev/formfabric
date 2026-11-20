import asyncio
import secrets
import time

from app.core import prefs

INAPP_HEADER = "x-formfabric-inapp"
INAPP_TOKEN = secrets.token_urlsafe(24)
TAB_TOKENS: set[str] = set()


def issue() -> str:
    token = secrets.token_urlsafe(32)
    TAB_TOKENS.add(token)
    return token


def revoke(token: str | None) -> None:
    TAB_TOKENS.discard(token or "")


def is_inapp(given: str | None) -> bool:
    if not isinstance(given, str) or not given:
        return False
    raw = given.encode("utf-8")
    return any(secrets.compare_digest(raw, t.encode("utf-8")) for t in [INAPP_TOKEN, *TAB_TOKENS])
LIVE_SECONDS = 4.0
DECLINE_SECONDS = 600.0
ASK_SECONDS = 90.0
SWITCHED_OFF = "Outside agents are switched off in formfabric (Preferences, Plugins, Allow outside agents). Ask the person to switch that on, then retry."

_last_poll = 0.0
_allowed: bool | None = None
_declined_until = 0.0
_pending: dict | None = None
_answer: asyncio.Event | None = None


def seen() -> None:
    global _last_poll
    _last_poll = time.monotonic()


def window_live() -> bool:
    return time.monotonic() - _last_poll <= LIVE_SECONDS


def pending() -> dict | None:
    return _pending


def switched_off() -> str | None:
    return SWITCHED_OFF if prefs.load().get("agentsAllowed") is False else None


def status() -> dict:
    return {"window": window_live(), "enabled": switched_off() is None, "allowed": _allowed, "asking": _pending is not None, "declined": time.monotonic() < _declined_until}


def answer(allow: bool) -> None:
    global _allowed, _declined_until, _pending
    _allowed = allow
    if not allow:
        _declined_until = time.monotonic() + DECLINE_SECONDS
    _pending = None
    if _answer is not None:
        _answer.set()


def reset() -> None:
    global _allowed, _declined_until
    _allowed = None
    _declined_until = 0.0


async def permit(tool: str, inapp: bool = False) -> str | None:
    global _pending, _answer
    if time.monotonic() < _declined_until:
        return "The person declined outside agents in the formfabric window in the last ten minutes. Ask them to allow agents there, then retry."
    if inapp or _allowed is True:
        return None
    if not window_live():
        return "The formfabric window is not open, so nobody can approve outside agents. Ask the person to open formfabric and allow agents there, then retry."
    if _pending is None:
        _answer = asyncio.Event()
        _pending = {"tool": tool, "asked": time.time()}
    event = _answer
    try:
        await asyncio.wait_for(event.wait(), ASK_SECONDS)
    except asyncio.TimeoutError:
        return "Nobody answered the permission prompt in the formfabric window. Ask the person to allow outside agents there, then retry."
    if _allowed:
        return None
    return "The person declined outside agents in the formfabric window. Do not retry unless they ask you to."
