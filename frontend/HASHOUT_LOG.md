# HASHOUT_LOG · frontend

Living context for the window. What would be comments in `frontend/` lives here, newest first. Read before editing.

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
