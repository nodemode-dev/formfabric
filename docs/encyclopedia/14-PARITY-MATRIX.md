# 14 · Parity matrix

Every feature the research found, mapped to the formfabric module that owns it, the name formfabric shows the user, and the build phase that delivers it (phases are defined in the plan of record, DEV_PLAN.md). Status is "planned" for every row until a phase lands; the column is updated as phases close.

Naming rule (family rule, as in formsculpt's CLAUDE.md): formfabric's features carry formfabric's own names in labels, tooltips, translations and MCP tool descriptions. Generic garment words (pattern, seam, dart, pleat, pin, tack, zipper, topstitch) stay as they are. Product-specific names are replaced, and the reference name appears only in this encyclopedia.

Module key: `ffb-core` mesh, BVH, scene, units · `ffb-pattern` 2D geometry and tools · `ffb-sew` seams · `ffb-mesh` pattern meshing · `ffb-fabric` physical and visual fabric · `ffb-sim` solver (CPU reference and wgpu compute) · `ffb-body` avatars, skeleton, skin, placement · `ffb-anim` timeline, keys, caches · `ffb-io` files · `ffb-uv` unified UV and baking · `ffb-commands` command registry and history · `ui/*` frontend workspaces · `service` FastAPI.

Chapter column points at the encyclopedia chapter with the behaviour detail.

## Workspace, files and shell

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Splash, title bar, menu bar, status bar, settings window | family shell | ui/shell, src-tauri | 1 | S1 |
| 3D window, 2D window, side panels, docking, maximize window | Workspace with 3D View and Pattern View | ui/workspace | 1, 5 | 01, 12 |
| Workspace modes (Simulation, Animation, Modular, UV, Sculpt) | Modes: Drape, Animate, Blocks, UV, Retopo | ui/modes | 1 then per phase | 01 |
| Project file .zprj, garment .zpac | Project `.ffb` (zip container: garment, body, scene, caches, view state) | ffb-io | 2 | 01 |
| Small asset files (.zfab, .sst, .btn, .bth, .pos, .mtn, .arr, .pan) | `.ffab` fabric, `.ffstyle` styles, `.ffpose`, `.ffmotion`, `.ffplace` | ffb-io | 6, 9, 12, 13 | 01 |
| Command history, undo and redo | History (typed edits, camera per step) | ffb-commands | 2 | 12 |
| History 3D state snapshots | Snapshots | ffb-commands, ui | 16 | 12 |
| Auto-save every 3 minutes with crash recovery | Autosave and Recovery | service | 16 | 12 |
| Hotkey editor with search and reset, active tool hotkey viewer | Shortcut Editor, Key Hints | ui/shell | 1, 16 | 12 |
| Layouts saved to file, tool sets | Layouts | ui/workspace | 16 | 12 |
| Library window (folders, favorites) | Library | ui/library, service | 16 | 12 |
| Object Browser (fabric, graphic, button, buttonhole, topstitch, puckering, trim, zipper styles, scene tree) | Asset Browser with typed style registry | ffb-commands, ui | 6, 12, 16 | 12, 07 |
| Property Editor | Inspector | ui/inspector | 3 | 12 |
| User settings file save and load | Settings export and import | ui/settings | 16 | 12 |
| Arithmetic in numeric fields | Expression fields | ui/controls | 3 | 11 |
| Statistics overlay | Stats overlay | ui/viewport | 5 | 11 |
| Interface languages | 30 family languages | ui/i18n | 1 | S1 |

## 2D pattern

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Polygon, Rectangle, Circle and Ellipse tools | Draw Panel, Rectangle, Ellipse | ffb-pattern | 3 | 02, 02b |
| Internal polygon, line, rectangle, circle | Inner Shape, Inner Line | ffb-pattern | 3 | 02 |
| Edit Pattern, Transform Pattern (A), Edit point (Z) | Edit, Transform | ffb-pattern | 3 | 02 |
| Curve point, Bezier handles, Edit Curvature, Add Point, Split Line, smooth | Curve tools | ffb-pattern | 3 | 02 |
| Numeric dialogs on right click or F1 for every tool | Exact Input | ui/pattern | 3 | 02 |
| Offset outline (extend, retract, corner modes, optional inner line) | Offset Outline and Seam Allowance | ffb-pattern | 4 | 02, 02b |
| Dart, internal dart | Dart | ffb-pattern | 4 | 02 |
| Slash and Spread | Slash and Spread | ffb-pattern | 4 | 02 |
| Fold Angle 0 to 360 (180 flat) and Fold Strength 0 to 20 on internal lines | Fold Line | ffb-pattern, ffb-sim | 4, 7 | 02, 03 |
| Symmetric linked editing, instance with count and interval, Unfold half pattern | Mirror Edit, Instances, Unfold | ffb-pattern | 4 | 02 |
| Clone as pattern with sewing | Duplicate with Seams | ffb-pattern, ffb-sew | 8 | 11 |
| Trace | Trace | ffb-pattern | 4 | 02b |
| Grain line display and edit | Grain Line | ffb-pattern, ffb-fabric | 4 | 02b |
| Notch (add on segment point, sewing notches) | Notch | ffb-pattern | 4 | 02b |
| Shrinkage weft and warp (rest state only) | Shrink | ffb-pattern, ffb-sim | 7 | 02 |
| Pattern properties (particle distance, layer, pressure) | Panel properties | ffb-pattern | 5, 7 | 02, 05 |
| Superimpose (compare states in 3D) | Overlay Compare | ui/viewport | 15 | 02b |
| 2D measurement, match measurements | Measure, Match Lengths | ffb-pattern | 4 | 02, 11 |
| DXF, AAMA, ASTM pattern import and export (not in MD, a CLO feature) | Pattern Exchange (DXF-AAMA) | ffb-io | 4 | 01, 02b |
| Snap and grid (not documented for MD) | Snap and Grid | ui/pattern | 3 | 02b |
| Styleline editing in 3D propagating to 2D | Style Lines | ffb-pattern, ui | 11 | 11 |
| Spiral tool | Spiral | ffb-pattern | 4 | 11 |

## Sewing

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Segment Sewing, Free Sewing (2D and 3D), 1:N with Shift, M:N with Enter | Seam (basic in 7), Free Seam, Multi Seam | ffb-sew | 7, 8 | 03 |
| Edit sewing, flip direction, directional notches | Edit Seam | ffb-sew | 8 | 03 |
| Sewing type Turned (locked 180 degree fold) or Custom angle | Seam Fold | ffb-sew, ffb-sim | 8 | 03 |
| Tension (ease or stretch, strength, ratio up to 200 percent) | Seam Tension | ffb-sew | 8 | 03 |
| Sublayer front and back, reverse direction | Seam Side | ffb-sew, ffb-sim | 8 | 03 |
| Length mismatch display and Check Sewing Length (default 5 mm) | Seam Check | ffb-sew | 8 | 03 |
| Tack (pattern to pattern, pattern to avatar) | Tack | ffb-sew, ffb-sim | 8 | 03 |
| Pleats (knife, box) | Pleats | ffb-pattern, ffb-sew | 8 | 03 |
| Gathering and shirring via seam ratio | Gather | ffb-sew | 8 | 03 |
| Auto Sewing (rule based, default placement only) | Auto Seam | ffb-sew, ffb-body | 9 | 03, 04 |
| Seam spring model (one to one lattice springs, fixed and moving halves) | Seam constraints | ffb-sim | 7 | 03 |
| Seamline rip, fray (2026.1) | Rip, Fray | ffb-sew, ffb-sim | 12 | 01 |

## Bodies and placement

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Default avatars and library | Bodies (formfabric's own parametric bodies) | ffb-body | 9 | 04 |
| Import avatar OBJ, FBX, Alembic, glTF, scale units | Import Body | ffb-io, ffb-body | 9 | 04 |
| Avatar Editor, size and measurement sliders | Body Editor | ffb-body | 9 | 04 |
| Skin offset (3 mm default), friction, soft body, divide mesh | Body collision | ffb-body, ffb-sim | 7, 9 | 04 |
| Joints, skin weights, X-ray joints | Skeleton | ffb-body | 9 | 04 |
| Pose library, pose transitions | Poses | ffb-body, ffb-anim | 9 | 04 |
| Morph target and timed morph, blend shapes 0 to 100 | Morph | ffb-body, ffb-anim | 13 | 04, 08 |
| Bounding volumes (cylinder, cuboid) and arrangement points (X, Y, offset, wrap, symmetry) | Placement Volumes, Placement Points | ffb-body | 9 | 04 |
| Arrange pattern on point (position, offset, orientation, shape style) | Place Panel | ffb-body, ui | 9 | 04 |
| Fitting Suit, Auto Fitting, Re-Target Draping | Fit Shell, Refit, Re-Drape | ffb-body, ffb-sim | 11 | 04, 11 |
| Avatar tape, attach and detach | Body Tape | ffb-sim | 11 | 04 |
| Scale custom-joint FBX avatar | Body scale | ffb-body | 9 | 11 |

## Simulation

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Simulate on and off (Space) | Drape (Space) | ffb-sim | 7 | 05 |
| Presets Normal, Animation, Fitting; quality Normal or Complete | Quality presets | ffb-sim | 10 | 05, 12b |
| Time step, substeps, CG finish condition, self-collision iterations, air damping, nonlinear, gravity -9800 | Solver settings | ffb-sim | 7, 10 | 05 |
| CPU and GPU simulation | CPU reference and GPU solver | ffb-sim | 7, 10 | 05, 12b |
| Particle distance 0.8 to 700 mm (default 20) | Mesh density | ffb-mesh | 5 | 05 |
| Collision thickness 1.5 mm per side, render thickness separate | Contact thickness | ffb-sim | 7 | 05, 06 |
| Layer integer | Layer | ffb-sim | 10 | 05 |
| Pressure (signed, mirrored pairs) | Pressure | ffb-sim | 7 | 05 |
| Freeze, Deactivate, Strengthen, Solidify | Freeze, Disable, Stiffen, Hold Shape | ffb-sim | 11 | 05 |
| Pin family (2D pin, box, lasso, segment, whole pattern, delete) | Pins | ffb-sim, ui | 7, 11 | 12b |
| Drag while simulating, quick pinching, advanced pinching (bump or linear, 1 to 500 mm, power -2 to 2), brush pinching | Grab, Pinch | ffb-sim, ui | 11 | 05, 11 |
| Fold arrangement | Fold Place | ffb-sim | 11 | 11 |
| Wind (spherical, planar, decay, turbulence) | Wind | ffb-sim | 11 | 05 |
| Intersection resolution toggle | Untangle | ffb-sim | 10 | 12b |
| Collision with props and rigid trims | Props | ffb-sim | 10, 12 | 05 |

## Fabric and materials

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Fabric presets | formfabric presets, measured where possible | ffb-fabric | 6 | 06 |
| Stretch weft, warp, shear; bending weft, warp, bias; buckling ratio and stiffness; damping; density; friction; thickness | Physical properties (with SI units shown beside sliders) | ffb-fabric, ffb-sim | 6 | 06 |
| Fabric Kit and Emulator digitization | Fabric Lab (measure and fit) | ffb-fabric | 6 | 06, 13b |
| Material types and PBR channels, front, back, side | Look | ffb-fabric, ui/viewport | 6 | 06 |
| Texture placement and transform, prints and graphics | Prints | ffb-fabric | 12 | 06, 07 |
| Colorways | Colorways | ffb-fabric | 6 | 06 |
| Fur, fur strands | Fur | ui/viewport | 15 | 06, 07 |

## Trims and details

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Buttons and buttonholes with styles | Buttons, Buttonholes | ffb-sim, ffb-io | 12 | 07 |
| Zippers (slider, puller, stopper, teeth, custom OBJ) | Zippers | ffb-sim | 12 | 07 |
| Topstitch (styles, stitch count 1 to 5) | Topstitch | ui/viewport | 12 | 07 |
| Puckering and seam puckering | Puckering | ui/viewport | 12 | 07 |
| Piping (width 1 to 15 mm) and binding (width 1 to 50 mm, length 50 to 200 percent) | Piping, Binding | ffb-sew, ffb-sim | 12 | 07 |
| Elastic (ratio default 80 percent) | Elastic | ffb-sim | 12 | 07 |
| Shirring and crease | Shirring, Crease | ffb-sim | 12 | 07 |
| Pressing, seam tape, bond, skive | Press, Tape, Bond | ffb-sim | 12 | 07 |
| Trims, buckles, rings from OBJ or FBX | Trims | ffb-io, ffb-sim | 12 | 07 |
| Padding and quilting via pressure clones | Quilt | ffb-sim | 12 | 07 |

## Animation

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Animation mode and editor, timeline, frame rate | Animate mode and Timeline | ffb-anim, ui | 13 | 08 |
| Record simulation to cache layers (pattern, trim, soft body) | Record | ffb-anim, ffb-sim | 13 | 08 |
| Cache editing (keys, cut, merge, move, lock, override layers) | Cache Layers | ffb-anim | 13 | 08 |
| Property keyframes (constant, linear, cubic, auto key) | Keys | ffb-anim | 13 | 08 |
| Motion import, Auto Convert to Motion, transition from T pose | Motion import, Retarget, Lead-in | ffb-anim, ffb-io | 13 | 08 |
| Wind, pin, tack and freeze animation | Animated controls | ffb-anim | 13 | 08 |
| Export Alembic, MDD, PC2, OBJ sequence, FBX caches, USD, glTF | Cache export | ffb-io | 14 | 08, 09 |

## Export, UV and retopology

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Shared export options (weld, thin or thick, single or multiple, scale, axis, range) | Export options | ffb-io | 14 | 09 |
| OBJ, FBX, Alembic, USD, glTF and GLB, MDD, PC2 | Exporters | ffb-io | 14 | 09 |
| Thickness export (quad meshes) | Thickness | ffb-io | 14 | 09 |
| Remeshing to aligned quads, polygon optimization | Quad Mesh | ffb-mesh | 14 | 09 |
| Retopology (point and face drawing, ghost points across seams) | Retopo mode | ffb-mesh, ui | 14 | 09 |
| Unified UV, UV editor, auto packing, UDIM naming | UV mode | ffb-uv | 14 | 09 |
| Texture baking (diffuse, normal, metal, roughness, opacity, displacement) | Bake | ffb-uv | 14 | 09 |
| Python API | formfabric Python API (headless engine module) | ffb-py | 17 | 09 |
| Plugins menu | Python plugins (family extension, formsculpt's PARAMS model) | service | 17 | 12 |
| (no reference feature) | GoF hand-off to formsculpt and formsurface (family extension) | service, ffb-io | 17 | S3 |
| Unreal LiveSync, USD with simulation data | Live Link and USD sim export | ffb-io, service | 17 | 09 |
| Game-ready pipeline (optimize, seal, rig and weights, UV pack, bake, FBX) | Game-Ready Export | ffb-io, ffb-uv | 14 | 11 |

## Visualization

| Reference feature | formfabric name | Module | Phase | Ch |
| --- | --- | --- | --- | --- |
| Display toggles and render styles (Alt+1 to Alt+8) | View styles | ui/viewport | 5, 15 | 10 |
| Schematic render | Schematic | ui/viewport | 15 | 10 |
| Lights, shadows, HDRI, background image | Stage | ui/viewport | 15 | 10 |
| Saved custom views with display state | Saved Views | ui/viewport | 15 | 10 |
| Snapshot (multi-view, transparent, ppi), video capture, turntable | Capture | ui/viewport, service | 15 | 10 |
| Strain, stress and pressure maps with units and bins | Fit Maps | ffb-sim, ui/viewport | 15 | 10 |
| 3D measure (circumference, length) | Measure 3D | ffb-body, ui | 15 | 10 |

## Beyond scope of v1.0.0 (recorded, not planned)

| Reference feature | Why deferred |
| --- | --- |
| AI Studio (texture, graphic, PBR map, pose generator, pattern drafter) | Needs model hosting and data licensing decisions; a later plan |
| Modular block system with structure files and ML labeling | Planned in Phase 16 as Blocks without the ML labeling |
| CONNECT asset marketplace | A service, not an app feature |
| Reading .zprj, .zpac or other proprietary files | Undocumented formats; interop goes through open formats |
