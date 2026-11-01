# Marvelous Designer: Product Overview, Editions, Version History, UI Layout and File Formats

Research slice 01 of the formfabric encyclopedia. Research date: 2026-11-01 (UTC). Latest release found: Marvelous Designer 2026.1 (August 2026). No 2026.2 was found in searches.

Confidence notation: unmarked claims were confirmed on a fetched primary page (official manual at support.marvelousdesigner.com or marvelousdesigner.com). [single source] means only one fetched source supports it. [secondary] means supported only by trade press (CGChannel, Digital Production). [unverified] means seen in passing or inferred and not confirmed. The official manual pages fetched through the tool were often thin (they carry text and omit screenshot-only detail), so several UI areas are only partially documented here; gaps are listed explicitly in the last section.

## 1. Summary

- Marvelous Designer (MD) is a 3D garment design and cloth simulation application made by CLO Virtual Fashion (Seoul-based; sister product of the apparel-industry application CLO 3D). MD targets 3D artists: games, VFX, animation, film, virtual production, and real-time/metaverse assets. CLO targets real-world apparel production (pattern making, grading, DXF/AI/PDF pattern exchange). MD is explicitly "not recommended for making real patterns" and does not import DXF or PDF patterns; the official reply points users needing that to CLO.
- Core workflow: draw 2D patterns, define seamlines, assign fabric physical properties, place patterns around an avatar in 3D, then run a particle-based cloth simulation that sews and drapes the garment. Output: a simulated mesh, cached animation, and textures exported to DCC/game tools.
- Software is rental-only since MD10 (late 2020). Personal: US$39/month or US$280 for 1 year prepaid. Student and Enterprise (network online) tiers exist. Enterprise standalone licence was discontinued 2025-12-02.
- Versioning: MD1 to MD12 (Nov 2022 = 12.0; 12.1 May 2023; 12.2 Nov 2023), then calendar naming starting with 2024.0 (the release that would have been 13.0). Cadence since 2024: roughly three releases per year (x.0 spring, x.1 summer, x.2 autumn), except 2026 which had 2026.0 (April) and 2026.1 (August) found so far.
- Simulation is CPU by default with an optional NVIDIA CUDA GPU simulation path. Intel/AMD-only systems get no GPU simulation speedup.
- Windows 11 (24H2 minimum) and Apple Silicon macOS are the supported desktop platforms as of April 2026; Linux (Rocky Linux 9.5 / RHEL equivalent) is offered only to Enterprise customers on request.
- Native formats are numerous small files: .zprj (project), .zpac (garment), .avt (avatar), .pos, .mtn, .zfab, .psp, .sst, .btn, .bth, .arr, .pan, .mea, .cmp, .cmt, .smp. Interchange: FBX, OBJ, Alembic, USD/USDZ, glTF 2.0/GLB (2026.0), COLLADA (import only), LXO (export only), MDD/PC2/MC caches, SBSAR (import only), images.

## 2. Product and Maker

| Item | Detail |
|---|---|
| Product | Marvelous Designer |
| Maker | CLO Virtual Fashion (also makes CLO 3D, CLO-SET collaboration platform, CONNECT asset marketplace) |
| Sister product | CLO; MD's own support replies call CLO the "sister software" made for real pattern making |
| Official site | marvelousdesigner.com |
| Official manual | support.marvelousdesigner.com (Zendesk Help Center; per-version "New Feature List" articles) |
| Developer API | developer.marvelousdesigner.com (Python-style API with ExportOBJ, ImportZPRJ, ImportAlembic, ExportUSD, etc.) |
| Related services | CONNECT (asset store, in-app store removed, now a menu), CLO-SET (cloud collaboration, single account sign-in since 2024.0), EveryWear (garment auto fitting/rigging/baking for real-time), LiveSync (Unreal Engine live link, not on macOS), Omniverse Connector (beta in 12.1), USD Layer window |
| Intended users | 3D artists, game and VFX studios, indie creators, students |

## 3. Editions and Licensing

All data from the official licence page (support.marvelousdesigner.com article 47358269350041), cross-checked with CGChannel (2024 to 2026 articles) and Digital Production (2026.0 and 2026.1 articles).

### 3.1 Licence tiers

| Tier | Price | Billing | Notes |
|---|---|---|---|
| Personal, Monthly | US$39/month | Auto-billed, cancel anytime | For individuals, freelancers, hobbyists, sole proprietors (clause 2.1.1) |
| Personal, 1 Year Prepaid | US$280 one time | Not auto-billed | Time-limited access, not perpetual |
| Student | US$8.25/month listed on the licence page; Digital Production 2026.0 article lists US$99 one-time | Not auto-billed | Requires verification via school email or uploaded student ID/enrolment certificate; can be purchased up to two times within four years after verification. The two sources disagree on the price presentation (monthly equivalent vs. one-time), treat as unresolved |
| Enterprise, Network Online Monthly | US$199/seat | Auto-billed | Online connection; node-locked to one device per licence |
| Enterprise, Network Online Annual | US$2,000/seat | Annual | Online; floating licence |
| Enterprise Linux Annual | US$2,300/seat | Annual | Online; floating; Linux build |
| Indie Studio | Contact sales | | Online; floating |
| Enterprise Network Offline | No listed price | | No in-app store (v11+); assets via CONNECT |
| Academic (institutions) | US$300/copy under 10 copies, US$150/copy over 10 | Annual network online | Contact sales |
| Free trial | Free, 14 days | | Personal: "Apply For A Free Trial" button; Enterprise: inquiry form |

### 3.2 Pricing history (secondary, CGChannel)

| Release | Personal | Enterprise node-locked | Enterprise floating |
|---|---|---|---|
| 12.2 (Nov 2023) | $39/mo or $280/yr | $1,700/yr | $2,000 to $2,500/yr |
| 2024.0 (Apr 2024) | $39/mo or $280/yr | $199/mo or $1,900/yr (raised from $1,700 after original post) | $2,000/yr |
| 2025.0 to 2026.1 | unchanged | $199/mo or $1,900/yr (2025), then network online monthly/annual in 2026 | $2,000/yr |

