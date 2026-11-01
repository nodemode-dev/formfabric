# HASHOUT_LOG · frontend

Living context for the window. What would be comments in `frontend/` lives here, newest first. Read before editing.

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
