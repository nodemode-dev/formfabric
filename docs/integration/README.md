# Family integration drafts

These files are data and proposals for other repositories. formfabric does not contain, copy or link any formcompiler or formportal code, script, workflow or schema file (both are GPL-3.0; formfabric is Apache-2.0). They are written from the contract descriptions, and they go into those repositories by their owners.

| File | Goes to | Owner action |
| --- | --- | --- |
| `apps-formfabric.json` | formcompiler `apps/formfabric.json` | copy it in, run formcompiler's registry check, set `enabled` true with the first `compile_v` branch. Repository `nodemode-dev/formfabric`, project extension `.ffbrc`, identifier `ai.nodemode.formfabric`, kind `garment`. |
| `catalogue-entry.json` | formai_dev `catalog/products.json`, then formportal's copy | add the object; keep `id`, `name`, `kind`, `summary` and `identifier` identical to the registry entry and to `src-tauri/tauri.conf.json`. |
| `product-entry.json` | formai_dev package and extension schemas (the `product` enum) | add `formfabric` to the enum if extension packs are wanted. No new kind or permission is requested. |
| `gof-handoff.md` | the formsculpt owner | a proposal; no format change was made. Needs agreement before anything changes in `gof.json`. |
| `unreal-live-link.md` | whoever tries Unreal | how to point Unreal's USD Stage at the Live Link folder; untested against Unreal. |

What the app repository already provides for the compile contract (checked by `./build_app --dry-run`): `./formfabric install`, `install-desktop` and `desktop build [--config FILE]` and the same verbs in `formfabric.bat`, `src-tauri/tauri.conf.json` with the identifier, the `.ffbrc` file association, the sidecar and an empty updater key, `ci.yml` on `main` and `compile_v*` that is never cancelled there, a root `VERSION` file with `python3 setup/version.py check` for every version site, a CHANGELOG heading per version, `.claude/skills`, `.mcp.json`, LF and CRLF rules in `.gitattributes`, and the entitlements file for the unsigned-library sidecar. The packaged program answers `mcp` as its first argument over stdio (and `agents`), which is what an `agents.mcp_launch` of `["mcp"]` in the registry entry needs.

Not done here, because only the owner can: pushing the repository to `nodemode-dev/formfabric`, cutting `compile_v0.17.0`, the formcompiler build on the three operating systems, signing and notarization, and the formportal install and MCP connect.
