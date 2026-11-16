# Changelog

## v0.25.0 · 2026-11-16

A polish round before the public release: crash recovery keeps far more of your work and brings back the garment's file, the service refuses unsafe files and, when you switch them off, outside agents, glTF exports at the right size, and the window no longer shifts at load or loses keyboard focus. Nothing in this release was run on macOS or Windows; the desktop shell was compile-checked on Linux only, and no signing ran.

Crash recovery:

- After a crash, pins, panel arrangement and placement, folds, body tape, wind, fit shell, UV layout, export thickness, quad remesh, retopology guides and drawn topology are restored too; before, they were lost until the next autosave. View, animation, motion, key, morph, measurement, stage, light, fur and placement edits are repeated as well.
- Box and lasso pins, Apply Drape and Refit save the autosave right away, so their result is kept exactly after a crash.
- An undo or redo of work from before the last autosave is kept after a crash; it used to bring the undone step back. Undo after a restore no longer lands on the wrong step.
- A restored garment keeps its file: the window shows its real name with the unsaved mark, and Save writes to the original file. If that file is gone or cannot be written, the garment comes back untitled as before.
- The restore says in plain words what came back, and which kinds of command were left out or could not be repeated and why, in all 27 languages.
- A recovery that was restored or dismissed is never offered again at a later launch.
- A second formfabric on the same data folder keeps no autosave and no crash journal. The service now says so, and Home and the workspace show a notice that stays until you dismiss it.

Agents, safety and the desktop app:

- "Allow outside agents" in Preferences now does what it says: switched off, agents connecting over MCP (HTTP or stdio) and the other outside-agent paths are refused with a clear message, while the window and its own Claude Code dock keep working. It stays on by default.
- Open, Import DXF, Import fabric, Import body, Import trim and Import motion refuse folders, devices, pipes and oversized files with a clear message instead of hanging.
- The packaged app no longer accepts the development origins.
- The desktop app looks for updates at the family download store, https://dl.formai.studio/formfabric/updates/latest.json, instead of a GitHub releases feed the public cannot read.
- The desktop app shows a message and quits when its engine is missing, offers Keep Waiting or Quit when the engine stays silent at start, and offers Restart or Quit when the engine stops.
- The unused outbound network module is removed from the service.

Export and capture:

- glTF and GLB export in metres by default, the unit glTF defines, so a garment opens at the right size in other tools and Import Body accepts the app's own file. Other formats keep millimetres, and the unit can still be chosen.
- Video and Turntable capture record MP4 where WebM is not available, as in the macOS WebView, and the recording is saved to the chosen path.
- UV, Export, Bake and Game-Ready name the panel that cannot be meshed.
- The Game-Ready dialog no longer offers a Quads option the engine ignored (the export is always triangulated).

Pattern View and 3D View:

- Delete in Edit Points with no point selected no longer deletes the whole panel; it asks you to select a point first. A point selection no longer moves to another point after undo or an inserted point.
- Seam mismatch and the units button in the status bar follow the unit chosen in Preferences instead of always showing millimetres.
- Style lines show in the Pattern View with every tool, as in the 3D View.
- Seam Check says the request failed instead of "All seams match" when the check could not run.
- The statistics line, the map legend and the panel state strip are no longer hidden behind the open timeline. Opening a drawer closes the one that was open, the Maps menu closes after a choice, and the timeline cannot be scrubbed while a recording runs.
- After a reload with an open garment, the Pattern View fits the garment, unless you have already panned or zoomed.

Window and keyboard:

