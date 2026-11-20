from __future__ import annotations

import json
import os
import time
from pathlib import Path

from app.core import config
from app.services import agent_lookup, agents, claude_code, terminal

DOCK_CONTRACT = "2.2.0"
PROGRAMS = {
    "codex": {"title": "Codex", "install": agent_lookup.install("codex")},
    "cursor": {"title": "Cursor", "install": agent_lookup.install("cursor")},
    "antigravity": {"title": "Antigravity", "install": agent_lookup.install("antigravity")},
}
TABS = tuple(PROGRAMS)
MCP_URL_VAR = "FFB_MCP_URL"
SERVER = "formfabric_tab"
_cache: dict = {"at": 0.0, "value": None}


def mcp_url() -> str:
    return config.engine_url() + "/api/mcp"


def tab_folder(kind: str) -> Path:
    folder = config.data_dir() / "agent-tabs" / kind
    folder.mkdir(parents=True, exist_ok=True)
    return folder


def _write_private(path: Path, data: dict) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    part = path.with_name(path.name + f".part-{os.getpid()}")
    with os.fdopen(os.open(part, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w", encoding="utf-8") as fh:
        fh.write(json.dumps(data, indent=2) + "\n")
    os.replace(part, path)


def _servers(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8") or "{}") if path.exists() else {}
    if not isinstance(data, dict) or not isinstance(data.get("mcpServers", {}), dict):
        raise OSError(f"{path} is not an MCP servers file formfabric can read, so it was left alone")
    return data


def antigravity_file(folder: Path) -> Path:
    return folder / ".agents" / "mcp_config.json"


def write_tab_entry(folder: Path, token: str) -> None:
    path = antigravity_file(folder)
    data = _servers(path)
    data.setdefault("mcpServers", {})[SERVER] = {"serverUrl": mcp_url(), "headers": {agents.INAPP_HEADER: token}}
    _write_private(path, data)


def remove_tab_entry(folder: Path, token: str) -> None:
    path = antigravity_file(folder)
    try:
        data = _servers(path)
        entry = data.get("mcpServers", {}).get(SERVER)
        if not isinstance(entry, dict) or (entry.get("headers") or {}).get(agents.INAPP_HEADER) != token:
            return
        del data["mcpServers"][SERVER]
        _write_private(path, data)
    except (OSError, ValueError):
        return


def cursor_config(folder: Path) -> None:
    path = folder / ".cursor" / "mcp.json"
    entry = {"url": "${env:" + MCP_URL_VAR + "}", "headers": {agents.INAPP_HEADER: "${env:" + claude_code.TOKEN_ENV + "}"}}
    try:
        data = _servers(path)
        if data.get("mcpServers", {}).get(SERVER) != entry:
            data.setdefault("mcpServers", {})[SERVER] = entry
            _write_private(path, data)
    except (OSError, ValueError):
        return


def codex_args() -> list[str]:
    return ["-c", f'mcp_servers.{SERVER}.url="{mcp_url()}"',
            "-c", f'mcp_servers.{SERVER}.env_http_headers={{"{agents.INAPP_HEADER}"="{claude_code.TOKEN_ENV}"}}']


def _program(kind: str, fresh: bool) -> dict:
    found = claude_code.located(kind, fresh)
    p = PROGRAMS[kind]
    return agent_lookup.answer(found, p["install"], claude_code.OTHERWISE) | {"signedIn": None, "install": "" if found.state == "runs" else p["install"], "label": p["title"]}


def status(fresh: bool = False) -> dict:
    if fresh or _cache["value"] is None or time.time() - _cache["at"] >= 60:
        _cache.update(at=time.time(), value={kind: _program(kind, fresh) for kind in PROGRAMS})
    claude = claude_code.info(fresh=fresh)
    return {"claude": claude | {"signedIn": None, "install": "" if claude["installed"] else agent_lookup.install("claude"), "label": "Claude Code"}, **_cache["value"]}


def open_terminal(kind: str, cols: int, rows: int) -> dict:
    if kind not in TABS:
        raise ValueError(f"No agent program called {kind}; the dock offers {', '.join(TABS)}")
    found = claude_code.located(kind)
    if found.state != "runs":
        _cache["value"] = None
        raise LookupError(agent_lookup.sentence(found, PROGRAMS[kind]["install"], claude_code.OTHERWISE))
    folder = tab_folder(kind) if kind == "antigravity" else claude_code.workdir()
    if kind == "cursor":
        cursor_config(folder)
    token = agents.issue()
    env = claude_code.child_env(found.path) | {claude_code.TOKEN_ENV: token, MCP_URL_VAR: mcp_url()}
    argv = agent_lookup.start_argv([found.path] + (codex_args() if kind == "codex" else []), env)
    try:
        if kind == "antigravity":
            write_tab_entry(folder, token)
        session = terminal.open_session(str(folder), cols, rows, PROGRAMS[kind]["title"], argv, env, kind)
    except BaseException:
        agents.revoke(token)
        if kind == "antigravity":
            remove_tab_entry(folder, token)
        raise
    session.on_close.append(lambda: agents.revoke(token))
    if kind == "antigravity":
        session.on_close.append(lambda: remove_tab_entry(folder, token))
    return session.describe()
