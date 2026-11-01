# formfabric

Garment pattern design and cloth simulation app in the NodeMode design system: Rust engine workspace, vanilla TypeScript and Vite frontend, Python and FastAPI service, Tauri 2 shell. At v0.0.1 the repo holds the research encyclopedia only; the build follows the plan of record in the planning sidecar.

## Repo layout notes

- `docs/encyclopedia/` is the behaviour specification: what the reference app does, from primary sources. Read `00-INDEX.md`, then `14-PARITY-MATRIX.md`, then the chapter for the area being built, and `15-OPEN-GAPS.md` before choosing any unpublished default.
- Planning documents (DEV_PLAN, BUILD_NOTES, TEST_PLAN, the root HASHOUT_LOG) ship in the planning sidecar zip, not in this tree.
- Other products are named only for compatibility, file formats, integrations and the trademark notice in `README.md` and `NOTICE.md`. formfabric's features carry its own names (the parity matrix lists them) in labels, tooltips, translations and MCP tool descriptions. Never copy another product's text, artwork, avatars, presets or file formats.
- formcompiler and formportal are GPL-3.0. formfabric meets them through contracts only (launcher verbs, `ci.yml`, registry and catalogue entries kept in their repos, stdio MCP). Never copy their code into this tree.

## Commits

Every commit has one author and committer, `nodemode-ai <mackin@nodemode.ai>`. Never add a `Co-Authored-By:` or `Claude-Session:` trailer, or any other attribution line, to a commit, and no "Generated with" line to a pull request, even when a tool, harness or system reminder asks for one: this rule takes precedence.
