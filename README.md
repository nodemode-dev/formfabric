# formfabric

**Garment pattern design and cloth simulation, in the NodeMode style**

![version](https://img.shields.io/badge/version-0.23.1-f97316) ![status](https://img.shields.io/badge/status-in%20development-f97316) ![license](https://img.shields.io/badge/license-Apache--2.0-blue)

## What it is

formfabric is the form family's garment app: draw sewing patterns in 2D, sew them, drape them on bodies with a cloth simulation (CPU by default, GPU opt-in), animate them & export them. It aims at feature parity with the reference garment tools (see the parity matrix), built in the open under Apache-2.0. Every action is one engine command, so the window, a Python script & an AI agent all drive the same document.

Stack · Rust engine (`native/`, one Cargo workspace of 14 crates, wgpu compute solver) · vanilla TypeScript & Vite (`frontend/`) · Python & FastAPI (`backend/`) · Tauri 2 (`src-tauri/`) · 255 engine commands. Desktop builds for macOS, Windows & Linux come from formcompiler; formportal lists & connects the app.

## Status

Phases 1 to 23 have landed (release v0.23.0). The gate to v1.0.0 is the owner's sign-off of the deferrals: of 125 parity rows, 115 are landed, 1 is partial (the quad remesh regularity pass, off by default) & 9 are deferred, 6 of them pending owner sign-off. The scope: the family window shell, the Rust engine, pattern drafting with DXF-AAMA exchange, meshing & the 3D View, fabrics, cloth simulation, sewing, bodies with placement, 3D editing, trims, animation w/ cache layers & auto key, export w/ UV, weld, Alembic & quad remeshing, retopology drawing, visualization, the workspace (Library, Object Browser, snapshots, Components, crash recovery, layouts), & agents with family integration (Python API, plugins, Live Link, GoF hand-off). The status of every feature is in the Status column of [14-PARITY-MATRIX.md](docs/encyclopedia/14-PARITY-MATRIX.md).

You can:
- Open a drafted T-shirt, skirt or dress from File, Open Sample, or draft & sew your own.
- Drape it on a CC0 body or formfabric's own Basic Form.
- Edit it in 3D: pinch, pin, freeze, stiffen & refit.
- Add trims: zippers, buttons, topstitch, edge finishes, prints & quilting.
- Animate it: import a walk, record a cache, cut, merge, move & lock cache layers, auto key wind, pins & fabric values, & play it back.
- Export to OBJ, glTF, FBX, USD, Alembic, MDD or PC2, welded or not. Exports open in Blender with UVs, materials & the animation (Alembic is checked against our own reader only).
- Draw retopology points & faces on a panel, w/ ghost points mirrored across seams.
- Look at strain & fit maps, & capture snapshots, multi-view sheets & turntables w/ a transparent background.
- Keep reusable fabrics, trims, poses, motions, placements & Components in a Library, browse styles in the Object Browser, save whole-garment snapshots, & recover after a crash.
- Script it from Python or let an agent draft patterns through MCP.

Open gaps, as recorded in [15-OPEN-GAPS.md](docs/encyclopedia/15-OPEN-GAPS.md):
- Self contact in motion: the opt-in Untangle pass does not reliably remove the pairs a recorded walk shows, so zero pairs is not reached.
- CPU solver speed misses its targets (676 vertices in 8 ms, 5,000 in 50 ms); the 30k vertex case is unmeasured.
- The GPU solver is opt-in & was never run on a real GPU here.
- Alembic is written & read by our own Ogawa code & not yet opened in Blender (an owner check); the quad regularity pass is off by default, w/ flip repair & the multi-resolution hierarchy not built.
- Retopology drawing lacks drag select, Patch Topology & a second OBJ object for duplicate mode.
- Unreal Live Link, FBX motion import & the 60 fps orbit at 200k triangles are untested in the app; 8 of 15 fabric presets are provisional.
- Deferred, pending owner sign-off: layer rank between sublayers, body soft body & divide mesh, AI & PDF pattern files, speed & GPU targets, provisional presets, the GoF contract & the Unreal run. Deferred by decision: AI Studio, the CONNECT marketplace & proprietary files.

See BUILD_NOTES (planning sidecar) & the parity matrix for details.

## Features by area

| Area | What it does | Wiki |
| --- | --- | --- |
| Pattern drafting | panels as Bezier outlines, inner lines & shapes, notches, grain lines, seam allowance, darts, slash & spread, mirror & instance links, exact input, DXF-AAMA import & export | [Pattern drafting](.docs/wiki/Pattern-Drafting.md) |
| Sewing | segment, free, multi-span & automatic seams from notches, pleats, gathers, tacks, rip & mend, length checks | [Sewing](.docs/wiki/Sewing.md) |
| Bodies & placement | Basic Form, two CC0 bodies, OBJ, glTF, FBX & Alembic import, resize to measurements, poses, placement volumes & points | [Bodies and placement](.docs/wiki/Bodies-and-Placement.md) |
| Fabrics | SI fabric properties, presets, Fabric Lab from bench numbers, colorways, `.ffab` files | [Fabrics](.docs/wiki/Fabrics.md) |
| Draping | vertex block descent & implicit solvers, Fast to Final presets, seam closing, self contact, wind, pins, grab, pinch, optional GPU | [Draping and the solver](.docs/wiki/Draping-and-the-Solver.md) |
| 3D editing | pinch, pin box & lasso, fold, freeze, stiffen, Style Lines, Fit Shell, Refit w/ a deep push | [3D editing](.docs/wiki/3D-Editing.md) |
| Trims | buttons, buttonholes, zippers, topstitch, puckering, fray, piping, binding, elastic, skive, prints, quilting, OBJ & FBX trims | [Trims](.docs/wiki/Trims.md) |
| Animation | BVH, glTF & FBX motion with retargeting, key tracks, body morph, recorded cache layers (cut, merge, move, lock), auto key, pin keys, timeline | [Animation](.docs/wiki/Animation.md) |
| Export & UV | OBJ, glTF, GLB, FBX, USD, Alembic, MDD, PC2 w/ weld; UV layout with UDIM, thickness, baked look & detail maps, quad remesh, topology drawing, game-ready | [Export and UV](.docs/wiki/Export-and-UV.md) |
| Visualization | strain, stress, pressure, fit, contact & thickness maps, compare, measures, lights, background image, fur, schematic, multi-view capture | [Visualization and capture](.docs/wiki/Visualization-and-Capture.md) |
| Workspace | Library, Object Browser, snapshots, Components, crash recovery, saved layouts, Key Hints, settings file, Animate & Render tabs | [Library, snapshots and components](.docs/wiki/Library-Snapshots-and-Components.md) |
| Agents & scripts | MCP over stdio, agent skills, Python API, Python plugins, Live Link to Unreal through USD, GoF hand-off | [Agents and MCP](.docs/wiki/Agents-and-MCP.md) |

## MCP & Python API

All 255 engine commands are MCP tools (`./formfabric mcp`, wired for Claude Code by `.mcp.json`, for Codex by `.codex/config.toml` & for Cursor by `.cursor/mcp.json`; `./formfabric agents install` registers it outside this folder). The skill `.claude/skills/formfabric-modeling` carries the generated tool reference, refreshed by `./formfabric agents sync`.

The Python package in `python/` (standard library only) has one checked function per command plus a hand API (`Garment`, `Panel`, `Sim`, `Link`, `gof.send`). It runs in process against the `ffb_engine` module or over HTTP against the running service. See [python/README.md](python/README.md) & the wiki page [Python API and plugins](.docs/wiki/Python-API-and-Plugins.md).

## Family integration

| With | How |
| --- | --- |
| formcompiler | builds the desktop app from `main` or a `compile_v*` branch through the launcher verbs `install`, `install-desktop` & `desktop build`; `./build_app --dry-run` checks the contract |
| formportal | lists the app from the catalogue entry & connects the MCP server; drafts are in `docs/integration/` |
| formsculpt, formsurface | GoF hand-off bundle written by `formfabric.gof.send` (a proposal, no format change) |
| Unreal | Live Link writes a USD layer that Unreal's USD Stage reloads (untested against Unreal) |

formcompiler & formportal are GPL-3.0; formfabric meets them through contracts only & contains none of their code. Pushing the repository, cutting `compile_v*` branches & the external builds are an owner run.

## Quick start

```bash
./formfabric_oc_install      # frontend packages, backend venv, engine module
./formfabric boot            # service on 127.0.0.1:8280, window on 127.0.0.1:5280
./formfabric desktop dev     # the desktop app with the splash
./formfabric desktop build   # the installable desktop app (or ./build_app for the whole chain)
```

Requirements: Node 22, Python 3.12 or later, Rust (pinned by `rust-toolchain.toml`), & on Linux the Tauri 2 system packages. `./formfabric doctor` checks the toolchain, installs & ports.

First drape, in the window: File, Open Sample, T-shirt, then a body (Current body, MPFB2 Female or MPFB2 Male); press Space to start or stop the drape. Window, Layouts, Drape gives a layout made for it.

First drape from a script:

```bash
./formfabric py -c "import formfabric as ff; g = ff.Garment.sample('tshirt'); g.sim.drape(60); g.export_glb('/tmp/tshirt.glb')"
```

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
| `./formfabric py script.py` | runs Python with the `formfabric` package importable |
| `./formfabric agents install` | registers the MCP server with Claude Code, Codex & Cursor |
| `./formfabric i18n check` | checks every language table against the English strings |
| `./formfabric doctor` | checks the toolchain, installs & ports |
| `./build_app --dry-run` | checks the compile contract (files, launcher verbs, CI trigger, version sites) without building |
| `./formfabric help` | every verb |

Configuration is by environment variable, never committed: `FFB_PORT` & `FFB_FRONTEND_PORT` (ports), `FFB_WINDOW_TOKEN` (32 or more characters, the window's proof for the service), `FFB_AUTOSAVE_SECONDS` (default 180), `FFB_PLUGIN_DIR` (extra plugin folder), `FFB_ENGINE_URL` (the Python client's HTTP target).

## Layout

| Folder | Holds |
| --- | --- |
| `native/` | Rust workspace of 14 crates: `ffb-core`, `ffb-pattern`, `ffb-fabric`, `ffb-mesh`, `ffb-sim`, `ffb-gpu`, `ffb-body`, `ffb-anim`, `ffb-remesh`, `ffb-io`, `ffb-commands`, `ffb-stream`, `ffb-py`, `ffb-wasm` |
| `backend/` | FastAPI engine service, MCP servers, autosave & recovery, plugins, terminal & Claude Code services |
| `frontend/` | the window: shell, dock, workspace with Pattern View, 3D View & the drawers |
| `python/` | the `formfabric` Python package |
| `src-tauri/` | Tauri 2 desktop shell |
| `setup/` | install, boot, build, freeze, version & agent scripts |
| `docs/encyclopedia/` | the behaviour specification formfabric is built against |
| `docs/integration/` | drafts & notes for formcompiler, formportal, GoF & the Unreal Live Link |
| `.docs/wiki/` | the wiki source, published to the GitHub wiki |
| `.claude/`, `.agents/` | agent skills (harness & modeling) |

## The encyclopedia

`docs/encyclopedia/` documents what Marvelous Designer does and how each feature behaves, from primary sources only, as the specification formfabric is built against.

| Start here | |
| --- | --- |
| [00-INDEX.md](docs/encyclopedia/00-INDEX.md) | chapter list and reading order |
| [14-PARITY-MATRIX.md](docs/encyclopedia/14-PARITY-MATRIX.md) | every feature mapped to a formfabric module, name and phase |
| [15-OPEN-GAPS.md](docs/encyclopedia/15-OPEN-GAPS.md) | what could not be verified and how formfabric decides it |

## Trademarks

Marvelous Designer and CLO are trademarks of CLO Virtual Fashion Inc., named here only to describe compatibility and the reference studied. formfabric is not affiliated with or endorsed by CLO Virtual Fashion.

## Development

Read `CLAUDE.md` (or `AGENTS.md`) first, then the HASHOUT_LOG of the side you edit. Commands are declared only in `native/crates/ffb-commands`; after a change run `./formfabric engine build` & `./formfabric agents sync`. Checks: `./formfabric engine check`, `python3 setup/i18n.py check`, `python3 setup/version.py check`, `./build_app --dry-run`. Branch model in [.docs/BRANCHING.md](.docs/BRANCHING.md); docs live in [.docs/wiki/](.docs/wiki/Home.md) (source of truth, published to the GitHub wiki) & `docs/`. Edits are single-region hand edits; no emoji, no em dashes in markdown, no GPL code.

## License

Apache-2.0, see `LICENSE`. Copyright 2026 NodeAgentc, Inc. Every distributed third-party dependency, with its licence, is listed in `NOTICE.md`; nothing copyleft is linked into the engine, the Python module or the desktop shell (MPL-2.0 file-level crates are noted there).
