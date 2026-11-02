# Agents and MCP

An agent drafts and drapes garments through the same 232 commands the window uses. There is no separate agent API.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/` | the registry; the catalogue binary is `src/bin/catalogue.rs` (`ffb-catalogue`) |
| `backend/app/mcp_stdio.py` | the stdio MCP server started by `./formfabric mcp` |
| `backend/app/services/mcp_tools.py`, `routers/mcp.py` | the tool list and the HTTP MCP endpoint `/api/mcp` with its consent flow |
| `backend/app/services/agents.py` | outside-agent consent state |
| `backend/app/services/claude_code.py`, `cmd_chat.py`, `cmd_terminal.py` | the Claude Code and terminal docks in the window |
| `setup/agents.py` | `agents sync` and `agents install` |
| `.mcp.json`, `.codex/config.toml`, `.cursor/mcp.json` | MCP registration for Claude Code, Codex and Cursor |
| `.claude/skills/`, `.agents/skills/` | the skills; `.agents` mirrors `.claude` for Codex |

## Connecting

- Inside this folder the three config files above start `./formfabric mcp` (stdio). `./formfabric agents install` registers formfabric for Claude Code, Codex and Cursor outside the folder; a packaged app sets `FORMFABRIC_LAUNCHER` so the registration points at the app.
- Over HTTP, `/api/mcp` serves the same tools. The first change asks the person in the window for consent once. The window routes need the window token; a Claude Code started inside the window gets an in-app token automatically.
- In the packaged app the frozen engine answers `formfabric-engine mcp` (stdio) and `formfabric-engine agents`, which formportal's connect relies on.

## Names

MCP tools are the command ids with dots as underscores: `sample.load` is `sample_load`, `sim.step` is `sim_step`. Errors come back as `{ok: false, error: {code, message}}` over HTTP and as tool errors over MCP. Common codes: `bad_args` (schema failed), `unknown_command`, `unsaved_changes` (`sample_load` without `force`), `unsupported`, `outside_library`, `port_mismatch`. A 503 or "engine module is not built" means run `./formfabric engine build`.

## The skills

| Skill | Use |
| --- | --- |
| `formfabric-modeling` | operating formfabric through its tools; ends with the generated tool reference between `[ffb-tools:begin]` and `[ffb-tools:end]` |
| `formfabric-harness` | changing formfabric's code; routes to sub-skills `engine`, `pattern`, `shell-ui`, `service`, `desktop`, `release`, `agents` |

After any command change run `./formfabric engine build`, then `./formfabric agents sync` (and `agents sync --check` to confirm). The reference is generated; never edit between the markers.

## A typical agent session

1. `system_capabilities` and `pattern_get` to see the build and the document.
2. `sample_load` (name `tshirt`, ask the person before `force`), or draft with `pattern_draw_panel` and sew with `sew_segment`.
3. `sim_reset`, `sim_step` (frames 60 to 120), `sim_state`, then `map_show` or `map_stats` to inspect.
4. `export_run` to an absolute path. Everything is undoable with `history_undo`; use `history_list` to find the last good step.

## Live Link to Unreal

`link_start` (absolute `folder`, `every_frames`, `format` usd) writes `garment.usda` into the folder at once and replaces it atomically every N simulation frames, on document changes, on a reset and 250 ms after the frame stops moving. `link_state` shows writes and the last error; `link_stop` ends it, and so does any `file_` command. Point Unreal's USD Stage at the file; the recipe is `docs/integration/unreal-live-link.md` and has not been run against Unreal. The whole layer is rewritten each time.

## GoF hand-off

`formfabric.gof.send` (Python only) writes a GLB or FBX bundle and merges an entry into `gof.json` for formsculpt and formsurface receivers. It needs no format change; extra fields are proposals in `docs/integration/gof-handoff.md`.

## Contract

Declare commands only in `ffb-commands` with a description, ranges and a note on whether the call is safe to repeat; the schema, the palette label, the MCP tool and the skill reference all come from it. Never hand an agent a token the window did not grant.
