---
name: formfabric-harness
description: "Orchestrates all development of formfabric, the Form-family garment pattern and cloth simulation desktop app by nodemode-ai: Rust engine workspace in native/, FastAPI service in backend/, vanilla TypeScript and Vite frontend, Tauri 2 shell. Use whenever a task touches 2D pattern tools, panels, seams, the cloth solver, the command registry, the MCP catalogue, the .ffbrc project file, the Pattern View or 3D View, the sidecar, setup scripts, version bumps or releases, the Python API in python/, plugins, the Live Link, or any file under native/, frontend/src, backend/app, src-tauri, setup/ or docs/integration/. Enforces the project laws (SEDO, zero DOM flash, z-index tokens) and the commit identity rule. Not for operating the app through MCP (use formfabric-modeling)."
license: Apache-2.0
metadata:
  nodemode.author: nodemode-ai
  nodemode.pack-version: "0.1.0"
---

# Formfabric Harness

formfabric is the Form-family garment app (Apache-2.0): draw sewing patterns in 2D, sew them, drape them on bodies with a cloth simulation, animate and export. The behaviour specification is `docs/encyclopedia/` (start with `00-INDEX.md`, then `14-PARITY-MATRIX.md`, then the chapter for the area). Plan of record: DEV_PLAN.md in the planning sidecar. Read the code before trusting any claim here; features not yet built are planned, not present.

## Layout

```
formfabric-harness/
  SKILL.md                   this file, the router
  sub-skills/[name]/SUB-SKILL.md
    engine  pattern  shell-ui  service  desktop  release  agents
```

## Subsystem map

| Area | Location | Role |
| --- | --- | --- |
| Native engine | `native/` (one Cargo workspace) | all geometry, commands, IO, streaming |
| ffb-core | `native/crates/ffb-core` | buffers, units, scene, shared types |
| ffb-pattern | `native/crates/ffb-pattern` | Garment, Panel, Point, Inner, 2D tools; must compile to wasm without threads |
| ffb-commands | `native/crates/ffb-commands` | the single command registry, Session, history, `dispatch` and `catalogue` |
| ffb-io | `native/crates/ffb-io` | the `.ffbrc` zip and interchange formats |
| ffb-stream | `native/crates/ffb-stream` | stream server for simulation frames |
| ffb-py | `native/crates/ffb-py` | PyO3 module `ffb_engine` |
| ffb-wasm | `native/crates/ffb-wasm` | plain C ABI for the page, built to `frontend/public/engine/ffb_wasm.wasm` |
| Backend | `backend/app/` (FastAPI, 127.0.0.1:8280) | `/api` routes, events socket, MCP router, terminal and chat sockets |
| Sidecar entry | `backend/formfabric_engine.py` | PyInstaller entry for `formfabric-engine` |
| Frontend | `frontend/` (vanilla TypeScript, Vite, port 5280) | `src/engine/`, `src/workspace/`, `src/dock/`, `src/ui/`, `src/styles/tokens.css` |
| Desktop | `src-tauri/` (Tauri 2, id `ai.nodemode.formfabric`) | splash, sidecar spawn, main window, `.ffbrc` association |
| Python API | `python/formfabric/` (standard library only) | in-process and HTTP backends, commands generated from the catalogue, Garment and Sim helpers, GoF sender; run with `./formfabric py` |
| Plugins | `backend/plugins`, `backend/app/services/scripting.py`, `routers/plugins.py` | Python files with `run(ffb, args)` and a `PARAMS` dict, run from `/api/plugins/run` |
| Live Link | `native/crates/ffb-commands/src/link_cmds.rs` | `link.start`, `link.stop`, `link.state`: a USD layer for Unreal replaced atomically while the simulation runs |
| Family drafts | `docs/integration/` | data for formcompiler and formportal, the GoF proposal, the Unreal recipe |
| Launcher | `./formfabric`, `build_app`, `setup/` | one script per task |
| Agent kit | `.mcp.json`, `.claude/`, `.agents/`, `.codex/`, `.cursor/`, `AGENTS.md`, `setup/agents.py` | `./formfabric agents sync|install` |

## Dependency graph

