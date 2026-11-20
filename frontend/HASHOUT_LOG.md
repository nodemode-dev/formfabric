# HASHOUT_LOG · frontend

Living context for the window. What would be comments in `frontend/` lives here, newest first. Read before editing.

## v0.29.0 · 2026-11-20

### The family's design names (FormVis's palette)

- FormFabric shares FormVis's palette: both token files are the same stylized-nodemode base, every shared `:root` and light token had FormVis's value already, and the theme profiles are FormVis's (FormVis has one more, AMOLED Black). So the step is FormVis's values, not names only.
- Added, with FormVis Studio Dark's values: `--cv-options-h`, `--cv-tabs-h`, `--cv-rail-w`, `--cv-layers-w`, `--cv-row-h`, `--cv-thumb`, `--cv-checker-a`, `--cv-checker-b`, `--cv-ants-a`, `--cv-ants-b`, `--cv-pixel-grid`, `--cv-handle-fill`, `--cv-doc-edge`, `--cv-action-blue`, `--cv-action-violet`, `--z-under`, `--z-local-2`, `--z-local-4`, `--z-local-5`, and `--dock-outline`. Every FormVis token name now resolves here.
- Own names kept: `--scrim` is now an alias of FormVis's `--bg-scrim` (FormVis's dock reads that name where formfabric's read `--scrim`), so a scrim is black at 60 percent instead of 45. `--shadow-panel`, `--uv-island-off`, `--axis-*` and `--trim-*` have no FormVis counterpart and stay as they are.
- Shared dock names that differed take FormVis's value: `--dock-raised` (`--surface-raised`), `--dock-tab-border` (`--line-2`, the same colour), `--dock-scrim` (`--bg-scrim`), `--dock-accent-line` (`--accent-line`), `--dock-term-font-size` (13 px, was 12). No rule, literal or layout was touched.

## v0.26.0 · 2026-11-20

### The agent dock 2.2.0 replaces formfabric's own dock

