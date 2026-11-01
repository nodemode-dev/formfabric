# Changelog

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
