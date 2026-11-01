# Marvelous Designer: Menus, Hotkeys, Preferences, Object Browser, History and Workspace

Researcher slice 12 of 13 for the formfabric encyclopedia.
Scope: menu bar, default hotkeys, User Settings/Preferences, Object Browser, Property Editor, History, Library, workspace layouts and docking, scene management, auto-save, language.

## 0. Summary and honesty statement

What was verified from approved primary sources (support.marvelousdesigner.com, marvelousdesigner.com, and the official 2024 Creator's Field Guide PDF hosted on s3.marvelousdesigner.com):

- The default hotkey set for the 3D and 2D windows (from the official PDF guide), the gizmo mode switch (Unified vs Divided) and its effect on the Pin key.
- Camera navigation (pan, rotate, zoom), the configurable "Viewport Control" settings, tablet pen presets, auto focus, smart scrolling.
- User Settings tab list and the contents of the tabs that could be fetched: Default Files, User Interface, 3D, Graphic Options, Viewport Control, Shortcuts, Others.
- Object Browser tabs, Library window controls, History window and 3D States, Custom UI Layout (.uix), toolbar customization (JSON tool sets), User Setting save/load, auto-save, supported languages.

What could NOT be verified (the official manual HTML pages are largely image-based tables, and the fetch tool returned only prose):

- The complete main menu tree item by item. Only partial evidence exists (section 2). The official pages for "Menu" and "Main Menu" describe only that a Main Menu exists, plus the right-click pop-up menus.
- A full official hotkey table for every tool. The PDF gives about 60 shortcuts (section 3). The in-app "Active Tool Hotkey Viewer" lists the rest and cannot be scraped.
- Numeric Undo/Redo depth. No approved source states one. History window numeric limits: not stated.
- The Simulation and GPU tabs of User Settings (the PDF only says "Fast (GPU)" simulation exists, Windows only).
- Mouse mode presets named "Maya" and "3ds Max": confirmed to exist [single source: Zoom/Rotation page], but exact bindings were not fetched.

Where a claim comes from a single source it is tagged [single source]. Items marked UNVERIFIED are gaps for the next pass, not facts. Nothing below is invented; where a recommendation for the clone is mine rather than documented, it is labeled "Clone note".

Versioning context: the manual covers Marvelous Designer 2025.0, 2025.1, 2025.2 and 2026.1 (version tags appear inline in the docs). Older numbering (ver 4.x, 5.x, 10, 11, 12) is used for when features were introduced.

## 1. Application window anatomy

Source: 2024 Creator's Field Guide PDF, section "Screen Environment" (single PDF; cross-checked with the Layout and Object Browser support pages).

| Area | Name | Purpose |
|---|---|---|
| A | 3D Garment Window | Arrange, drape and visualize the 3D garment |
| B | 2D Pattern Window | Create and edit patterns, sewing, topstitching, puckering |
| C | Library | Default avatars and linked folders |
| D | Object Browser | Fabrics, buttons, buttonholes, topstitches, puckering, points of measure |
| E | Property Editor | Properties of the selected object (content changes with selection) |
| F | 3D Toolbar | Main 3D tools |
| G | 2D Toolbar | Main 2D tools |
| H | 3D Vertical Toggle Toolbar | Components, views, rendering styles, fit maps for garment and avatar |
| I | 2D Vertical Toggle Toolbar | Sewing, pattern, information and fabric displays; pattern locks |
| J | History | Access the 3D state or return to an earlier design point |

Window types (right-click menus on title bars, ver 2025.0+; source: Main Menu 2025.0 page):

| Title bar | Windows that can be opened |
|---|---|
| Viewport Title Bar | 3D Window, 2D Pattern Window, UV Editor, Pattern Drafter (Beta, ver 2025.1) |
| Side Panel Title Bar | Library (ver 2025.2 merged legacy library into the new Library), Modular Library, Object Browser, Property Editor, Animation Editor, History, USD Layer, Omniverse Connector |

Clone note: this gives two window classes. Viewport windows (up to 4 shown at once, cannot be closed or hidden) and side panels (closable, collapsible to edge tabs, tab-groupable).

## 2. Menu bar

### 2.1 What is verified

| Fact | Detail | Source |
|---|---|---|
| Main Menu location | Top-left of the title bar; "shows menus for each function" | Main Menu 2025.0 page |
| Tear-off menus | Clicking the dashed line (scissors icon) at the top of a drop-down, or pressing Enter, pops the menu out as a floating window | Main Menu page; UI Changes (Above ver 11) |
| Window menu | Until ver 12 it lived at Main Menu > Display > Window. From ver 12.1 it is a top-level Window menu (Main Menu > Window) | UI Changes page |
| Object Browser path (older) | Main Menu > Display > Window > Object Browser (Show) | Object Browser page (older) |
| Script menu renamed | In 2025.2 the Script menu became Plugins | 2025.0/2025.1/2025.2 New Feature List |
| CONNECT menu | EveryWear moved to CONNECT in the Main Menu in 2025.0 (EveryWear became a Beta plug-in in 2025.2) | New Feature List |
| Settings menu | Settings > User Settings (older: Settings > User Customization; ver 11: Settings/Preferences > Preferences; 2025.0: Settings > Preferences) | Several Set* pages |
| Help menu | Help > Language (language selection, restart required) | Set Language page |
| History | Main Menu > History opens the History window | History Window page |

### 2.2 Menu items confirmed to exist (partial, by menu)

This is NOT a complete listing; the official pages do not enumerate it. Each row is an item explicitly named in an official source.

| Menu | Item | What it does | Evidence |
|---|---|---|---|
| File | Open (Ctrl+O) | Open a file | PDF |
| File | Open > User Setting | Load a saved user-settings configuration file; prompts for restart | Save User Setting page |
| File | Open > Tool Set | Load a toolbar group configuration (JSON), 2026.1 | Customize Toolbar page |
| File | Open > Auto Save file | Opens dialog for opening auto-saved .zpac/.zprj | Open Auto Save File page |
| File | Save (Ctrl+S) | Save in current format | PDF |
| File | Save As (Ctrl+Shift+S) | Save as new name/format | PDF |
| File | Save As > User Setting | Save configuration file (language, units, shortcuts and more) | Save User Setting page |
| File | Save As > Tool Set | Save toolbar group config | Customize Toolbar page |
| File | Save As / Open > Project (*.zprj) | Garment + avatar + other modes, default save type | PDF; Project File page |
| File | Save As / Open > Garment (*.zpac) | Garment only, smaller | PDF; 3D Garment (*.zpac) page |
| File | Project/Garment with Meta Data | Saves metadata file together with project or garment (needs "Show Option to Save MetaData" in Others tab) | Project with Meta Data page; Others tab |
| File | Add > Avatar; Add > Project (*.zprj) | Add content to the open scene | PDF; Project File Add page |
| File | Import / Import (Add) / Export | 3D objects, animation, textures, graphics, color palettes, trims (OBJ among formats) | PDF |
| Edit | Undo (Ctrl+Z), Redo (Ctrl+Y) | History stepping | PDF |
| Avatar | Avatar Editor | Create and save custom avatar sizes; Auto Convert to Avatar moved here in 2025.0 | PDF; New Feature List |
| Window | Layout > Layouts (System Default), Custom Layouts, Export Layout, Browse Directory | Layout management (.uix) | Custom UI Layout page |
| Window | Library, Modular Library, Object Browser, Property Editor, Animation Editor, History, USD Layer, Omniverse Connector, 2D Pattern Window, 3D Window, UV Editor | Open or show windows | Main Menu page; New Feature List |
| Settings | User Settings / Preferences (tabs listed in section 5) | Defaults dialog | Set* pages |
| Help | Language | Choose UI language | Set Language page |
| CONNECT | EveryWear (Beta plug-in) | Connect to external service | New Feature List |
| Plugins | (formerly Script) | Scripting and plug-in entry | New Feature List |
| History | (opens History window) | 3D States | History Window page |

Top-level menu names with evidence: File, Settings, Avatar, Window, History, Help, CONNECT, Plugins (formerly Script). Names such as Edit, 3D Garment, 2D Pattern, Display (pre-12.1), Render are common in the product but were not confirmed as menu names by the pages fetched. UNVERIFIED: the full item list of Edit, 3D Garment, 2D Pattern, Display, and any Render menu. Next pass should open the application or capture the tool Index.

### 2.3 Right-click (context) menus (verified names, contents not listed)

| Window | Context menu | Notes |
|---|---|---|
| 3D | 3D Background Options | Contains Simulation Property, Wind Property entries (per Property Editor page); Show/Hide Color (PDF) |
| 3D | Avatar Options | |
| 3D | Pattern Options | |
| 2D | 2D Background Options | |
| 2D | Pattern Options | Includes "Select All with Same Property" (Particle Distance, Layer or Fabric) |
| 2D | Line/Point Options | |
| 2D | Sewing line Options | |
| 2D | Topology Options | |
| 2D | Texture Options | |
| 2D | Topstitch Options | |
| Library | Folder context menu | Delete, Rename, Show in Explorer (ver 12.1), Add to Workspace |
| Object Browser | Selection context menu | Merge to new Fabric/Button/Buttonhole/Topstitch (ver 11+) |
| History | 3D State context menu | Delete, Rename |
| Toolbars | Right-click | Group Tools, Rearrange Tools, Reset to Default (2026.1) |
| Zipper slider | Pop-up | Added 2025.2 |

## 3. Default hotkeys

Source: official 2024 Creator's Field Guide PDF (s3.marvelousdesigner.com). The PDF itself states its list is complete only for what it states and some tool keys appear only in tool headings. Windows keys shown; the CLO shortcut guide for Mac and PC exists as a separate download (not fetched). Mac Command/Option mapping: UNVERIFIED.

### 3.1 Global, file and edit

| Shortcut | Function |
|---|---|
| Ctrl+O | Open |
| Ctrl+S | Save (current format) |
| Ctrl+Shift+S | Save As |
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Ctrl+A | Select all in active window (3D, 2D or Edit Pattern points/segments) |
| Ctrl+F | Reset 2D and 3D arrangement for selected pieces |
| F5 | Refresh textures and graphics |
| Spacebar | Toggle Simulation on/off |
| Shift+F | Toggle Arrangement Points |
| Shift+W | Show Garment (reveal all hidden pieces) |
| Shift+Q | Hide 3D Pattern |
| Alt+7 | Toggle Strain Map |
| Alt+W | Toggle between current layout and full-screen of the selected window (ver 12.1) |
| Shift+T | Texture Surface display in 2D (ver 2025.0) |
| + and - | Zoom in / out with keyboard (added v2.4.0) |
| Enter | Pop a menu or tool group out as a window (when on tear-off line); also completes selection groups |

### 3.2 Tool keys

| Key | Tool | Window |
|---|---|---|
| Q | Select/Move (Unified gizmo: select, move, rotate in one) | 3D |
| A | Transform Pattern | 2D |
| Z | Edit Pattern | 2D |
| H | Polygon | 2D |
| S | Rectangle | 2D |
| E | Ellipse | 2D |
| I | Trace | 2D |
| G | Internal Polygon/Line | 2D |
| R | Internal Ellipse | 2D |
| C | Edit Curvature | 2D |
| V | Edit Curve Point | 2D |
| X | Add Point/Split Line | 2D |
| M | Free Sewing | 2D |
| N | Segment Sewing | 2D |
| B | Edit Sewing | 2D |
| K | Segment Topstitch | 2D |
| L | Free Topstitch | 2D |
| J | Edit Topstitch | 2D |
| ; (semicolon) | Seamline Topstitch | 2D |
| U | Create Topology | 2D |
| Y | Edit Topology | 2D |

### 3.3 Pattern and garment commands

| Shortcut | Function |
|---|---|
| Ctrl+D | Create Symmetric Pattern (with sewing) |
| Ctrl+[ | Superimpose Over |
| Ctrl+] | Superimpose Under |
| Ctrl+H | Strengthen (toggle to unstrengthen) |
| Ctrl+K | Freeze (toggle to unfreeze) |
| Ctrl+J | Deactivate/Activate (pattern) |
| W + Left click | Single Pin (Unified gizmo mode) |
| P + Left click | Single Pin (Divided gizmo mode) |

### 3.4 Gizmo modes (User Settings > 3D)

| Mode | Behaviour | Source |
|---|---|---|
| Unified Gizmo (default per PDF where Q is Select/Move) | One gizmo handles select, move and rotate | 3D settings page |
| Divided Gizmo | Separate tools: Select (Q), Move (W), Rotate (E). Pin shortcut moves from W to P; if P is already assigned, Pin gets none (ver 2.5.0) | 3D settings page; PDF |

Gizmo facts (PDF): lines move along X/Y/Z, circles rotate around axes, yellow square moves freely. Coordinate spaces: Screen (camera aligned), Local (object native, for avatar joint posing), World (grid axes). The gizmo only appears when simulation is off.

### 3.5 Modifier behaviours inside tools

| Key | Behaviour |
|---|---|
| Shift | Multi-select toggle; smart guides; locks angles 90/45 in Internal Polygon/Line; constrains Rectangle/Ellipse to square/circle; straight 3D lines |
| Ctrl (held) | Curved lines or curve points while drawing Polygon and 3D Line tools |
| Enter | Complete selection group (M:N sewing); extract pieces (Flatten, Trace) |
| Delete / Backspace | Undo last step while drawing Polygon |
| Esc | Restart Polygon drawing |
| Right-click | Context menus and precision dialogs (precision move, scale input) |

### 3.6 Camera navigation (default scheme)

| Action | 3D window | 2D window |
|---|---|---|
| Pan | Hold mouse wheel and drag; Mac Magic Mouse: Alt + left drag | Hold mouse wheel and drag |
| Zoom | Wheel (direction per PDF: scroll away to zoom in); Alt + left drag (up = out, down = in); keyboard + and - | Wheel |
| Rotate | Hold right mouse button and drag | Not applicable |
| View presets (front/back/left/right/top) | UNVERIFIED hotkeys; the Zoom/Rotation page confirms a "default lens distance" used by shortcut-based view control | n/a |

Note: two sources disagree on wheel direction wording for zoom (Zoom page: scroll down zooms in; PDF: scroll away zooms in). Treat as configurable; clone should follow the platform convention and expose an invert option.

### 3.7 Active Tool Hotkey Viewer

A dockable panel (Dropdown window selection > Active Hotkey Viewer, introduced Ver 12 / 2024.2) that lists hotkeys of the currently selected tool in collapsible groups, including hotkeys that have no UI button. It updates when the tool changes. Clone note: this implies each tool should carry its own hotkey metadata list, rendered live.

## 4. Hotkey editor (User Settings > Shortcuts)

Sources: Set Shortcuts (Above ver 4.2.0), Shortcuts/Hotkeys 2025.0.

| Feature | Behaviour |
|---|---|
| Location | Main Menu > Settings > User Settings > Shortcuts (ver 11: Settings/Preferences > Preferences > Shortcuts) |
| Structure | Browse by menu hierarchy; commands appear in a tree matching menus |
| Assign | Click a command, press the key combination |
| Search (ver MD10) | Find a command by menu name or by shortcut key; selecting a result jumps to its menu location |
| Reset (per item) | Returns that hotkey to program default |
| Delete | Removes a user-set shortcut |
| Restore Default button | Bottom of dialog: "Restore the Default Shortcut Keys" |
| Conflicts | Not documented. UNVERIFIED |
| Import/export | Via File > Save As > User Setting configuration file (includes shortcuts) |
| Applying | Takes effect as default after program restart |

## 5. User Settings / Preferences dialog

Menu path by version: Main Menu > Settings > User Settings (standard, ver 4 to 10 era "User Customization"), Settings/Preferences > Preferences (ver 11), Settings > Preferences (2025.0). Changes become defaults after restart. The dialog covers: shortcuts, menus, graphic options, view controls, default files, fonts, miscellaneous.

Tabs confirmed: User Interface, 3D, Graphic Options, Viewport Control (View Control), Shortcuts, Default Files, Others, CLOSET (listed in search results as "CLOSET (Above ver4.2.0)", not fetched; UNVERIFIED contents). Simulation, GPU and Language are not tabs of this dialog per the evidence (language is under Help; GPU appears as simulation option).

### 5.1 User Interface tab

| Option | Detail |
|---|---|
| Unit | inch, centimeter or millimeter; restart required; Ligne (L) mention truncated in source |
| Button Width | Toggle button-size adjustment in Ligne (v4.0.0) |
| Default Topstitch Thread Thickness Unit | v4.2.0; default became millimeters in 2025.2 |
| 3D Toolbar position | top, bottom, left or right (v4.0.0) |
| 2D Toolbar position | top, bottom, left or right (v4.0.0) |
| Group Tools | v5.1.431; groups tools by category, fewer hidden tools; manuals assume ON |
| Appearance > Line Pattern colors | Outline (2D), Internal Line (3D and 2D), Wireframe (3D), Selected, Hover |
| Font Size | For 2D measurements |
| Thickness (pixels) | 2D pattern outline thickness |
| Highlight 3D Pattern colors | Selected, Frozen, Strengthen, Layer, Bond (3D/2D), Solidify |
| Font Type | Font family |
| Resolution | Default Automatic |
| Scale Display | v4.2.0; on: auto resolution for monitor; off: pick a Scaled resolution |
| UI Color Customization | Added 2025.2 |
| Reset all "Don't Show Again" dialogs | Added 2025.2 |

### 5.2 3D tab

| Option | Detail |
|---|---|
| Mesh | Default mesh type: Triangle or Quad |
| Default Arrangement Shape Type | Curved or flat arrangement around avatar bounding volume |
| Default Arrangement Point Size | Radius of arrangement points; shrink for small avatars |
| Sync Fold Arrangement Angle | Fold Arrangement tool adjusts Fold Angle of selected Internal Line or Pattern Outline (v4.2.0) |
| Gizmo | Unified Gizmo or Divided Gizmo (Q/W/E); see 3.4 |

### 5.3 Graphic Options tab

| Option | Detail |
|---|---|
| Show Shadow | Shadow of virtual model |
| Use Shader | Softer lighting |
| Use Normal Map | Realistic seams and fabric texture |
| Anti-aliasing | Higher is smoother and slower |
| Use VBOs | Faster rendering |
| Image Resolution | 2025.0; maximum render image resolution, larger images downscaled during rendering, uses more memory |

### 5.4 Viewport Control (View Control) tab

Source: "Set View Control (Above ver. 8)" page.

| Option | Detail |
|---|---|
| Pan | Choose key and mouse button |
| Zoom | Choose key and mouse button |
| Rotate (3D) | Choose key and mouse button |
| Pop-up menu | Choose key and mouse button for context menu in 3D |
| Rule | Each key/button maps to one function only; duplicates cause malfunction |
| Presets | Default, Maya, 3ds Max schemes exist [single source: Zoom/Rotation page]; exact bindings UNVERIFIED. A forum-free official community bug report indicates not every custom combination works (shift-based rotation with middle-button pan failed, alt-based worked) [single source, community post on the official site; treat as unconfirmed] |
| Axis movement key | Moves objects along horizontal, vertical or 45 degree directions |
| Smart scrolling | ver 2.5.0; screen follows cursor during some functions |
| Auto focus | ver 5.0.0; 3D focus point follows clicks |
| Reset focus when nothing selected | Appears when auto focus is on |
| Default Y-axis position | View default |
| Default X-axis position | View default |
| Default lens distance | ver 4.2.0; camera distance when using shortcut view control |
| Mac | Right-click off by default on Mac presets; enable Secondary Click in OS settings |

Tablet pen presets (ver 10 adds Pressure support):

| Pen type | Buttons | Move | Zoom | Rotate | Extra |
|---|---|---|---|---|---|
| Two-button (XP-PEN or Wacom preset) | Top = middle click, bottom = right click | Hold top button while hovering, move pen | Hold top button while tip touches, move vertically | Hold bottom button, tip touching or near | XP pens rotate only when near the tablet |
| Three-button (Wacom preset) | Top = spacebar, middle = middle click, bottom = right click | Hold middle button while hovering | Hold middle button near device, move vertically | Hold bottom button | Top button toggles simulation |

### 5.5 Default Files tab

| Group | Setting | Detail |
|---|---|---|
| Avatar and Garment | Project (ZPRJ) | Default project loaded at start |
| Avatar and Garment | Current Viewpoint (2025.0) | Thumbnail from current 3D viewpoint |
| Avatar and Garment | Save Thumbnails in 4 Views | Front, Right, Back, Left (MD10) |
| Avatar and Garment | Custom Viewpoint (2025.0) | Multi-view snapshot (MVS) file |
| Material | Physical Property (PSP) | Default fabric physical properties file |
| Material | Texture Images: Open from | Default path for loading textures |
| Material | Refresh from Custom Folder Path (5.1) | Refresh texture path |
| Simulation Property | Simulation Property (SMP) | Default simulation property file |
| Auto Save | Time Interval (min) | Auto-save period (default value not stated here; the auto-save article says 3 minutes) |
| Auto Save | File Type | Auto-save file type |
| Auto Save | File Path | Auto-save directory |
| Custom Color Palette | ACO file | Default palette |
| Garment Information | File path (4.1.0) | Default garment info template |

### 5.6 Others tab

| Option | Detail |
|---|---|
| Show Option to Save MetaData | Offers saving project/garment with metadata file |
| Maintain Block Selections with Same Name | Modular mode: keep pattern selections when blocks share a name |
| Maintain Fabric Properties for Patterns with Same Name | Modular mode: keep fabric applications |

### 5.7 Language

Help > Language. Supported: Korean, Chinese (simplified), English, Japanese, French, Portuguese, Taiwanese (traditional Chinese), Italian, Spanish. Restart required. Docs themselves are published in en, ja, ko, zh-cn. German and Russian: not listed (not supported per source).

### 5.8 Settings export

File > Save As > User Setting writes a configuration file (extension not stated) holding language, units, shortcuts and more. File > Open > User Setting reloads it, then prompts a restart. Toolbar layouts are saved separately as JSON "Tool Set" files (2026.1).

### 5.9 Items requested but not found

| Item | Status |
|---|---|
| GPU settings in Preferences | Not found. PDF only notes "Fast (GPU)" simulation option, Windows only |
| Simulation defaults tab | Not a tab; defaults come from the default SMP file (Default Files tab) and Property Editor simulation properties |
| Auto-save toggle on/off | Not documented; only interval, type, path |
| Maya/3ds Max hotkey set | Only view-control presets confirmed |

## 6. Object Browser

Sources: Object Browser (2025.0 version and older version pages), Fabric and PDF sections.

Open via Main Menu > Window > Object Browser (older: Display > Window), or right-click the Side Panel Title Bar. Since 2025.0 the Object Browser contains all Style windows plus the Scene window.

| Tab | Contents | Operations |
|---|---|---|
| Scene | Every item and structure in 3D and 2D windows, in creation order; primary way to find and select | Select; click Name header to sort A to Z / Z to A (second click reverses) |
| Fabric | All fabrics | Select, add, copy, reset; drag to reorder (since 4.1.0); only tab with reorder and reset |
| Graphic | All graphics | Select, add, copy |
| Button | All buttons | Select, add, copy |
| Buttonhole | All buttonholes | Select, add, copy |
| Topstitch | All topstitches | Select, add, copy; add via drag from Library or + Add |
| Puckering | All puckering styles | Select, add, copy; edit Material, Density %, Width in Property Editor |
| Trim | All trims | Select, add, copy |
| Zipper | All zippers (ver 2025.1) | Select, add, copy |
| Point of Measure (PDF 2024) | Measurement points | Listed in the PDF object browser table; absent from 2025 tab list. UNVERIFIED in 2025 |

Operations common to the browser:

| Operation | Detail |
|---|---|
| Assign | Select lines or pieces in a window then click the Assign arrow beside the item name (PDF) |
| Merge | Multi-select, right-click, "Merge to new Fabric / Button / Buttonhole / Topstitch" yields a new default item (ver 11+) |
| Show/Hide scene and props | "Show and Hide Scene and Props" added 2025.1; per-icon behaviour not documented in fetched pages. UNVERIFIED |
| Select All with Same Property | Right-click 2D pattern: by Particle Distance, Layer or Fabric; also in UV Editor (2025.2) |

Clone note: an Object Browser is a typed registry of "styles" (fabric, button, buttonhole, topstitch, puckering, trim, zipper, graphic) plus a Scene tree. Styles are shared, copy-able, mergeable, and assigned to geometry by reference.

## 7. Property Editor

Source: Property Editor page; PDF section 6.

| Aspect | Behaviour |
|---|---|
| Role | Edits properties of the selected asset or style group; content is context sensitive |
| Types shown | Pattern, Internal Shape, Sewing, Stitch, Fabric, Button, Buttonhole, Avatar (pattern related properties; Eye Control added 2025.0), Simulation, Wind, 2D Window |
| Opening Simulation/Wind | Right-click 3D background > Simulation Property / Wind Property |
| Simulation Property | Open, save, edit simulation settings (SMP files) |
| Fabric material | Maps, color, reflection; Texture Map Options with Open/Add with Another App (2025.0) |
| Physical properties | Details section for manual fabric values |
| Fold | Fold Angle and Fold Strength for internal lines and seams |
| Elastic | Enable, Strength and Ratio (default 80 percent) |
| Pressure | Simulation property; positive pushes to back face, negative to front |
| Puckering | Material, Density %, Width |

Per-property field lists (fabric physical sliders and so on) belong to other researchers' slices.

## 8. History window and Undo/Redo

Source: History Window page; PDF.

| Item | Behaviour |
|---|---|
| Open | Main Menu > History, or Side Panel title right-click > History |
| Purpose | Tracks workflow; return to earlier points; save and compare 3D States |
| 3D State | Named snapshot of the garment state; shown with thumbnail; hover preview (ver 12.2) |
| Revert | Double-click a 3D State then OK |
| Rename / Delete | Right-click a 3D State |
| Loading a 3D State | Deletes the current History (warning dialog: "Loading the 3D State will delete the current History. Would you like to proceed?") |
| Undo / Redo keys | Ctrl+Z, Ctrl+Y |
| Numeric undo depth | Not stated in any approved source. UNVERIFIED |
| History count cap | Not stated. UNVERIFIED |

Clone note: model as two layers: a linear command undo stack (depth configurable) and user-named full-state snapshots with thumbnails that reset the undo stack when restored.

## 9. Library window

Sources: Library Window page (2025.0), PDF section 3.

| Item | Behaviour |
|---|---|
| Open | Main Menu > Window > Library, or Side Panel title bar right-click (2025.0). In 2025.2 "Library (Legacy)" merged into the new Library; 2025.2 adds a docked widget for universal browser |
| Default content | Default avatars, poses, shoes and other assets that can be downloaded (Download, MD10); folders marked with an N sign contain new downloadable files |
| Add Folder | Link a folder (bookmark); persists until deleted or Reset |
| Reset | Restore defaults, removes custom linked folders |
| Refresh | Apply added/deleted files or folders |
| Select All | Replaces Refresh when download checkboxes are enabled |
| List / Icon view | ver 2.3.0 |
| Search | ver 3.0.0 |
| Open a file | Drag from library into the 3D window, or double-click |
| Add to existing file | Right-click > Add to Workspace / "Add to existing file" |
| Folder context menu | Delete (removes bookmark only), Rename (bookmark label only), Show in Explorer (ver 12.1) |
| Preview | Hover over a Project or Garment thumbnail to see front, back, side views |
| Favorites | No separate "Favorites" feature documented; user-linked bookmark folders serve this role. UNVERIFIED whether a star/favorite flag exists |
| Default folder names | UNVERIFIED (not in fetched text) |

Related: Modular Library is a separate window (Modular mode).

## 10. Workspace, layouts and docking

Sources: Custom UI Layout (2025.0), UI Changes (Above ver 11).

| Feature | Behaviour |
|---|---|
| Docking | Windows dock to preferred positions; resizable width and height |
| Workspace windows | Up to 4 visible at once (viewport windows); cannot be closed or hidden |
| Side windows | Close with X; hide with the arrow at top-left; hidden windows become edge tabs |
| Pop-ups | Duplicate-window icon opens a window as a pop-up; resize by dragging edges; drag a pop-up onto a side window to group as bottom tabs |
| Window type switch | Top-center dropdown (removed after 2025.0; windows now opened from Main Menu > Window) |
| Layout menu | Main Menu > Window > Layout: Layouts (System Default), Custom Layouts, Export Layout, Browse Directory |
| Export Layout | Name it, saved as .uix in the default directory; appears under Custom Layouts |
| Backup/restore | Copy .uix files into the layout default folder (the list syncs with the folder) |
| Reset layout | Toggle in bottom-right restores default layout (layout icons switch between 3D/2D views) |
| Full-screen | Alt+W toggles between current layout and a single full-screen window (12.1) |
| Toolbar placement | Any window edge via left handle; show/hide toggle at the far right of view icons; positions also set in User Interface tab |
| Tool group tear-off | Dashed line with scissors or Enter pops tool group as window |
| Modular Editor | Toggle icon on 2D toolbar |
| Toolbar customization (2026.1) | Right-click toolbar: Group Tools (drag reorder, move between groups, split group, hide with eye-off button), Rearrange Tools, Reset to Default (irreversible); no moving between 3D and 2D toolbars; saved as JSON via Save As / Open > Tool Set |

## 11. Scene and object management

Evidence is thin; recorded as found.

| Capability | Evidence | Status |
|---|---|---|
| Hide / Show 3D patterns | Shift+Q hide, Shift+W show all (PDF) | Verified |
| Show/Hide Color (strengthen, freeze indicators) | Right-click 3D background (PDF; 2024.1 article) | Verified |
| Show / Hide Scene and Props | Added 2025.1 | Verified existence, details UNVERIFIED |
| Show/Hide sewing, measurement, seam taping, trims display | Vertical toggle toolbars (PDF, 2D Display Icons page) | Verified |
| Show wireframe for avatars | 2025.1 | Verified |
| 2D pattern locks | Lock All Pattern Outlines, All Internal Lines, All Guidelines, All Remesh Patterns (locked items cannot be selected) | Verified |
| Freeze / Deactivate | Ctrl+K, Ctrl+J | Verified |
| Layers | "Layer" in the garment sense: pattern layer numbers (Show Pattern Layer ver 2024.1, Layer Clone Over/Under, Superimpose Over/Under). The USD Layer window is a separate USD workflow. No generic scene-layer system documented | Verified as stated |
| Grouping | Pinned areas from Pin (Box) move as a group; no scene-object Group command documented | UNVERIFIED for scene groups |
| Scene tree | Scene tab lists all objects in creation order; select from there | Verified |
| Select All with Same Property | Particle Distance, Layer, Fabric | Verified |

2D display toggles (vertical toolbar, 2D Window Display Icons page):

| Group | Toggles |
|---|---|
| Sewing | Show Sewing Lines, Show 2D Topstitches, Puckering toggle |
| Pattern/Garment | Show 3D Pen line, Show Symmetric/Instance Lines |
| Information | Pattern Names, Pattern Layer (2024.1), Line Length/Angle, Grainline, 2D Measurement, Ruler (2024.1), Ruler guideline |
| Rendering style | Texture Surface (Shift+T), Monochrome, Opaque/Translucent Surface, Transparent, Mesh/Wireframe, Back Texture, Random Color (ver 10) |
| Pattern lock | Outlines, Internal Lines, Guidelines, Remesh Patterns |

3D vertical toggle (PDF): Show Garment, 3D Trims Display, Front/Back Texture Surface, Textured and Thick Textured Surface, Show Statistics, Show Pressure Points, fit maps (Strain Map Alt+7, Stress Map added 2025.0).

## 12. Auto-save and backup

Sources: Open Auto Save File page (3.1.0+), Default Files tab, PDF.

| Item | Detail |
|---|---|
| Introduced | Project auto-save since 3.1.0 |
| Interval | 3 minutes by default; work in the first 3 minutes after launch is not saved. Configurable via Default Files > Auto Save > Time Interval |
| Default folder (Windows) | C:\Users\Public\Public Documents\Marvelous Designer\Assets_ver_x.x.x\Autosave |
| File types | .zpac and .zprj autosave files; File Type selectable |
| Recovery | On relaunch after a crash a pop-up offers to open the auto-saved file; or double-click the autosave file; or File > Open auto-saved dialog |
| Change path | Default Files > Auto Save > File Path (older: Settings > User Customization) |
| Explicit backup | None documented. "Backup" in docs refers to layouts (.uix copy). UNVERIFIED if rolling backups exist |
| File name/path length | 255 character limit removed in 2025.2 |

Related file types: ZPRJ (project, default), ZPAC (garment), PSP (fabric physical properties), SMP (simulation property), ACO (palette), MVS (multi-view snapshot), UIX (layout), JSON (tool set), POS (pose), user setting config file (extension unknown).

## 13. Manual section map (for scoping the clone)

Section counts from the official manual index (articles): Installation and Requirements 31, Troubleshooting 7, Setting 20, Window 16, 2D Pattern Window Environment 12, 3D Window Environment 26, Open/Save 16, Import/Export 22, Avatar 29, Measure 16, Trim 10, Scene and Props (n/a), 2D Pattern Shape 17, Edit 2D Pattern 48, Edit 3D Garment 18, Garment Properties 9, Arrangement 19, Sewing 26, Fabric 34, Material 13, Graphic 10, Select Mesh/Pin 7, Tack 7, 3D Lines 11, Topstitch 33, Puckering, Button 53, Zipper 10, Piping/Binding, Retopology, Animation 13, UV Editor, Modular 12, Script (now Plugins), AI Studio.

## 14. Clone implementation notes (formfabric)

| Area | Recommendation (clone note, not source fact) |
|---|---|
| Command registry | One registry of commands with id, menu path, default hotkey, tool group; drives Main Menu, Shortcut editor with search, Active Hotkey Viewer and toolbar Group Tools |
| Hotkey conflicts | MD does not document conflicts; formfabric should detect and warn |
| Settings persistence | Settings apply at restart in MD; a clone can apply live. Provide export/import of one settings file plus separate layout (.uix equivalent) and tool-set JSON |
| View control | Provide presets Default, Maya, 3ds Max, Blender; enforce one-binding-per-function validation |
| Gizmo | Unified (Q) and Divided (Q/W/E) modes with Pin key relocation |
| Layout | Docking with up to 4 viewports, side panel tabs, edge-collapse, named layouts, full-screen toggle |
| History | Command stack plus named 3D State snapshots with thumbnails |
| Library | Linked folder bookmarks, list/icon view, search, hover preview, drag into 3D |
| Autosave | Timer (default 3 min), separate directory, crash-recovery prompt at startup |
| Languages | At minimum EN, KO, ZH-CN, ZH-TW, JA, FR, PT, IT, ES match MD |

## 15. Open questions for a follow-up pass

1. Complete item lists of Edit, 3D Garment, 2D Pattern, Display, Render menus (not found in fetched official text).
2. Numeric undo depth and History cap.
3. Exact Maya and 3ds Max view-control bindings; view preset hotkeys (front/back/left/right/top).
4. Mac modifier mapping for hotkeys.
5. CLOSET tab contents in Preferences.
6. Default Library folder names and any favorites flag.
7. Scene tab icons (eye, lock) and group semantics.
8. Any GPU selection setting beyond "Fast (GPU)" simulation.

## Sources

Fetched successfully:

- https://support.marvelousdesigner.com/hc/en-us/articles/47358409175577 (Others tab)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358260608793 (Default Files tab)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358402111769 (3D tab)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358169509273 (User Interface tab)
- https://support.marvelousdesigner.com/hc/zh-cn/articles/47358263791769 (Graphic Options)
- https://support.marvelousdesigner.com/hc/zh-cn/articles/47358287009049 (View Control; also en: Set-View-Control-Above-ver-8)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358395237529-Set-Shortcuts-Above-ver4-2-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358120094361-Shortcuts-Hotkeys-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358326346521-Active-Tool-Hotkey-Reference-Window-Ver-12-2024-2
- https://support.marvelousdesigner.com/hc/en-us/articles/47358315717017-Zoom-In-Out-Rotation
- https://support.marvelousdesigner.com/hc/en-us/articles/47358207865881-Set-Language-Above-ver-7-1-1
- https://support.marvelousdesigner.com/hc/en-us/articles/47358171722265-Object-Browser-Ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358294806169-Object-Browser
- https://support.marvelousdesigner.com/hc/en-us/articles/47358318005913-Property-Editor
- https://support.marvelousdesigner.com/hc/en-us/articles/47358253334169-Library-Window
- https://support.marvelousdesigner.com/hc/en-us/articles/47358269290393-History-Window
- https://support.marvelousdesigner.com/hc/en-us/articles/47358183889817-UI-Changes-Above-ver-11
- https://support.marvelousdesigner.com/hc/en-us/articles/47358157097881-Custom-UI-Layout-ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358379603609-Save-User-Setting-as-Configuration-File
- https://support.marvelousdesigner.com/hc/en-us/articles/59782795610393-Customize-Toolbar
- https://support.marvelousdesigner.com/hc/en-us/articles/47358120307353-Marvelous-Designer-2025-0-2025-1-2025-2-New-Feature-List
- https://support.marvelousdesigner.com/hc/en-us/articles/47358205499033-2D-Window-Display-Icons-Ver-5-1-4-and-above
- https://support.marvelousdesigner.com/hc/ja/articles/47358131751449-Main-Menu-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358240361625-Menu
- https://support.marvelousdesigner.com/hc/en-us/articles/47358246789529-2D-Window
- https://support.marvelousdesigner.com/hc/zh-cn/articles/47358409002265-Open-Auto-Save-File (zh-cn URL with encoded title)
- https://support.marvelousdesigner.com/hc/ko/community/posts/48267453063577 (community bug report on official site)
- https://support.marvelousdesigner.com/hc/en-us/categories/51985515993625-Manual
- https://www.marvelousdesigner.com/manuals
- https://www.marvelousdesigner.com/support/news/view/d9fffb13e83a401baa91b639c7795f15 (shortcut guide announcement; no key list on page)
- https://s3.marvelousdesigner.com/newmdweb/case/20240626/MD+User+Guide+2024.pdf (Creator's Field Guide 2024, source of the hotkey tables)

Attempted but not retrievable: manual.marvelousdesigner.com Confluence pages (blocked by robots), the Divided Gizmo article, the Open/Save section index, the community shortcut post.
