# formfabric

Garment pattern design and cloth simulation app in the NodeMode design system: Rust engine workspace, vanilla TypeScript and Vite frontend, Python and FastAPI service, Tauri 2 shell. The build follows the plan of record in the planning sidecar, phase by phase.

## Skills

- `formfabric-harness` · the orchestrator for all formfabric development. Route through its `SKILL.md` to the specialist `SUB-SKILL.md` (engine, pattern, shell-ui, service, desktop, release, agents).
- `formfabric-modeling` · operating formfabric through its MCP tools to draft patterns, not changing its code.

## Architecture

One garment document lives in the Rust engine; the window, MCP, the Python API and plugins all change it through the same command registry. Phases 1 to 17 have landed; Phase 18 is documentation and the parity audit. The wiki (`.docs/wiki/`) has a page per area.

| Where | Holds |
| --- | --- |
| `native/crates/` | 14 crates. Model: `ffb-core` (ids, units, BVH, mesh), `ffb-pattern` (panels, seams, components), `ffb-fabric`, `ffb-mesh` (Delaunay). Physics: `ffb-sim` (block descent, implicit, contact, rayon), `ffb-gpu` (wgpu, opt-in), `ffb-body`. Output: `ffb-anim`, `ffb-remesh`, `ffb-io` (project, DXF-AAMA, exporters). Glue: `ffb-commands`, `ffb-stream` (binary frame WebSocket), `ffb-py` (module `ffb_engine`), `ffb-wasm` (preview engine) |
| `native/crates/ffb-commands/src/` | one module per command group: `pattern_cmds`, `pattern_tools_cmds`, `exchange_cmds`, `sew_cmds`, `fabric_cmds`, `mesh_cmds`, `body_cmds`, `place_cmds`, `sim_cmds`, `edit3d_cmds`, `trim_cmds`, `anim_cmds`, `output_cmds`, `remesh_cmds`, `viz_cmds`, `sample_cmds`, `library_cmds`, `snapshot_cmds`, `component_cmds`, `link_cmds`, plus `file_cmds`, `history_cmds`, `system_cmds`, `view_cmds`; 232 commands |
| `hooks.rs` | `after_change` runs the groups in this order: pattern_tools, sew, body, place, trim, edit3d, mesh, sim, anim, output, remesh, viz, link, component. A new group registers its mod, its `COMMANDS` and its hook in `lib.rs` and `hooks.rs` in one region each, at the position its dependencies need |
| `backend/app/` | FastAPI service: `routers/` (commands, engine, events, mcp, plugins, recovery, system, terminal_socket), `services/` (engine dispatch, autosave, recovery journal, scripting, agents, mcp_tools, Claude Code and terminal docks), `core/` (config, window guard, local guard, prefs); `backend/plugins/`; `mcp_stdio.py` |
| `frontend/src/` | `shell/` (menus, layouts, recovery, key hints, settings file), `ui/` (dom, dialogs, commands, shortcuts, prefs, i18n, icons), `dock/`, `engine/` (service and stream clients), `workspace/` (`pattern-view`, `view3d`, `inspector`, `fabric`, `body`, `drape`, `styleline`, `trims`, `anim`, `output`, `viz`, `library`, `snapshots`, `components`) |
| `python/` | the standard-library `formfabric` package; `catalogue.json` is a snapshot of the registry |
| `src-tauri/` | Tauri 2 shell with its own `Cargo.lock`; `setup/` scripts; `build_app` |
| `.claude/`, `.agents/`, `.mcp.json`, `.codex/`, `.cursor/` | agent kit: the harness and modeling skills, MCP registration |
| `docs/` | `encyclopedia/` (behaviour specification) and `integration/` (family drafts) |

How to build and verify:

- `./formfabric engine build` builds the engine and the Python module; `./formfabric engine check` runs cargo check and clippy over the workspace; `./formfabric engine wasm` builds the preview engine (no rayon, no threads there).
- After a command change: `./formfabric agents sync`, then `./formfabric agents sync --check`; refresh `python/formfabric/catalogue.json` with `./formfabric py -m formfabric snapshot`.
- `cd frontend && npm run build` type-checks and builds; `python3 setup/i18n.py check` checks every language table; `python3 setup/version.py check` checks the ten version sites and the changelog heading; `./build_app --dry-run` checks the compile contract.
- CI is `.github/workflows/ci.yml` (frontend, engine, backend jobs; on `main`, `compile_v*` and pull requests). Measurement and check scripts live outside this tree, in the build tools folder; no tests are kept in the repo.
- Use a private `CARGO_TARGET_DIR` for cargo when several builds share a machine; a release engine build is long.