### 3.3 Licence model history

- Before MD10: perpetual licences with hardware keys, plus Steam versions (a Steam page for "Marvelous Designer 9 for Steam" exists in search results; Steam was not covered by any fetched primary page, so the current Steam status is [unverified]).
- MD10 (late Nov 2020): all new licences subscription-only. Personal US$39/month auto-renewing, 40 percent discount for prepaying 12 months. Enterprise Standalone Annual US$1,700 (new) and Network Annual raised to US$1,900. Student US$15/month plus a 3-year discount, once per user. Hardware keys no longer provided for Personal and Enterprise Standalone. All licences except Enterprise Network Offline need an internet connection.
- Perpetual holders: existing licences stay usable indefinitely; those less than 3 versions behind could do a Version Update up to MD12; "License Trade" converts perpetual to subscription (Personal perpetual to 1-year subscription, Enterprise perpetual to 2-year). MD12 was "the final one to which holders of old perpetual licences can upgrade" (CGChannel).
- 2025-12-02: Enterprise Standalone licence discontinued, replaced by Network Online monthly/annual.

### 3.4 Clone implications

- No licence enforcement is needed for formfabric, but the trial/subscription structure shows the product assumes online account sign-in (CLO-SET account) for asset download and CONNECT.
- The Linux build exists only for Enterprise, so an open-source cross-platform clone fills a real gap.

## 4. Release History

Dates: where the manual and press differ, both shown. Official manual feature lists are the primary source.

### 4.1 Early versions (pre-12)

Only fragments were verified from approved sources in this pass. A full chronology of MD1 to MD11 was not obtained. Known fragments:
- Object Browser Fabric drag-reorder: 4.1.0. Library list/icon view: 2.3.0. Library search: 3.0.0. AI Bezier curve drawing (Illustrator-style): 3.0.0. UV Editor mode: 7.5. Sculpt mode: MD8. Random Color Surface: 5.0.0. Opaque surface: 5.1.4. Alembic export on Mac: 5.1.431. Library "Download" button: MD10. Object Browser merge: 11+. No in-app store in network offline: 11+. "Show in Explorer" in Library: 12.1.
- MD10 introduced subscription-only licensing (Nov 2020).
- A CGPress item exists titled "marvelous designer 12 1 released with usd support" which agrees with the manual (USD in 12.1).

### 4.2 MD12 family (2022 to 2023)

| Version | Date | Headline features |
|---|---|---|
| 12.0 | 2022-11-15 | Automatic keyframes when recording simulation caches (editable, loopable, blendable); styleline editing in 3D view updating 2D patterns; retopology options (select topology points, snap to internal lines/shapes); Spiral tool; custom OBJ as zipper slider/puller with per-part materials; Substance materials on trims, buttons, avatars; automatic fitting suit on character import; merge vertices by proximity at export; joint keyframe animation exported in FBX; hotkey reference window for active tool; import FBX as trims; normal map blend method change; simulation speed and stability improvements |
| 12.1 | 2023-05-25 | USD file compatibility; Omniverse Connector (beta); drag-and-drop background images; viewport layout switching by hotkey; statistics display; New project/New garment commands; Show all avatars; fit frame range and animation layers; retopology face selection and 3D topology-only display; window main menu |
| 12.2 | 2023-11-21 | Automatic UV packing (preserve rotation/scale, separate UV tiles per garment); FBX import/export with multiple avatars; Roll Up (sleeve/cuff fold); double-click selects all sewn patterns; USD unified UV coordinates; clone as pattern (with sewing); optimize all curve points dialog; improved History 3D state preview; auto fitting keeps texture size; silent install; FBX axis conversion; default avatars replaced with Mia and Luka; topstitch alignment to pattern outline; Library select-all for downloads, list-view downloads, file extension display (zprj, zpac); reworked Snapshot dialog |

### 4.3 2024 releases (calendar naming begins; would have been 13.0)

| Version | Date (manual / press) | Headline features |
|---|---|---|
| 2024.0 | Feb 28 2024 (manual) / Apr 29 2024 (CGChannel article; build 2024.0.173 later added UE5.4 USD workflows) | Soft body simulation (simple skin elasticity for cloth to skin contact; default avatars only; skin deformation exports as Alembic only); multiple wind controllers; automatic conversion of imported characters to native avatars; scaling of custom FBX/OBJ avatars; EveryWear (auto fitting, rigging, texture baking, LOD creation); LiveSync with Unreal Engine (not macOS); animation recording up to 1080p; Puckering (fine wrinkles at seams); remesh reorganised into Quad (Grid) and Triangle/Quad (Optimized); polygon count optimisation for buttons, zippers, trims; USD export single or multiple objects, USD export includes simulation data; 18 MetaHuman body types in asset library; single CLO-SET account sign in; arithmetic in numeric fields; Ctrl+drag partial pattern selection; project thumbnail as file icon; Alembic import axis conversion; expiry date in About dialog |
| 2024.1 | 2024-06-17 | Layer depth shown in 2D window; improved UV texture seam handling; show/hide colour menu; new UV packing nesting algorithm; "Density (g/m2)" renamed "Weight (gsm)"; Daz avatar Auto Convert dialog change; opacity map improvements; trim collision handling improvements |
| 2024.2 | 2024-10-21 | Trim simulation with collisions; Pinching supports soft selection; EveryWear masking thin/thick and better auto rigging; pins duplicated to symmetric pattern; roughness map range expanded; buttonhole Z-offset; template avatar smoothing; USD layer carries simulation data; cached avatar animation exports to USD; FBX imports as scenes and props; axis conversion in USD import/export; thick/thin mesh in USD layer export; custom thumbnail when saving ZSE files; register poses and motions for avatars; piping closed-end option; Wireframe on Monochrome (Thick) display style |

