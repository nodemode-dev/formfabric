# 12b. Menus, Hotkeys and Simulation Gap Pass

Scope: gap-filling for 12-menus-hotkeys-prefs.md and 05-simulation.md. Primary sources only (support.marvelousdesigner.com, support.clo3d.com, the official 2024 User Guide PDF). Tags: [single source] = one primary page; [CLO only] = documented only on support.clo3d.com; NOT PUBLISHED = searched, no primary source states it; never filled by guess.

Search note: the web search budget ran out partway through this pass, so several leads (CLO "Set View Control", a CLO shortcut PDF, the Tool Index) were not opened. Those are listed under Remaining gaps.

## 1. Result summary

| Gap item | Result | Status |
|---|---|---|
| Full item list of every top menu | Not published as text in any fetched page; only navigation paths and pop-up menu names found | Partly closed (paths only) |
| macOS key mapping | Not published in fetched primary pages; Mac mouse notes only | Open |
| Maya / 3ds Max view-control bindings | Schemes exist; bindings not published as text | Open |
| View preset hotkeys | Not published in fetched pages; only "Switch Viewport Layout by Hotkey" (12.1) is named | Open |
| Undo depth | Not published; Ctrl+Z / Ctrl+Y confirmed | Open (keys closed) |
| Simulation and GPU preferences | Simulation Properties panel and Graphic Options documented; no GPU tab; GPU is a Processor choice | Closed |
| Hard pin vs soft pin | Neither term appears in MD or CLO pages fetched | Closed (does not exist as named feature) |
| Pin tool family | Documented in full below | Closed |
| Fix / Settle commands | No such commands found; Freeze and Pin cover the use cases | Closed (not found) |
| Intersection resolution tools | A scene-level toggle "Intersection Resolution" (Normal / Flipped Normal); no "Untangle" command found | Closed |
| Preset to time step / substep / CG mapping | Numeric mapping NOT PUBLISHED | Open (confirmed unpublished) |
| CPU vs GPU limits | Documented below | Closed |

## 2. Top menus: what is verifiable

### 2.1 Menu names with evidence

| Menu | Evidence | Source |
|---|---|---|
| File | Paths "File > Open", "File > Export", "File > Import", "File > Add > Avatar" | 2024 User Guide PDF |
| 3D Garment | Paths "Main Menu > 3D Garment > Simulate", "> Pin", "> Select/Move" | MD Simulate, Create Pin |
| Avatar | Path "Avatar > Avatar Editor" | 2024 User Guide PDF |
| Materials | Path "Main Menu > Materials > Steam > Steam" | MD Steam page |
| Display | Path "Main Menu > Display > Environment > Show Wind Controller" | MD Wind Controller Setting |
| Settings | Paths "Settings > User Settings > Shortcuts / Graphic Options / View Control / 3D / Others" | MD user settings pages |
| Window | "Open all window types from Main Menu > Window or right-click Title Bar" (2025.0) | MD New Feature List |
| CONNECT | "EveryWear ... location has been moved to CONNECT in the Main Menu" (2025.0) | MD New Feature List |
| Plugins | "Change the Script menu to Plugins" (2025.2) | MD New Feature List |
| Help | Language and help paths (from first pass) | first pass |
| Edit | No fetched page names it as a menu | NOT PUBLISHED in fetched pages |
| 2D Pattern | No fetched page names it as a menu | NOT PUBLISHED in fetched pages |
| Render | No fetched page names it as a menu | NOT PUBLISHED in fetched pages |

The Menu page (article 47358240361625) and Main Menu 2025.0 (47358131751449) do not list top-menu items. They list pop-up menus only.

### 2.2 Pop-up (context) menus, ver 2025.0

| View | Pop-up menus |
|---|---|
| 3D window | 3D Background Options, Avatar Options, Pattern Options |
| 2D window | 2D Background Options, Pattern Options, Line/Point Options, Sewing line Options, Topology Options, Texture Options, Topstitch Options |

### 2.3 Window menu and title-bar menus (2025.0 and later)

| Title bar | Entries |
|---|---|
| Viewport Title Bar | 3D window, 2D Pattern Window, UV Editor, Pattern Drafter (Beta, 2025.1) |
| Side Panel Title Bar | Library (2025.2), Modular Library, Object Browser, Property Editor, Animation Editor, History, USD Layer, Omniverse Connector |