- `dock/` · the family's shared module, 2.2.0, copied byte for byte from formvis (15 files and `MANIFEST.json`; every hash matches). Never edit it here; a change goes to formvis first. `setup/i18n.py` skips this folder: its `t(key, english)` calls pass ids such as `lead`, not English keys; the English goes through the adapter's `t`, which looks it up in the language tables.
- Removed: `dock/index.ts` (`mountDock`) and `dock/terminal-dock.ts` (the old tab strip, terminal and ask line). Moved with `git mv` to `agent/`: `chat-pane.ts` (formfabric's Chat, with its usage meter, Claude Code hand-off and adopt), `agent-link.ts` (the outside-agent consent dialog and the engine epoch poll), `link.ts`, `dom.ts`, `markdown.ts`, `sse.ts`; `engine/service.ts` and `engine/events.ts` import `agent/sse`. `styles/dock.css` became `styles/agent.css` without the old dock's rules.
- `dock-adapter.ts` · `DockAdapter<'antigravity'>` over the service's window commands: tabs Chat, shell, Claude Code, Codex, Cursor, Antigravity. Chat is formfabric's own pane through `chat.pane(host)` with `ask: 'line'`, so the ask line sends to Chat on the Chat tab and opens Claude Code with the text elsewhere, as the old dock did. The socket's exit frame is the text `{"type": "exit"}`; the adapter turns it into `onExit`. No `consent` or `storedChoice`: the outside-agent dialog stays formfabric's own (`agent/agent-link.ts`). No activity tab: formfabric has no such feed.
- `app.ts` · `new Dock(adapter).mount(.cv-dock)`, once; `dock.onTab` keeps the title bar buttons in step; Cmd or Ctrl+Shift+J toggles Claude Code and Alt+J Chat (the old dock's chords), Cmd or Ctrl+J is the module's own. `ff:dock-toggle` (menus, layouts) still opens and closes it.
- Layout: the dock lives in the shell grid's own track (`.cv-app` row 3, `--ft-dock-h`), and `styles/terminal.css` adds the right-side column (`--ft-dock-w`), so opening, closing and resizing change a track size, never a node outside the dock. `index.html`'s pre-paint seed now also sets `data-ft-dock-side` and the width from `formfabric.dock`. The storage key is unchanged, so a saved height and open state carry over; the first-run guide shows once, since the old key has no `seen`.
- `styles/dock-tokens.css` maps every `--dock-*` token onto formfabric's tokens.
- Checked in headless Chrome against the dev server and the live service, CPU throttled 4x: the dock opened in its region, showed the Chat and shell tabs and the Claude Code, Codex, Cursor and Antigravity launchers, ran `echo ui-dock-ok` in the shell, and a `MutationObserver` on `.cv-app` saw no node added or removed outside `.cv-dock` across open, close, open, close; no console errors. The Home screen covers the window, the dock included, as it did before.

## v0.25.0 · 2026-11-16

Polish round: workspace bar, Preferences, outside agents, export units, shortcuts.

- `styles/pattern.css` :: `.ff-bar` · the bar is a grid with one fixed column per drawer button (Fabric, Body, Trims, Timeline, the two Output buttons, Library, Objects) and a 48 px zoom column, each button placed by its class. The buttons are mounted one at a time after `body.app-ready` by their own panels (`insertBefore(zoom)`); in the old flex row each one pushed the earlier ones 36 px left, six layout shifts on every load with a garment open. The columns hold their width while empty, so mount order no longer moves anything. A new drawer button needs its own column and a `grid-column` rule here.
- `ui/settings.ts` :: `later`, `soon` · a preference that nothing reads is shown disabled with the translated note "Not available yet" under its group: Look for updates at start (no updater), Match system appearance, Light theme, Dark theme, Use the graphics processor, Frame pacing, Worker threads, History steps kept, History kept in projects, Renderer, Antialiasing, Shadows, Show the floor, Background, NodeMode address; Solver device was already disabled and now carries the note. The values stay stored, so wiring one later is a one-line change here.
- `workspace/pattern-view/tools-garment.ts` :: `SeamTool` · a panel with no seam allowance yet starts the Seam Allowance dialog at Preferences, Pattern, Default seam allowance (was a fixed 10 mm).
- `app.ts` :: `pushAgents` · "Allow outside agents" is sent to the service (`agents.prefs`, `agentsAllowed`) when the engine link comes up and on every change; the sends are chained so two quick flips cannot arrive out of order. The window's stored value wins over the service's file. The guard itself is in the service (`backend/HASHOUT_LOG.md`, v0.25.0).
- `shell/window-commands.ts` :: `app.close` · Close Garment is Mod+W in the desktop app and Mod+Alt+W in a browser, where Mod+W closes the tab (formvis `close-tab`). Nothing else in the catalogue or the window binds W with a modifier.
- `index.html` · links `/favicon.ico` (32x32) and `/favicon.png` as formvis does, instead of `/icon.png`.
- Export dialog: glTF and GLB start at metres because `export.formats` now says so (`native/HASHOUT_LOG.md`, v0.25.0). A unit the person chose before is kept in localStorage (`export.gltf`, `export.glb`) and still wins.
- Menus: the catalogue marks no command as internal (no flag, no naming rule, no group: `pattern.begin_drag` and `pattern.end_drag` are ordinary `pattern` commands), so File, Export (35 rows) and Pattern (61 rows) are left as they are.

### Pattern and 3D view, workspace panels (polish)

- `pattern-view/tools-edit.ts` :: `EditTool.keydown` · Delete or Backspace with no point selected says "Select a point first" and stops there. It used to fall through to `view.ts` `keydown`, which deletes the selected panel; deleting a panel belongs to Select.
- `state.ts` :: `Selection.prune` · takes the outline length per panel (`layout.ts` passes it on every document change) and drops `pts` and `seg` when their panel's point count changed, because they are indices and would land on another point after undo, redo or an inserted point.
- `pattern-view/render-sew.ts` :: `mmText(v, unit)` · seam mismatch follows the unit preference in the canvas tags, the Seam Check list (`loadPrefs().units`, repainted from `syncPrefs`) and the inspector, as the lengths beside it do.
- `pattern-view/render.ts` :: `Scene.under` · style lines are drawn with every tool, as the 3D View always shows them; `view.ts` passes `StyleLineTool.lines` while another tool is active.
- `doc.ts` :: `pullSew` · a failed `sew.list` or `sew.check` sets `SewSnapshot.failed`; Seam Check then says the request failed instead of "All seams match". A canvas pick repaints the Seam Check rows (`selected` hook).
- `anim/dock.ts` :: `setOpen` · the open timeline puts `has-timeline` on the 3D pane. `anim.css` sets `--ff-an-lift: 274px` there; the map legend, the measure hint and the panel state strip (`drape/ui.ts`, now 34 px up and clear of the legend) add it to their `bottom`, and `View3D.paintOverlay` lifts the statistics line by the dock's height. A class `MutationObserver` on the pane repaints the overlay.
- `anim/dock.ts` · while a recording runs, the ruler does not scrub (`scrub` hook), the Frame field is disabled (`paintAvail`) and the play shortcut does nothing; `cache.frame` would only show cached frames and the view jumped.
- Drawers: Fabric, Body, Trims, Output and Stage close every other open `.ff-body > aside[data-k]` through its close button when they open, as Library and Objects already did; Body and Fabric only closed each other, and Stage covered Body. Esc closes the open drawer (`layout.ts` keydown, after menus and popovers); `viz/pop.ts` marks its Esc handled so it does not also reach the drawer or the tool.
- `viz/maps.ts` · a map choice closes the Maps menu. `viz/pop.ts` returns focus to its button when it closes with focus inside, and buttons in the 3D toolbar, the drape bar, the tool rail and the timeline blur only after a pointer click (`e.detail > 0`), so a keyboard user keeps their place.
- `viz/capture.ts` :: `videoType` · Video and Turntable record WebM when `MediaRecorder` supports it and MP4 otherwise; the Type labels, the path extension and the file follow. `capture.write` accepts png, webm and gif only, so an MP4 is written through the engine only when its schema lists `mp4` (`writesKind`), and is downloaded otherwise. Not run on WebKit.
- `output/bake.ts` :: `GameReadyDialog` · the Quads toggle is gone: `gameready.run` documents `quads` as reserved and always triangulates.
- `offline.ts` · `engineDown`, `onEngineDown`, `emptyLabel`: empty notes in the Body, Fabric and Trims lists, the Objects list, the Library, Snapshots and Components tabs, the timeline hint and the new Pattern View note ("No panels yet ...") say the engine is not running while the link is down, instead of "nothing here" or "not provided by this engine".
- `shell/home/second.ts` · when `/api/recovery` reports `autosave_active: false` with `autosave_off_reason: "second_service"` (contract in the service's v0.25.0 entry), a toast that does not time out says this window keeps no autosave and no crash recovery. Checked at start and on every link-up; the toast root sits above Home.

### Engine link, status bar units, focus (polish leftovers)

- `dock/sse.ts` :: `recheckStreams` · every open event stream registers an `again` hook once its response is open. `recheckStreams()` aborts each open stream and reconnects it at once without reporting an error; only a failed reconnect goes through `retry` and `handlers.error`, so a healthy service shows no "Engine offline" blink. Called on a network-level failure (a `TypeError` from `fetch`) by `engine/service.ts` `call` and `catalogue` and by `dock/agent-link.ts` `tick` (polls every 900 ms). Why: a service stopped without a graceful limit (plain `uvicorn` without `--timeout-graceful-shutdown`) closes its listening socket but stays in "Waiting for connections to close" and keeps both `/api/events` streams alive with `: ping` every 15 s, so the stream never ended, the link stayed `up` and no reconnect happened while every other request was refused. A clean end of the stream was already link down (`events.ts` `error`, `seen`). Dropping the streams also lets that service finish stopping.
- `app.ts` :: `load` · a catalogue that arrives after the link went down is dropped (`loaded` stays false, the next link-up loads again). On a slow network the start-up catalogue (several hundred kB, fetched four times at start) could land after the service stopped and set the status bar back to "Engine 0.24.0" while it was offline.
- `app.ts` :: `refresh` · the status bar's units button shows the unit preference (`loadPrefs().units`), set once at start and on every refresh, as the Pattern View and the inspector do; it showed the document's `view.units` (mm for every sample) while the preference was cm or in.
- Body, Fabric and Output drawer buttons and the bar's drawer toggles (Fabric, Body, Trims, Timeline, Output, UV Editor, Library, Objects) blur only after a pointer click (`e.detail > 0`), the rule the 3D toolbar already uses. The blur is not only there to hide the ring: a button left focused by a mouse click takes Space (`layout.ts` `locked`, so the global Space runs Simulate instead of the pattern pan) and Enter. A keyboard activation keeps focus and shows the `:focus-visible` ring. The two Enter-commits in text fields (`fabric/look.ts`, `fabric/physical.ts`) are unchanged.

### Verification pass (polish check)

- `app.ts` :: the window's shortcut `keydown` · Space on a button that has keyboard focus (`:focus-visible`) is left to the button. Before, the global Simulate shortcut (`sim.toggle`, key Space) took it and called `preventDefault`, so Space on a keyboard-focused drawer toggle or tool started the simulation and did not press the button; with focus now kept after a keyboard activation this happened on the second press. A button focused by a mouse click does not match `:focus-visible`, so Space there still runs Simulate as before. `layout.ts` `locked` and `View3D.onKey` already leave Space on a button alone.

### After a restore, Home placeholders, the Pattern View fit, same-value writes (polish after a restore)

- `app.ts` :: `offerRecovery` callback; `shell/recovery.ts` :: `RecoveryDialog.act`; `workspace/library/api.ts` :: `recoveryAct` · the restore reply's `path` (service v0.25.0, "A restore keeps the document's path") becomes `session.path` with the unsaved mark, so the title shows the real name and Save writes to that file with no dialog. `null` keeps "Untitled". `recoveryAct` now returns the reply object, or `null` on failure.
- `shell/recovery.ts` :: `report`, `reason`, `showReport` · the window builds its own translated sentences from the reply's numbers (`replayed`, `logged`, `skipped_count`, `failed_count`, `truncated`, `base`) instead of showing the service's English `summary`. Everything back: the dialog closes and the toast says what came back. Anything left out, failed or a full journal: the same dialog stays open, its three lines say what came back, what did not and why, Discard hides and Restore becomes Close. The service's reason strings map by their start to translated reasons; an engine error message is shown as the engine wrote it, as everywhere else. The body keeps its height as a minimum and a taller report grows downward (`margin-top` equal to the growth keeps the centred frame's top still).
- `shell/home/index.ts` :: `TPL_SKELETON`, `tplReady` · the Start and Templates grids start with four `is-skeleton` cards (the Recent skeleton's markup, keys `__skeleton0` to `__skeleton3` so `patchList` removes each); `renderTemplates` leaves the grids alone until the first template load has answered, then patches them, also when that load failed, so a failure shows an empty grid rather than placeholders forever. Throttled: 212 by 173 placeholders for about 18 s, then real cards of the same size, no layout shift.
- `workspace/pattern-view/view.ts` :: `moved`, `fitLater`, `fitUnlessMoved`, `fitIfPending`; `workspace/layout.ts` · a fit is pending from the start and runs on the first document the engine answers with (the doc store's change event), once the pane has a size; it runs again when the engine link comes up. A pan or wheel zoom in this page session sets `moved` and stops these automatic fits; Open, New, a sample, DXF as new and a restore still fit as before. An empty first document gives the old default camera, so the first drawn panel does not refit.
- `ui/dom.ts` :: `setAttr` · writes an attribute only when its value differs. Used where the throttled workspace walk found the most same-value writes: `workspace/anim/dock.ts` `paintTip` (aria-label of every timeline button on every repaint) and `paintLayers` (`aria-pressed` of Auto Key, runs every frame); `ui/settings.ts` `markSection` (`aria-selected` of the rail tabs on scroll); `workspace/fabric/look.ts` surface tabs and `workspace/fabric/list.ts` rows (`aria-selected`). Also `workspace/anim/dock.ts` `paintAvail` leaves Cancel and the layer buttons (Cut, Merge, Lock, Move Earlier, Move Later) to `paintLayers` and the line after the loop, which always run after it and set them; before, each repaint enabled them and disabled them again. `workspace/fabric/look.ts` map Pick and Clear use `setDisabled`; `workspace/output/uv.ts` Layout title uses `setTitle`; `shell/home/index.ts` `fill` uses `setClass` for `is-blank`. Same walk, same machine: 150 same-value writes before, 21 after. Left: Remove Key's `disabled` (6, `paintAvail` then `paintKeys`; the job path calls `paintAvail` without `paintKeys`, so skipping it there would leave the button enabled during a recording) and single writes in dialogs and pop-ups.

## v0.24.0 · 2026-11-15

Pre-release audit of the shell, the engine link and the Output, Drape, Fabric and Library panels.

- Menus: `shell/menus.ts` `group()` compares group names without case. The catalogue's groups are lower-case (`sim`, `sew`, `fabric`) and the window's own are capitalised (`Sew`, `Window`), so Drape and Fabric showed only the placeholder and Sew, Body, Pattern, View, Help and File, Export lacked their engine commands. The Output menu also lists `app.output.topology`.
- Save: `runEngine` in `app.ts` fills `file.save` with the session path before the required-argument check, so Save on a named project saves at once. Before, the Exact Input path dialog opened on every save and the fill-in line below it could never run.
- Unsaved changes: `file.new` and `file.open` (Open Recent included) ask through `askSaveChanges` when the session is dirty. Save runs `file.save` first and stops if that save did not finish. The dialog and its strings existed in every table but nothing called them. Exit and the window close button still do not ask.
- Home: plain keys do not run commands while the Home screen covers the workspace; combos with the modifier still do.
- Engine link: `engine/events.ts` reports `down` for any stream failure after the first successful open (`seen`), so a dead service reads Engine offline and blocks engine commands with the reason. Before the first open a network failure is still `connecting` (the service is starting). `load()` in `app.ts` skips commands that are already registered, because `Commands.add` throws on a duplicate and the reconnect path lost its refresh, recent list and Home update.
- Layouts: `ff:dock-toggle` takes an optional `on`. `WorkspaceLayouts.apply` sends the wanted state; `toggleKind('shell')` opened a shell tab instead of closing the dock when a Claude Code or Chat tab was in front.
- Save As uses the native save dialog in the desktop app (`pickPath`), the name prompt only in the browser. The engine refuses relative paths.
- Export dialog: `show(format, thickness)` sets every format's Thickness toggle from the project's `output.thickness.enabled`. The toggle used to start off and was sent as `thickness: false`, which overrode Output, Thickness.
- Bake dialog: map toggles repaint the Bake button (it stayed disabled after the last map was re-ticked, and enabled with none). UV Editor: Padding and Tiles limits follow `uv.layout` (0 to 0.1, 1 to 16).
- Fabric drawer: `loadPresets` runs again when the drawer opens, since the one call at mount fails for good when the engine is not up yet.
- Settings file: `settings-io.ts` asks for a native path only when `@tauri-apps/plugin-fs` loads. It is not a dependency, so the desktop app showed a save dialog and then ignored the chosen path (export), or showed two pickers (import). Until the shell can write a file, export is a download and import a file input everywhere.
- Number fields: `ui/number-fields.ts` `leave` compares the tidied text with the text held at focus as numbers too, so a field shown as `12.50` no longer fires `change` on a plain focus and blur (it wrote the rounded value back and could add an undo step). An emptied field that held nothing still fires when a number is typed.
- Exact Input: a number whose schema says `unit: mm` is shown and read in the unit preference (`loadPrefs().units`, the same factors as `workspace/state.ts` `UNIT_MM`) and sent to the engine in millimetres; limits clamp after the conversion. The label carries the unit, `Width (in)`. Array and point fields stay raw JSON in millimetres and say `(mm)`. `JsonSchema` gained `unit`. A second `ask` while the dialog is open settles the first promise with `null` instead of leaving it pending.
- Mesh stream: `EngineStream` reconnects by itself when its socket closes without `close()` (0.5 s doubling to 8 s, eight tries, only while something is subscribed), and `connect()` does not open a second socket when two connects overlap. `onStreamState` is exported. The service resends every topology to a new client, so `MeshFeed` needs no change; its own retry on link-up still covers a service that was down longer.
- Open: Clear Recent only clears the window's copy (the service has no route to clear `recent.json`); sixteen preferences are stored but read by nothing (list in the audit report); `ChatPane.load` does not retry after a failed first load.

### Pattern View, trims and inspector (release audit)

- `pattern-view/view.ts` `onDown` blurs the focused element before the tool sees the press. Inspector fields commit on blur against the current selection, and the press changes the selection before the browser blurs, so without this a typed value lands on the object just clicked.
- `view.ts` `onDown`, `onMove`, `onUp` ignore non-primary pointers. A second finger used to call the tool's `down` twice for one release pair, leaving `DocStore.holding` above zero; `refresh()` returns early while a hold is open, so the window stopped following the engine until reload.
- `render.ts` `drawScene`: the hover hit is kept across document changes, so its segment index can be gone; an absent segment draws nothing instead of throwing mid-frame.
- `render.ts` `rulers`: label decimals follow the major step (0.1 mm above 20000 % zoom), not a fixed count per unit.
- `layout.ts` `refreshed`: `sample.load` is a new document like `file.new` and `file.open` (ids restart at 1); a new document cancels the active tool, then clears the selection.
- `doc.ts` `cancelTrial` puts the base garment back and reloads it into the preview engine before the refresh. A trial begun before that refresh lands would otherwise take the cancelled result as its base (Match Lengths cancels a trial on every hover change).
- `tools-edit.ts` `SelectTool.up`: an unchanged move, scale or rotate cancels the preview; the engine records `pattern.transform` with scale 1 or translate 0 as an undo step.
- `tools-create.ts` `InnerShapeTool.finish` uses `ellipseMode` (Alt at press), the flag the preview draws with.
- `inspector/index.ts` `commit`: Position sends the edited coordinate from the field and the other one from the document, not from its rounded display text.
- `trims/panel.ts` `optional`: the four "empty means the style's value" inputs carry `data-plain`. `ui/number-fields.ts` restores the old value when a field is emptied and stops the native `change`, so they could not be cleared.
- Open: a point selection is an index and can move to another point after undo. (The number field and Exact Input items this review left open are closed in the notes above.)

### 3D View, maps, timeline and body drawer (release audit)

- Maps: the stream decides which analysis map is shown. `Garment.viz.map` is never written by the engine, so `Maps.sync` only acts on a real name; before, every document refresh read the missing field as "none" and switched the map off (legend hidden, colours cleared) while the engine kept it. `MeshFeed.scalarFrame` forwards the engine's empty clear frame, `Maps.onScalars` turns the map off on it, adopts the map the frames carry when it differs (not while a menu choice is in flight) and resets range and unit. `VizUi` also turns it off on `file.new` and `file.open`, where the old panels can be unsubscribed before their clear frames arrive.
- Topology tool: its capture `pointerup` marks a click with `preventDefault()` and `View3D.onUp` skips its own click when the event is prevented. It used `stopImmediatePropagation()`, so `View3D` never ended the drag it began on `pointerdown` and the camera orbited on every later mouse move.
- Context loss: `View3D` prevents `webglcontextlost` and pauses painting (`lost`). `restore()` runs `Scene.forget()` (null handles, all dirty), `setup()` again (programs and the mesh, extras, stage, fur and style renderers; seams and pins carried over), `Maps.contextRestored()` (ramp textures) and `applyViz()` with a cleared `vizSource` so stage, fur and style lines are pushed again.
- Stream: `MeshFeed.streaming` only guards the connect in flight. It used to stay true after the first connect, so a closed socket was never reopened. (`engine/stream.ts` now reconnects on its own, see above.)
- Style lines: `pullStyle` reschedules when a request is in flight instead of dropping the pull (the late reply of the previous document could stay drawn).
- Overlay: the statistics line sets its font and bottom baseline on every paint; tapes and measures set a middle baseline on the same context.
- Stage: a failed HDRI load only marks `failed` when it is still the selected image, as the backdrop already did.
- Capture: recorder tracks are stopped in `onstop`. No supported WebM type raises "This window cannot record video" before recording, because `capture.write` refuses anything else. The recorder is created before `anim.play`.
- Timeline: `paintList` drops picked cache ids that are no longer layers (Merge Layers stayed enabled on merged-away ids) and re-clamps the row scroll.
- Body drawer: `BodyStore.originals` is cleared on `file.new`, `file.open` and `sample.load`. Body ids restart at 1 per document, so Reset to original could apply the previous document's measurements.
- Open: a stream-sourced panel keeps its last mesh when the engine sends an empty topology; fabric textures are cached for the life of the window; a pixel ratio change with no resize keeps the old backing store; the timeline shortcuts have no default keys.

### Splash, menu bar, quit and opened files (family shell)

- Splash: `public/splash.html` is formvis's file with formfabric's name in it, nothing else differs. The bar uses the family gradient `linear-gradient(90deg, #3A7BFF, #9A5CFF 30%, #FF4F7A 62%, #FFAE2B)`; removed: `linear-gradient(90deg, #f97316, #ffb347 33%, #e8577a 66%, #7c5cff)`. The picture is `favicon.png`, a byte copy of `public/icon.png` (which `index.html` still links). Removed after the `<img>`: the commented-out inline SVG of the weave mark (`<!-- <svg ... id="splash-tile" ... id="splash-warp" ... id="splash-weft" ... </svg> -->`, 3798 bytes, last present in commit 4906f93). `public/favicon.ico` is a byte copy of `src-tauri/icons/icon.ico`, as formvis keeps one, so the browser's own `/favicon.ico` request for the splash page is answered.
- One menu list: `shell/menus.ts` `buildMenus` feeds the window bar, the system menu bar and the palette. File starts with `app.home`, then New, Open, Open Recent, Open Sample, Save, Save As, Export, `app.close` (Close Garment) and Exit last. Help is Keyboard Shortcuts, Demos (a submenu of `ui/demos.ts` `DEMOS` commands, present only when the list has entries), Source Code, Report an Issue, the engine's System group, About last. Removed: `'-', 'app.home'` after Save As in File, `'-', 'app.home'` at the end of Window, `'app.about'` as the first Help item.
- `menus.ts` Pattern: clears `used` and marks the three Edit rows (`EDIT_PATTERN`: Duplicate, Delete Panels, Rename Panel) itself. Before, `used` came from whichever menu was built last, so the window bar showed those rows in Pattern after File but not after Edit, while the system menu (built in order) never did.
- `ui/native-menu.ts` runs only on macOS in the desktop app with View, Menus in Window off (`wantsNativeMenus`). It is the formdeck form on top of formvis's `text_edit` route:
  - `accelerator()`: a combo becomes a menu key only with Mod, Ctrl or Alt, or when it is F1 to F12. The tool keys (A, Z, B, Space, Delete) stay with the page, because a bare key as a menu key equivalent swallows typing. `,` `.` `/` `;` `'` `\` and the arrows have names now, so Preferences shows Cmd+, and Keyboard Shortcuts Cmd+/.
  - `SYSTEM_KEYS` (Cmd+H, Cmd+Q, Cmd+M, Ctrl+Cmd+F) belong to the system items; an app row with one of them, or with a key an earlier row already has, is built without it (File, Exit has Mod+Q in the desktop app and so shows no key beside the system Quit). Home keeps Alt+Cmd+H although Hide Others has it too: decision of record, the same as formdeck. Which of the two macOS runs was not tried.
  - The application menu is built here, not from the list: About (name, version, copyright, licence), Preferences, Services, Hide, Hide Others, Show All, Quit. The five system rows of Window (Minimize, Maximize, Full Screen, Bring All to Front) are appended to the app's own Window menu, which is registered as the Windows menu; Help is registered as the Help menu.
  - `shapeOf` is the menu's signature: every row's id, translated label, first key and whether it ticks. `refresh()` (150 ms after a pointer or key release, a focus change, a session change, an engine change) rebuilds when the signature changed, otherwise sends only the enabled states that changed and the tick states. This is what rebuilds Open Recent, the layout slots, the catalogue menus once the engine answers, a changed shortcut and a changed language.
  - A build touches the page in one place: the constructor sets `has-native-menus` on `<html>` before the first paint, so the window bar is never shown and then hidden; a failed first build removes it and the window bar is the menu.
  - Rows are disabled while a dialog is open (the six Edit rows stay on in a text field), and a row's action does nothing then.
  - `claim(id, by)`: a command reached by its key and by its menu row within 100 ms runs once. `app.ts` asks it before the page's key handler runs a command.
- `shell/desktop.ts` `listenShell`: listens for `formfabric://open-paths` and `formfabric://exit-requested`, then calls `take_pending_opens`, in that order so no file is lost. That call is also what tells the shell the page can answer a quit. `app.ts` opens the first `.ffbrc` of a batch through `file.open` once the catalogue is loaded (`engineReady`), so the Unsaved Changes question applies.
- Quit: `exitApp` in `app.ts` answers the shell's quit and close requests. Unsaved work asks through `askSaveChanges`; Save runs `file.save` and stops the quit when that save did not finish; Cancel answers `confirm_exit` with `cancel`; otherwise `confirm_exit` lets the app exit. File, Exit closes the window, which raises the same request. In a browser the page cannot close itself, so Exit sends `ff:exit` to the same `exitApp`, which asks about unsaved work, tries `window.close()` and says "This page can be closed now." Removed from `app.exit`: `else window.close();`, which did nothing in a tab the person opened.
- `app.menus_in_window` says in a notice that the change applies from the next start.
- Settings file: left as a download and a file input. formvis has no file system plugin either: its settings-like files (tool presets, connection notes) are downloads, and its exports go through the save panel and an engine route.
- Not run: anything on macOS. The system menu tree was built in headless Chrome with the Tauri IPC replaced by a recorder.

### Home (formdeck's structure, the garment library)

- `shell/home/index.ts` replaces `shell/home.ts` (the two-page formvis clone, class `HomeScreen`), by the owner's requirement that Home be structured as formdeck's. The old file and its rules in `styles/shell.css` (`.cv-home*` on `--z-launcher`, `.cv-home-action`, `.cv-home-size*`, `.cv-home-file*`, `.cv-home-none`, the 900 px rule) are removed, and so is the token `--z-launcher`. `.cv-key` stays in `shell.css`: the Shortcuts dialog uses it.
- Structure: one template string built once in the `Home` constructor, the same regions and class suffixes as `formdeck/frontend/src/ui/home.ts` under the `cv-` prefix. Lists are patched by key (`patchList`), text by `setText`, visibility by `hidden`; nothing in the live tree is rebuilt. The card menu's rows are rebuilt only while the menu is hidden.
- Layer: `.cv-home` is `position: fixed; inset: 0` on `--z-home`. `body.nm-home` is set while it is open and raises `#overlay-root` to `--z-home-modal` and `#toast-root` to `--z-home-preview` (rules in `shell/home/home.css`), so dialogs, the card menu and notices sit above it. Inside Home only `--z-raised` is used (edge buttons, confirm bar, drag strip).
- Title bar allowances formdeck does not need (it keeps the system title bar): under `.has-overlay-titlebar` (macOS) the side nav starts below the traffic lights and `.cv-home-drag` is a drag strip over the top row; under `.has-caption-buttons` (Custom Title Bar on Windows and Linux) Home starts below the title bar so Minimize, Maximize and Close stay reachable. Neither was run here.
- Visibility is formvis's one rule in `Home.sync()`: `forced || (!hasGarment && (hadGarment ? homeOnLastClose : autoShowHome))`. `app.ts` calls it from `title()` on every session change. `file.new` and `file.open` call `home.leave()`, since this app replaces its one document and the garment count never rises a second time. Esc closes the card menu, then the confirm bar, then the selection, and leaves Home only when a garment is open.
- `Session.close()` is new: there was no way for the document to close, so `homeOnLastClose` could never apply. `app.close` (Close Garment) asks about unsaved changes, sends `file.new` straight to the engine and closes the session. Trashing the open garment from Home closes it the same way, or, when it has unsaved changes, keeps it open as an untitled dirty document so nothing is lost.
- Previews: `shell/home/thumb.ts` asks the 3D View for a 480 by 270 capture (`ff:thumb`, the event the library drawer already uses) after every successful `file.save` and `PUT`s it as WebP (JPEG where WebP encoding is missing). Home loads previews through `fetch` and a blob URL, not `<img src>`, because the window token header (`x-ffb-window`) can only be added to `fetch`. A preview is decoded off screen and then swapped into a box that already has its 16:9 size; the old picture stays until then.
- Default save place: `savePlace()` in `app.ts` gives Save and Save As the library's next free `Untitled.ffbrc` path when the document has no path.
- Language: Home wraps `t()` to remember which English string each shown string came from, records the static text nodes and attributes once (`collect`), and rewrites them in `relabel()` on `onLanguage`. Dynamic text goes through the render functions again.
- Confirm bar: `position: sticky` with a fixed height and negative margins that cancel it, so asking moves nothing on the page (formdeck's bar pushes the page down by its height). The Start row of recents hides its second row with `grid-auto-rows: 0`; the bottom padding of that grid is 0 so the hidden row does not show a 6 px sliver at narrow widths.
- Brand: `.cv-brand` in the title bar (`titlebar.ts`) runs `app.home` on click, Enter and Space. It shares grid column 1 with the menu bar (`grid-row: 1` on the bar's children, a start margin on the menu bar) so the title bar's column template is untouched.
- `app.home` is spelled `Mod+Alt+H` in `keys`: `comboOf` in `ui/commands.ts` writes Mod first, where formdeck writes `Alt+Mod+H` for the same chord.
- Four icons added to `ui/icons.ts` from formdeck's icon module (same family, Apache-2.0): `upload`, `template`, `inbox`, `cloudOff`.
- Layer table: `--z-raised` 8 (inside Home), `--z-menu` 200 (`#overlay-root` normally), `--z-modal` 1000 (dialog scrim, inside the overlay root), `--z-toast` 3000 (`#toast-root` normally), `--z-home` 10000 (Home), `--z-home-modal` 10010 (`#overlay-root` while Home is open), `--z-home-preview` 10100 (`#toast-root` while Home is open).
- Open: Home has no rename (decision H9); the undo notice is the app's own toast with an action, bottom right, not formdeck's centred toast; file drops in the desktop app arrive through Tauri's `onDragDropEvent` with real paths and were not run.

### Verification pass (release check)

- `app.ts` :: `adopt` · when no garment is open in the window and the engine holds panels, the window takes the document as an unsaved untitled garment (`session.set(null, true)`). It runs once the catalogue has loaded and after every engine change that is not a `file.` command. Two cases needed it: an agent drafting over MCP while Home is up (the window stayed on Home, which covers everything, and Esc does nothing there without a garment), and a page reload in web mode while the engine still held unsaved work (Home showed, and the first Open or template replaced the work without asking). Settings-only changes (`view.units`, `wind.set`) on an empty document do not leave Home, because the test is the panel count. The path is not known to the window after a reload, so the garment reads Untitled and Save offers the library folder; an engine command that reports the open path would be the fuller fix.
- `ui/menu.ts` :: `PopupMenu.openAt`, `MenuBar.open` · a menu opened from the bar takes `below = true`: it keeps its top under the bar and gets a `max-height` down to the window edge (formdeck's `openAt`). With the engine commands back in the menus Pattern has 61 rows; the clamp that keeps a pointer menu inside the window moved it up to the top edge, over the menu bar and its own button, so the neighbouring menus could not be reached by hover. Context menus keep the old placement.
- `workspace/view3d/index.ts` :: `onDown`, `onMove`, `onUp` · only the primary pointer is handled, as in `pattern-view/view.ts`. A second finger replaced `this.drag`, so the first finger's release never ended its grab, gizmo drag or box tool. Not run with two real pointers (headless Chrome did not deliver a touch grab); the mouse paths were re-run.
- `engine/wasm.ts` :: `WasmEngine.send`, `stopped` · a trap or any other throw inside the preview engine marks it stopped and every call answers `{ ok: false, code: internal }` from then on. The module is built with panic abort and holds its session in a `RefCell`, so after one trap every later call trapped too, and `pattern.snap` is called on pointer moves without a `try`. `pattern-view/view.ts` `snap` falls back to grid snapping while `stopped`. Run in Node against the built wasm with a throwing export.
- `shell/home/home.css` · three rules brought to formdeck's values after a side-by-side measurement: `.cv-home-brand` (gap 10, padding 0 8px, no clipping, so "by nodemode.ai" wraps as in formdeck and is no longer cut), `.cv-badge.is-count` (padding 0 5px, line-height 1) and the glow on the hero's primary button. Left as they are on purpose: the active chip keeps weight 400 (500 would move the chips beside it), the confirm bar sits in the top padding (formdeck's pushes the page down by 58 px).

## v0.23.1 · 2026-11-02

From the owner's test run (fixes made on v0.18.0, landed by hand on v0.23.0).

- 3D View fabric maps: the engine's albedo key is `base_color`; `firstMap` checks it first, then `base`, `color`, `albedo`, `diffuse`.
- `map_scale` arrives as `[width_mm, height_mm]`; `scaleOf` takes a number or a pair and uses the first value, because the shader has one scale uniform. A non-uniform repeat needs a second uniform.
- Local texture paths load through `previewUrl` (`fabric/files.ts`: fetch with the window token, cached blob URL); only `http(s):`, `blob:` and `data:` go straight to `img.src`. An `img` request never carries the token.
- Textures upload without `UNPACK_FLIP_Y`: texture v = 0 is the image's top row and lands at local y = `map_offset.y` (the hem), so an image meant to read upright on a panel is stored upside down.
- App icon: `assets/formfabric_icon.png` is now the owner's 1024 px icon (the pinwheel of eight gradient bars on a dark rounded tile) and `src-tauri/icons/*` were regenerated from it with `tauri icon`; `assets/icon.svg` and `frontend/public/icon.svg` (the window favicon) draw the same bars on a 32 unit grid; `ui/brand.ts` already drew this pinwheel and is unchanged.
- Open: prints (`print.add`, `print.set`) draw only in the Pattern View (`trims/draw.ts`), not in 3D. A Vite hot reload reloads the document from the autosave, which can be older than the live engine document and clears undo.

## v0.19.0 to v0.23.0 · 2026-11-02

- Title bar: Animate and Render tabs are live and every tab applies its layout through `applyWorkspace` (the dock is untouched); UV and Retopo are Window items that dispatch `ffb:output-section` on `window` with `detail.section`, which `output/panel.ts` handles without re-rendering the drawer.
- Object Browser (`workspace/objects/`): keyed row patching, fixed 28 px rows and a 58 px tab row, a sort header, state in guarded localStorage. The Graphic tab lists and selects only.
- Stage background picker and the `stagegl.ts` backdrop quad. Capture: eight camera presets, 2 to 10 views, combine or separate files, ppi written into the PNG pHYs chunk (`viz/png.ts`), transparent GIF turntable (`viz/gif.ts`). The Capture dialog body must keep one height across kinds.
- Timeline: auto key toggle, cut, merge (shift or ctrl click selects several), lock and move by N frames in a fixed 30 px row (dock height 274 px), a lock chip reserved in every cache row, pin tracks as rows; playback and scrub resolve layers in the engine by omitting the id.
- Export dialog: weld choice with a reserved 88 px panel picker for weld_selected (persisted as `export.weldPanels`); bake dialog map toggles and a Seam fill row (`bake.opts`).
- `view3d/topology.ts` draws on its own overlay canvas patched by a signature; `View3D` gains `toolbox()` and `pickAt()`. The Output drawer has a Retopo section.
- Trims: finish kind Skive with a Percent row and `--trim-skive`; the drape settings popup has an Untangle select. Body import lists `abc`.

## v0.7.0 to v0.9.0 · 2026-11-01

### Sewing in the Pattern View

- Tool keys: Seam V, Free Seam Shift+V, Edit Seam B, Tack Shift+T, Pleats Shift+P, Gather Shift+G. Each is one undo step with Exact Input.
- Each seam's sides share one colour; connectors at intervals show direction, so a reversed seam is visible. Mismatches reported by `sew.check` turn the seam a warning colour with the difference in mm.
- Seam Side is shown only as a per-end property in the Inspector: no guideline arrow plus operation symbol, per PATENT_NOTES. Multi-seam orientation is a heuristic; Flip fixes crossed connectors.
- Seam Check is a tool-strip toggle; there is no View menu entry yet.

### Drape in the 3D View

- The Drape bar sits in the 3D HUD (`workspace/drape/`, `view3d/extras.ts`): on and off, Step, Reset, presets, Apply, Pin, Clear pins, frame time and vertex count. Space toggles the drape when the pointer is over the 3D pane in the split layout.
- `draping` is `sim.state.running`; the gizmo hides while draping. Grab sends `sim.grab` on a camera-facing plane through the picked point and releases on pointer up. P toggles the pin tool in the 3D pane.
- Bodies render from stream objects 1,000,000 and up with a skin material, unpickable but occluding. Seam lines come from `sew.list` plus mesh boundary parameters and hide while draping. Pin and seam overlays lag the document by about 200 ms (up to 2 s at 4x throttling).
- Alt+1 to Alt+4 pick the view style; Alt+0 toggles stats. Preferences push substeps, gravity and thickness to `sim.settings`; the solver-device row stays disabled.
- The Drape workspace tab only highlights: `onWorkspace` in `app.ts` does nothing yet.

### Body drawer

- `workspace/body/` follows the Fabric drawer pattern and mounts from `layout.ts`. It holds the library (MPFB2 female and male with the CC0 line, Basic Form), import, the Body Editor (measurements with arithmetic, Reset to original), poses and joint sliders committed on release, collision, placement points with Place, and Auto Seam proposals with Apply.
- `body.set` and the other body commands take `id`, not `body`.

### Service guard

- The window must be served from the configured frontend port (`FFB_FRONTEND_PORT`); other origins get 403 from engine calls. Test harnesses must set it.

## v0.4.0 to v0.6.0 · 2026-11-01

### Shell

- Menus build their items once per command list and refresh labels, enabled state, checks and key hints in place; a second open adds or removes no node.
- `StatusBar.setCursor(text)` owns the fixed-width cursor slot; the workspace no longer inserts its own.
- Exact Input still builds its rows on each open, while the dialog is hidden, then shows it once: nothing visible moves.
- `setup/i18n.py` reads the TypeScript tables and every `t()` and `tn()` call; strings reached only through arrays (fabric labels and help) are not found by `extract` but are in every table.
- JetBrains Mono 400 is preloaded, but `--font-mono` lists `ui-monospace` first, so macOS uses the system monospace.

### Pattern View, Phase 4 tools

- Keys: Seam Allowance W, Notch K (1 to 4 pick the kind), Grain Line G, Dart D (Alt for an internal dart), Slash and Spread X, Mirror Edit Y, Instances J (digits set the count), Unfold O, Spiral Q, Trace T, Match Lengths H.
- Trial protocol for the new tools (`DocStore.beginTrial`, `trial`, `commitTrial`, `cancelTrial`): every pointer move reloads the gesture-start garment into wasm, runs the command, and reuses unchanged panel objects; release sends the command once to the service.
- Drag start: Select and Edit open the wasm drag group on pointer down and prime the caches with a zero-delta command and a snap call (`DocStore.prime`); a click without a drag discards the group. This took the first pointer move from up to 42 ms to under 3 ms at 4x throttling.
- Derived panels draw dimmed with a link badge; edits on them are snapped back by the engine.

### 3D View

- Data priority per panel: stream frames, then `mesh.get`, then a local outline triangulation built synchronously on every garment change, so the view is never empty.
- Colours: `look.front.color` and `look.back.color` (the back defaults to the front times 0.72; no fabric is grey 0.62, 0.63, 0.66), with the active colorway's colour overriding the front.
- Shading: linear lighting with view-space key, fill and rim lights, hemisphere ambient, a highlight scaled by one minus roughness and a Fresnel sheen; two-sided through `gl_FrontFacing`. The light idea follows formsculpt's renderer; the shader is new.
- The camera refits when the panel count changes until the user moves it. Keys 1 to 5, 0 and F act only when the pointer is over the 3D pane, the canvas has focus or the layout is 3D.
- Selection is the workspace's shared `Selection`: a click sets it, Shift toggles, an empty click clears.
- The gizmo appears for exactly one selected panel and only when `arrange.place` exists; dragging writes the panel's model matrix (no buffer copy) and commits once on release.
- Measured on this sandbox's software GL at 4x throttling with a 200k-triangle panel: feed 152 ms, in-place position update 54 ms, a completed frame about 1 s. No GPU was available to measure the 60 fps target.

### Fabric panel

- A drawer docked against the Inspector (`right: var(--ff-insp)`, 344 px, `z-index: var(--z-chrome)`), opened from the workspace bar; the Inspector never moves.
- Every edit is one `fabric.*` or `colorway.*` command. Store refreshes are coalesced, skipped while the drawer is closed and replayed when it opens. Lists have reserved heights and update through keyed patching.
- Map picking is desktop-only (native dialog); the path field works in both builds.
- No shortcut or palette entry opens the drawer yet.

## v0.1.0 to v0.3.0 · 2026-10-31

### Shell (from formvis 1.98.6, Apache-2.0, nodemode-ai)

- Copied and renamed from formvis: `ui/dom.ts`, `dialogs.ts`, `dialog-history.ts`, `dropdown.ts`, `number-fields.ts`, `toast.ts`, `menu.ts`, `native-menu.ts`, `caption-buttons.ts`, `commands.ts`, `command-palette.ts`, `i18n.ts`. The copies were produced by a throwaway script that stripped comments and renamed formvis to formfabric while creating the new files; later changes are hand edits.
- `ui/themes.ts` holds formvis's 28-theme table verbatim; `index.html` holds the same table as `PROFILES` for the pre-paint theme boot. The two must agree: change both in the same commit.
- Keys: `cv-` class prefix and token names stay family-wide; the shared `nodemode_ui_settings` and `nm:ui-settings-changed` stay; app keys are `formfabric.*` (`formfabric.layout` holds the layout string, `formfabric.layout.split` the splitter ratio, `formfabric.dock`, `formfabric.recent`).
- Language tables: `ui/i18n/[lang].ts`, 30 languages. `es-MX` and `fr-CA` carry only regional deltas over their parents.
- Menus come from commands. Pattern, Sew, Drape, Body, Fabric, View and Help append every catalogue command of their group; an empty menu shows a disabled entry. Window-only commands use `app.*` ids.
- `comboOf` builds modifiers in the order Mod, Alt, Shift, the order every command registers its keys in. It was Alt, Shift, Mod until v0.3.0, which left Mod+Shift+Z and Mod+Shift+S dead.
- Exact Input (`ui/exact-input.ts`) builds a numeric dialog from any command's JSON schema; menus and the palette open it for commands with required arguments. Values are millimetres even when the display unit is cm or in (to fix when units reach the dialog).
- The session marks itself dirty on any `engine.changed` whose name does not start with `file.`.
- Recent projects come from `GET /api/recent`, with the local copy as a fallback when the service is down.
- Desktop: Open and Save use the Tauri dialog plugin with an `.ffbrc` filter (`shell/file-dialog.ts`); the browser build asks for a path. `set_window_title` and `text_edit` are invoked only when present.
- Dropped from formvis as unused: the jobs list, the title-bar quick search (Mod+K covers it), tabs, home files.
- Preferences: the Drape rows (substeps, gravity, collision thickness, solver device) are present and disabled until Phase 7. Sew, Drape, Animate and Render workspaces and the four templates are disabled until their phases.

### Engine clients

- `engine/service.ts`: base URL from `window.__FFB_ENGINE__` (desktop) or relative (Vite proxy, or a page served by the engine).
- `engine/events.ts`: one shared `EventSource` on `/api/events`; the dock opens its own, by design.
- `engine/stream.ts`: binary frames from the native stream server (format in `native/HASHOUT_LOG.md`); no DOM access. Not yet consumed; Phase 5 draws them.
- `engine/wasm.ts`: loads `public/engine/ffb_wasm.wasm` (built by `./formfabric engine wasm`, git-ignored) and wraps the C ABI. The page mirrors the document with `ffb_load` after every `pattern.get`.

### Dock (from formsculpt, Apache-2.0)

- Terminal, Claude Code and Chat with formsculpt's wire protocol unchanged. The dock row is reserved through `--ft-dock-h`, set before first paint from `formfabric.dock`, so opening it shifts nothing. Robot head for Claude Code, speech bubble for Chat.

### Workspace and Pattern View

- `workspace/layout.ts` builds the DOM once (tools 44 px, pattern pane, 6 px splitter, 3D pane, inspector 296 px). The 2D and 3D layouts collapse the other pane to zero through CSS only; nothing is created or destroyed on toggle. Every later update writes one text, value, class or style variable.
- Pattern View is one canvas with the 2D API. The retained model is the last `pattern.get` plus a per-panel geometry cache in a `WeakMap`; a drag replaces only the touched panel objects, so the rest of the cache stays valid.
- Preview protocol: pointer down opens a wasm drag group; each pointer move runs the same command in wasm and re-reads only the touched panels, synchronously, with no service traffic; pointer up sends `begin_drag`, the final command and `end_drag` to the service, one undo step. The first move of a drag is the slowest (16 to 26 ms at 4x throttling); the rest average about 5 ms.
- Snap runs `pattern.snap` in wasm: points, midpoints and inner shapes (Snap to Points), segments (Snap to Edges), grid (Snap to Grid, the larger of the grid preference and the visible minor step). Tolerance 8 px expressed in mm. Panels being dragged are excluded.
- Tool keys: A select and transform, Z edit points, P draw panel, R rectangle, E ellipse, I inner shape (Alt for an ellipse), L inner line, S split, C curvature, M measure, N snap, U rulers, F frame. Shift snaps directions to 45 degrees and makes squares and circles; Mod constrains to horizontal or vertical; Alt draws from the centre.
- Draw Panel: click corners, press and drag for a smooth curve point, click the first point or Enter to close, Delete pops the last point, Esc cancels, right click opens Exact Input with the points so far.
- Every `engine.changed` except `view.units` refreshes the document (coalesced to one in-flight request).
- The status bar has no cursor slot yet; `layout.ts` inserts one fixed-width span once at mount. Move it into the shell's StatusBar when that grows a `setCursor`.
- 3D View: formsculpt's `OrbitCamera` and `math.ts` (Apache-2.0, comments stripped) over one shader drawing a fading three-decade floor grid with red x and blue z axes, colours read from the theme tokens. Meshed panels arrive in Phase 5.
- Known limits: inner shape points are not editable yet; rotate and scale previews accumulate incrementally (the committed result is exact).

### v0.10.0 to v0.12.0

- Window token: every engine call goes through one fetch wrapper in `engine/service.ts` that adds `x-ffb-window`; the terminal socket passes `ffb_window` as a query parameter. In dev, Vite's proxy adds the header server-side, WebSocket upgrades included. `dock/agent-link.ts` marks the window live with POST.
- 3D tools: the selection overlay is one canvas (`ff3-select`) created once; the wind arrow and tape labels paint on the existing HUD overlay. The renderer has a `uState` uniform and `PanelBuf.stateKind` (0 none, 1 frozen, 2 disabled). The picker's `hitTri`, `hitU`, `hitV` are public for fold mapping. Tool keys come only from the catalogue `keys`, titled with `formatKeys`.
- Trims UI: a drawer plus eight Pattern View tools (Shift+B, H, F, Z, S, E, I, A), drawn in the existing canvas pass with no DOM per item. The selected trim item lives in `trims/state.ts`, pruned on `doc.onChange` in `layout.ts`; Selection is untouched. `trims/hook3d.ts` gives the 3D View line and point buffers.
- Dev ports: Vite's HMR uses the frontend port plus one, so parallel checks must leave a gap of two between frontend ports. A snapshot of `src` served from /tmp avoids reloads while other edits land (needs `server.fs.allow` for the linked node_modules).

### v0.13.0 to v0.15.0

- Timeline dock (`workspace/anim/`): DOM built once, one ruler canvas. Scrubbing is coalesced (latest frame wins, one `anim.set` and one `cache.frame` in flight). Playback prefers the engine's `anim.play`. Cache start and end are 0-based frame indices.
- Output (`workspace/output/`): the Export dialog renders each format's option schema and remembers it per format. File, Export comes from `app.export.*`. `export.run` takes flat arguments, and synchronous output commands do not poll `job.state`. The UV Editor sends one command per gesture. Retopo guides are drawn on an overlay canvas over the Pattern pane and stored per panel in panel space.
- Visualization (`workspace/viz/`, view3d): kind-3 scalar frames decode through `engine/stream.ts` `registerKind`. The view3d module has the Maps menu and legend, the stage (lights, one shadow map, HDRI, ground), saved views, 3D measures, fur strands and style lines. Turntable capture pushes each rendered frame with `requestFrame()`.
- Dock speed:
  - The chat pane batches tokens per frame, scrolls on requestAnimationFrame with a pinned flag, caps the log at 400 nodes and uses content-visibility on settled messages.
  - The terminal writes through a frame queue with a debounced fit. The SSE reader parses in one pass with an idle watchdog.
  - Resizing is two-stage: a CSS stretch while dragging, then one backing-store resize and paint 140 ms after release (`data-ft-dock-drag` on html).
  - Backend: position-tagged terminal chunks (no replay duplicates), coalesced socket frames, pty backpressure (high water at 48 chunks, 1 s cap), and a cached Claude Code lookup.

### v0.16.0 to v1.0.0

- Library drawer (`workspace/library/`):
  - Three tabs: Library, Snapshots and Components.
  - The thumbnail grid is virtualized over pooled cells.
  - Snapshots carry a 3D thumbnail rendered through the capture path.
  - Components pick ports as edge spans in the Pattern View (`components/pick.ts`).
- Shell:
  - `layouts.ts` holds saved layouts (six defaults) in prefs.
  - `settings-io.ts` exports and imports prefs, shortcuts and layouts. On the web it uses download and upload, with no `plugin-fs`.
  - `keyhints.ts` is the Key Hints overlay, shown when you hold the modifier.
  - `recovery.ts` is the start-up recovery dialog.
- `ui/commands.ts` keeps a list per key combo with `scopeOf`, `scopesCollide` and `conflicts`, so the Shortcut Editor detects conflicts per scope.
- One-region hooks into `workspace/layout.ts`, `view3d/index.ts` (the thumbnail render) and `app.ts`.