### 4.4 2025 releases

| Version | Date | Headline features |
|---|---|---|
| 2025.0 | April 2025 (CGChannel 2025-04-30) | Keyframing extended to avatar joint positions, fabric and simulation properties and wind; IK joint mapping for imported avatars (auto for Daz 3D, Mixamo, Character Creator, MetaHuman naming); Auto Convert to Motion (FBX to MTN); Fur strands (experimental, in 3D window and test renders, exportable in USD); UV generation for sides and back; better all-quad conversion; recolouring base colour map on PBR export; two-way zippers; USDZ export; modular library (reusable garment parts, integrated with CLO-SET and CONNECT); new Library window; Schematic render (Alt+0); Auto Sewing; garment fit properties; Object Browser tab layout with Graphic Style Browser; custom UI layout save/load; Avatar editor with IK joint feature and Fitting Suit moved in; Sculpt integrated in 3D window; Animation Editor with keyframe animation; 2D snapshot and multi-view 3D snapshot (turnaround); advanced pinching; tack across multiple vertices; many new seamline property types; custom OBJ topstitch assets; UI renames; window switcher removed, hotkey list removed, in-app CONNECT store removed |
| 2025.1 | August 2025 (CGChannel 2025-08-21) | 3D pen strokes off the avatar; keep mesh topology after auto fitting; Pattern Drafter (beta); soft body for custom avatars; denim wet wash effect; AI pose generator (beta); zipper style options; group internal lines/shapes; mirror-paste trims; blend prints with fabric; avatar wireframe display; import/export presets; show/hide scenes and props; enhanced masking selection; partial auto rigging; clamp animation range to selected layers; schematic render improvements; warning before sculpt simulation; flatten multiple areas into one merged pattern; Zipper tab in Object Browser |
| 2025.2 | November 2025 (CGChannel 2025-11-17, Digital Production 2025-11-18) | Keyframable shrinkage, pressure, solidify, tack, trim weight; convert patterns to Trim Styles or avatar accessories (trims are rigid, not simulated); local FBX export from EveryWear (no CONNECT upload); MetaHuman DNA import (native, round trip to Unreal Engine); Pleats tool (knife, box, accordion; number, depth, interval, fold angle); Library (Legacy) merged into new Library window |

Platform at 2025.0 to 2025.2 press time: Windows 10+ and macOS 12+. Windows 10 support ended on 2025-10-14 per the April 2026 requirements page.

### 4.5 2026 releases

| Version | Date | Headline features |
|---|---|---|
| 2026.0 | April 2026 (Digital Production 2026-04-15; CGChannel 2026-04-20) | Lacing tool (click holes in order, Enter or double-click ends, Backspace undoes last step, Esc/Ctrl+Z cancels; generates an editable lace pattern; reversible cord direction and curvature slider); 3D Pencil (sketch on avatar, locked front/back view, eraser, converts to editable 3D pen lines); preserve 3D shape when 2D patterns are scaled; template-based rigging for EveryWear (beta); Toon material type ("Express Cartoon": base colour/opacity, shadow colour/hardness/shift, emission, MatCap, rim light with fresnel power and lift, outlines with mode/mask/colour/width in mm); GPU accelerated trim simulation; steam eraser brush; keep graphics and seamlines on fur; fabric-aware strain maps; Pattern Drafter edit mode; fit maps shown on 2D patterns; pattern archive; symmetric line selection in half-symmetry pattern trace; isolate selection; animation timeline markers; open selected material from ZFAB and swap front/back; zipper usability; arithmetic in ghost/numeric fields updates in real time; CONNECT download folder path; docked library preview; expanded glTF 2.0 and VRM support [secondary: the manual page fetched only describes glTF]; play-region selection for FBX and glTF export; keyframe baking for joint animation export; FBX import options (Open/Add, scale, axis conversion, joint/cache animation toggles, trim collision handling and glue); FBX export options (single/multiple objects, weld/un-weld, thin/thick, unified UV and texture with size and seam fill); "Weld Turned Sewing Lines" for vertex merge at export |
| 2026.1 | August 2026 (CGChannel 2026-08-24; Digital Production 2026-08-25) | Seamline Rip tool (beta; tear opens when simulated force exceeds Rip Resistance, derived from Yield Strength and Damage Rate; batch creation; paired rip object); Fray system (generation threshold, bridging, root offset, gravity, fuzz, loose warp, loose weft, exposed strands; ZFRAY presets; disabled on Mac; not yet exportable as mesh); Brush Pinching (Size, Focal Radius, Lazy Distance, Strength, Push Inward, Average Radius); Pinching Simulation Override (Override Strength 0 to 1 blends original and override values; Update/Reset Rest Shape); Blend Shape Animation (weights 0 to 100, keyframes); camera interchange via FBX and glTF 2.0/GLB (one Main Camera; Default Camera not deletable); Variable Fold Angle (gradient from chosen angle 0 to 360 degrees to flat 180); zipper placement and Extend property; Autosave (interval, max files, retention days, file type, path); toolbar customisation (reorder, hide, regroup, save as JSON tool sets; cannot move tools between 2D and 3D toolbars) |

2026 pricing as listed: Personal $39/mo, $280 prepaid year; Enterprise Network Online $199/seat monthly, $2,000/seat annual; Linux $2,300/seat annual; 14-day trial.

## 5. System Requirements (official page "System Requirements April 2026")

### 5.1 Windows