### 2.4 Known commands by menu (from paths and hotkey lists, not a complete listing)

| Menu | Confirmed items | Source |
|---|---|---|
| File | Open (Ctrl+O), Save (Ctrl+S), Save As (Ctrl+Shift+S), Import, Export, Add > Avatar | PDF 2024 |
| 3D Garment | Simulate, Pin (Pin Box / Pin Lasso), Select/Move | MD Simulate, Create Pin |
| Avatar | Avatar Editor | PDF 2024 |
| Materials | Steam > Steam | MD Steam |
| Display | Environment > Show Wind Controller | MD Wind |
| Settings | User Settings (Shortcuts, Graphic Options, View Control, 3D, Default Files, Others) | MD |

Clone note: until the live application or the Tool Index is inspected, treat full menu contents as unknown. Build the menu bar as data (a registry of command id, label, hotkey, location) so items can be filled later.

## 3. Hotkeys

### 3.1 Confirmed from the 2024 User Guide PDF (Windows keys)

| Function | Key | Source |
|---|---|---|
| Undo | Ctrl+Z | PDF 2024 |
| Redo | Ctrl+Y | PDF 2024 |
| Simulate | Spacebar | PDF 2024 |
| Single pin | W + click (Select/Move tool) | PDF 2024; MD Create Pin |
| Select/Move | Q | PDF 2024 |
| Transform Pattern | A | PDF 2024 |
| Edit Pattern | Z | PDF 2024 |
| Edit Sewing | B | PDF 2024 |
| Edit Topstitch | J | PDF 2024 |
| Strengthen | Ctrl+H | PDF 2024 |
| Freeze | Ctrl+K | PDF 2024 |
| Deactivate | Ctrl+J | PDF 2024 |
| Hide 3D Pattern | Shift+Q | PDF 2024 |
| Select All | Ctrl+A | PDF 2024 |
| Arrangement Points | Shift+F | PDF 2024 |
| Symmetric Pattern | Ctrl+D | PDF 2024 |
| Refresh Textures | F5 | PDF 2024 |
| Strain Map | Alt+7 | PDF 2024 |
| Quick Pinching (while simulating) | Q | MD Quick Pinching page |
| Advanced Pinching (up to 2024.2) | Shift+E | MD Quick Pinching page |

### 3.2 Pin-related hotkeys

| Action | Key | Source |
|---|---|---|
| Create single pin | W + click with Select/Move | MD Create Pin |
| Delete pin | W + click on an existing pin | MD (pin page 47358146159769) [single source] |
| Add pinned area in 2D | Shift+Ctrl + drag on pattern | MD Create Pin |
| Remove pinned area in 2D | Ctrl + drag on pattern | MD Create Pin |
| Pin key under Divided Gizmo | P instead of W (ver 2.5.0), unless P is already assigned | MD User Settings 3D page |
| Korean input language | W does not create pins; switch to English | MD Create Pin |

### 3.3 macOS key mapping

| Question | Finding |
|---|---|
| Command/Option mapping | NOT PUBLISHED in any fetched primary page. The Shortcuts and Set Shortcuts pages give Windows keys only and say nothing on platform differences |
| Mac mouse | 3D pan on a Magic Mouse: hold Alt and drag with the left button (MD Zoom In/Out/Rotation). Mac right-click needs the system secondary-click setting enabled (MD View Control page, zh-cn copy) |
| Remapping | Every function is remappable in Settings > User Settings > Shortcuts; a Restore Default button exists |

Clone note: map Ctrl to Cmd on macOS as a convention and keep Alt as Option; this is a design choice, not documented MD behavior.

### 3.4 View-control bindings

