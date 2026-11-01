---
name: service
description: "Covers the formfabric Python service in backend/app: FastAPI routes under /api, health, version, capabilities, engine catalogue and dispatch, the /api/events socket, the MCP router and stdio shim, terminal and chat sockets, the loopback guard and the PyInstaller sidecar entry. Load when editing backend/ or the MCP surface."
---

# Service

## Purpose

Own `backend/`. FastAPI bound to 127.0.0.1 only, port 8280 (0 in the desktop app, read from stdout). Hands back routes that match the contract.

## Procedure

1. Routes: `GET /api/health`, `/api/version`, `/api/capabilities`, `/api/engine/catalogue`, `POST /api/engine/dispatch`, `GET /api/engine/stream`, WebSocket `/api/events` (`engine.changed` with epoch and name after every non-read-only dispatch).
2. MCP: `POST /api/mcp` and `./formfabric mcp` stdio, both generated from the catalogue; no hand-written tool list.
3. The loopback guard applies to every route. When `ffb_engine` is missing, engine routes answer 503 with `the engine module is not built; run ./formfabric engine build`.
4. The sidecar prints the uvicorn `Uvicorn running on http://127.0.0.1:PORT` line; `--selfcheck` imports the app and exits 0.
5. Env: `FFB_PORT`, `FORMFABRIC_DATA_DIR`. Requirements files carry only permissively licensed packages.

## Gate

Selfcheck exits 0, catalogue route matches the engine, a dispatch emits an event.

## Hands back

The edit and the selfcheck output.