| | Minimum | Recommended |
|---|---|---|
| OS | Windows 11 64-bit 24H2 | Windows 11 64-bit 25H2 |
| CPU | AMD Ryzen 3000 series or Intel 8th Gen | AMD Ryzen 9000 or higher, or Intel i9 14th Gen or higher |
| RAM | 16 GB | 64 GB |
| GPU | AMD/Intel/NVIDIA with OpenGL 1.6 or higher (footnote says OpenGL Compatibility Profile 1.3+; the page conflicts with itself) | NVIDIA GeForce RTX 4000 series or RTX A / RTX Ada / RTX Pro Blackwell |
| VRAM | not specified | not specified |
| Storage | SSD, 10 GB | SSD, 20 GB |
| Display | 1920x1080 | 2560x1440 or 3840x2160 |
| Input | 3-button mouse, internet connection | same |

### 5.2 macOS

- Minimum Apple Silicon (page prints "M11", almost certainly a typo for M1); recommended M1 to M5.
- macOS: table says Sonoma 14 minimum; a footnote says minimum became Monterey 12 from the 2024 release; recommended Sequoia 15 or Tahoe 26. Intel Macs: Apple ends support with macOS Tahoe.
- RAM 16 GB minimum, 64 GB recommended. Display 1920x1080 min; Retina 4K/5K recommended. Boot Camp and desktop virtualisation unsupported.

### 5.3 Linux

Rocky Linux 9.5 or RHEL equivalent. Enterprise only, on request; separate setup guide exists.

### 5.4 June 2024 page (historic comparison)

Windows 10 64-bit 1903+ (min) / Windows 11 23H2 (rec); CPU Ryzen 5 1500X or Core i5-6400 (min); RAM 8 GB min, 32 GB rec; GPU GTX 960 or Quadro P2200 min, RTX series rec; macOS Monterey 12 min, Intel Macs after 2018 or Apple Silicon recommended. Windows N editions need the Media Feature Pack. Catalina supports up to 12.2; v11+ does not run on Mojave or earlier.

### 5.5 GPU and CPU simulation

- "Marvelous Designer utilizes Nvidia CUDA acceleration technology for GPU-Simulation" (also rendering). Computers with only Intel or AMD graphics do not get the GPU speedup; simulation still runs on CPU. The page names no CUDA compute capability or VRAM minimum.
- 2026.0 adds GPU accelerated trim simulation [secondary, Digital Production].
- 2026.1 press notes list a "macOS GPU Simulation" feature but the processing backend is not specified [single source, treat as unverified].
- Memory guidance (official): 16 GB basic, 32 GB smoother, 64 GB or more for very heavy garments and animation.
- Other notes: garment pattern digitisers unsupported; Wacom tablets tested; 32-bit OS, mobile OS, ChromeOS unsupported; installers are .exe or .dmg only; VMs and cloud run but are unsupported.

### 5.6 Clone implications

- The reference product is CPU-capable with an optional CUDA path. A clone should treat GPU simulation as a backend plug-in (compute shaders or Vulkan/wgpu would be the open equivalent).
- Quoting 64 GB recommended shows the simulation is memory heavy: dense particle meshes plus collision structures.

## 6. Workspace Modes

Source: official "Mode" article. Switch with the button at the top right of the window.

| Mode | Purpose | Introduced |
|---|---|---|
| Simulation | Create/edit patterns in 2D; drape garments, poses, actions in 3D | original |
| Animation | Record, play back, edit garment animation (Animation Editor, keyframes, layers) | original; keyframe system expanded 12.0 and 2025.0 |
| Modular | Combine garment components (sleeves, collars) as blocks to test designs | exists in docs; modular library 2025.0 |
| UV Editor | Edit UV mapping of the garment | 7.5 |
| Sculpt | Shape garments with brush tools; integrated into the 3D window in 2025.0, with a warning dialog before sculpt simulation in 2025.1 | MD8 |

Also: Pattern Drafter (beta, 2025.1) listed among viewport windows; Avatar Editor (separate editor with size, measurement, arrangement, IK joint, Fitting Suit); EveryWear exporter (CONNECT menu).

## 7. UI Layout

Primary sources: Main Menu 2025.0, Layout Window, Object Browser, Property Editor, Library Window, Menu, Functional Improvement (2025.0 UI changes) articles.

### 7.1 Overall structure

- Main Menu at top left of the title bar. Menus include File, 3D Garment, 2D Pattern, Display, Window, plus Render and CONNECT (added 2025.0; EveryWear moved under CONNECT). Full per-item menu text was not recoverable from the fetched pages (screenshots only). Known paths: Main Menu > 3D Garment > Select/Move; Main Menu > 2D Pattern > Create > Polygon / Internal Polygon/Line; Main Menu > Display > Viewpoint > Zoom Extends All; Main Menu > File > Import/Export > (format); Main Menu > Window (or Display > Window in older builds) for opening windows.
- Workspace windows (the central viewport area): 3D Window, 2D Pattern Window, UV Editor, Pattern Drafter (beta 2025.1). Up to four can be shown at once; they cannot be closed or hidden. Alt+W toggles between the current layout and a full-screen view of the selected window (12.1). Viewport layout switching by hotkey was added in 12.1.
- Side panel windows: Library (new unified Library in 2025.2), Modular Library, Object Browser, Property Editor, Animation Editor, History, USD Layer, Omniverse Connector. Side windows can be closed (X), hidden to an edge tab (arrow), grouped as tabs by dragging a pop-up onto an existing side window, resized by dragging edges, duplicated as pop-ups.
- Right-clicking a title bar opens the window list: viewport title bar lists viewport windows; side panel title bar lists side windows. Windows can also be opened from Main Menu > Window.
- Toolbars: separate 2D and 3D toolbars with tool groups (hover a group to show sub-tools). The control handle at the toolbar's left moves it to top, bottom, left, or right. A show/hide toggle sits to the right of the view icons. Clicking the dashed scissors line or pressing Enter duplicates a tool group or the main menu as a pop-up. Modular Editor toggle sits on the 2D toolbar. 2026.1: tools reorderable, hidable, regroupable within the 2D or the 3D toolbar, and saved as JSON tool sets.
- Custom UI Layout (2025.0): save and load layouts; "Reset to Default Layout" at the bottom right restores the default. Light, Wind and Camera controller icons changed in 2025.0.
- Removed in 2025.0: Window Switcher drop-down, Hotkey List window, in-app CONNECT Store. 12.0 had added a per-tool hotkey reference window.
- Statistics display added in 12.1.