| Item | Documented value | Source |
|---|---|---|
| 3D pan | Hold mouse wheel and drag | MD Zoom In/Out/Rotation |
| 3D zoom | Scroll wheel; or Alt + left button drag (up = zoom out, down = zoom in) | same |
| 3D zoom (keyboard) | + and - keys (ver 2.4.0) | same |
| 3D rotate | Right mouse button drag | same |
| Maya scheme | Exists ("supported alongside the default method"); bindings NOT PUBLISHED | same [single source] |
| 3ds Max scheme | Exists; bindings NOT PUBLISHED | same [single source] |
| Configurable items in View Control tab | Mouse and key assignments for pan, zoom, rotate (3D); pop-up menu button; axis-movement key (constrain to horizontal, vertical or 45 degrees); smart scrolling; auto-focus; default X/Y axis positions; default lens distance | MD View Control page (zh-cn) [single source] |
| Constraint | Each key or mouse button can be assigned to only one function; duplicates break normal operation | same |
| Pen tablets | Pressure support; 2-button and 3-button pen presets (XP-PEN, Wacom) for move, zoom, rotate | same |
| View presets front/back/left/right/top | Hotkeys NOT PUBLISHED. Only "Switch Viewport Layout by Hotkey" (12.1) is listed in the Key Features index | MD Quick links to Key Features |

### 3.5 Undo depth

| Item | Finding | Source |
|---|---|---|
| Undo / redo keys | Ctrl+Z / Ctrl+Y | PDF 2024 |
| Numeric undo limit | NOT PUBLISHED. The History window page states no step cap | MD History Window |
| History behavior | History tracks workflow; 3D States can be saved; loading a 3D State deletes the current History (warning dialog: "Loading the 3D State will delete the current History") | MD History Window |
| Preference for undo count | None in the Others, 3D or Graphic Options tabs | MD user settings pages |

## 4. Preferences relevant to simulation and GPU

### 4.1 Preferences dialog tabs (Settings > User Settings; v11 path Settings/Preferences > Preferences)

| Tab | Contents confirmed | Source |
|---|---|---|
| Shortcuts | Click a menu, press a key; search field; Delete; per-item Reset; Restore Default button | MD Set Shortcuts |
| Graphic Options | Show Shadow, Use Shader, Use Normal Map, Anti-aliasing, Use VBOs, Image resolution (2025.0, caps max render resolution) | MD Graphic Options |
| View Control | See 3.4 | MD View Control |
| 3D | Default mesh (Triangle or Quad), arrangement shape type and point size, Sync Fold arrangement angle (4.2.0), Gizmo mode (Unified or Divided) | MD 3D tab |
| Others | Show Option to Save MetaData; Maintain Block Selections with Same Name (Modular Mode); Maintain Fabric Properties for Patterns with Same Name (Modular Mode) | MD Others |
| Default Files, Language, User Interface, Color Theme | Exist (CLO section index) | CLO Preferences/Settings index |

There is no Simulation tab and no GPU tab in Preferences. Simulation settings live in the scene Simulation Properties (Property Editor) and the Simulate toolbar.

### 4.2 Other CLO Preferences/Settings section articles [CLO only index]

Set Graphic Options, Set View Control, Set Shortcuts, User Interface, Set Color Theme, 3D, Other, Set Default Files, Set Language, Smart Guide, Snap to Pattern, Snap to Grid, Fix Grid Display, Show/Hide Ruler, Show/Hide Guidelines, Ruler and Guidelines, Edit Ruler Properties, 3D Background Setting, Show 3D Garment Window Size, Gizmo Axis Setting, Unified Gizmo, Divided Gizmo, Ground Setting, Move with Arrow Keys, Show Camera, Wind Controller Setting, Show 3D Shadow, Show Simulation Speed, Compatible with Wacom 3D Pen (ver 6.0).

Clone note: "Show Simulation Speed" is a display toggle worth replicating; "Fix Grid Display" here is a 2D grid setting, not a simulation fix.

### 4.3 Simulation Properties panel (scene level), re-verified