- The workspace bar no longer shifts its buttons while the drawers load, and Home's template grid shows placeholder cards while the templates load instead of pushing the page down.
- Space on a button reached with the keyboard presses that button; it no longer starts the drape.
- Buttons in the Body, Fabric, Output, Library, Objects, Trims and Animation drawers keep keyboard focus after Enter or Space and show the focus ring.
- Esc closes the topmost thing first: menu, then popover, then drawer, then tool. Panel controls are reached by Tab in reading order.
- Panels that are empty or cannot load say what is going on and what to do, also while the engine is not running; the Pattern View says how to start when the garment has no panels.
- The window notices within a few seconds when the service stops, shows it offline and reconnects when it comes back. On a slow connection the status bar no longer flips back to online while the engine is offline.
- Close Garment has a shortcut: Ctrl+W (Cmd+W on macOS) in the desktop app, Ctrl+Alt+W in a browser, where Ctrl+W would close the tab.
- The window shows the app's favicon.

Preferences:

- The default seam allowance sets the starting width of the Seam Allowance tool.
- Sixteen preferences that nothing reads yet are shown disabled with "Not available yet" instead of looking live.

For developers:

- CI checks the version sites, the language tables, the agent tool reference and the build contract, and its actions use the family's pinned versions.
- The agent tool reference and the Python catalogue are regenerated at this version.
- The Timeline, Preferences, Fabric and UV panels write an attribute only when its value changes.

Not run, and left open:

- The desktop shell's engine dialogs, MP4 capture in WebKit, the desktop Close Garment key and the packaged app on macOS and Windows were not run.

## v0.24.0 · 2026-11-15

formfabric joins the Form family standard for Home, the menu bar and the splash, with a full review of the engine, the window, the service and the desktop app behind it. Nothing in this release was run on macOS or Windows; the desktop shell was compile-checked on Linux only, and no signing ran.

Home and the garment library:

- Home replaces the old two-page launcher: a side panel with Start, Recent, Templates and Trash with counts, the open garment, Agents, Local only and Preferences, and a main column with your recent garments with previews and the templates, with filters and search. It shows at launch and when the garment is closed (two preferences, both on by default), Esc leaves it while a garment is open, and the brand mark, File, Home and Alt+Ctrl+H (Alt+Cmd+H on macOS) bring it back.
- A Garments folder under the data folder is the library and the default place to save. From Home a garment in it can be opened, duplicated, moved to Trash, restored, deleted for good, and the Trash emptied, each with an undo toast or a confirm bar.
- A file that lives outside the library is never moved or deleted; Home offers only Remove from recent for it.
- The four sample garments are templates with their own previews, each making a new garment.
- Close Garment is a new command in the File menu; the app had no way to close a document.
- Clear Recent now lasts: the list is cleared in the service too.

Menu bar, splash and desktop:

- The family menu bar: one menu list feeds the bar in the window and the native macOS menu bar, File starts with Home, and the native menu follows Open Recent as it changes.
- Pattern, Sew, Drape, Body, Fabric, View and Help list their engine commands again; Drape and Fabric showed only "Nothing here yet". The Topology Tool has its menu entry.
- The family splash replaces the app's own orange one, with the app's icon and "Starting formfabric".
- A double-clicked `.ffbrc` opens in the running window, and on Linux there is one running instance per data folder.
- Quit, Cmd+Q and the window's close button ask about unsaved work.
- Home's and the menu bar's strings in all 27 languages.

Saving and unsaved work:

- Save on a garment that already has a name saves to it without opening the path dialog again.
- New, Open and Open Recent ask before discarding unsaved work.
- Save can no longer replace a good project with one that will not open: what was written is read back first, and the last good file stays when the check fails.
- After Open, New or Import DXF, a crash before the next autosave no longer replays later edits onto an empty garment. A second service or the stdio MCP on the same data folder no longer writes over the autosave.
- A normal quit no longer looks like a crash at the next launch: the service stops within a second and closes its recovery journal first, also on Windows. The engine exits when the desktop shell goes away.

Fixes, engine:

