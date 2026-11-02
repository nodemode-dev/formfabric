<p align="center">
  <img src="assets/formfabric_icon.png" alt="formfabric logo" width="160" />
</p>

<h1 align="center">formfabric</h1>

<p align="center"><b>The garment studio of the form family: draw sewing patterns, sew them, drape them on a body w/ a real cloth simulation, animate & export.</b></p>

<p align="center">
  <img src="https://img.shields.io/badge/version-v0.23.2-f97316" alt="version v0.23.2" />
  <img src="https://img.shields.io/badge/Node.js-20.19%2B-339933?logo=nodedotjs&logoColor=white" alt="Node.js 20.19+" />
  <img src="https://img.shields.io/badge/Python-3.12%2B-3776AB?logo=python&logoColor=white" alt="Python 3.12+" />
  <img src="https://img.shields.io/badge/Rust-1.99-000000?logo=rust&logoColor=white" alt="Rust 1.99" />
  <img src="https://img.shields.io/badge/license-Apache--2.0-555555" alt="license Apache-2.0" />
</p>

## What it is

formfabric is a desktop garment studio for macOS, Linux & Windows, built in the image of its siblings formvis (image) & formsculpt (sculpting). You draft flat pattern pieces in the Pattern View, sew them edge to edge, & drape them on a body in the 3D View, where a native cloth solver settles the fabric under gravity, contact & seams. Garments animate on a walk, export to every common 3D format w/ their UVs & caches, & save as `.ffbrc` projects. Every action is one engine command, so the window, a Python script & an AI agent over MCP all drive the same document.

**Stack:** Rust engine (14 crates) · vanilla TypeScript + Vite · Python + FastAPI · Tauri 2 · **Scale:** 255 engine commands, every interface string in 30 languages

## Highlights

- **Pattern drafting** panels, rectangles, ellipses, inner shapes & lines, curve tools, exact numeric input w/ arithmetic, snapping, darts, slash & spread, seam allowance, notches, grain lines, mirror & linked instances, unfold, measure & match lengths
- **Pattern exchange** DXF-AAMA import & export in the ASTM D6673 dialect, round trip within 0.05 mm
- **Sewing** seams, free & multi seams, seam side, fold angle & tension, tack, pleats, gather, a live seam check that shows mismatches in millimetres
- **Real cloth** a native solver (block descent & implicit) w/ orthotropic stretch, directional bending, seams, body collision w/ skin offset & friction, stable self contact, pins, wind, pressure & shrink; fast, normal, quality & final presets; an opt-in GPU path on Metal, Vulkan & DX12
- **Bodies** CC0 MPFB2 female & male bodies & the Basic Form, measured & resized, posed, imported from OBJ, glTF, FBX & Alembic; placement volumes & points, Auto Seam proposals
- **3D editing** grab, pinch, pins, fold place, freeze, stiffen, hold shape, body tape, style lines, fit shell, refit to a new body & re-drape
- **Fabrics** physical values in SI units, 15 presets, a Fabric Lab that turns test measurements into parameters, PBR looks front, back & side, colorways, `.ffab` files
- **Trims & details** zippers, buttons & buttonholes, topstitch, piping, binding, elastic, press, tape, bond, skive, prints, quilting, rigid trims from OBJ & FBX
- **Animation** a timeline w/ motion import & retargeting, recorded caches in layers you can cut, merge, move & lock, keys w/ auto key, keyable wind, fabric values & pins
- **Export** OBJ, FBX, glTF & GLB, USD, Alembic, MDD, PC2 & OBJ sequences, weld or unweld, thick or thin, quad remesh w/ seam-matched borders, retopology drawing w/ ghost points across seams, texture baking & game-ready export
- **Visualization** strain & fit maps, 3D measures, overlay compare, a Stage w/ lights, HDRI, shadows & a background image, saved views, snapshots up to ten views at any ppi, video & transparent turntables
- **Workspace** a Library of fabrics, trims, poses, motions, arrangements & garment Components, an Object Browser, snapshots, named layouts, crash recovery from autosave & a command journal, settings export
- **Agents & scripting** every command is an MCP tool; a standard-library Python package w/ plugins; a Live Link that streams a USD layer to Unreal; drafted sample garments to start from
- **The form family shell** the same splash & reveal, title bar, menus, Home screen, Preferences & status bar as formvis, w/ 28 shared themes & zero flash on load

## Quick Start

### Requirements

Node 20.19+ (22 recommended), Python 3.12+, Rust (the version pinned by `rust-toolchain.toml`, installed by `rustup`). Desktop builds on Linux also need the Tauri 2 system packages. `./formfabric doctor` checks them.