| Property | Detail | Source |
|---|---|---|
| Preset | Selects a predefined mode; Open/Save icons import or store custom settings | MD Simulation Properties |
| Time Step | Smaller is more refined and slower; "default values are recommended"; no numbers | MD, CLO |
| Simulations Per Frame (Simulation Quality) | Splits each time step into substeps; example 0.03 with 10 gives 0.003 | MD, CLO |
| CG Finish Condition | Iteration or Residual | MD, CLO |
| CG Iteration Count | Fixed count; faster, less accurate | MD, CLO |
| CG Residual | Error threshold; slower, more accurate; no default given | MD, CLO |
| Self Collision Iteration Count | Use default | MD, CLO |
| Air Damping | Resistance from air; no value given | MD |
| Gravity | Default -9800 | MD |
| Number of CPU in Use | Up to 12 cores auto-detected; 16-core CPU still uses 12 | MD, CLO |
| Nonlinear Simulation | Realistic but slower; selected automatically when Fitting runs | MD, CLO |
| Ground | Collision and friction; defaults recommended | MD |
| Avatar-Cloth Collision Detection | Toggle | MD |
| Self-Collision Detection | Toggle; Avoidance Stiffness default 0.001000 | MD |
| Proximity Detection | Toggle | MD |
| Intersection Resolution | Toggle; Normal or Flipped Normal sets resolve direction | MD |
| Layer Based Collision Detection | Toggle; layers and sublayers set collision order | MD |
| GPU settings | The panel mentions none | MD |

## 5. Pin tool family

### 5.1 Tools and commands

| Tool or command | Location | Behavior | Source |
|---|---|---|---|
| Pin (Box) | Main Menu > 3D Garment > Pin, or hold the Pin button on the 3D Toolbar for over 2 seconds | Click-drag a box to pin all vertices inside; mesh turns red in 3D and 2D | MD Create Pin |
| Pin (Lasso) | Same flyout | Lasso selection variant of the same pin creation | MD Create Pin |
| Vertex pin | Pin tool, click a mesh vertex | Pins one vertex | MD Create Pin |
| Single pin with hotkey | Select/Move + W + click | Creates a pin at that point | MD Create Pin |
| Pin in 2D | 2D patterns accept pins; Shift+Ctrl + drag adds, Ctrl + drag removes | Pins sync to the 3D mesh | MD Create Pin |
| Segment pin | Double-click a pattern outline in 2D or 3D | In 2D, adjacent vertices pinned when angle difference is under 20 degrees | MD Create Pin |
| Whole-pattern pin | Double-click a pattern in 2D or 3D | Pins the whole pattern | MD Create Pin |
| Internal line pin | Same double-click on internal lines and shapes (ver 4.0.0) | Pins internal lines | MD Create Pin |
| Move pinned mesh | Drag, or use the Gizmo, in 3D | Pinned mesh moves as a group | MD Create Pin |
| Delete Pin / Delete All Pins | Right-click a pin; Delete Pin options (single, all, all on selected patterns) | Removes pins; this is the "Unpin" equivalent | MD pin page 47358146159769; PDF 2024 ("Right-Click on pin to remove selected pin or all pins") |
| W + click on a pin | Select/Move | Deletes that pin | MD pin page [single source] |
| Attach to Avatar / Detach from Avatar | Right-click pin | Binds pin to nearest avatar part so garment does not slide in simulation or animation; attached pins cannot be selected or moved | MD pin-to-avatar page 47358452805913 [single source] |
| Pin Group Selection | Added 2025.2 | Listed under Effortless and Intuitive; details not fetched | MD New Feature List [single source] |

The Help Center section "Select Mesh/Pin" holds 7 articles; titles were not retrievable.

### 5.2 Related simulation-time pulling and locking tools

| Tool | Hotkey | Behavior | Source |
|---|---|---|---|
| Quick Pinching | Q (on by default while simulating) | Pulls a mouse-picked point during simulation | MD Quick Pinching page |
| Advanced Pinching | Shift+E (to 2024.2); more features added 2025.0 | Only while simulating. Falloff Shape: Bump or Linear. Falloff Distance 1 to 500 mm. Falloff Power -2.0 to +2.0. Distance Measure: Surface (one layer) or Straight (all patterns in range). Options: Limit to Selected Patterns, Distance Indicator, Display Weights | same |
| Brush Pinching | Hold Quick Pinching over 2 seconds (2026.1) | Brush-stroke deformation. Size, Focal Radius, Lazy Distance, Display Weights, Strength, Push Inward, Average Radius | MD 2026.1 note [single source] |
| Freeze / Unfreeze | Ctrl+K (PDF); right-click in 3D (Select/Move) or 2D (Transform Pattern) | Pattern acts as a solid during simulation; frozen patterns still collide, deactivated ones do not | MD Freeze page |
| Deactivate / Activate | Ctrl+J | Removes pattern from collision and simulation | PDF 2024 |
| Strengthen | Ctrl+H | Stiffens a pattern | PDF 2024 |

