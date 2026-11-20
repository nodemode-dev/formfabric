import json
import os
import shlex
import time
from pathlib import Path

from app.core import config, prefs, seal
from app.services import agent_lookup, agents, terminal
from app.services.commands import CommandError, command

KEEP_ENV = {"CLAUDE_CODE_OAUTH_TOKEN", "CLAUDE_CODE_USE_BEDROCK", "CLAUDE_CODE_USE_VERTEX", "CLAUDE_CONFIG_DIR"}
OVERRIDES = {"claude": "FFB_CLAUDE_BIN", "codex": "FFB_CODEX_BIN", "cursor": "FFB_CURSOR_BIN", "antigravity": "FFB_ANTIGRAVITY_BIN"}
OTHERWISE = "give its path in the Agents settings"
INSTALL = "Claude Code is not installed: install it from https://claude.com/claude-code, run claude once in a terminal to sign in, then open it here again"
NOTE = ("You are Claude Code running in the terminal dock inside formfabric, the garment pattern and 3D drape app open on this computer. "
        "Work with the formfabric MCP server's tools: start with pattern_get for the panels and their outlines, and system_capabilities for what the engine can do. "
        "The tools act on the garment open in the window, the person sees every change happen, "
        "and each change is one undo step.")
TOKEN_ENV = "FFB_INAPP_TOKEN"
_cache: dict = {"at": 0.0, "value": None}


def child_env(path: str | None = None) -> dict:
    return agent_lookup.child_env(path, terminal.base_env(), keep=KEEP_ENV)


def _pref(key: str) -> str:
    value = prefs.load().get(key)
    return value.strip() if isinstance(value, str) else ""


def _runnable(path: str) -> bool:
    return agent_lookup.runnable(path)


def located(program: str = "claude", fresh: bool = False) -> agent_lookup.Found:
    setting = _pref("claudeBin") if program == "claude" else None
    found = agent_lookup.find(program, setting, (OVERRIDES[program],), env=terminal.base_env(), fresh=fresh)
    return agent_lookup.check(found, child_env(found.path) if found.path else None, fresh=fresh)


def cli() -> str | None:
    found = located("claude")
    return found.path if found.state == "runs" else None


def info(fresh: bool = False) -> dict:
    if not fresh and _cache["value"] is not None and time.time() - _cache["at"] < 60:
        return _cache["value"]
    found = located("claude", fresh)
    _cache["value"], _cache["at"] = agent_lookup.answer(found, agent_lookup.install("claude"), OTHERWISE), time.time()
    return _cache["value"]


def mcp_config() -> str:
    return json.dumps({"mcpServers": {"formfabric": {"type": "http", "url": config.engine_url() + "/api/mcp", "headers": {agents.INAPP_HEADER: "${" + TOKEN_ENV + "}"}}}})


def workdir() -> Path:
    raw = _pref("claudeFolder") or os.environ.get("FFB_AGENT_DIR", "").strip()
    folder = Path(raw).expanduser() if raw else config.data_dir() / "Agent"
    try:
        folder.mkdir(parents=True, exist_ok=True)
    except OSError as e:
        raise CommandError(f"Claude Code cannot start in {folder}: {e.strerror or e}; choose another folder in the Agents settings", 400) from None
    return folder


def argv(path: str, prompt: str | None, resume: str | None = None) -> list[str]:
    cmd = [path, "--mcp-config", mcp_config(), "--allowedTools", "mcp__formfabric", "--append-system-prompt", NOTE]
    if resume:
        cmd += ["--resume", resume]
    if prompt:
        cmd.append(prompt)
    if os.name == "nt":
        return cmd
    return terminal.login_shell() + ["-c", "exec " + shlex.join(cmd)]


@command("claudecode.status", "Claude Code",
         description="Whether the Claude Code CLI is installed (its path and version), the folder it starts in, and the terminal tab it runs in when open.")
def status() -> dict:
    tab = next((s.describe() for s in terminal.SESSIONS.values() if s.kind == "claude"), None)
    chosen = _pref("claudeBin")
    note = f"{chosen} is not a program formfabric can run, so it uses the one found on this computer" if chosen and not _runnable(chosen) else ""
    return info(fresh=True) | {"folder": str(workdir()), "terminal": tab, "binNote": note, "firstMessage": _pref("claudeFirstMessage")}


@command("claudecode.open", "Open Claude Code", {"prompt": {"type": "string", "description": "a first message to start the session with"},
                                                  "cols": {"type": "integer"}, "rows": {"type": "integer"}, "fresh": {"type": "boolean"},
                                                  "resume": {"type": "string", "description": "a Claude Code session id to continue, such as the Chat tab's"}},
         description="Starts the person's own Claude Code sign-in in a terminal tab of formfabric's dock, connected to formfabric's MCP tools without the outside-agent prompt. Reuses the running tab unless fresh is true or resume names a conversation to continue. The window only.")
def open_(prompt: str | None = None, cols: int | None = None, rows: int | None = None, fresh: bool | None = None, resume: str | None = None) -> dict:
    running = next((s for s in terminal.SESSIONS.values() if s.kind == "claude" and s.alive), None)
    if running is not None and not fresh and not resume:
        if prompt:
            terminal.write(running.id, prompt + "\r")
        return running.describe()
    found = info()
    if not found["installed"]:
        found = info(fresh=True)
    if not found["installed"]:
        raise CommandError(found.get("said") or INSTALL, 400)
    prompt = prompt or _pref("claudeFirstMessage") or None
    token = agents.issue()
    try:
        session = terminal.open_session(str(workdir()), cols or 100, rows or 30, "Claude Code (from Chat)" if resume else "Claude Code",
                                        argv(found["path"], prompt, resume), child_env(found["path"]) | {TOKEN_ENV: token}, "claude")
    except BaseException:
        agents.revoke(token)
        raise
    session.on_close.append(lambda: agents.revoke(token))
    return session.describe()


@command("agents.prefs", "Agent Settings", {"claudeFolder": {"type": "string"}, "claudeBin": {"type": "string"}, "claudeFirstMessage": {"type": "string"}, "agentsAllowed": {"type": "boolean"}},
         description="Reads, or changes when values are given, the folder Claude Code starts in, the Claude Code program to run, a first message for new sessions and whether outside agents may run commands. The window only.")
def agent_prefs(claudeFolder: str | None = None, claudeBin: str | None = None, claudeFirstMessage: str | None = None, agentsAllowed: bool | None = None) -> dict:
    changes = {k: v for k, v in {"claudeFolder": claudeFolder, "claudeBin": claudeBin, "claudeFirstMessage": claudeFirstMessage, "agentsAllowed": agentsAllowed}.items() if v is not None}
    current = prefs.update(changes) if changes else prefs.load()
    if agentsAllowed is False:
        agents.reset()
    _cache["value"] = None
    return {k: current[k] for k in (*prefs.DEFAULTS, *seal.REPORTS)}