### macOS & Linux

```shell
chmod +x formfabric_oc_install formfabric
./formfabric_oc_install
./formfabric
```

Desktop app:

```shell
./formfabric desktop dev
./formfabric desktop build
```

First drape: File, Open Sample, T-shirt, pick a body, then press Space.

### Windows

```shell
formfabric_oc_install.bat
formfabric.bat
```

### Ports

| Service | URL | Notes |
|---|---|---|
| Frontend | http://127.0.0.1:5280 | Vite dev server, `server.host` false, proxies `/api/*` to the backend |
| Backend | http://127.0.0.1:8280 | FastAPI/uvicorn, local only; the desktop app picks a free port |
| Frame stream | 127.0.0.1, OS-assigned port | binary WebSocket from the native engine, token required |

## Configuration

| Variable | Purpose |
|---|---|
| `FFB_PORT` | the backend port (desktop app: 0, a free port) |
| `FFB_FRONTEND_PORT` | the frontend port the backend accepts the window from |
| `FFB_WINDOW_TOKEN` | the window's proof for the backend, 32 or more characters (the desktop app makes one per launch) |
| `FORMFABRIC_DATA_DIR` | where the engine keeps autosaves, recovery, the Library & settings (default: the platform's app data folder) |
| `FFB_AUTOSAVE_SECONDS` | the autosave interval (default 180) |
| `FFB_PLUGIN_DIR` | an extra folder of Python plugins |
| `FFB_ENGINE_URL` | the Python package's HTTP target when it is not run in process |

Set them in the shell. No `.env` is read or committed.

## Architecture

```
formfabric/
├── native/                the engine: one Cargo workspace of 14 crates
│   ├── crates/ffb-pattern/    panels, seams, trims, components, the garment document
│   ├── crates/ffb-sim/        the cloth solver: block descent, implicit, contact, untangle
│   ├── crates/ffb-gpu/        the opt-in wgpu compute solver
│   ├── crates/ffb-body/       bodies, skeletons, placement, body import
│   ├── crates/ffb-io/         the .ffbrc project, DXF-AAMA, every exporter, Alembic
│   ├── crates/ffb-commands/   the one command registry behind menus, MCP & Python
│   └── crates/ffb-stream/     the binary frame stream to the 3D View
├── frontend/              the interface (vanilla TS + Vite): shell, dock, Pattern View, 3D View, drawers
├── backend/               the engine service (FastAPI): dispatch, events, MCP, autosave & recovery, plugins
├── python/                the formfabric Python package
├── src-tauri/             the desktop shell (Tauri 2): splash, reveal, title bar, engine sidecar
├── setup/                 install, boot, build, freeze, version & agent scripts behind ./formfabric
├── docs/integration/      drafts for formcompiler, formportal, GoF & the Unreal Live Link
├── .docs/wiki/            the GitHub wiki pages
├── .github/workflows/     CI: frontend build, engine check & clippy, service selfcheck, desktop shell check
├── build_app              the desktop build, start to finish
└── assets/                the app icon, bodies & sample garments
```

## Status

Phases 1 to 23 of the plan have landed: the family shell, the Rust engine & command registry, pattern drafting & exchange, meshing & the 3D View, fabrics, the cloth solver, sewing, bodies & placement, 3D editing, trims, animation w/ cache layers, export w/ weld, Alembic & retopology, visualization, the workspace, & agents w/ the Python API, plugins & Live Link. v1.0.0 follows the owner's sign-off of the items still deferred (the GPU path & solver speed on real hardware, the Unreal run, body soft body, and a few others). The Linux build is checked here; the macOS & Windows desktop builds come from formcompiler.

## Development

Changes follow the family's rules: single-region hand edits, zero DOM flash, tokens only, no emoji, decisions & removed code in `frontend/HASHOUT_LOG.md`, `backend/HASHOUT_LOG.md` & `native/HASHOUT_LOG.md`. Commands are declared once, in `native/crates/ffb-commands`; `./formfabric agents sync` refreshes the agent tool reference after a change. Commits are Conventional Commits as `nodemode-ai <mackin@nodemode.ai>` w/o attribution trailers.

## Contributors

- nodemode-ai · owner

## Licensing

Apache-2.0, see `LICENSE` & `NOTICE.md`. Bodies are MPFB2 (CC0). Marvelous Designer & CLO are trademarks of CLO Virtual Fashion Inc. & are named for comparison only; formfabric contains none of their code, artwork or text.