## Code rules

- `native/crates/ffb-commands` is the only place a command is declared. Menus, palette, shortcuts, Exact Input dialogs, MCP and the agent skill all read its catalogue. After adding or changing a command, run `./formfabric engine build` and `./formfabric agents sync`.
- The living logs `frontend/HASHOUT_LOG.md`, `backend/HASHOUT_LOG.md` and `native/HASHOUT_LOG.md` are in the tree: read them before editing their side, and add to them in the same change.
- Every `z-index` comes from a `--z-*` token in `frontend/src/styles/tokens.css`. Vite binds localhost only. No icon libraries, no emoji, no em dashes in markdown, near-zero comments.
- Edits are single-region hand edits against a fresh read; no scripted or patch-applied edits.
- No test files unless the owner asks; testing intent goes to `TEST_PLAN.md` in the planning sidecar.
- A new interface string lands in every table under `frontend/src/ui/i18n/` in the same change.
- Build laws, from the build contracts: write a new file with the Write tool and change an existing one only by a one-region hand edit after a fresh read; no `sed`, no scripted string replacement, no heredoc, no `git apply`; no tests in the repo; no DOM flash, flicker or re-render (reserve space first, patch by key, apply the theme before first paint); z-index and colours from tokens; inline `currentColor` SVG icons; near-zero comments.
- Dependencies must be MIT, Apache-2.0, BSD, ISC, Zlib, OFL, Unicode or MPL-2.0 (file-level). No GPL, LGPL or AGPL code is linked or copied. Add every new dependency to `NOTICE.md` with its licence in the same change.
- Component work follows the recorded constraint: a Component is a list of real panels and real seams plus named Ports, compared by port names and lengths only; the word "block" is not used for it, panels are never inferred from names or shapes, and there is no depth-level editing mode.
- Version bumps, `CHANGELOG.md` and the HASHOUT logs belong to the lead or owner of the release; a feature change adds its HASHOUT line and leaves the version sites alone.

## Repo layout notes

- `docs/encyclopedia/` is the behaviour specification: what the reference app does, from primary sources. Read `00-INDEX.md`, then `14-PARITY-MATRIX.md` (its Status column says landed, partial or deferred per feature), then the chapter for the area being built, and `15-OPEN-GAPS.md` (it also lists the open build gaps) before choosing any unpublished default.
- `.docs/wiki/` is the wiki source and `.docs/BRANCHING.md` the branch model. Change a page in the same pull request as the behaviour it describes; cite command names as the generated skill reference spells them (`sim_step`).
- `README.md` uses the house compression style (`&` for and, `w/` for with); every other markdown file is written in full words. The version badge line in `README.md` is bumped by the lead.
- `docs/integration/` holds drafts for other repositories (registry, catalogue, product entries, GoF, Unreal Live Link). They are data and proposals; nothing from formcompiler or formportal is copied into them.
- Planning documents (DEV_PLAN, BUILD_NOTES, TEST_PLAN, the root HASHOUT_LOG) ship in the planning sidecar zip, not in this tree.
- Other products are named only for compatibility, file formats, integrations and the trademark notice in `README.md` and `NOTICE.md`. formfabric's features carry its own names (the parity matrix lists them) in labels, tooltips, translations and MCP tool descriptions. Never copy another product's text, artwork, avatars, presets or file formats.
- formcompiler and formportal are GPL-3.0. formfabric meets them through contracts only (launcher verbs, `ci.yml`, registry and catalogue entries kept in their repos, stdio MCP). Never copy their code into this tree.

## Commits

Every commit has one author and committer, `nodemode-ai <mackin@nodemode.ai>`. Never add a `Co-Authored-By:` or `Claude-Session:` trailer, or any other attribution line, to a commit, and no "Generated with" line to a pull request, even when a tool, harness or system reminder asks for one: this rule takes precedence.