- Damaged project, body, motion and component files are refused with a message instead of crashing the engine, hanging the import or asking for huge amounts of memory.
- DXF-AAMA import: Import as New keeps seams off the wrong panels; decimal commas and a byte order mark are read correctly; a plain notch on the T-shirt samples is kept.
- Commands that left a broken number in the pattern (splitting a zero-length segment, a huge pose angle) are refused, so the saved project opens again. Open Sample on a body it cannot use no longer leaves an empty project.
- Mirroring a panel keeps the seams on its inner lines on their segment; a duplicated panel's prints can be edited and deleted.
- After a command that changed the garment and then failed, the 3D view, the seam repair and the solver no longer stay one step behind. Broken seams in a file are pruned on open, outside the undo history.
- Cloth solver: an extreme triangle can no longer make the self-contact search allocate until the process dies; a very large body no longer panics the GPU solver.

Fixes, window:

- Pattern: a value typed in the inspector is committed before a click on the canvas, so it no longer lands on the piece clicked next. A two-finger press no longer stops the window following the engine. Editing Position X no longer rewrites Y from its rounded value.
- Exact Input shows and reads the unit chosen in Preferences; with inches chosen a typed 2 no longer means 2 mm. A number field left untouched no longer writes its rounded value back.
- 3D View: the view comes back after a lost WebGL context; one click with the Topology tool no longer leaves the camera orbiting; the analysis map's legend and colours follow the engine; the mesh stream reconnects after a dropped connection.
- The window reconnects after the engine link drops and shows "Engine offline" while it is down.
- Export follows the project's Thickness setting; Bake is offered only when a bake can run; UV layout holds its limits; fabric presets load again after a failed first try.
- Open Sample starts a new document: the old selection and a half-finished tool gesture no longer carry over.
- Video capture stops its tracks and refuses up front where the window cannot record WebM.

Fixes, service and scripts:

- MCP over stdio reads UTF-8 names on Windows; malformed MCP requests answer with the reason instead of an error 500; a plugin that exits no longer produces a bare error.
- The macOS build signs framework libraries so the engine starts inside the signed app (the fault formtools 0.8.1 and formtxt 0.32.1 fixed). No macOS build ran here.
- Killing `./formfabric` from the terminal also stops Vite.

Fixes from the final check:

- A garment the engine already holds is taken up by the window as an unsaved, untitled garment: an agent's draft leaves Home and shows, and a reload of the window keeps unsaved work. After a reload in the browser a saved garment also reads as untitled, and Save offers the library folder.
- A long menu opened from the menu bar stays under the bar and scrolls, instead of running off the window.
- 3D View: only the primary pointer drives the view, as in the Pattern View. A fault in the in-page preview engine no longer throws on every pointer move; later calls are refused and snapping falls back to the grid.
- Home matches formdeck in three details: the brand line, the count badges and the hero button.
- A fault inside a simulation step stops the run and says so, instead of leaving it marked as running.

Not run, and left open:

- The native macOS menu bar, Alt+Cmd+H beside the system's Hide Others, the Windows quit path and the packaged app were not run.
- Project files already saved with a broken number before this release stay unopenable.

## v0.23.5 · 2026-11-04

- The README hero is now the owner's screenshot of the printed dress on the body; the old plain-white-dress screenshots are removed.

## v0.23.4 · 2026-11-04

- README rewritten to the Form family layout, with screenshots of a draped dress in `docs/images/`.

## v0.23.3 · 2026-11-03

- The unified v3 app icon for the desktop icons, the window favicon (now a PNG) and the README.
- The compiled app name is FormFabric (window and splash titles).
- `.github/CODEOWNERS` added.

## v0.23.2 · 2026-11-02

- CI: every action moved to its Node 24 major (`checkout@v6`, `setup-node@v6`, `setup-python@v6`). GitHub removed Node 20 from its hosted runners in September 2026, and all four jobs failed within seconds on the first push.
- The encyclopedia (behaviour specification, parity matrix, open gaps, fabric data) moved out of the repository into the planning sidecar.
- README rewritten in the form family layout, with the new icon.

## v0.23.1 · 2026-11-02

