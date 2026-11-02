---
name: agents
description: "Covers formfabric's agent and family surfaces: the Python API in python/formfabric (in-process and HTTP backends, generated commands module), Python plugins in backend/plugins (the PARAMS model, scripting.py, routers/plugins.py), the stdio MCP server and .mcp.json, the Live Link for Unreal through USD (link_cmds.rs), the GoF hand-off proposal and the formcompiler and formportal drafts in docs/integration. Load when scripting formfabric, writing a plugin, changing the MCP surface, the link commands or any file in python/ or docs/integration/."
---

# Agents and family integration

## Purpose

Keep the ways other programs drive formfabric (Python, plugins, MCP, the Live Link) in step with the command registry, and keep the family drafts data only.

## Procedure

1. Python API: `python/formfabric/` is pure standard library, Apache-2.0. `backend.py` holds the two backends (`InProcessBackend` over `ffb_engine`, `HttpBackend` over `/api/engine/dispatch` with the `x-ffb-window` header from `FFB_WINDOW_TOKEN`, or `x-formfabric-inapp` from `FFB_INAPP_TOKEN`). `commands.py` builds one function per command from `catalogue.json`; after any command change run `./formfabric py -m formfabric snapshot` against a current engine so the snapshot matches. `model.py` is the small hand API (Garment, Panel, Sim, Link); `gof.py` writes GoF-compatible bundles.
2. Plugins: a `.py` file in `backend/plugins`, `~/.formfabric/plugins` or `FFB_PLUGIN_DIR` with a top-level `run(ffb, args)` and an optional literal `PARAMS` dict (types number, integer, boolean, choice, text). `scripting.resolve_args` fills defaults and validates before `run`; `ffb.call(name, **args)` raises `ScriptError` with the engine's code. The freeze bundles `backend/plugins` as `plugins/`.
3. MCP: `./formfabric mcp` is the stdio server (`app/mcp_stdio.py`), `POST /api/mcp` the HTTP one; both list every catalogue command. The packaged program answers `mcp` and `agents` as its first argument and hands them to the frozen engine. `./formfabric agents sync` rewrites only the generated block of the modeling skill.
4. Live Link: `link.start{folder, every_frames, format: usd}`, `link.stop`, `link.state` in `native/crates/ffb-commands/src/link_cmds.rs`. It reuses the USD exporter through `output_cmds::link_scene`, follows `Session.sim.core` like the viz map follower and stops on any `file.` command. Document changes go in `docs/integration/unreal-live-link.md`, which stays marked untested against Unreal until someone runs it.
5. Family drafts: `docs/integration/` holds `apps-formfabric.json`, `catalogue-entry.json` and `product-entry.json` as data for the formcompiler and formportal owners, plus `gof-handoff.md`, a proposal. Write them from the contract descriptions only; never copy a formcompiler or formportal script, workflow or schema file, and never change formsculpt's `gof.json` unilaterally.

## Gate

`python3 -m py_compile` over `python/formfabric`, the family check (`/home/claude/ff/tools/family/check_family.py`) for both backends, `./formfabric agents sync --check`, and a JSON-RPC `tools/list` from `./formfabric mcp` whose tool count equals the catalogue.

## Hands back

The files changed, the check output and anything left untested (Unreal, the external builds).
