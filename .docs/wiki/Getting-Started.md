# Getting started

How to install formfabric, boot it, open a sample garment and drape it.

## Requirements

Node 22, Python 3.12 or later, Rust (the version is pinned by `rust-toolchain.toml`), and on Linux the Tauri 2 system packages (webkit2gtk 4.1 among them). `./formfabric doctor` checks the toolchain, the installs and the ports.

## Install and boot

| Step | Command | Result |
| --- | --- | --- |
| Install | `./formfabric_oc_install` (or `./formfabric install`) | frontend packages, backend virtual environment, the `ffb_engine` Python module |
| Boot | `./formfabric boot` | service on `127.0.0.1:8280`, window on `127.0.0.1:5280` |
| Desktop, development | `./formfabric desktop dev` | the Tauri app with the splash; the app picks a free service port |
| Desktop, installable | `./formfabric desktop build` or `./build_app` | see [[Building-Desktop-Apps]] |

`./formfabric engine build` rebuilds the Rust engine and the Python module after a native change. The first release build takes a long time on a small machine. `./formfabric engine wasm` builds the in-page preview engine used when no native engine is attached.

## First drape in the window

1. File, Open Sample, then T-shirt, Skirt or Dress, then a body: Current body, MPFB2 Female or MPFB2 Male. The sample is drafted from the body's measurements, sewn and placed. If the open project has unsaved edits the window asks first (the command `sample_load` refuses without `force`).
2. Press Space (the Drape button) to start the simulation. Press it again to stop. A sample settles in roughly 60 to 120 frames.
3. Window, Layouts, Drape for a layout made for this work, or View for the 2D, split and 3D layouts.
4. File, Save writes a `.ffbrc` project. File, Export writes OBJ, glTF, GLB, FBX, USD, MDD or PC2 (see [[Export-and-UV]]).

The menu bar is File, Edit, Pattern, Sew, Drape, Body, Fabric, Output, View, Window and Help. Edit, Command palette finds any command by name; Edit, Shortcuts lists and changes keys; hold Ctrl (Cmd on macOS) for 600 ms to see Key Hints for the current view.

## First drape from a script

```bash
./formfabric py -c "import formfabric as ff; g = ff.Garment.sample('tshirt'); g.sim.drape(60); g.export_glb('/tmp/tshirt.glb')"
```

The same steps as commands: `sample_load` (name `tshirt`, `force` true), `sim_reset`, `sim_step` (frames 60), `export_run` (format `glb`, absolute path). See [[Python-API-and-Plugins]].

## Units, files and where data lives

- All geometry is in millimetres, x to the right, y up on the pattern plane. `view_units` changes only the display unit.
- Project files end in `.ffbrc` (a zip with `manifest.json`, `garment.json` and optional snapshot entries). Fabrics are `.ffab`, styles `.ffstyle`, poses `.ffpose`, motions `.ffmotion`, components `.ffcomp`.
- The data directory holds `autosave/` (see [[Recovery]]), `library/` and `library_roots.json`.

## Where to go next

[[Pattern-Drafting]] to draw your own panels, [[Sewing]] to join them, [[Draping-and-the-Solver]] to understand the presets, [[Troubleshooting]] when something does not start.
