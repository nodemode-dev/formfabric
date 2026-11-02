# 14 · Parity matrix

Every feature the research found, mapped to the formfabric module that owns it, the name formfabric shows the user, and the build phase that delivers it (phases are defined in the plan of record, DEV_PLAN.md). The Status column gives one of three values for every row: "landed vX.Y.Z" (the release that delivered it, from CHANGELOG.md), "partial: what is missing", or "deferred: reason"; rows still being built in the current release read "in progress vX.Y.Z" and are rewritten at release. Release to phase map: v0.1.0 Phase 1, v0.2.0 Phase 2, v0.3.0 Phase 3, v0.6.0 Phases 4 to 6, v0.9.0 Phases 7 to 9, v0.12.0 Phases 10 to 12, v0.15.0 Phases 13 to 15, v0.16.0 Phase 16, v0.17.0 Phase 17.

Naming rule (family rule, as in formsculpt's CLAUDE.md): formfabric's features carry formfabric's own names in labels, tooltips, translations and MCP tool descriptions. Generic garment words (pattern, seam, dart, pleat, pin, tack, zipper, topstitch) stay as they are. Product-specific names are replaced, and the reference name appears only in this encyclopedia.

Module key: `ffb-core` mesh, BVH, scene, units · `ffb-pattern` 2D geometry and tools · `ffb-sew` seams · `ffb-mesh` pattern meshing · `ffb-fabric` physical and visual fabric · `ffb-sim` solver (CPU reference and wgpu compute) · `ffb-body` avatars, skeleton, skin, placement · `ffb-anim` timeline, keys, caches · `ffb-io` files · `ffb-uv` unified UV and baking · `ffb-commands` command registry and history · `ui/*` frontend workspaces · `service` FastAPI.

Chapter column points at the encyclopedia chapter with the behaviour detail.

## Workspace, files and shell

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Splash, title bar, menu bar, status bar, settings window | family shell | ui/shell, src-tauri | 1 | S1 | landed v0.1.0 |
| 3D window, 2D window, side panels, docking, maximize window | Workspace with 3D View and Pattern View | ui/workspace | 1, 5 | 01, 12 | landed v0.6.0 |
| Workspace modes (Simulation, Animation, Modular, UV, Sculpt) | Modes: Drape, Animate, Blocks, UV, Retopo | ui/modes | 1 then per phase | 01 | partial: tabs Pattern, Sew and Drape work; Animate (timeline dock), UV and Retopo (Output panel) exist as docks and panels while the Animate and Render tabs are not enabled; Components (Blocks) in progress v0.16.0 |
| Project file .zprj, garment .zpac | Project `.ffbrc` (zip container: garment, body, scene, caches, view state) | ffb-io | 2 | 01 | landed v0.2.0 |
| Small asset files (.zfab, .sst, .btn, .bth, .pos, .mtn, .arr, .pan) | `.ffab` fabric, `.ffstyle` styles, `.ffpose`, `.ffmotion`, `.ffplace` | ffb-io | 6, 9, 12, 13 | 01 | partial: `.ffab` landed v0.6.0 and styles live inside the project (v0.12.0); standalone `.ffstyle`, `.ffpose`, `.ffmotion` and `.ffplace` files arrive with the Library in progress v0.16.0 |
| Command history, undo and redo | History (typed edits, camera per step) | ffb-commands | 2 | 12 | landed v0.2.0 |
| History 3D state snapshots | Snapshots | ffb-commands, ui | 16 | 12 | in progress v0.16.0 |
| Auto-save every 3 minutes with crash recovery | Autosave and Recovery | service | 16 | 12 | in progress v0.16.0 (the 3 minute autosave loop exists since v0.15.0; the crash journal and recovery offer are being built) |
| Hotkey editor with search and reset, active tool hotkey viewer | Shortcut Editor, Key Hints | ui/shell | 1, 16 | 12 | partial: Shortcut Editor with search and reset landed v0.3.0; Key Hints in progress v0.16.0 |
| Layouts saved to file, tool sets | Layouts | ui/workspace | 16 | 12 | partial: three pane layouts (2D, split, 3D) remembered since v0.1.0; named saved layouts in progress v0.16.0 |
| Library window (folders, favorites) | Library | ui/library, service | 16 | 12 | in progress v0.16.0 |
| Object Browser (fabric, graphic, button, buttonhole, topstitch, puckering, trim, zipper styles, scene tree) | Asset Browser with typed style registry | ffb-commands, ui | 6, 12, 16 | 12, 07 | partial: the typed style registry (`style.*`, fabric and colorway lists) landed v0.12.0; a single Asset Browser window with scene tree is not built |
| Property Editor | Inspector | ui/inspector | 3 | 12 | landed v0.3.0 |
| User settings file save and load | Settings export and import | ui/settings | 16 | 12 | partial: preferences persist and reset (v0.3.0); file export and import of settings is not built |
| Arithmetic in numeric fields | Expression fields | ui/controls | 3 | 11 | landed v0.3.0 |
| Statistics overlay | Stats overlay | ui/viewport | 5 | 11 | landed v0.6.0 |
| Interface languages | 30 family languages | ui/i18n | 1 | S1 | landed v0.1.0 |

## 2D pattern

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Polygon, Rectangle, Circle and Ellipse tools | Draw Panel, Rectangle, Ellipse | ffb-pattern | 3 | 02, 02b | landed v0.3.0 |
| Internal polygon, line, rectangle, circle | Inner Shape, Inner Line | ffb-pattern | 3 | 02 | landed v0.3.0 |
| Edit Pattern, Transform Pattern (A), Edit point (Z) | Edit, Transform | ffb-pattern | 3 | 02 | landed v0.3.0 |
| Curve point, Bezier handles, Edit Curvature, Add Point, Split Line, smooth | Curve tools | ffb-pattern | 3 | 02 | landed v0.3.0 |
| Numeric dialogs on right click or F1 for every tool | Exact Input | ui/pattern | 3 | 02 | landed v0.3.0 |
| Offset outline (extend, retract, corner modes, optional inner line) | Offset Outline and Seam Allowance | ffb-pattern | 4 | 02, 02b | landed v0.6.0 |
| Dart, internal dart | Dart | ffb-pattern | 4 | 02 | landed v0.6.0 |
| Slash and Spread | Slash and Spread | ffb-pattern | 4 | 02 | landed v0.6.0 |
| Fold Angle 0 to 360 (180 flat) and Fold Strength 0 to 20 on internal lines | Fold Line | ffb-pattern, ffb-sim | 4, 7 | 02, 03 | landed v0.9.0 |
| Symmetric linked editing, instance with count and interval, Unfold half pattern | Mirror Edit, Instances, Unfold | ffb-pattern | 4 | 02 | landed v0.6.0 |
| Clone as pattern with sewing | Duplicate with Seams | ffb-pattern, ffb-sew | 8 | 11 | landed v0.9.0 |
| Trace | Trace | ffb-pattern | 4 | 02b | landed v0.6.0 |
| Grain line display and edit | Grain Line | ffb-pattern, ffb-fabric | 4 | 02b | landed v0.6.0 |
| Notch (add on segment point, sewing notches) | Notch | ffb-pattern | 4 | 02b | landed v0.6.0 |
| Shrinkage weft and warp (rest state only) | Shrink | ffb-pattern, ffb-sim | 7 | 02 | landed v0.9.0 |
| Pattern properties (particle distance, layer, pressure) | Panel properties | ffb-pattern | 5, 7 | 02, 05 | landed v0.12.0 |
| Superimpose (compare states in 3D) | Overlay Compare | ui/viewport | 15 | 02b | landed v0.15.0 |
| 2D measurement, match measurements | Measure, Match Lengths | ffb-pattern | 4 | 02, 11 | landed v0.6.0 |
| DXF, AAMA, ASTM pattern import and export (not in MD, a CLO feature) | Pattern Exchange (DXF-AAMA) | ffb-io | 4 | 01, 02b | partial: DXF-AAMA round trip landed v0.6.0; ASTM and AI files are not read or written |
| Snap and grid (not documented for MD) | Snap and Grid | ui/pattern | 3 | 02b | landed v0.3.0 |
| Styleline editing in 3D propagating to 2D | Style Lines | ffb-pattern, ui | 11 | 11 | landed v0.15.0 |
| Spiral tool | Spiral | ffb-pattern | 4 | 11 | landed v0.6.0 |

## Sewing

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Segment Sewing, Free Sewing (2D and 3D), 1:N with Shift, M:N with Enter | Seam (basic in 7), Free Seam, Multi Seam | ffb-sew | 7, 8 | 03 | landed v0.9.0 |
| Edit sewing, flip direction, directional notches | Edit Seam | ffb-sew | 8 | 03 | landed v0.9.0 |
| Sewing type Turned (locked 180 degree fold) or Custom angle | Seam Fold | ffb-sew, ffb-sim | 8 | 03 | landed v0.9.0 |
| Tension (ease or stretch, strength, ratio up to 200 percent) | Seam Tension | ffb-sew | 8 | 03 | landed v0.9.0 |
| Sublayer front and back, reverse direction | Seam Side | ffb-sew, ffb-sim | 8 | 03 | partial: front, back, auto and reverse landed v0.9.0; the layer rank between sublayers is not built (PATENT_NOTES) |
| Length mismatch display and Check Sewing Length (default 5 mm) | Seam Check | ffb-sew | 8 | 03 | landed v0.9.0 |
| Tack (pattern to pattern, pattern to avatar) | Tack | ffb-sew, ffb-sim | 8 | 03 | landed v0.9.0 |
| Pleats (knife, box) | Pleats | ffb-pattern, ffb-sew | 8 | 03 | landed v0.9.0 |
| Gathering and shirring via seam ratio | Gather | ffb-sew | 8 | 03 | landed v0.9.0 |
| Auto Sewing (rule based, default placement only) | Auto Seam | ffb-sew, ffb-body | 9 | 03, 04 | landed v0.9.0 |
| Seam spring model (one to one lattice springs, fixed and moving halves) | Seam constraints | ffb-sim | 7 | 03 | landed v0.9.0 |
| Seamline rip, fray (2026.1) | Rip, Fray | ffb-sew, ffb-sim | 12 | 01 | landed v0.12.0 |

## Bodies and placement

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Default avatars and library | Bodies (formfabric's own parametric bodies) | ffb-body | 9 | 04 | landed v0.9.0 (MPFB2 CC0 female and male plus the Basic Form) |
| Import avatar OBJ, FBX, Alembic, glTF, scale units | Import Body | ffb-io, ffb-body | 9 | 04 | partial: OBJ, glTF, GLB and FBX with units and up axis landed v0.9.0; Alembic import is deferred |
| Avatar Editor, size and measurement sliders | Body Editor | ffb-body | 9 | 04 | landed v0.9.0 |
| Skin offset (3 mm default), friction, soft body, divide mesh | Body collision | ffb-body, ffb-sim | 7, 9 | 04 | partial: skin offset and friction landed v0.9.0; body soft body and divide mesh have no command and are not built |
| Joints, skin weights, X-ray joints | Skeleton | ffb-body | 9 | 04 | landed v0.9.0 (joints listed and drawn, weights computed on import; a dedicated X-ray toggle not verified) |
| Pose library, pose transitions | Poses | ffb-body, ffb-anim | 9 | 04 | landed v0.9.0 (named poses; transitions through pose keys and Lead-in, v0.15.0) |
| Morph target and timed morph, blend shapes 0 to 100 | Morph | ffb-body, ffb-anim | 13 | 04, 08 | landed v0.15.0 |
| Bounding volumes (cylinder, cuboid) and arrangement points (X, Y, offset, wrap, symmetry) | Placement Volumes, Placement Points | ffb-body | 9 | 04 | landed v0.9.0 |
| Arrange pattern on point (position, offset, orientation, shape style) | Place Panel | ffb-body, ui | 9 | 04 | landed v0.9.0 |
| Fitting Suit, Auto Fitting, Re-Target Draping | Fit Shell, Refit, Re-Drape | ffb-body, ffb-sim | 11 | 04, 11 | partial: Refit and Re-Drape landed v0.12.0 and Fit Shell v0.15.0; Refit still leaves cloth inside the new body (22.3 mm at v0.12.0) |
| Avatar tape, attach and detach | Body Tape | ffb-sim | 11 | 04 | landed v0.12.0 |
| Scale custom-joint FBX avatar | Body scale | ffb-body | 9 | 11 | landed v0.9.0 |

## Simulation

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Simulate on and off (Space) | Drape (Space) | ffb-sim | 7 | 05 | landed v0.9.0 (runner starvation that made Space show nothing fixed v0.15.0) |
| Presets Normal, Animation, Fitting; quality Normal or Complete | Quality presets | ffb-sim | 10 | 05, 12b | landed v0.12.0 (Fast, Normal, Quality, Final; the Quality preset's self contact is unstable on sewn cloth in motion, fix in progress v0.16.0) |
| Time step, substeps, CG finish condition, self-collision iterations, air damping, nonlinear, gravity -9800 | Solver settings | ffb-sim | 7, 10 | 05 | landed v0.12.0 |
| CPU and GPU simulation | CPU reference and GPU solver | ffb-sim | 7, 10 | 05, 12b | partial: CPU solver landed v0.12.0 but misses the speed targets (676 vertices 15 to 22 ms, target 8 ms; 5,000 vertices 200 to 260 ms, target 50 ms); wgpu VBD is opt-in, has no self contact, and wind direction and layered contact are wrong; speed and GPU fixes in progress v0.16.0; 30 fps on 30k vertices unmeasured (no GPU in the sandbox) |
| Particle distance 0.8 to 700 mm (default 20) | Mesh density | ffb-mesh | 5 | 05 | landed v0.6.0 |
| Collision thickness 1.5 mm per side, render thickness separate | Contact thickness | ffb-sim | 7 | 05, 06 | landed v0.9.0 |
| Layer integer | Layer | ffb-sim | 10 | 05 | landed v0.12.0 |
| Pressure (signed, mirrored pairs) | Pressure | ffb-sim | 7 | 05 | landed v0.9.0 |
| Freeze, Deactivate, Strengthen, Solidify | Freeze, Disable, Stiffen, Hold Shape | ffb-sim | 11 | 05 | landed v0.12.0 |
| Pin family (2D pin, box, lasso, segment, whole pattern, delete) | Pins | ffb-sim, ui | 7, 11 | 12b | landed v0.12.0 |
| Drag while simulating, quick pinching, advanced pinching (bump or linear, 1 to 500 mm, power -2 to 2), brush pinching | Grab, Pinch | ffb-sim, ui | 11 | 05, 11 | landed v0.12.0 |
| Fold arrangement | Fold Place | ffb-sim | 11 | 11 | landed v0.12.0 |
| Wind (spherical, planar, decay, turbulence) | Wind | ffb-sim | 11 | 05 | landed v0.12.0 (directional and radial kinds; the wind sign on the GPU path is reversed, fix in progress v0.16.0) |
| Intersection resolution toggle | Untangle | ffb-sim | 10 | 12b | partial: Untangle and the self-collision toggle landed v0.12.0; cloth still intersects itself in 92 of 120 walk frames, fix in progress v0.16.0 |
| Collision with props and rigid trims | Props | ffb-sim | 10, 12 | 05 | partial: bodies, including any imported object, collide (v0.9.0), and buttons and imported trims are rigid discs or meshes glued to a panel (v0.12.0); there is no separate prop type and no hanger or mannequin |

## Fabric and materials

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Fabric presets | formfabric presets, measured where possible | ffb-fabric | 6 | 06 | partial: 15 presets landed v0.6.0, 8 are provisional until measured data is found |
| Stretch weft, warp, shear; bending weft, warp, bias; buckling ratio and stiffness; damping; density; friction; thickness | Physical properties (with SI units shown beside sliders) | ffb-fabric, ffb-sim | 6 | 06 | landed v0.6.0 |
| Fabric Kit and Emulator digitization | Fabric Lab (measure and fit) | ffb-fabric | 6 | 06, 13b | landed v0.6.0 (a measurement model; formulas not yet cited from verified sources) |
| Material types and PBR channels, front, back, side | Look | ffb-fabric, ui/viewport | 6 | 06 | landed v0.6.0 |
| Texture placement and transform, prints and graphics | Prints | ffb-fabric | 12 | 06, 07 | landed v0.12.0 |
| Colorways | Colorways | ffb-fabric | 6 | 06 | landed v0.6.0 |
| Fur, fur strands | Fur | ui/viewport | 15 | 06, 07 | landed v0.15.0 |

## Trims and details

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Buttons and buttonholes with styles | Buttons, Buttonholes | ffb-sim, ffb-io | 12 | 07 | landed v0.12.0 |
| Zippers (slider, puller, stopper, teeth, custom OBJ) | Zippers | ffb-sim | 12 | 07 | landed v0.12.0 |
| Topstitch (styles, stitch count 1 to 5) | Topstitch | ui/viewport | 12 | 07 | landed v0.12.0 |
| Puckering and seam puckering | Puckering | ui/viewport | 12 | 07 | landed v0.12.0 |
| Piping (width 1 to 15 mm) and binding (width 1 to 50 mm, length 50 to 200 percent) | Piping, Binding | ffb-sew, ffb-sim | 12 | 07 | landed v0.12.0 |
| Elastic (ratio default 80 percent) | Elastic | ffb-sim | 12 | 07 | landed v0.12.0 |
| Shirring and crease | Shirring, Crease | ffb-sim | 12 | 07 | landed v0.12.0 (Shirring is an alias of Elastic and Crease is a fold line) |
| Pressing, seam tape, bond, skive | Press, Tape, Bond | ffb-sim | 12 | 07 | partial: Press, Tape and Bond landed v0.12.0; Skive is not built |
| Trims, buckles, rings from OBJ or FBX | Trims | ffb-io, ffb-sim | 12 | 07 | partial: OBJ trims landed v0.12.0 as rigid meshes glued to a panel point; FBX trims are not read |
| Padding and quilting via pressure clones | Quilt | ffb-sim | 12 | 07 | landed v0.12.0 |

## Animation

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Animation mode and editor, timeline, frame rate | Animate mode and Timeline | ffb-anim, ui | 13 | 08 | landed v0.15.0 (a timeline dock; the Animate workspace tab is not enabled) |
| Record simulation to cache layers (pattern, trim, soft body) | Record | ffb-anim, ffb-sim | 13 | 08 | partial: panel caches and body pose recorded in the project since v0.15.0; trims and body soft body are not cached; the walk still shows cloth self intersection in 92 of 120 frames, fix in progress v0.16.0 |
| Cache editing (keys, cut, merge, move, lock, override layers) | Cache Layers | ffb-anim | 13 | 08 | partial: layers can be recorded, renamed, shown, hidden, made active and removed (v0.15.0); cut, merge, move, lock and override are not built |
| Property keyframes (constant, linear, cubic, auto key) | Keys | ffb-anim | 13 | 08 | partial: step, linear and cubic keys landed v0.15.0; auto key is not built |
| Motion import, Auto Convert to Motion, transition from T pose | Motion import, Retarget, Lead-in | ffb-anim, ffb-io | 13 | 08 | landed v0.15.0 (BVH, glTF and FBX; FBX motion not yet tried in the app; no Auto Convert) |
| Wind, pin, tack and freeze animation | Animated controls | ffb-anim | 13 | 08 | partial: wind, panel state (freeze and similar) and fabric tracks landed v0.15.0; pins and tacks cannot be keyed |
| Export Alembic, MDD, PC2, OBJ sequence, FBX caches, USD, glTF | Cache export | ffb-io | 14 | 08, 09 | partial: all landed v0.15.0 except Alembic, which is deferred (needs an Ogawa writer) |

## Export, UV and retopology

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Shared export options (weld, thin or thick, single or multiple, scale, axis, range) | Export options | ffb-io | 14 | 09 | partial: thick or thin, single or per panel, scale, units, axis and frame range landed v0.15.0; there is no weld option |
| OBJ, FBX, Alembic, USD, glTF and GLB, MDD, PC2 | Exporters | ffb-io | 14 | 09 | partial: all landed v0.15.0 and open in Blender 5.2 except Alembic, which is deferred |
| Thickness export (quad meshes) | Thickness | ffb-io | 14 | 09 | partial: thickness export landed v0.15.0 for triangle meshes; it cannot be combined with quad output |
| Remeshing to aligned quads, polygon optimization | Quad Mesh | ffb-mesh | 14 | 09 | partial: field-aligned quad remesh with seam-matched borders landed v0.15.0; the QuadriFlow min-cost-flow regularity step is not implemented |
| Retopology (point and face drawing, ghost points across seams) | Retopo mode | ffb-mesh, ui | 14 | 09 | partial: retopo guides and settings landed v0.15.0 in the Output panel; point and face drawing with ghost points across seams was not verified |
| Unified UV, UV editor, auto packing, UDIM naming | UV mode | ffb-uv | 14 | 09 | landed v0.15.0 |
| Texture baking (diffuse, normal, metal, roughness, opacity, displacement) | Bake | ffb-uv | 14 | 09 | partial: normal, ambient occlusion and displacement maps landed v0.15.0; diffuse, metal, roughness and opacity are not baked |
| Python API | formfabric Python API (headless engine module) | ffb-py | 17 | 09 | in progress v0.16.0 (the `ffb-py` engine module exists since v0.2.0; the Python package is being built) |
| Plugins menu | Python plugins (family extension, formsculpt's PARAMS model) | service | 17 | 12 | in progress v0.16.0 |
| (no reference feature) | GoF hand-off to formsculpt and formsurface (family extension) | service, ffb-io | 17 | S3 | in progress v0.16.0 (needs the contract agreed with formsculpt's owner) |
| Unreal LiveSync, USD with simulation data | Live Link and USD sim export | ffb-io, service | 17 | 09 | in progress v0.16.0 (USD time-sample export landed v0.15.0; Live Link not yet tried against Unreal) |
| Game-ready pipeline (optimize, seal, rig and weights, UV pack, bake, FBX) | Game-Ready Export | ffb-io, ffb-uv | 14 | 11 | landed v0.15.0 |

## Visualization

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Display toggles and render styles (Alt+1 to Alt+8) | View styles | ui/viewport | 5, 15 | 10 | landed v0.15.0 (view style buttons with Alt+number keys since v0.6.0) |
| Schematic render | Schematic | ui/viewport | 15 | 10 | landed v0.15.0 (SVG and PDF export) |
| Lights, shadows, HDRI, background image | Stage | ui/viewport | 15 | 10 | partial: lights, shadows, HDRI and ground landed v0.15.0; a background image is not verified |
| Saved custom views with display state | Saved Views | ui/viewport | 15 | 10 | landed v0.15.0 |
| Snapshot (multi-view, transparent, ppi), video capture, turntable | Capture | ui/viewport, service | 15 | 10 | partial: PNG snapshot, WebM video, GIF and turntable landed v0.15.0; multi-view, transparent background and ppi options were not verified; the exit capture used the pre-fix jacket and the re-capture on a drafted sample is a v0.16.0 exit |
| Strain, stress and pressure maps with units and bins | Fit Maps | ffb-sim, ui/viewport | 15 | 10 | landed v0.15.0 |
| 3D measure (circumference, length) | Measure 3D | ffb-body, ui | 15 | 10 | landed v0.15.0 |

## Beyond scope of v1.0.0 (recorded, not planned)

| Reference feature | Why deferred | Status |
| --- | --- | --- |
| AI Studio (texture, graphic, PBR map, pose generator, pattern drafter) | Needs model hosting and data licensing decisions; a later plan | deferred: model hosting and data licensing undecided |
| Modular block system with structure files and ML labeling | Planned in Phase 16 as Components (the word block is not used) without the ML labeling | in progress v0.16.0 for Components; ML labeling deferred: no training data or hosting |
| CONNECT asset marketplace | A service, not an app feature | deferred: a service, not an app feature |
| Reading .zprj, .zpac or other proprietary files | Undocumented formats; interop goes through open formats | deferred: undocumented proprietary formats, by decision |