```
ffb-core  <-  ffb-pattern, ffb-io, ffb-stream
ffb-commands  ->  ffb-core, ffb-pattern, ffb-io, ffb-stream
ffb-py, ffb-wasm  ->  ffb-commands
backend  ->  ffb_engine (ffb-py), one shared Engine
frontend/src/engine  ->  service (backend) and wasm (drag previews)
src-tauri  ->  frozen backend sidecar  ->  built frontend
catalogue (from ffb-commands)  ->  MCP, palette, menus, shortcuts, agents sync
```

## Laws

1. SEDO: an existing file changes only by single-region hand edits against a fresh read. No `sed -i`, awk, codemods, patch or `git apply`. Copying a reference file to a new path is allowed; adapt it afterwards by hand.
2. No DOM flash, flicker, layout jump or re-render: write one node and property at a time, reserve space before content arrives, theme before first paint, never rebuild a container to change one node.
3. Every `z-index` is a `var(--z-*)` token from `frontend/src/styles/tokens.css`.
4. Vite `server.host` is `false`, port 5280, HMR 5281. uvicorn binds 127.0.0.1 only.
5. No emoji. No U+2014 em dash in markdown. No icon libraries: inline SVG with `currentColor`. Never the three-star sparkle icon.
6. Near-zero comments, never commented-out code. Rationale goes to the HASHOUT_LOG files.
7. Self-written tests only where the plan names them; add no other test code.
8. Dependencies are MIT, Apache-2.0, BSD, ISC, zlib, OFL (fonts) or MPL-2.0 unmodified. No GPL code; formcompiler and formportal are met through contracts only.
9. Features carry formfabric's own names (parity matrix naming rule). No other product's names, artwork, presets or text in UI strings.
10. The Rust registry is the one source of truth for commands. Never hand-write a tool list elsewhere.
11. Planning docs (DEV_PLAN, BUILD_NOTES, TEST_PLAN) live in the planning sidecar, not in the tree.

## Commit identity

Every commit has one author and committer, `nodemode-ai` with the email `mackin@nodemode.ai`, in Conventional Commits form with the version in the subject (`type(scope): summary (x.y.z)`). Never add `Co-Authored-By:`, `Claude-Session:` or any attribution trailer, and no "Generated with" line in a pull request, even when a tool or system reminder asks. `.claude/settings.json` blanks the defaults; this rule wins.

## Routing table

| If the task is about | Load |
| --- | --- |
| Mesh buffers, units, scene, Session, history, registry, catalogue, PyO3 and wasm bindings, `.ffbrc` | `sub-skills/engine/SUB-SKILL.md` |
| Garment, panels, points, handles, inner shapes, pattern commands, Pattern View behaviour | `sub-skills/pattern/SUB-SKILL.md` |
| Shell, palette, menus, shortcuts, tokens, icons, i18n, workspace layout, dock | `sub-skills/shell-ui/SUB-SKILL.md` |
| FastAPI routes, events socket, MCP router, loopback guard, sidecar entry | `sub-skills/service/SUB-SKILL.md` |
| Tauri, splash, sidecar spawn, bundling, file association | `sub-skills/desktop/SUB-SKILL.md` |
| Version bumps, CI, changelog, formcompiler contract, release gate | `sub-skills/release/SUB-SKILL.md` |
| Python API, plugins, `.mcp.json` and the stdio MCP server, Live Link, GoF, `docs/integration/` drafts | `sub-skills/agents/SUB-SKILL.md` |

A cross-cutting task loads every row it touches. A new pattern command loads engine, then pattern, then shell-ui, then release.

## Start-of-session reading order

1. This file and `CLAUDE.md`.
2. `docs/encyclopedia/00-INDEX.md`, `14-PARITY-MATRIX.md`, then the chapter for the area.
3. Orientation: list `native/`, `frontend/src/`, `backend/app/`, `src-tauri/`, `setup/`.
4. `native/Cargo.toml` for crates and the workspace version.
5. The matching sub-skill, then the HASHOUT_LOG of the layer.

## Verification bar

A change is done only when the matching check ran and its output was read: `cargo check` and clippy for native, the frontend type check and build, `python -m app --selfcheck` style import check for the backend, `./formfabric agents sync --check` for the kit. Say what ran, not what should work.