### 7.2 3D Window

- Purpose: simulate garments and pose avatars to make animations.
- Opened via Main Menu > Window > 3D Window or title bar right-click.
- Right-click pop-ups: 3D Background, Avatar, Pattern menus.
- Select/Move tool (Main Menu > 3D Garment > Select/Move or 3D toolbar): with Simulate active, dragging a garment part moves it physically (fine-tune fit); with Simulate inactive the window enters pattern arrangement mode and dragging moves individual patterns.
- Pinching (2024.2 soft selection; 2025.0 advanced pinching; 2026.1 brush pinching and simulation override).
- Sculpt integrated in the 3D window (2025.0).
- Copy/paste patterns in 3D (article exists; contents not fetched).
- Styleline editing in 3D (12.0): reshape existing stylines or draw new ones in the 3D view; 2D patterns update.
- 3D Pencil and Lacing tools (2026.0).

### 7.3 3D camera navigation

| Action | Input |
|---|---|
| Rotate | Hold right mouse button and drag |
| Pan | Hold mouse wheel (middle button) and drag; Mac Magic Mouse: Alt + left drag |
| Zoom | Mouse wheel (documented as scroll down zooms in, scroll up zooms out); Alt + left drag (drag up zooms out, down zooms in); + and - keys (hold) |
| Preset views | Numeric keys: 2 Front, 3 3/4 Left, 8 Back, 6 Left, 4 Right, 1 3/4 Right, 5 Top, 0 Bottom (the CLO page labels it "Button", presumably Bottom) |
| Frame selection | F enlarges and centres the selected object |
| Zoom to extent | Main Menu > Display > Viewpoint > Zoom Extends All |
| Alternate schemes | Maya and 3ds Max navigation styles are supported (stated by the manual); settings not detailed |

Note: the CLO-hosted view controls page is CLO 3D documentation; MD documentation says the same keys but only the rotate/pan/zoom portion was confirmed on the MD site. The 2D window navigation controls were not documented on the pages fetched [unverified].

### 7.4 3D display modes (official "3D Render Style" article)

Render modes:
- Quality Render (toggle icon; slower, better quality).
- Schematic Render (Alt+0, 2025.0).

Garment rendering styles:

| Style | Hotkey | Notes |
|---|---|---|
| Thick Textured Surface | Alt+1 | Textures plus thickness |
| Textured Surface | Alt+2 | Texture, no thickness |
| Thick Monochrome Surface | Alt+3 | 2025.0 |
| Monochrome Surface | Alt+3 / Alt+4 (page lists both) | No textures |
| Translucent Surface | Alt+4 / Alt+5 (page lists both) | Opaque Surface also listed (5.1.4) |
| Thick Textured Surface (Back) | | Both sides keep original colour, 2025.0 |
| Random Color Surface | | Distinguish patterns (5.0.0) |
| Wireframe on Monochrome (Thick) | | 2024.2 |

Options: Show Wireframe; Show Fur Strand and Random Fur Strand Color (2025.0).

Garment fit maps: Stress Map (Alt+6, below v12), Strain Map (Alt+7, percentage of external force effect), Fit Map (Alt+8, tightness colour, below v12), Pressure Points renamed Pressure Map in 2025.0 (avatar-to-garment contact). 2025.0 added a Stress map option in 3D display fit maps; 2026.0 added fabric-aware strain maps and fit maps on 2D patterns.

Avatar rendering styles: Textured Surface, Monochrome Surface, Translucent Surface (2025.1), Mesh/Show Wireframe (2025.1).

### 7.5 2D Pattern Window

- Purpose: design patterns, set seamlines, edit fabric textures.
- Pop-ups: 2D Background, Pattern, Line/Point, Sewing Line, Topology, Texture, Topstitch menus.
- Polygon / Internal Polygon/Line tool uses Illustrator-style Bezier drawing: click-drag to place a point and set curvature with the blue handle; Esc or Ctrl+Z cancels the whole polygon; Delete or Backspace removes the last point (since 3.0.0).
- 2025.0: Layer depth shown in 2D (2024.1), 2D snapshot, new 2D garment rendering styles, UV editor anti-aliased line option, individual editing range for curve points, many new seamline property types. 2025.1: pattern drafter beta. 2026.0: pattern drafter edit mode, symmetric line selection, point selection on smooth curves.
- Right-click a 2D pattern: Select All with Same Property (Particle Distance, Layer, Fabric).
- Detailed 2D tooling is the subject of research slice 02 (see 02-pattern-2d.md).

### 7.6 Object Browser (2025.0 version)

Open via Main Menu > Window > Object Browser or side panel title bar right-click.

| Tab | Content / operations |
|---|---|
| Scene | Every item and its structure across 3D and 2D windows; creation order; click Name header to sort A to Z then Z to A; quickest way to find and select components; now includes all style windows and scene window (2025.0) |
| Fabric | All fabrics; select, add, copy, reset; drag to reorder (4.1.0+) |
| Graphic | Select, add, copy |
| Button | Select, add, copy |
| Buttonhole | Select, add, copy |
| Topstitch | Select, add, copy |
| Puckering | Select, add, copy |
| Trim | Select, add, copy |
| Zipper | 2025.1; select, add, copy |

Merge (v11+): select multiple items in a tab, right-click, choose merge; the selection combines into a new default item (Fabric, Button, Buttonhole, Topstitch). Graphic Style Browser added in 2025.0.

### 7.7 Property Editor