### 5.3 Hard pin versus soft pin

| Question | Finding |
|---|---|
| Does MD or CLO define "hard pin" or "soft pin"? | No. Both terms are absent from the Create Pin, pin-to-avatar, Simulate, Freeze and Simulation Properties pages. The pin page states it "does not mention hard or soft" pins |
| Closest documented behavior | Pins act like Freeze for the pinned vertices (infinite-mass constraint). Soft-like behavior is available via Advanced Pinching falloff (Bump or Linear weights), which is a transient pull, not a pin |
| Clone note | Model a pin as a position constraint with optional animated target and an optional weight (1.0 = hard). This is a design suggestion, not documented MD behavior |

## 6. Fix, Settle, and intersection tools

| Name | Exists as MD command? | Evidence |
|---|---|---|
| Fix | No command found. "Fix Grid Display" (CLO) is a 2D grid setting; Freeze and Pin cover locking | CLO Preferences index; MD Freeze |
| Settle | No command found in any fetched page | none |
| Resolve Intersection | No standalone command found. A toggle "Intersection Resolution" in Simulation Properties uses Normal or Flipped Normal to set resolve direction | MD Simulation Properties |
| Untangle | No command found | none |
| Layer (property) | Pattern property: 0 = off; sets drape order; set back to 0 after draping for stability. Layers 1 to 23 have distinct colors (repeat from 24); negatives reverse order | MD Layer page |
| Show Layer Depth | Ver 2024.1; Show/Hide Color in 3D Background Options hides the layer outline | MD Layer page |
| Layer Based Collision Detection | Scene toggle; layers and sublayers set collision | MD Simulation Properties |
| Freeze inner layers then drape outer | Documented workflow to reduce instability | MD Freeze page |
| Instant Simulation Stop | 2025.2; stops even under heavy load | MD New Feature List |
| Sculpt warning dialog | 2025.1: warns when starting simulation in Sculpt | MD New Feature List |

Clone note: implement intersection handling as (a) a per-step resolve pass with a direction mode (mesh normal or flipped) and (b) a layer ordering system, plus freeze-and-drape as a user workflow. No dedicated "untangle" solver is documented.

## 7. Simulation presets and parameter mapping

### 7.1 Preset names by product

| Product | Presets | Source |
|---|---|---|
| MD (2024.2 and later) | Normal (Default), Animation (Stable), Fitting (Accurate Fabric); Processor CPU or GPU chosen alongside each | MD Simulate |
| CLO (current Edit Simulation Properties page) | "Normal, Fitting, and GPU simulations are primarily used" | CLO Edit Simulation Properties [CLO only] |
| CLO (older Simulation article) | Simulate (GPU), Simulate (Normal), Simulate (Complete), Simulate (Complete Nonlinear) | CLO Simulation article 115000304968 [CLO only] |

Older CLO naming: Complete is described as "more defined results, but possibly slower", for finished garments; Complete Nonlinear as "the most defined results, with more realistic fabric stretch". Normal is "fast, suited to working on a garment".

### 7.2 Preset to numeric value mapping

| Preset | Time Step | Substeps (Number of Simulation) | CG iterations / residual | Source |
|---|---|---|---|---|
| Normal | NOT PUBLISHED | NOT PUBLISHED | NOT PUBLISHED | none |
| Animation (Stable) | NOT PUBLISHED | NOT PUBLISHED | NOT PUBLISHED | none |
| Fitting | NOT PUBLISHED | NOT PUBLISHED | NOT PUBLISHED; Nonlinear Simulation auto-selected | MD, CLO |
| GPU | NOT PUBLISHED | NOT PUBLISHED | NOT PUBLISHED | none |
| Documented example only | 0.03 | 10 (effective 0.003) | n/a | MD, CLO |

Neither vendor publishes the numbers. They are visible in the app (Property Editor shows values after choosing a preset) but not in primary documentation. Do not cite any numbers from third parties as MD fact.

## 8. CPU versus GPU simulation limits

