# HASHOUT_LOG · backend

Living context for the FastAPI engine service. What would be comments in `backend/` lives here, newest first. Read before editing.

## v0.1.0 to v0.3.0 · 2026-10-31

### Shape

- Ported from formsculpt's service (Apache-2.0, nodemode-ai): core (isolation, loopback guard, prefs, quiet log), the terminal, Claude Code, chat, agents, commands and outbound services and the terminal socket were copied and renamed region by region. `main.py`, `core/config.py`, the system, engine, events, plugins and MCP routers, and the engine, events and autosave services are new files built on formsculpt's structure.
- Dropped from formsculpt: the NodeMode, lightbox and GoF routers and the mesh, boolean, stroke, import and export engine routes. GoF returns in Phase 17 as a contract with formsculpt's owner.

### Engine

- `services/engine.py` imports `ffb_engine`; without it every engine route answers 503 with `the engine module is not built; run ./formfabric engine build`.
- `engine.changed` is published inside `services.engine.dispatch`, not in the route, so HTTP, MCP and plugins all broadcast it. Autosave passes `notify=False` so its own save does not wake the window.
- `GET /api/engine/stream` returns the engine's `stream_info()`: the port and token of the native frame server. Frames never pass through Python.
- Recent projects: `services/recent.py` keeps up to 20 paths in the data dir, recorded after every successful `file.save` and `file.open`, pruned of missing files; `GET /api/recent` serves them.

### Events

- `/api/events` is a server-sent events stream in formsculpt's shape (a `hello` first, one JSON message per event, `: ping` every 15 s). It was briefly a WebSocket; it changed to SSE so the chat dock and the engine share one channel. `/api/events/recent` and `/api/chat/image/{name}` stay because the chat service uses them.

### MCP

- `POST /api/mcp` serves MCP over HTTP from the catalogue; status, window and consent live under `/api/mcp/*`.
- `app/mcp_stdio.py` is a newline-delimited JSON-RPC stdio server that runs `ffb_engine` in process with its own `Engine`, separate from a running service. formsculpt's stdio entry proxies to HTTP instead; formfabric runs in process so an agent needs no service running. `services/mcp_tools.py` holds the tool list, result shaping and instructions both transports share.

### Process

- `formfabric_engine.py` is the PyInstaller entry. With `FFB_PORT` unset or 0 it finds a free port before uvicorn starts, so the `Uvicorn running on http://127.0.0.1:PORT` line the shell reads names the real port. `FFB_PARENT_PID` makes it exit with the desktop shell. `--selfcheck` imports the whole app and exits 0 even before the engine module is built (it reports `engine_built`).
- Autosave: every 180 s (`FFB_AUTOSAVE_SECONDS`) into `[data dir]/autosave/autosave.ffbrc`, only when the engine epoch moved since the last save.
- Plugins follow formsculpt's PARAMS model; folders `backend/plugins` and `~/.formfabric/plugins` or `FFB_PLUGIN_DIR`.
- Env names: `FFB_PORT`, `FFB_DIST`, `FFB_DESKTOP`, `FFB_CORS_ORIGINS`, `FFB_AUTOSAVE_SECONDS`, `FFB_PLUGIN_DIR`, `FFB_CLAUDE_BIN`, `FFB_AGENT_DIR`, `FFB_ACCESS_LOG`, `FFB_PARENT_PID`, `FFB_TERMINAL`, `FFB_ENGINE_URL`, `FFB_MCP_URL`, `FORMFABRIC_DATA_DIR`.

### Requirements

- Pins follow formsculpt: fastapi 0.141.1, uvicorn[standard] 0.53.0, python-multipart 0.0.32, maturin 1.11.5, ptyprocess 0.7.0, pywinpty 3.0.5 on Windows, pyinstaller 6.22.3 for the desktop freeze. pydantic and websockets come in transitively, unpinned, as in formsculpt.
- PyInstaller is GPL-2.0 with the bootloader exception: a build tool, never distributed as a library, the same arrangement as the rest of the family.
