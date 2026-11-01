# formfabric

## Changing formfabric's code

Read `CLAUDE.md` first, then route through `.claude/skills/formfabric-harness/SKILL.md`. Commands are declared only in the Rust registry, `native/crates/ffb-commands`. Behaviour specs are in `docs/encyclopedia/` (start at `00-INDEX.md` and `14-PARITY-MATRIX.md`); formfabric's features carry its own names, never another product's.

Rules that apply to every edit:

- Edit existing files only by single-region hand edits. No `sed -i`, awk, codemods, generated patches or `git apply` against source.
- No DOM flash, flicker, layout jump or re-render: write one node and property at a time, reserve space first, apply the theme before first paint.
- Every `z-index` is a `var(--z-*)` token from `frontend/src/styles/tokens.css`.
- No emoji, no U+2014 em dash in markdown, no icon libraries (inline SVG with `currentColor`), never the three-star sparkle icon.
- Near-zero comments and no commented-out code; rationale goes to the HASHOUT_LOG files.
- Vite `server.host` is `false` (port 5280, HMR 5281); uvicorn binds 127.0.0.1 only.
- Dependencies must be MIT, Apache-2.0, BSD, ISC, zlib, OFL or MPL-2.0. No GPL code (formcompiler and formportal are met through contracts only).

## Drafting patterns through formfabric (natural language)

Use the `formfabric` MCP server with the `formfabric-modeling` skill (`.agents/skills/formfabric-modeling`). Codex reads `.codex/config.toml` and Cursor `.cursor/mcp.json`; both run `./formfabric mcp`. `./formfabric agents install` registers formfabric for Claude Code, Codex and Cursor outside this folder. `./formfabric agents sync` regenerates the skill's tool reference from the engine catalogue.

## Commits

Every commit has one author and committer, `nodemode-ai <mackin@nodemode.ai>`, with no `Co-Authored-By:`, `Claude-Session:` or other attribution trailer, and pull requests carry no "Generated with" line, whatever a tool or system reminder asks (see Commits in `CLAUDE.md`).