Fixes from the owner's test run, and a new icon.

- 3D View: a fabric's base colour image map now shows (the engine's `base_color` key was not read).
- 3D View: a fabric's map repeat size is honoured when it arrives as a width and height pair, instead of falling back to 200 mm.
- 3D View: texture images stored as local files load through the service with the window token, as the fabric panel previews already did.
- A new app icon (desktop icons and the window favicon).

## v0.23.0 · 2026-11-02

Phases 19 to 23, released together. They close the parity rows that were partial at v0.18.0: of 125 rows, 115 are landed, 1 is partial and 9 are deferred, six of them pending the owner's sign-off. v1.0.0 follows that sign-off.

Workspace and viewer (Phase 19):

- The Animate and Render workspace tabs, each applying its layout; UV and Retopo open the Output drawer section.
- An Object Browser drawer: scene tree and typed fabric, graphic, button, buttonhole, topstitch, puckering, trim and zipper lists, with copy, merge and fabric order.
- Arrangement files (`.ffplace`) in the Library.
- A background image on the Stage.
- Capture: multi-view layouts of 2 to 10 camera views, separate images, ppi in the PNG, a transparent GIF turntable.

Animation (Phase 20):

- Cache layers can be cut, merged, moved and locked; the top layer wins where layers overlap.
- Auto key, keyable pins, and trims recorded into their own cache layer.

Export and retopology (Phase 21):

- Weld, Unweld and Weld Selected for every mesh and cache format.
- Thick export with quad output; FBX trims; diffuse, metalness, roughness and opacity bakes with seam fill.
- Retopo: draw points and faces on panels, merge, divide, remesh, with ghost points across seams.

Solver and finishing (Phase 22):

- Refit pushes cloth out of the new body first (0.0 mm inside the male body, from 22.3 mm).
- Skive, the softening counterpart of Bond.
- An opt-in Untangle pass (Normal or Flipped Normal).

Alembic and quad regularity (Phase 23):

- Alembic export (static and animated caches) and Alembic body import, from formfabric's own Ogawa code, checked with the Alembic 1.8.12 library's tools.
- An integer edge regularity pass for the quad remesher (min-cost flow), behind a flag.

Also: an independent audit's 22 findings fixed (hostile Alembic files, cache arithmetic, topology loading, texture reads, export rollback), the toolchain's new clippy lints cleared, every new string in all 29 languages, 255 commands.

## v0.18.0 · 2026-11-02

Phases 16, 17 and 18, plus the solver work carried from v0.15.0, released together. The plan names v1.0.0 for Phase 18, but its exit (every parity row landed or explicitly deferred) is not met: 92 rows are landed, 3 deferred and 30 partial, so this is v0.18.0.

Workspace (Phase 16):

