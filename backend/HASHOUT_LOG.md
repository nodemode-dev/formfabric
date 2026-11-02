# HASHOUT_LOG · backend

Living context for the FastAPI engine service. What would be comments in `backend/` lives here, newest first. Read before editing.

## v0.9.0 · 2026-11-01

- `core/window_guard.py`: engine dispatch, terminal and Claude Code commands, `/mcp/consent` and `/mcp/window` need a browser Origin exactly equal to a configured window origin (scheme, host and port; `FFB_FRONTEND_PORT` sets the dev one) or the in-app token. Read-only routes stay open. A local program can still forge an Origin; a per-launch window token from the desktop shell is the planned fix.
- The in-app token goes to Claude Code through the environment (`FFB_INAPP_TOKEN`), never argv; untested against a real Claude Code.
- `terminal.resize` clamps columns and rows to 1 to 1000.
- Test servers must set `FFB_FRONTEND_PORT` to the Vite port they use, or the window is refused.

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
- Window token (v0.10.0): the desktop shell makes a 32-byte token per launch and passes it as `FFB_WINDOW_TOKEN`; `./formfabric boot` generates one too. With a token set, `window_guard` accepts only the `x-ffb-window` header (constant-time compare), or, on the terminal WebSocket, the token as the first text frame within 5 s; without one it falls back to the exact-origin rule and logs a warning. Since v0.12.0 the service pops the token from its environment after reading it (children never inherit it), and a token under 32 characters fails closed. `/api/events`, `/api/events/recent` and `/api/recent` follow the window rule; print images come from `/api/files/image`. Every guarded route also requires an allowed Host (127.0.0.1 or localhost on a configured port) against DNS rebinding. `GET /api/mcp/window` is read-only; the window marks itself live with `POST /api/mcp/window`.
- Env names: `FFB_PORT`, `FFB_DIST`, `FFB_DESKTOP`, `FFB_CORS_ORIGINS`, `FFB_AUTOSAVE_SECONDS`, `FFB_PLUGIN_DIR`, `FFB_CLAUDE_BIN`, `FFB_AGENT_DIR`, `FFB_ACCESS_LOG`, `FFB_PARENT_PID`, `FFB_TERMINAL`, `FFB_ENGINE_URL`, `FFB_MCP_URL`, `FORMFABRIC_DATA_DIR`.

### Requirements

- Pins follow formsculpt: fastapi 0.141.1, uvicorn[standard] 0.53.0, python-multipart 0.0.32, maturin 1.11.5, ptyprocess 0.7.0, pywinpty 3.0.5 on Windows, pyinstaller 6.22.3 for the desktop freeze. pydantic and websockets come in transitively, unpinned, as in formsculpt.
- PyInstaller is GPL-2.0 with the bootloader exception: a build tool, never distributed as a library, the same arrangement as the rest of the family.

### v0.16.0 to v1.0.0

- Recovery:
  - `services/recovery.py` keeps a crash journal: the last autosave plus every mutating command since, recorded by the dispatch path in `services/engine.py` and cut at each autosave (`autosave.save_now(force)`).
  - `main.py` opens the journal at startup and closes it cleanly at shutdown.
  - `GET /api/recovery` reports a journal left by an unclean exit; `POST /api/recovery/restore` opens the autosave and replays the log; `discard` clears it.
- Plugins: `scripting.resolve_args` fills PARAMS defaults and validates (`bad_args`); `ScriptError` carries a code and the command; `Ffb.call` takes an optional dict of arguments; `try_call` and `commands` are added. An unavailable engine inside a plugin answers 503. `backend/plugins/drape_and_export.py` is the sample.
- The frozen engine honours `mcp` (stdio MCP) and `agents` as its first argument, because the packaged desktop program passes both. The freeze bundles `backend/plugins` and `.claude/skills`.
- `setup/version.py` and `VERSION` compare ten version sites and the changelog heading; `build_app --dry-run` runs it.
- Python API: `python/formfabric` generates one function per command from `catalogue.json` (refresh it with `./formfabric py -m formfabric snapshot` after command changes), with an in-process backend and an HTTP backend that sends `x-ffb-window` from the environment.