Opened from the side panel title bar (2025.0). Context-sensitive editors for: Pattern, Internal Shape, Sewing, Stitch, Fabric (via fabric list), Button, Buttonhole, Avatar (pattern-related properties), Simulation (right-click 3D window > Simulation Properties; open/save/edit simulation settings; saved as .smp), Wind (right-click 3D window > Wind Property), and the 2D Window. Sections and parameter ranges are not described on the fetched page; they belong to other slices (fabric, simulation). 2025.0 renamed some avatar property editor items and added eye control; "Simulation Quality" became "Number of Simulation".

### 7.8 Library Window

- New Library (2025.0; Legacy Library merged into it in 2025.2). Open via Main Menu > Window > Library.
- Bookmarked folders at top or bottom; folders with an "N" flag contain new downloadable items.
- Shows thumbnails for Project and Garment files; hover previews front, back, side views. Open by drag into the 3D window or double-click.
- Buttons: Download (MD10, fetch new default files), Reset (registered folders), Add Folder, Refresh, Select All (replaces Refresh when the downloadable-checkbox is on), List/Icon (2.3.0), Search (3.0.0).
- Folder context menu: Delete (bookmark only), Rename (bookmark only), Show in Explorer (12.1), Add to Workspace.
- 2026.0: docked-library preview.
- Modular Library (2025.0): garment parts saved as reusable blocks, integrated with CLO-SET and CONNECT.

### 7.9 History, Animation Editor and other panels

- History window: lists project states with a 3D state preview (improved in 12.2). Details (undo depth, step naming) not documented on fetched pages [unverified].
- Animation Editor: keyframe animation (2025.0), animation layers and fit frame range (12.1), clamp animation range to selected layers (2025.1), timeline markers (2026.0), min/max range saved in project (2024.0).
- USD Layer window and Omniverse Connector window (12.1) exist for USD live workflows.

### 7.10 Clone implications

- Layout model: four fixed workspace viewports plus dockable, tabbable, hideable side panels plus movable toolbars with JSON-saved tool sets. A docking framework (Qt-ADS, Dear ImGui docking, or web equivalents) maps well.
- Hotkeys documented: Alt+0 to Alt+8 for render style/fit maps, F, numeric keypad view presets, Alt+W window maximise, +/- zoom.

## 8. File Formats

### 8.1 Native formats (official "Marvelous Designer File Format" article)

| Extension | Type | Stores | Notes |
|---|---|---|---|
| .zprj | Project | Bundles the files used for garment creation into one file; includes the AVT and ZPac files plus animation file if present | Largest container. The project load can be partial (API ImportZPRJ: append, load garment, avatar, scene and props, render properties, custom view, display settings, with translation offset) |
| .zpac | Garment | Garment, pattern, sewing and fabric information | Library shows file extension (zprj, zpac) since 12.2 |
| .avt | Avatar | Model, sizing, poses, motions, accessories, bounding volumes, arrangement points | Encrypted MD/CLO avatars (including MetaHuman body types in avatar folders) cannot be exported |
| .pos | Pose | Avatar pose | Saved automatically with garment; poses from other software saved only for COLLADA avatars with joints |
| .mtn | Joint Motion | Avatar motion | Only for COLLADA avatars with joints and animation; 2025.0 Auto Convert to Motion turns FBX into MTN |
| .zfab | Fabric | Fabric property values: texture and physical properties | 2026.0 allows opening a selected material from ZFAB and swapping front/back |
| .psp | Physical Property | Detailed fabric physical property values | |
| .sst | Topstitch | Type, number of lines, thread thickness, SPI, length | |
| .btn | Button | Type, size, colour | |
| .bth | Buttonhole | Style, size, colour | |
| .arr | Arrangement Point | Blue points around avatar for placing patterns | Sits on top of the arrangement bounding volume |
| .pan | Arrangement Bounding Volume | Green cylinders covering body parts; each end bound to an avatar joint so they follow pose | |
| .mea | Tape Measure | Tape measure data created on avatar | |
| .cmp | Camera Projection | Field of view, near/far | |
| .cmt | Camera Transformation | Camera view for the 3D garment window | |
| .smp | Simulation Property | Simulation settings | |

Additional extensions seen in release notes but not described in the file format article:
- ZFRAY: fray presets (2026.1, Digital Production) [secondary, single source].
- ZSE: custom thumbnail can be set when saving ZSE files (2024.2). Contents and meaning [unverified]; likely a scene/style file, not confirmed.
- Not found in any approved source: .zmdr, .zpos. They could not be verified; do not assume they exist. Note the requested list included them and they may come from third-party format-explainer sites (renderhub guides appeared in search results but were not used).
- Version introduction dates for most native extensions were not published; the .zprj/.zpac/.avt set exists in all current versions and is inherited from the MD1 to MD12 lineage. The Library showing `.zprj`/`.zpac` extensions dates to 12.2.

### 8.2 Interchange formats (official "Compatible File Format" and "Which file formats are compatible" articles)