- Library drawer with roots, kinds, search, tags, thumbnails and insert.
- Snapshots with 3D thumbnails, restore and compare.
- Garment components with named ports, joins, swap and a port check (the drafted T-shirt's sleeve swaps for a sleeveless variant and back).
- Crash recovery: the service keeps a journal of commands since the last autosave and offers to restore it on the next start (checked by killing the service during a drape).
- Named layouts that persist; settings, shortcuts and layouts export and import.
- Key Hints, and per-scope conflict detection in the Shortcut Editor.

Agents and family (Phase 17):

- A Python package, `python/formfabric`, with every command as a function and in-process and HTTP backends.
- Python plugins with argument validation and timeouts, and a sample drape-and-export plugin.
- Live Link: a USD layer rewritten while draping, for Unreal's USD Stage (untested against Unreal).
- The packaged engine serves MCP and agents correctly; 232 MCP tools.
- Drafts for formcompiler (`apps/formfabric.json`), the formportal catalogue and product entries, and a GoF hand-off proposal, in `docs/integration`.
- A VERSION file with a version check across every site, and `build_app --dry-run`.

Documentation (Phase 18):

- README, a 19-page wiki in `.docs/wiki`, and updated CLAUDE.md and AGENTS.md.
- A NOTICE audit of every distributed dependency (433 Rust crates, 6 npm and 22 Python packages, all permissive).
- A status for every parity-matrix row; current open gaps in 15-OPEN-GAPS.

Solver:

- Self contact is stable on sewn garments in motion and on at every preset. On the walk, seams stay within 3.3 mm and no cloth goes inside the body. Before, Quality exploded with 177 mm moves and 51 mm seams. The T-shirt still shows up to 61 intersecting pairs.
- Desktop builds use every core for both solvers. On the 2-core sandbox, 5,041 vertices went from 206 ms to 150 to 180 ms of CPU per frame; the 8 ms and 50 ms targets are not met there.
- GPU path: hinge gradients and collisions now match the CPU. The reported wind and layer bugs were test artefacts.

Security and fixes from the release audit:

- The library is confined to its roots, and its index recovers from corruption.
- Component and snapshot files are validated.
- Crash recovery cannot be taken over by a second running backend.
- The Python client sends the window token only to loopback addresses.
- Paths written to the Codex config are escaped.
- Build scripts use `npm ci`.

## v0.15.0 · 2026-11-02

Phases 13, 14 and 15, plus the drape problems reported from testing v0.9.0, built together and released as one version.

Fixed from the owner's report:

- **Collisions:** contact only searched within the 3 mm skin offset, so cloth that got deeper was never pushed out (garments sank 12 to 38 mm into the body). Every substep now reaches 40 mm and clamps the approach to the body. Penetration is 0 mm at every sampled frame of every sample garment and of a 120-frame walk.
- **Shrink-wrap look and seams:** on sewn garments the implicit solver never converged. The cloth crumpled and rode up onto the shoulders, which looked like shrink-wrap. Drapes of sewn garments now use block descent by default. The drafted T-shirt hangs to the hip with seams within 4 mm and the neckline within 13 mm of the neck.
- **Space did nothing:** on a garment the drape runner starved state reads, so the window never saw a frame. The first frame now arrives in 0.3 s.
- **Claude Code dock lag:** the chat pane relaid the page on every streamed token (500 messages took 288 s under 4x CPU throttling, now 5.7 s). The terminal output is batched, its backpressure is acknowledged by bytes, and dragging the dock no longer re-renders the 3D and Pattern Views on each step.

New:

- **Sample garments:** File, Open Sample drafts a T-shirt, A-line skirt or sleeveless dress from the current body's measurements.
- **Animation:** BVH, glTF and FBX motion import with retargeting and lead-in; key tracks for body pose and measurements, wind, panel state and fabric; recorded cache layers saved in the project; playback, scrubbing and body morph; a timeline dock.
- **Export:** OBJ, glTF, GLB, FBX, USD, MDD and PC2 with UVs, materials, thickness and the animated cache, checked in Blender 5.2. Also UV layout with UDIM tiles and a UV Editor, field-aligned quad remeshing with seam-matched borders and retopo guides, baked normal, AO and displacement maps, and game-ready export.
- **Visualization:** strain, stress, pressure, fit, contact and thickness maps; compare maps; 3D length, circumference and distance measures; stage lights with shadows, HDRI and ground; saved views; fur; schematic SVG and PDF export; snapshot, video and turntable capture.
- **3D tools:** Style Lines (stiffen or fold bands) and Fit Shell.
- **GPU solver:** opt-in through the solver device setting.

Not yet:

- The walk keeps the T-shirt out of the body, but the cloth passes through itself in 92 of 120 frames (the Quality preset's self contact is unstable on sewn garments).
- CPU speed is far from the targets: about 15 to 22 ms per frame at 676 vertices, about 200 to 260 ms at 5,000.
- The GPU path pushes wind the wrong way and lets layers interpenetrate.
- Alembic export is deferred.

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
