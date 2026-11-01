import asyncio
import os
import shutil
import sys
import threading
import time
import uuid
from collections import deque
from pathlib import Path

from app.core import config
from app.services import events

if os.name == "nt":
    from winpty import PtyProcess
else:
    from ptyprocess import PtyProcessUnicode as PtyProcess

SCROLLBACK = 256 * 1024


class Session:
    def __init__(self, sid: str, title: str, cwd: str, argv: list[str], env: dict, kind: str) -> None:
        self.id = sid
        self.title = title
        self.cwd = cwd
        self.argv = argv
        self.env = env
        self.kind = kind
        self.proc = None
        self.chunks: deque[str] = deque()
        self.size = 0
        self.alive = True
        self.subscribers: set[asyncio.Queue] = set()
        self.lock = threading.Lock()
        self.gen = 0
        self.cols = 100
        self.rows = 30
        self.pos = 0

    def describe(self) -> dict:
        return {"id": self.id, "title": self.title, "cwd": self.cwd, "alive": self.alive, "kind": self.kind}

    def keep(self, text: str) -> int:
        with self.lock:
            self.chunks.append(text)
            self.size += len(text)
            self.pos += len(text)
            while self.size > SCROLLBACK and len(self.chunks) > 1:
                self.size -= len(self.chunks.popleft())
            return self.pos

    def scrollback(self) -> str:
        with self.lock:
            return "".join(self.chunks)

    def snapshot(self) -> tuple[str, int]:
        with self.lock:
            return "".join(self.chunks), self.pos


SESSIONS: dict[str, Session] = {}
_loop: asyncio.AbstractEventLoop | None = None


def start(loop: asyncio.AbstractEventLoop) -> None:
    global _loop
    _loop = loop


def login_shell() -> list[str]:
    if os.name == "nt":
        return [os.environ.get("COMSPEC", "cmd.exe")]
    shell = os.environ.get("SHELL") or shutil.which("bash") or "/bin/sh"
    return [shell, "-l"]


def base_env() -> dict:
    env = dict(os.environ)
    env.pop("FFB_WINDOW_TOKEN", None)
    env.pop("FFB_INAPP_TOKEN", None)
    env["TERM"] = "xterm-256color"
    env["COLORTERM"] = "truecolor"
    env["FFB_TERMINAL"] = "1"
    env["FFB_ENGINE_URL"] = config.engine_url()
    env["FFB_MCP_URL"] = config.engine_url() + "/api/mcp"
    if os.name != "nt" and not getattr(sys, "frozen", False):
        env["PATH"] = str(config.REPO) + os.pathsep + env.get("PATH", "")
    return env


HIGH_WATER = 48
HARD_LIMIT = 20000


def _deliver(session: Session, text: str | None, pos: int) -> None:
    item = None if text is None else (text, pos)
    for q in list(session.subscribers):
        if q.qsize() < HARD_LIMIT or text is None:
            q.put_nowait(item)


def _backpressure(session: Session) -> None:
    waited = 0.0
    while waited < 1.0 and any(q.qsize() > HIGH_WATER for q in list(session.subscribers)):
        time.sleep(0.004)
        waited += 0.004


def _send(session: Session, text: str | None, pos: int = -1) -> None:
    if _loop is not None and not _loop.is_closed():
        _loop.call_soon_threadsafe(_deliver, session, text, pos)


def _pump(session: Session, proc, gen: int) -> None:
    while True:
        try:
            text = proc.read(65536)
        except (EOFError, OSError):
            break
        if not text:
            continue
        if session.gen != gen:
            return
        _send(session, text, session.keep(text))
        _backpressure(session)
    if session.gen != gen:
        return
    session.alive = False
    _send(session, None)
    events.publish("terminal.exited", terminal=session.describe())


def _spawn(session: Session) -> None:
    where = session.cwd if os.path.isdir(session.cwd) else str(Path.home())
    proc = PtyProcess.spawn(session.argv, cwd=where, env=session.env, dimensions=(session.rows, session.cols))
    session.proc = proc
    session.alive = True
    threading.Thread(target=_pump, args=(session, proc, session.gen), name=f"pty-{session.id}-{session.gen}", daemon=True).start()


def open_session(cwd: str | None = None, cols: int = 100, rows: int = 30, title: str | None = None,
                 argv: list[str] | None = None, env: dict | None = None, kind: str = "shell") -> Session:
    where = cwd or str(Path.home())
    if not os.path.isdir(where):
        raise FileNotFoundError(f"No folder {where}")
    sid = uuid.uuid4().hex[:12]
    session = Session(sid, title or f"Terminal {len(SESSIONS) + 1}", where, argv or login_shell(), env or base_env(), kind)
    session.cols, session.rows = min(1000, max(10, cols)), min(1000, max(4, rows))
    _spawn(session)
    SESSIONS[sid] = session
    events.publish("terminal.opened", terminal=session.describe())
    return session


def restart(sid: str) -> Session:
    session = get(sid)
    old = session.proc
    session.gen += 1
    try:
        old.terminate(force=True)
    except Exception:
        pass
    with session.lock:
        session.chunks.clear()
        session.size = 0
    _send(session, "\x1bc")
    _spawn(session)
    events.publish("terminal.restarted", terminal=session.describe())
    return session


def write(sid: str, data: str) -> None:
    session = get(sid)
    if session.proc is not None and session.alive:
        session.proc.write(data)


def resize(sid: str, cols: int, rows: int) -> None:
    session = get(sid)
    session.cols, session.rows = min(1000, max(1, cols)), min(1000, max(1, rows))
    if session.alive and session.proc is not None:
        session.proc.setwinsize(session.rows, session.cols)


def close(sid: str) -> None:
    session = SESSIONS.pop(sid, None)
    if session is None:
        return
    session.gen += 1
    try:
        session.proc.terminate(force=True)
    except Exception:
        pass
    events.publish("terminal.closed", id=sid)


def get(sid: str) -> Session:
    session = SESSIONS.get(sid)
    if session is None:
        raise KeyError(sid)
    return session


def close_all() -> None:
    for sid in list(SESSIONS):
        close(sid)
