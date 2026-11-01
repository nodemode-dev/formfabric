# Changelog

## v0.12.0 · 2026-11-01

Phases 10, 11 and 12, built in parallel and released together.

Phase 10, solver:

- An implicit solver (Gauss-Newton with preconditioned conjugate gradients) is now the default. It converges: a hanging 1 m square stays within 0.007 mm of the reference after 3 s.
- Cloth self contact, Untangle, and layer order from each panel's layer.
- Presets: Fast, Normal, Quality and Final.
- A GPU solver on wgpu compute, checked against the CPU on a hanging square. It is not yet used by Drape.
- Not yet met: speed on large garments (about 22 ms per frame at 676 vertices and 273 ms at 5,041 on the Normal preset), and GPU self contact.

Phase 11, 3D editing:

- Pinch; box and lasso pins; Freeze, Disable, Stiffen, Hold Shape and Shrink per panel; Wind with an arrow in the 3D View; Fold Place; Body Tape.
- Refit to another body by blend, measurement sweep or re-drape.
- A seam-closing phase at the start of a drape.
- Choice of solver method and device.
- Not yet met: a whole shirt hanging cleanly on the shoulders, and Refit leaving no cloth inside the new body. Style Lines and Fit Shell move to Phase 13.

Phase 12, trims:

- Buttons and buttonholes with fastening; zippers.
- Topstitch and puckering.
- Edge finishes: piping, binding, elastic (shirring), tape, bond, press and crease.
- Prints, quilting, rip, mend and fray.
- Imported trim meshes.
- All of it works in the Pattern View, the 3D View, the inspector and MCP, with undo that follows seam edits.

Security and fixes from two independent audits:

- The engine and service now use a per-launch window token from the desktop shell. It is never inherited by terminal or agent processes and never appears in a URL.
- Host checks against DNS rebinding.
- Import caps on file size, vertex and triangle counts, and glTF buffers confined to the model folder.
- Bounded simulation calls; undo history capped by size.
- Quilting no longer explodes the drape.
- Placement wraps panels around the body.
- Seams close relative to the cloth's stiffness.

## v0.9.0 · 2026-11-01

Phases 7, 8 and 9, built in parallel and released together.

Phase 7, solver and drape:

- A CPU cloth solver (Vertex Block Descent) with an orthotropic membrane, directional bending with buckling, seams, fold lines, pins, grab, pressure, body and primitive collisions with friction, optional self collision, and an implicit conjugate-gradient reference for comparison.
- Drape from the window or MCP: on and off with Space, Step, Reset, Normal, Fast and Fine presets, pins, grab while draping, Apply.
- Not yet met: the plan's 2 mm accuracy against the reference and interactive speed on large garments (see BUILD_NOTES); Phase 10's GPU solver addresses them.

Phase 8, sewing:

- Seam, Free Seam, Multi Seam, Edit Seam, Seam Fold, Seam Tension, Seam Side, Seam Check, Tack, Pleats and Gather, in the Pattern View and over MCP; seam lines in 3D.

Phase 9, bodies and placement:

- Default female and male bodies from MakeHuman community assets via MPFB2 (CC0 1.0), and formfabric's own Basic Form.
- Import OBJ, glTF and GLB with skins, and FBX on desktop; the Body Editor resizes to target measurements; A, T and joint poses; skin offset and friction.
- Placement volumes and points from the skeleton, Place Panel with wrapping, Auto Seam proposals.

Security and fixes from an independent audit: engine and terminal calls only from the window or with the in-app token; absolute paths, required extensions and atomic writes; size caps on project, fabric and DXF files; control characters refused in names; engine panics no longer stop the service; stream connection limits; dropdowns and caption buttons update in place; per-item shortcut reset.

## v0.6.0 · 2026-11-01

Phases 4, 5 and 6, built in parallel and released together.

Phase 4, pattern tools:

- Seam allowance with miter, square, round and mirror corners and per-segment widths; offset outlines.
- Notches (slit, V, T, U) that stay in place through point edits and transforms; grain lines; fold and dart roles on inner lines with fold angle and strength; match lengths.
- Darts on the outline and internal darts, slash and spread, Mirror Edit and Instances with linked copies that undo with their source, unfold, spiral, trace.
- DXF-AAMA (ASTM D6673) and plain DXF import and export.
- Eleven new Pattern View tools with keys and Exact Input; the Inspector edits seam allowance, notches, inner roles and links.

Phase 5, meshing and 3D View:

- Constrained Delaunay meshing of every panel at its particle distance, minimum angle 20 degrees, with per-segment vertex counts ready for seams.
- Meshes stream from the native engine to the 3D View as panels change.
- The 3D View draws meshed panels with their fabric's front and back surfaces, four view styles, stats, view presets, picking shared with the Pattern View, and a placement gizmo.

Phase 6, fabric:

- Fabrics with physical properties in SI units, 15 presets traced to measured data (8 marked provisional), Fabric Lab, looks with PBR maps, colorways, `.ffab` files.
- The Fabric panel beside the Inspector.

Fixes: drag start no longer spikes; menus update in place; the i18n check reads the real tables and every string is translated; projects reopen bit-exact; enhanced selects show one control.

## v0.3.0 · 2026-10-31

Phase 3, the pattern core.

- Pattern View: draw and edit panels on a millimetre grid with ten tools (Transform, Edit, Draw Panel, Rectangle, Ellipse, Inner Shape, Inner Line, Split, Curvature, Measure), Bezier curve points, snapping, rulers and framing.
- Exact Input for every tool, with arithmetic in numeric fields.
- Drags preview in the in-page engine and land as one undo step.
- Inspector for name, position, rotation, grain, particle distance and layer, with perimeter and area.
- Split workspace with Pattern View, a 3D View floor and a resizable splitter; 2D, split and 3D layouts.
- 24 pattern commands in the engine, reachable from the window and over MCP.
- Interface strings translated into 27 languages.
- Fixes: change events only when the document changes; Mod+Shift shortcuts fire; the installer builds the preview engine.

## v0.2.0 · 2026-10-31

Phase 2, the engine skeleton.

- Rust engine workspace with one command registry that drives menus, palette, shortcuts, Exact Input dialogs and MCP; typed undo history.
- `.ffbrc` project files: save and reopen, unknown entries preserved.
- The `ffb_engine` Python module behind the service, a wasm build for in-page previews, and the binary frame stream the solver will use.
- MCP over stdio and HTTP generated from the catalogue; harness and modeling skills kept in sync with it.
- CI checks on `main` and `compile_v*` branches.
- Recent projects from the service, native Open and Save dialogs in the desktop app.

## v0.1.0 · 2026-10-31

Phase 1, the family shell.

- Launcher and setup scripts for macOS, Linux and Windows, including the install, install-desktop and desktop build verbs formcompiler runs.
- Tauri 2 desktop shell with the family splash, reveal and title bar, and the `.ffbrc` file association.
- FastAPI engine service with the events stream, HTTP and stdio MCP, and the terminal, Claude Code and Chat services.
- The window shell: 28 themes, 30 languages, menus, palette, shortcut editor, status bar, launcher home screen, Preferences.
- The terminal, Claude Code and Chat dock.

## v0.0.1 · 2026-10-31

The planning release. No application code yet.

- The formfabric encyclopedia of Marvelous Designer in `docs/encyclopedia/`: 16 research chapters from primary sources, an index, a parity matrix mapping every feature to a formfabric module, name and build phase, and the list of open gaps.
- Apache-2.0 license, copyright 2026 NodeAgentc, Inc., with the trademark notice.
