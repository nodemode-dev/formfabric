# Troubleshooting

Start with `./formfabric doctor`: it checks the toolchain, the installs and the ports.

| Symptom | Cause | Fix |
| --- | --- | --- |
| 503, "engine module is not built" | the `ffb_engine` Python module is missing or stale | `./formfabric engine build`, then restart the service |
| A new command is missing in menus, MCP or the skill | the engine was not rebuilt or the reference not synced | `./formfabric engine build`, then `./formfabric agents sync` |
| Window opens but nothing draws; 401 or 403 on `/api/` | the window token does not match the service | start both through `./formfabric boot`; set `FFB_WINDOW_TOKEN` (32 or more characters) for both when starting by hand |
| Port already in use | 8280 or 5280 taken | stop the other process or set `FFB_PORT` and `FFB_FRONTEND_PORT`; the Vite HMR port is the frontend port plus one |
| `sample_load` fails with `unsaved_changes` | the open project has edits | save, or pass `force` true after asking the person |
| The drape does nothing | no placement, or cloth not built | `sim_reset` first; place panels with `place_panel` or `arrange_place` when the garment is hand-drafted |
| Cloth crumples or seams stay open | seam lengths differ, or the wrong method | `sew_check`; leave `method` on `auto` (block descent for sewn garments); raise `close_frames` |
| Cloth passes through itself in motion | self contact limits | known open gap; use the Quality preset on short runs and read the cache validation numbers |
| Drape is slow | CPU speed above a few thousand vertices | raise `particle distance` with `pattern_set_props`, use the Fast preset, or try `device` gpu if the build has the gpu feature |
| `device` gpu reports unavailable | no adapter or no gpu feature | the simulation stays on the CPU; this is expected on machines without a GPU |
| `bad_args` | arguments failed the schema | read the parameter list in the skill reference and resend |
| `outside_library` | a path outside the library roots | use `library_roots` to see the roots, or `library_set_roots` |
| `port_mismatch` on `component_swap` | a joined port is missing or out of tolerance | `component_check`, then fix the port or pass `ease` |
| Restore dialog on start | the last session ended without a clean shutdown | Restore or Discard, see [[Recovery]] |
| Python script cannot reach the service | token or host | see [[Python-API-and-Plugins]]; the window terminal has no token on purpose |
| Export opens wrong in Blender | axis or units | pass `axis` z_up and the units the target expects; glTF is always metres and Y up |
| Alembic export refused | not available | use USD, glTF, MDD, PC2, FBX or an OBJ sequence |
| `build_app --dry-run` reports a file not executable | a lost file mode | restore the executable bit on `build_app` and the `setup/` scripts |
| Browser console shows a missing WebGPU adapter | headless or software rendering | expected; the solver runs natively, not in the webview |

## Getting help

Run the narrowest failing command through the Python API or the MCP server and read the engine's `code` and message; they are the engine's own. Report issues from Help, Report Issue. Include the version (`system_info`), the platform and the steps. Open items are listed in `docs/encyclopedia/15-OPEN-GAPS.md`.