| Format | Ext | Import | Export | Contents and notes |
|---|---|---|---|---|
| FBX | .fbx | Yes | Yes | Mesh, material, joint, animation, camera. Only joints imported as FBX can be exported as FBX. Multi-avatar since 12.2; axis conversion since 12.2; joint keyframes since 12.0; import as trim/scene/prop (12.0, 2024.2); camera interchange 2026.1; options expanded 2026.0 |
| OBJ | .obj | Yes | Yes | Geometry and materials; mainly avatar import; custom OBJ for zippers (12.0) and topstitch assets (2025.0) |
| Alembic | .abc | Yes | Yes | Vertex and animation data; Mac export from 5.1.431; axis conversion on import 2024.0; export can include current frame 2024.0; skin deformation exports as Alembic |
| OpenCOLLADA | .dae | Yes | No | Geometry, material, rigging, skinning, animation; needs OpenCOLLADA plugin when exporting from 3ds Max or Maya |
| USD | .usd | Yes | Yes | Mesh, material, joints, animation, cameras, scene layout; introduced 12.1; unified UV 12.2; single/multiple objects and simulation data 2024.0; avatar cache animation 2024.2; fur strands 2025.0 |
| USDZ | .usdz | No | Yes | Compressed package for AR/VR; 2025.0 |
| glTF 2.0 / GLB | .gltf, .glb | Yes | Yes | Added to File > Import/Export in 2026.0; see 8.4 |
| Animation caches | .mc, .pc2, .mdd | Yes | Yes | Vertex animation point caches |
| LXO | .lxo | No | Yes | Modo; geometry and materials |
| 3D snapshot | .png | N/A | Yes | Single view of 3D window; multiview turnaround since 2025.0 |
| Textures | .jpg, .png, .tif, .psd, .bmp, .tga, .dds, .hdr, .exr, .jp2 and many more | Yes | Yes | Long list in manual: JPEG, BMP, WBMP, ICO, TGA, TIFF, GIF, MNG, JNG, DDS, HDR, KOA, LBM, IFF, PBM, PCD, PCX, PGM, PPM, RAS, CUT, XMB, XPM, SGI, G3, EXR, J2K, JP2, PFM, PICT, PCT |
| Substance | .sbsar | Yes | No | Procedural textures; channels (colour, normal, metalness, roughness, opacity) can be baked and exported |
| VRM | .vrm | [unverified] | [unverified] | Mentioned only by Digital Production as "expanded glTF and VRM support" in 2026.0 |
| Video | MP4 etc. | N/A | Yes | Animation video export (API VideoExportOption: width, height, fps 24/25/30/60, start/end frame); 1080p max from 2024.0 |

### 8.3 Pattern exchange formats: DXF, AI, PDF

- Marvelous Designer does not import or export DXF, AI or PDF patterns. Official support replies state PDF and DXF patterns cannot be imported and AI/PDF pattern export is a CLO premium feature, not an MD one. Use File > Export and the UV Editor window for exports.
- Illustrator curve workflow: MD has an Illustrator-style Bezier tool (3.0.0) but no direct import of .ai curves is documented. A Compatibility section article "Are the Adobe Illustrator curves able to be imported and edited" exists; its text was not retrieved [unverified].
- 2D background images (JPG, PNG) can be imported as a tracing reference; 12.1 added drag and drop. 2026.0 mentions "pattern trace" from images [secondary].
- Clone implication: formfabric can legitimately exceed MD by supporting DXF-ASTM, AI/SVG and PDF pattern import, since MD lacks these.

### 8.4 glTF export and import options (2026.0 manual)

Import (Main Menu > File > Import > glTF 2.0): Open Type (open or add to scene); Object Type (avatar or trim); Arrangement Points auto-generate (bounding volume and arrangement points); Scale (value or unit, default metres for glTF); Axis Conversion (invert axes).

Export (File > Export > glTF 2.0): Object (Select All, Avatars, Patterns, Graphics, Trims), single or multiple objects, Thick or Thin, Unified UV Coordinates; animation: Light, Garment Animation, Avatar Joint Animation, Bake Keyframes (a key per frame), Avatar Cache Animation; Basic: scale/unit and axis conversion; Animation range: entire region or Play Region only; format: glTF-Embedded, Save and Compress (ZIP), Diffuse Color Combined on Texture (hidden when Unified UV is on), Exclude Ambient Map and Color (Modo, automatic); Save with Meta Data (extras): patterns mesh, physical properties and sewing information stored as XML metadata.

### 8.5 API-documented option parameters (developer.marvelousdesigner.com ApiTypes)

Useful as the de facto schema of import/export options:
- ImportExportOption (OBJ, glTF, FBX and import): scope (export garment/avatar, include hidden objects, include inner shapes, selected components); OBJ structure (single object, weldType, thin); unified UV and textures (cloth and avatar unified UV, texture size, seam fill, bake margin, per-map toggles for diffuse, normal, metalness, roughness, opacity, displacement); maps and colour (metallic-roughness map for glTF, opacity map, combined diffuse, exclude ambient); scale and axes with per-axis invert; packaging (save in zip, embedded, export light, export fabric, metadata, fbxSdkVersion where 0 = 2020 through 6 = 2012, include avatar shape for FBX/Alembic); import (object type: 0 Avatar, 1 Trim, 2 Garment; size and pose from avatar, auto translate, create camera, move garment, add arrangement points, auto create fitting suit, add, trace 2D patterns from UV map, translation X/Y/Z); animation (avatar joint, avatar cache, avatar blend shape, cloth cache); component lists (pattern, button, graphic, zipper).
- ImportZPRJOption: append, load garment, avatar, scene and props, render properties, custom view, display settings, translation and unit.
- ExportUSDOption: up axis (1 = Y up, 2 = Z up), metersPerUnit, avatar/garment/scene-and-prop assignment with prim paths, export simulation data, export strand data.
- ImportAlembicOption: load type, unit, scale, load object type (0 Avatar, 1 Morph Target), morph frame count, arrangement points, fitting suit, move garment to start, trace 2D patterns from UV map, axis conversion, translation.
- VideoExportOption: width, height, fps (0=24, 1=25, 2=30, 3=60, -1 system), start and end frame.
- 52 option types listed; APIs named include ExportOBJ, ExportAnimationVideo, ImportZPRJ, ImportAlembic, ImportPose, ExportUSD, GenerateZippersFromObj, wind controller get/set, pattern snapshot size/line/image/information options, avatar measurements, GetObjectBrowserItems, GetMaterialProperties/SetMaterialProperties, PlaceButtonPairsOnEdge, PlaceTopstitchOnInternalLine, graphic reposition/replace/remove, hi-res tolerance.
- Material options cover base colour, normal, displacement, opacity, roughness, metalness and reflection maps, with map transformation.

### 8.6 Avatar import rules (official)

