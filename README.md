# formfabric

**Garment pattern design and cloth simulation, in the NodeMode style**

![version](https://img.shields.io/badge/version-0.3.0-f97316) ![status](https://img.shields.io/badge/status-in%20development-f97316) ![license](https://img.shields.io/badge/license-Apache--2.0-blue)

## What it is

formfabric is the form family's garment app: draw sewing patterns in 2D, sew them, drape them on bodies with a GPU cloth simulation, animate them and export them. It aims at full feature parity with Marvelous Designer, built in the open under Apache-2.0.

Planned stack · Rust engine (`native/`, one Cargo workspace, wgpu compute solver) · vanilla TypeScript and Vite (`frontend/`) · Python and FastAPI (`backend/`) · Tauri 2 (`src-tauri/`). Desktop builds for macOS, Windows and Linux come from formcompiler; formportal lists and connects the app.

## Status

Phases 1 to 3 of 18 have landed: the family window shell, the Rust engine skeleton and the 2D pattern core. You can draw and edit panels with exact dimensions, undo every step, and save and reopen `.ffbrc` projects, from the window or through MCP. Sewing, simulation, bodies and export follow in later phases (see the parity matrix).

## Quick start

```bash
./formfabric_oc_install      # frontend packages, backend venv, engine module
./formfabric boot            # service on 127.0.0.1:8280, window on 127.0.0.1:5280
./formfabric desktop dev     # the desktop app with the splash
```

Requirements: Node 22, Python 3.12 or later, Rust (pinned by `rust-toolchain.toml`), and on Linux the Tauri 2 system packages.

| Service | Address | Notes |
| --- | --- | --- |
| Window (Vite) | http://127.0.0.1:5280 | bound to localhost only |
| Engine service | http://127.0.0.1:8280 | FastAPI, API under `/api`; the desktop app picks a free port |
| Frame stream | 127.0.0.1, OS-assigned port | binary WebSocket from the native engine, token required |

## Commands

| Command | Does |
| --- | --- |
| `./formfabric engine build` | builds the Rust engine and the `ffb_engine` Python module |
| `./formfabric engine wasm` | builds the in-page preview engine |
| `./formfabric mcp` | serves every engine command over MCP's stdio transport |
| `./formfabric agents sync` | regenerates the agent skill's tool reference from the engine |
| `./formfabric desktop build` | builds the installable app; forwards `--config FILE` to Tauri |
| `./formfabric help` | every verb |

## Layout

| Folder | Holds |
| --- | --- |
| `native/` | Rust workspace: `ffb-core`, `ffb-pattern`, `ffb-commands`, `ffb-io`, `ffb-stream`, `ffb-py`, `ffb-wasm` |
| `backend/` | FastAPI engine service, MCP servers, terminal and Claude Code services |
| `frontend/` | the window: shell, dock, workspace with Pattern View and 3D View |
| `src-tauri/` | Tauri 2 desktop shell |
| `setup/` | install, boot, build and freeze scripts |
| `docs/encyclopedia/` | the behaviour specification formfabric is built against |

## The encyclopedia

`docs/encyclopedia/` documents what Marvelous Designer does and how each feature behaves, from primary sources only, as the specification formfabric is built against.

| Start here | |
| --- | --- |
| [00-INDEX.md](docs/encyclopedia/00-INDEX.md) | chapter list and reading order |
| [14-PARITY-MATRIX.md](docs/encyclopedia/14-PARITY-MATRIX.md) | every feature mapped to a formfabric module, name and phase |
| [15-OPEN-GAPS.md](docs/encyclopedia/15-OPEN-GAPS.md) | what could not be verified and how formfabric decides it |

## Trademarks

Marvelous Designer and CLO are trademarks of CLO Virtual Fashion Inc., named here only to describe compatibility and the reference studied. formfabric is not affiliated with or endorsed by CLO Virtual Fashion.

## License

Apache-2.0, see `LICENSE`. Copyright 2026 NodeAgentc, Inc. Third-party notices in `NOTICE.md`.