| Item | Finding | Source |
|---|---|---|
| Processor choice | CPU or GPU; selectable in the simulation toolbar with each preset since 2024.2 | MD Simulate |
| Parity | Since 2024.2 GPU collision handling matches CPU "with minimal differences" | MD Simulate |
| Older CLO caveat | GPU "results differ from CPU simulation" | CLO Simulation article [CLO only] |
| Hardware | GPU simulation and rendering rely on NVIDIA CUDA; Intel or AMD only machines get no GPU simulation | MD System Requirements April 2026 |
| Older CLO hardware | Windows only, NVIDIA GeForce GTX 960 or higher | CLO Simulation article [CLO only, dated] |
| macOS | Apple Silicon only (M1 minimum); the page does not say whether GPU simulation runs on Mac | MD System Requirements April 2026 |
| Unsupported accelerations | In 2024.2, GPU did not cover Trim Simulation or Soft Body Simulation | MD Simulate |
| Soft body | "Soft Body Simulation in GPU" added 2025.2 | MD New Feature List |
| Trim simulation | 3D Trim context menu Simulation: Freeze, Activate, Deactivate (2025.0); Show All Trim Status Color (2025.1) | MD Simulate Trim |
| CPU cores | Up to 12 used, auto-detected | MD, CLO |
| VRAM | No VRAM requirement stated | MD System Requirements |
| Recommended GPU | NVIDIA GeForce RTX 4000 series or RTX A, RTX Ada, RTX Pro Blackwell | MD System Requirements |
| Minimum GPU | AMD, Intel or NVIDIA with OpenGL 1.6 or higher; footnote requires OpenGL Compatibility Profile 1.3 or higher | MD System Requirements |
| RAM | 16 GB minimum, 64 GB recommended | MD System Requirements |
| CPU (Windows) | Minimum Ryzen 3000 or Intel 8th gen; recommended Ryzen 9000 or Intel i9 14th gen | MD System Requirements |
| OS | Windows 11 64-bit (24H2 minimum, 25H2 recommended); macOS Sonoma 14 listed, footnote says Monterey 12 since 2024 (page inconsistent) | MD System Requirements |
| Max particle or garment size on GPU | NOT PUBLISHED | none |

Clone note: model CPU as the reference backend; treat the GPU path as optional and gated on CUDA-class hardware. Do not copy the 12-core cap as a requirement.

## 9. Version markers found in this pass

| Version | Item |
|---|---|
| 2.4.0 | + and - keys zoom 3D view |
| 2.5.0 | Divided Gizmo moves Pin hotkey from W to P |
| 4.0.0 | Pins on internal lines and shapes |
| 4.2.0 | Preset flyout on Simulate icon; Sync Fold arrangement angle preference |
| 5.1.0 | Starting simulation switches to Select/Move |
| 12 | Active Tool Hotkey Reference Window; Simulation Speed and Stability Improvements |
| 12.1 | Switch Viewport Layout by Hotkey |
| 2024.1 | Show Layer Depth |
| 2024.2 | Animation (Stable) in toolbar; CPU/GPU parity; CPU or GPU selectable per preset |
| 2025.0 | Advanced Pinching expanded; EveryWear moved to CONNECT; Window menu opens all windows; Simulate Trim; Image resolution graphic option |
| 2025.1 | Pattern Drafter (Beta); Show All Trim Status Color; Sculpt simulation warning |
| 2025.2 | Soft Body Simulation in GPU; Pin Group Selection; Instant Simulation Stop; Script menu renamed Plugins; Library window |
| 2026.1 | Brush Pinching |

## 10. Remaining gaps (need the live app or unopened pages)

| Gap | Suggested route |
|---|---|
| Item list of File, Edit, 3D Garment, 2D Pattern, Avatar, Materials, Render, Display, Help menus | Inspect the application, or the Tool Index linked from the 2024 PDF |
| macOS Command/Option mapping | CLO downloadable shortcut guide for Mac and PC (not fetched) |
| Maya and 3ds Max binding tables | Open the View Control tab in the app; or CLO "Set View Control" article |
| Front/back/left/right/top hotkeys | In-app Shortcuts tab (search by "view"); "Switch Viewport Layout by Hotkey" article (12.1) |
| Undo depth | In-app test; no documentation found |
| Preset numeric values | In-app Property Editor, Simulation Properties, after selecting each preset |
| Pin Group Selection (2025.2) and the 7 Select Mesh/Pin articles | Fetch the Select Mesh/Pin section on the MD Help Center |
| GPU on macOS | Not stated in System Requirements |

