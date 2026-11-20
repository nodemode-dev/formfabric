import re

from app.services import agent_programs, terminal
from app.services.commands import CommandError, command

ID = {"type": "string", "description": "terminal id from terminal.open or terminal.list"}
ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07]*(\x07|\x1b\\)|\x1b[@-_]")


def _session(sid: str) -> terminal.Session:
    try:
        return terminal.get(sid)
    except KeyError:
        raise CommandError(f"No terminal {sid}; terminal.list gives the ids", 404) from None


@command("terminal.open", "New Terminal", {"cwd": {"type": "string"}, "cols": {"type": "integer"}, "rows": {"type": "integer"}, "title": {"type": "string"}},
         description="Starts the person's login shell in a real terminal in formfabric's terminal dock (default folder: the home folder) and returns its id. The window only; outside agents cannot reach it.")
def open_(cwd: str | None = None, cols: int | None = None, rows: int | None = None, title: str | None = None) -> dict:
    try:
        return terminal.open_session(cwd, cols or 100, rows or 30, title).describe()
    except FileNotFoundError as exc:
        raise CommandError(str(exc), 404) from None


@command("terminal.list", "Terminals", description="Open terminals with their ids, titles, folders, kind (shell or claude) and whether the program is still running.")
def list_() -> dict:
    items = [s.describe() for s in terminal.SESSIONS.values()]
    return {"terminals": items, "count": len(items)}


@command("terminal.write", "Type into a Terminal", {"id": ID, "data": {"type": "string", "description": "text to type; end with \\r to press Enter"}},
         description="Types text into a terminal as if at the keyboard. The window shows it live.")
def write(id: str, data: str) -> dict:
    session = _session(id)
    if not session.alive:
        raise CommandError(f"Terminal {id} has exited; restart it or open a new one", 409)
    terminal.write(id, data)
    return {"written": len(data)}


@command("terminal.read", "Read a Terminal", {"id": ID, "chars": {"type": "integer", "description": "how much of the end to return, default 4000"}},
         description="The end of a terminal's output as plain text, colour codes removed.")
def read(id: str, chars: int | None = None) -> dict:
    session = _session(id)
    text = ANSI.sub("", session.scrollback()).replace("\r\n", "\n").replace("\r", "")
    n = max(1, min(int(chars or 4000), 200000))
    return {"id": id, "alive": session.alive, "text": text[-n:]}


@command("terminal.resize", "Resize a Terminal", {"id": ID, "cols": {"type": "integer"}, "rows": {"type": "integer"}})
def resize(id: str, cols: int, rows: int) -> dict:
    _session(id)
    cols, rows = min(1000, max(1, cols)), min(1000, max(1, rows))
    terminal.resize(id, cols, rows)
    return {"id": id, "cols": cols, "rows": rows}


@command("terminal.restart", "Restart a Terminal", {"id": ID},
         description="Ends the terminal's program and starts it again in the same tab and folder; its output so far is cleared.")
def restart(id: str) -> dict:
    _session(id)
    return terminal.restart(id).describe()


@command("terminal.close", "Close a Terminal", {"id": ID}, description="Ends the program and closes the terminal. Closing a terminal that is already gone succeeds.")
def close(id: str) -> dict:
    terminal.close(id)
    return {"closed": id}


@command("terminal.agent", "Open an Agent Program", {"kind": {"type": "string", "description": "codex, cursor or antigravity"}, "cols": {"type": "integer"}, "rows": {"type": "integer"}},
         description="Starts Codex, Cursor's agent or Antigravity in a terminal tab of formfabric's dock, connected to formfabric's MCP tools with a key made for that tab and ended when it closes. The window only.")
def agent(kind: str, cols: int | None = None, rows: int | None = None) -> dict:
    try:
        return agent_programs.open_terminal(kind, min(1000, max(10, cols or 100)), min(1000, max(4, rows or 30)))
    except LookupError as exc:
        raise CommandError(str(exc), 404) from None
    except OSError as exc:
        raise CommandError(f"The program could not start: {exc}", 500) from None


@command("agents.status", "Agent Programs", {"fresh": {"type": "boolean"}},
         description="Whether Claude Code, Codex, Cursor's agent and Antigravity are installed and start: for each, its path, version, how it was found, where formfabric looked and the line that installs it. The window only.")
def agents_status(fresh: bool | None = None) -> dict:
    return agent_programs.status(bool(fresh))