- Avatars without joints: OBJ, FBX, Alembic, COLLADA.
- Avatars with joints and animation cache: FBX and Alembic recommended.
- Auto conversion to native avatar and scaling arrived in 2024.0; IK joint mapping and Auto Convert to Avatar/Motion in 2025.0; MetaHuman DNA import in 2025.2.

## 9. Behaviour Notes Relevant to Building a Clone

1. Project container: .zprj bundles garment (.zpac), avatar (.avt), animation and optionally scene/props, render properties, custom views, and display settings. A clone should define a zip-style container with separately addressable sub-documents.
2. Fabric, topstitch, button and buttonhole are first-class reusable assets with their own file types, shown as tabs in the Object Browser.
3. Simulation and wind are separate property sets (.smp), editable and savable.
4. Camera and arrangement files (.cmp, .cmt, .arr, .pan) show that views and avatar placement helpers are persisted independently.
5. Interchange emphasis is game/VFX pipeline: FBX, Alembic, USD, glTF, caches (MDD, PC2, MC). No DXF/PDF/AI pattern exchange.
6. Trims are rigid, non-simulated geometry (2025.2); fur, fray, rip, and lace are newer layered systems on top of the cloth core.
7. Autosave exists only since 2026.1 with configurable interval, max count, retention days, type and path.

## 10. Gaps and Items Not Verified

- Full chronological list of MD1 to MD11 release dates and features (not retrieved from an approved primary source).
- Complete Main Menu item list and all hotkeys (the official pages are screenshots).
- 2D window navigation and display options; History window details; Toolbar tool inventory.
- .zmdr, .zpos, VRM file details, ZSE meaning, .ai curve import answer.
- Whether a Steam edition is still sold in 2026.
- Whether a 2026.2 release exists after 2026.1.
- Student price discrepancy (US$8.25/month vs US$99 one-time).
- macOS GPU simulation backend.

## Sources (URLs actually fetched)

- https://support.marvelousdesigner.com/hc/en-us/articles/47358120307353 (2025.0 and 2025.1 feature list)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358170563481 (2024.0 to 2024.2 feature lists)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358241113625-Marvelous-Designer-12-12-1-12-2-New-Feature-List
- https://support.marvelousdesigner.com/hc/en-us/articles/55837641308313 (2026.0)
- https://support.marvelousdesigner.com/hc/en-us/articles/55686813557913/ (2026.0 glTF import/export)
- https://www.marvelousdesigner.com/support/news/view/448aa11bc06d4b91a63668ca1d12a96e (2026.1 news)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358219834649-System-Requirements-April-2026
- https://support.marvelousdesigner.com/hc/en-us/articles/360036950292-System-Requirements-June-2024
- https://support.marvelousdesigner.com/hc/en-us/articles/47358269350041-What-kind-of-licenses-does-Marvelous-Designer-offer
- https://www.marvelousdesigner.com/support/news/view/ed03ffe524f444bf80d86d28bdab32c1 (licence changes, MD10)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358169252633 (Marvelous Designer File Format)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358272195609-Compatible-File-Format
- https://support.marvelousdesigner.com/hc/en-us/articles/47358199862553 (supported formats table)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358186497817-Which-file-formats-are-compatible-with-Marvelous-Designer
- https://support.marvelousdesigner.com/hc/en-us/articles/47358210334745-Can-I-import-my-own-avatar
- https://support.marvelousdesigner.com/hc/en-us/sections/47114871506457-Compatibility
- https://support.marvelousdesigner.com/hc/ja/community/posts/48268925915673/comments/48268941020441 (AI/PDF pattern answer)
- https://support.marvelousdesigner.com/hc/ja/community/posts/48267712674585/comments/48267712824089 (PDF/DXF import answer)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358334390809-Create-AI-Bezier-Curve
- https://support.marvelousdesigner.com/hc/en-us/articles/47358131751449-Main-Menu-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358240361625-Menu
- https://support.marvelousdesigner.com/hc/en-us/articles/47358171722265-Object-Browser-Ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358294806169 (Object Browser, older)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358318005913-Property-Editor
- https://support.marvelousdesigner.com/hc/en-us/articles/47358253334169-Library-Window
- https://support.marvelousdesigner.com/hc/en-us/articles/47358183889817 (Layout Window)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358233197593-3D-Window
- https://support.marvelousdesigner.com/hc/en-us/articles/47358246789529-2D-Window
- https://support.marvelousdesigner.com/hc/en-us/articles/47358168697497-Select-Move-3D
- https://support.marvelousdesigner.com/hc/en-us/articles/47358315717017-Zoom-In-Out-Rotation
- https://support.marvelousdesigner.com/hc/en-us/articles/47358200025497-3D-Window-Display-Icons
- https://support.marvelousdesigner.com/hc/en-us/articles/47358375599769-Mode
- https://support.marvelousdesigner.com/hc/en-us/articles/47358126161177 (Functional Improvement / 2025.0 UI changes)
- https://support.clo3d.com/hc/en-us/articles/115000446087-3D-Garment-View-Controls (official CLO sister site, view hotkeys)
- https://developer.marvelousdesigner.com/optiontype.html
- https://www.cgchannel.com/2025/04/clo-virtual-fashion-releases-marvelous-designer-2025-0/
- https://www.cgchannel.com/2024/04/clo-virtual-fashion-releases-marvelous-designer-2024-0/
- https://www.cgchannel.com/2023/11/clo-virtual-fashion-releases-marvelous-designer-12-2/
- https://www.cgchannel.com/2022/11/clo-virtual-fashion-releases-marvelous-designer-12/
- https://digitalproduction.com/2025/11/18/marvelous-designer-2025-2-metahumans-pleats-keyframes/
- https://digitalproduction.com/2026/04/15/marvelous-designer-2026-0-adds-3d-pencil-and-lacing/
- https://digitalproduction.com/2026/08/25/marvelous-designer-2026-1-rips-seams/