## Sources

All fetched during this pass. Primary vendor domains only.

| Short name | URL |
|---|---|
| MD Menu | https://support.marvelousdesigner.com/hc/en-us/articles/47358240361625-Menu |
| MD Main Menu 2025.0 | https://support.marvelousdesigner.com/hc/ja/articles/47358131751449-Main-Menu-2025-0 |
| MD Create Pin | https://support.marvelousdesigner.com/hc/en-us/articles/47358422800537-Create-Pin |
| MD Pin page (delete pin, W key) | https://support.marvelousdesigner.com/hc/en-us/articles/47358146159769 |
| MD Pin to Avatar | https://support.marvelousdesigner.com/hc/en-us/articles/47358452805913 |
| MD Zoom In/Out/Rotation | https://support.marvelousdesigner.com/hc/en-us/articles/47358315717017-Zoom-In-Out-Rotation |
| MD View Control (zh-cn) | https://support.marvelousdesigner.com/hc/zh-cn/articles/47358287009049 |
| MD Graphic Options (zh-cn) | https://support.marvelousdesigner.com/hc/zh-cn/articles/47358263791769 |
| MD Others tab | https://support.marvelousdesigner.com/hc/en-us/articles/47358409175577 |
| MD 3D tab | https://support.marvelousdesigner.com/hc/en-us/articles/47358402111769 |
| MD Set Shortcuts | https://support.marvelousdesigner.com/hc/en-us/articles/47358395237529-Set-Shortcuts-Above-ver4-2-0 |
| MD Shortcuts Hotkeys 2025.0 | https://support.marvelousdesigner.com/hc/en-us/articles/47358120094361-Shortcuts-Hotkeys-2025-0 |
| MD Active Tool Hotkey Reference | https://support.marvelousdesigner.com/hc/en-us/articles/47358326346521-Active-Tool-Hotkey-Reference-Window-Ver-12-2024-2 |
| MD Simulate | https://support.marvelousdesigner.com/hc/en-us/articles/47358232630937 |
| MD Simulation Properties | https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321/ |
| MD Quick/Advanced Pinching | https://support.marvelousdesigner.com/hc/en-us/articles/47358163705369 |
| MD Layer | https://support.marvelousdesigner.com/hc/en-us/articles/47358288780441 |
| MD Wind Controller Setting | https://support.marvelousdesigner.com/hc/en-us/articles/47358383927065-Wind-Controller-Setting |
| MD Freeze/Unfreeze | https://support.marvelousdesigner.com/hc/en-us/articles/47358315135897-Freeze-Unfreeze |
| MD Steam | https://support.marvelousdesigner.com/hc/en-us/articles/47358436596633 |
| MD Simulate Trim | https://support.marvelousdesigner.com/hc/en-us/articles/47358369726233 |
| MD Brush Pinching 2026.1 | https://support.marvelousdesigner.com/hc/en-us/articles/59781111736345 |
| MD History Window | https://support.marvelousdesigner.com/hc/en-us/articles/47358269290393-History-Window |
| MD New Feature List | https://support.marvelousdesigner.com/hc/en-us/articles/47358120307353 |
| MD Key Features by release | https://support.marvelousdesigner.com/hc/en-us/articles/47358241113625 |
| MD System Requirements April 2026 | https://support.marvelousdesigner.com/hc/en-us/articles/47358219834649-System-Requirements-April-2026 |
| MD Manual category | https://support.marvelousdesigner.com/hc/en-us/categories/51985515993625-Manual |
| MD User Guide 2024 PDF | https://s3.marvelousdesigner.com/newmdweb/case/20240626/MD+User+Guide+2024.pdf |
| CLO Edit Simulation Properties [CLO only] | https://support.clo3d.com/hc/en-us/articles/30201166554649-Edit-Simulation-Properties |
| CLO Simulation [CLO only] | https://support.clo3d.com/hc/en-us/articles/115000304968-Simulation |
| CLO Preferences/Settings index [CLO only] | https://support.clo3d.com/hc/en-us/sections/115002522408-Preferences-Settings |
