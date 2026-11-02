# formfabric

## Changing formfabric's code

Read `CLAUDE.md` first, then route through `.claude/skills/formfabric-harness/SKILL.md`. Commands are declared only in the Rust registry, `native/crates/ffb-commands`. Behaviour specs are in the planning sidecar's `encyclopedia/` folder, outside this tree (start at `00-INDEX.md` and `14-PARITY-MATRIX.md`); formfabric's features carry its own names, never another product's.

Rules that apply to every edit:

- Edit existing files only by single-region hand edits. No `sed -i`, awk, codemods, generated patches or `git apply` against source.
- No DOM flash, flicker, layout jump or re-render: write one node and property at a time, reserve space first, apply the theme before first paint.
- Every `z-index` is a `var(--z-*)` token from `frontend/src/styles/tokens.css`.
- No emoji, no U+2014 em dash in markdown, no icon libraries (inline SVG with `currentColor`), never the three-star sparkle icon.
- Near-zero comments and no commented-out code; rationale goes to the HASHOUT_LOG files.
- Vite `server.host` is `false` (port 5280, HMR 5281); uvicorn binds 127.0.0.1 only.
- Dependencies must be MIT, Apache-2.0, BSD, ISC, Zlib, OFL, Unicode or MPL-2.0 (file-level). No GPL, LGPL or AGPL code (formcompiler and formportal are met through contracts only). List each new dependency with its licence in `NOTICE.md`.
- Create new files with the Write tool; no heredoc, no scripted string replacement, no tests kept in the repo.
- After a command change run `./formfabric engine build`, then `./formfabric agents sync`; after interface strings run `python3 setup/i18n.py check`; `./build_app --dry-run` checks the compile contract and `python3 setup/version.py check` the version sites.
- Version bumps, `CHANGELOG.md` and the HASHOUT logs are the lead's; add your own HASHOUT line for a feature change.

## Repo map

| Where | Holds |
| --- | --- |
| `native/crates/` | 14 Rust crates (core, pattern, fabric, mesh, sim, gpu, body, anim, remesh, io, commands, stream, py, wasm); `ffb-commands/src/*_cmds` has one module per command group (including `scene_cmds`), 255 commands, with `hooks.rs` running each group's `after_change` in a fixed order |
| `backend/app/` | FastAPI service: routers, engine dispatch, autosave and recovery journal, plugins, MCP, docks; `backend/plugins/` |
| `frontend/src/` | `shell/`, `ui/`, `dock/`, `engine/` and `workspace/` (pattern-view, view3d, fabric, body, drape, trims, anim, output, viz, library, objects, snapshots, components) |
| `python/` | the `formfabric` Python package (standard library only) |
| `src-tauri/`, `setup/`, `build_app` | desktop shell, scripts, the start-to-finish app build |
| `docs/`, `.docs/wiki/` | behaviour specification, family integration drafts, the wiki source |

`CLAUDE.md` has the full map, the hook order and how to build and verify.

## Drafting patterns through formfabric (natural language)

Use the `formfabric` MCP server with the `formfabric-modeling` skill (`.agents/skills/formfabric-modeling`). Codex reads `.codex/config.toml` and Cursor `.cursor/mcp.json`; both run `./formfabric mcp`. `./formfabric agents install` registers formfabric for Claude Code, Codex and Cursor outside this folder. `./formfabric agents sync` regenerates the skill's tool reference from the engine catalogue.

## Commits

Every commit has one author and committer, `nodemode-ai <mackin@nodemode.ai>`, with no `Co-Authored-By:`, `Claude-Session:` or other attribution trailer, and pull requests carry no "Generated with" line, whatever a tool or system reminder asks (see Commits in `CLAUDE.md`).
