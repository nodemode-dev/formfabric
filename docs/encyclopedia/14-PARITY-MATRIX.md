# 14 · Parity matrix

Every feature the research found, mapped to the formfabric module that owns it, the name formfabric shows the user, and the build phase that delivers it (phases are defined in the plan of record, DEV_PLAN.md). The Status column gives one of three values for every row: "landed vX.Y.Z" (the release that delivered it, from CHANGELOG.md), "partial: what is missing", or "deferred: reason". A deferral that waits for the owner's decision reads "deferred, pending owner sign-off: reason"; a row that landed in part reads "landed vX.Y.Z ...; deferred: what remains". Release to phase map: v0.1.0 Phase 1, v0.2.0 Phase 2, v0.3.0 Phase 3, v0.6.0 Phases 4 to 6, v0.9.0 Phases 7 to 9, v0.12.0 Phases 10 to 12, v0.15.0 Phases 13 to 15, v0.18.0 Phases 16 to 18, v0.23.0 Phases 19 to 23. Phases 19 to 23 were built as v0.19.0 (workspace), v0.20.0 (animation), v0.21.0 (export and retopology), v0.22.0 (solver) and v0.23.0 (Alembic and edge regularity); the Status column names the build that delivered each row, and all five ship in the v0.23.0 release.

Naming rule (family rule, as in formsculpt's CLAUDE.md): formfabric's features carry formfabric's own names in labels, tooltips, translations and MCP tool descriptions. Generic garment words (pattern, seam, dart, pleat, pin, tack, zipper, topstitch) stay as they are. Product-specific names are replaced, and the reference name appears only in this encyclopedia.

Module key: `ffb-core` mesh, BVH, scene, units · `ffb-pattern` 2D geometry and tools · `ffb-sew` seams · `ffb-mesh` pattern meshing · `ffb-fabric` physical and visual fabric · `ffb-sim` solver (CPU reference and wgpu compute) · `ffb-body` avatars, skeleton, skin, placement · `ffb-anim` timeline, keys, caches · `ffb-io` files · `ffb-uv` unified UV and baking · `ffb-commands` command registry and history · `ui/*` frontend workspaces · `service` FastAPI.

Chapter column points at the encyclopedia chapter with the behaviour detail.

## Workspace, files and shell

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Splash, title bar, menu bar, status bar, settings window | family shell | ui/shell, src-tauri | 1 | S1 | landed v0.1.0 |
| 3D window, 2D window, side panels, docking, maximize window | Workspace with 3D View and Pattern View | ui/workspace | 1, 5 | 01, 12 | landed v0.6.0 |
| Workspace modes (Simulation, Animation, Modular, UV, Sculpt) | Modes: Drape, Animate, Blocks, UV, Retopo | ui/modes | 1 then per phase | 01 | landed v0.19.0 (every title bar tab is enabled and applies its saved layout: Pattern, Sew, Drape, Animate, Render; UV and Retopo are Window menu items that open the Output drawer on their section; Components landed v0.18.0) |
| Project file .zprj, garment .zpac | Project `.ffbrc` (zip container: garment, body, scene, caches, view state) | ffb-io | 2 | 01 | landed v0.2.0 |
| Small asset files (.zfab, .sst, .btn, .bth, .pos, .mtn, .arr, .pan) | `.ffab` fabric, `.ffstyle` styles, `.ffpose`, `.ffmotion`, `.ffplace` | ffb-io | 6, 9, 12, 13 | 01 | landed v0.19.0 (`.ffab` v0.6.0; `.ffstyle`, `.ffpose` and `.ffmotion` through the Library v0.18.0; `.ffplace` v0.19.0 holds body volumes and arrangement points and inserts onto the current body as one undo step) |
| Command history, undo and redo | History (typed edits, camera per step) | ffb-commands | 2 | 12 | landed v0.2.0 |
| History 3D state snapshots | Snapshots | ffb-commands, ui | 16 | 12 | landed v0.18.0 (snapshots with 3D thumbnails, restore as one undo step, compare) |
| Auto-save every 3 minutes with crash recovery | Autosave and Recovery | service | 16 | 12 | landed v0.18.0 (autosave plus a crash journal of commands since it; recovery offered on the next start and replayed; checked by killing the service during a drape) |
| Hotkey editor with search and reset, active tool hotkey viewer | Shortcut Editor, Key Hints | ui/shell | 1, 16 | 12 | landed v0.18.0 (Shortcut Editor since v0.3.0, per-scope conflict detection and the Key Hints overlay in v0.18.0) |
| Layouts saved to file, tool sets | Layouts | ui/workspace | 16 | 12 | landed v0.18.0 (named layouts with six defaults, persisted across restarts; settings, shortcuts and layouts export and import as JSON; tool sets deferred: no reference behaviour beyond layouts was found) |
| Library window (folders, favorites) | Library | ui/library, service | 16 | 12 | landed v0.18.0 (user library roots, kinds, search, tags, thumbnails, insert; favorites are tags) |
| Object Browser (fabric, graphic, button, buttonhole, topstitch, puckering, trim, zipper styles, scene tree) | Asset Browser with typed style registry | ffb-commands, ui | 6, 12, 16 | 12, 07 | landed v0.19.0: Object Browser drawer with Scene, Fabric and the six style tabs (select, add, copy, merge, fabric reorder, sort by name); deferred: add and copy on the Graphic tab, which lists and selects panel prints only |
| Property Editor | Inspector | ui/inspector | 3 | 12 | landed v0.3.0 |
| User settings file save and load | Settings export and import | ui/settings | 16 | 12 | landed v0.18.0 (preferences persist and reset since v0.3.0; settings, shortcuts and layouts export and import as one JSON file) |
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
| DXF, AAMA, ASTM pattern import and export (not in MD, a CLO feature) | Pattern Exchange (DXF-AAMA) | ffb-io | 4 | 01, 02b | landed v0.6.0 for DXF with AAMA and ASTM D6673 layers; deferred, pending owner sign-off: AI and PDF pattern files, no verified reference behaviour (02b section 4) |
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
| Sublayer front and back, reverse direction | Seam Side | ffb-sew, ffb-sim | 8 | 03 | deferred, pending owner sign-off: the layer rank between sublayers overlaps the claims in PATENT_NOTES 1.1 and needs the owner's design-around decision; front, back, auto and reverse landed v0.9.0 |
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
| Import avatar OBJ, FBX, Alembic, glTF, scale units | Import Body | ffb-io, ffb-body | 9 | 04 | landed: OBJ, glTF, GLB and FBX with units and up axis (v0.9.0) and Alembic Ogawa (v0.23.0: polygon meshes of the first or a chosen sample, units and up axis as for OBJ, faces reversed from clockwise, files written by the Alembic 1.8.12 library import; transforms are ignored; HDF5 Alembic is not read) |
| Avatar Editor, size and measurement sliders | Body Editor | ffb-body | 9 | 04 | landed v0.9.0 |
| Skin offset (3 mm default), friction, soft body, divide mesh | Body collision | ffb-body, ffb-sim | 7, 9 | 04 | deferred, pending owner sign-off: body soft body and divide mesh have no verified reference behaviour (04 section 6.3, 05 section 13); skin offset and friction landed v0.9.0 |
| Joints, skin weights, X-ray joints | Skeleton | ffb-body | 9 | 04 | landed v0.9.0 (joints listed and drawn, weights computed on import; a dedicated X-ray toggle not verified) |
| Pose library, pose transitions | Poses | ffb-body, ffb-anim | 9 | 04 | landed v0.9.0 (named poses; transitions through pose keys and Lead-in, v0.15.0) |
| Morph target and timed morph, blend shapes 0 to 100 | Morph | ffb-body, ffb-anim | 13 | 04, 08 | landed v0.15.0 |
| Bounding volumes (cylinder, cuboid) and arrangement points (X, Y, offset, wrap, symmetry) | Placement Volumes, Placement Points | ffb-body | 9 | 04 | landed v0.9.0 |
| Arrange pattern on point (position, offset, orientation, shape style) | Place Panel | ffb-body, ui | 9 | 04 | landed v0.9.0 |
| Fitting Suit, Auto Fitting, Re-Target Draping | Fit Shell, Refit, Re-Drape | ffb-body, ffb-sim | 11 | 04, 11 | landed v0.22.0: Refit ends with 0.0 mm of cloth inside the target body (sweep, blend and redrape measured on the drafted T-shirt; a deep push reaches cloth up to 400 mm inside); maintain {curvature_pct, graphic_size, texture_size, topology} is accepted at its maintaining values only because Refit never edits the pattern (PATENT_NOTES 2.5); deferred: lower maintain values and pattern updates for the new body; Re-Drape v0.12.0, Fit Shell v0.15.0 |
| Avatar tape, attach and detach | Body Tape | ffb-sim | 11 | 04 | landed v0.12.0 |
| Scale custom-joint FBX avatar | Body scale | ffb-body | 9 | 11 | landed v0.9.0 |

## Simulation

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Simulate on and off (Space) | Drape (Space) | ffb-sim | 7 | 05 | landed v0.9.0 (runner starvation that made Space show nothing fixed v0.15.0) |
| Presets Normal, Animation, Fitting; quality Normal or Complete | Quality presets | ffb-sim | 10 | 05, 12b | landed v0.12.0 (Fast, Normal, Quality, Final; self contact made stable on sewn cloth in motion and on at every preset in v0.18.0) |
| Time step, substeps, CG finish condition, self-collision iterations, air damping, nonlinear, gravity -9800 | Solver settings | ffb-sim | 7, 10 | 05 | landed v0.12.0 |
| CPU and GPU simulation | CPU reference and GPU solver | ffb-sim | 7, 10 | 05, 12b | deferred, pending owner sign-off: the speed targets and 30 fps at 30k vertices need the owner's hardware, the 2-core sandbox cannot meet them (676 vertices about 15 to 18 ms CPU, target 8 ms; 5,041 vertices 150 to 180 ms, target 50 ms); the CPU solver landed v0.12.0 and is multi-threaded since v0.18.0, and the opt-in wgpu VBD path matches the CPU for self contact, wind and layered contact since v0.18.0 |
| Particle distance 0.8 to 700 mm (default 20) | Mesh density | ffb-mesh | 5 | 05 | landed v0.6.0 |
| Collision thickness 1.5 mm per side, render thickness separate | Contact thickness | ffb-sim | 7 | 05, 06 | landed v0.9.0 |
| Layer integer | Layer | ffb-sim | 10 | 05 | landed v0.12.0 |
| Pressure (signed, mirrored pairs) | Pressure | ffb-sim | 7 | 05 | landed v0.9.0 |
| Freeze, Deactivate, Strengthen, Solidify | Freeze, Disable, Stiffen, Hold Shape | ffb-sim | 11 | 05 | landed v0.12.0 |
| Pin family (2D pin, box, lasso, segment, whole pattern, delete) | Pins | ffb-sim, ui | 7, 11 | 12b | landed v0.12.0 |
| Drag while simulating, quick pinching, advanced pinching (bump or linear, 1 to 500 mm, power -2 to 2), brush pinching | Grab, Pinch | ffb-sim, ui | 11 | 05, 11 | landed v0.12.0 |
| Fold arrangement | Fold Place | ffb-sim | 11 | 11 | landed v0.12.0 |
| Wind (spherical, planar, decay, turbulence) | Wind | ffb-sim | 11 | 05 | landed v0.12.0 (directional and radial kinds; the GPU path matches the CPU direction, checked in v0.18.0) |
| Intersection resolution toggle | Untangle | ffb-sim | 10 | 12b | landed v0.22.0 as an opt-in post-step pass (`sim_settings` untangle off, normal, flipped; default off) beside the self-collision toggle (v0.12.0); deferred, pending owner sign-off: zero intersecting pairs is not reached (on a hard drafted T-shirt walk the worst frame went 84 to 37 pairs, on a gentler walk 23 to 65, so there is no reliable gain; the skirt reaches 0 at Quality) |
| Collision with props and rigid trims | Props | ffb-sim | 10, 12 | 05 | landed v0.12.0: imported bodies and trims collide; the reference has no separate prop type, mannequins are avatars and hangers are trims (04 section 15) |

## Fabric and materials

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Fabric presets | formfabric presets, measured where possible | ffb-fabric | 6 | 06 | deferred, pending owner sign-off: 8 of 15 presets stay provisional until measured data is found (16-FABRIC-DATA); the 15 presets landed v0.6.0 |
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
| Pressing, seam tape, bond, skive | Press, Tape, Bond | ffb-sim | 12 | 07 | landed v0.22.0: Press, Tape and Bond v0.12.0; Skive finish (percent 0 to 100) lowers bending stiffness and solver thickness on a band along the chosen edge, the inverse of bond, stretch is not changed, saved and reopened with the project |
| Trims, buckles, rings from OBJ or FBX | Trims | ffb-io, ffb-sim | 12 | 07 | landed v0.21.0: OBJ trims v0.12.0 as rigid meshes glued to a panel point; `trim_import` reads .fbx through the body FBX reader (millimetres, Y up from the file, units from the file header, scale is an extra multiplier, up is ignored); the 8 MB and 20,000 triangle limits are kept |
| Padding and quilting via pressure clones | Quilt | ffb-sim | 12 | 07 | landed v0.12.0 |

## Animation

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Animation mode and editor, timeline, frame rate | Animate mode and Timeline | ffb-anim, ui | 13 | 08 | landed v0.15.0 (a timeline dock; the Animate workspace tab has been enabled since v0.19.0) |
| Record simulation to cache layers (pattern, trim, soft body) | Record | ffb-anim, ffb-sim | 13 | 08 | landed v0.20.0 for panels, body pose and hardware trims (buttons, snaps and trim items are stored as a cache layer of kind trim); deferred: strip, zipper and stitch trim caches (they follow the panels) and the body soft body cache (see the Body collision row) |
| Cache editing (keys, cut, merge, move, lock, override layers) | Cache Layers | ffb-anim | 13 | 08 | landed v0.20.0: cut, merge (the top layer wins, gaps are filled linearly), move by frames, lock, top-down frame resolution and re-record into a layer, on top of record, rename, show, hide and remove (v0.15.0); deferred: layer reorder by drag and per-layer override blending |
| Property keyframes (constant, linear, cubic, auto key) | Keys | ffb-anim | 13 | 08 | landed v0.20.0 for wind, panel state, fabric and pin keys with auto key (one undo step per amended key); step, linear and cubic keys v0.15.0; deferred: body pose and measurement keys, which live outside the garment document |
| Motion import, Auto Convert to Motion, transition from T pose | Motion import, Retarget, Lead-in | ffb-anim, ffb-io | 13 | 08 | landed v0.15.0 (BVH, glTF and FBX; FBX motion not yet tried in the app; no Auto Convert) |
| Wind, pin, tack and freeze animation | Animated controls | ffb-anim | 13 | 08 | landed v0.20.0 for pins (`key_set` target `pin:ID`, evaluated before every solver step while recording) beside wind, panel state and fabric tracks (v0.15.0); deferred: tack keys, because the reference documents no tack properties to key |
| Export Alembic, MDD, PC2, OBJ sequence, FBX caches, USD, glTF | Cache export | ffb-io | 14 | 08, 09 | landed: Alembic (v0.23.0, own Ogawa writer, uniform time sampling at the cache fps, P per frame with constant topology) joins the formats landed v0.15.0; cache layers resolve top down per frame and trim layers are skipped (v0.21.0); HDF5 layout not written; Blender acceptance is an owner check |

## Export, UV and retopology

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Shared export options (weld, thin or thick, single or multiple, scale, axis, range) | Export options | ffb-io | 14 | 09 | landed v0.21.0: weld modes unweld (default), weld and weld_selected for OBJ, FBX, glTF, USD and the cache formats, one index map for all frames, UVs stay per corner; thick or thin, single or per panel, scale, units, axis and frame range v0.15.0; deferred: seams that are not a single full-length rim segment in quad meshes, and welded output as separate objects |
| OBJ, FBX, Alembic, USD, glTF and GLB, MDD, PC2 | Exporters | ffb-io | 14 | 09 | landed: all formats; OBJ, FBX, USD, glTF and GLB, MDD and PC2 v0.15.0 (open in Blender 5.2), Alembic (Ogawa) v0.23.0 verified with the Alembic 1.8.12 library's own tools and reader (positions, faces, UVs and normals bitwise equal), Blender acceptance pending the owner check |
| Thickness export (quad meshes) | Thickness | ffb-io | 14 | 09 | landed v0.21.0: quad faces extrude with reversed back quads and quad side walls along boundary edges, and the walls follow the welded boundary when weld is on; triangle meshes v0.15.0; limit: a welded thick quad mesh can leave 4 open wall edges where the quad remesh leaves a notch triangle on a sewn rim |
| Remeshing to aligned quads, polygon optimization | Quad Mesh | ffb-mesh | 14 | 09 | partial: field-aligned quad remesh with seam-matched borders landed v0.15.0; v0.23.0 adds integer edge offsets, singularity detection, a min-cost flow (self-written, successive shortest paths), zero-edge collapse and triangle pairing behind `regularize` (default off); flip repair, SAT and the multi-resolution hierarchy are not built; measured 263 against 300 irregular interior vertices over the four T-shirt panels at densities 1, 1.5, 2 and 3 (12 percent fewer in total), but worse at densities 1 and 1.5 |
| Retopology (point and face drawing, ghost points across seams) | Retopo mode | ffb-mesh, ui | 14 | 09 | landed v0.21.0 as per-panel topology drawing in the 3D View (points, faces of 3 or 4 points, open lines, merge, merge all by distance, divide, remesh replace or duplicate) with ghost points mirrored across sewn edges, undoable and stored in the project; deferred: Edit Topology drag and box select, Patch Topology, face loop add by Shift+Ctrl, and a second OBJ object for duplicate mode |
| Unified UV, UV editor, auto packing, UDIM naming | UV mode | ffb-uv | 14 | 09 | landed v0.15.0 |
| Texture baking (diffuse, normal, metal, roughness, opacity, displacement) | Bake | ffb-uv | 14 | 09 | landed v0.21.0: diffuse, metalness, roughness and opacity maps beside normal, ambient occlusion and displacement (v0.15.0), seam fill `fill_px` 0 to 256 (0 gives transparent outside islands), PNG at the requested square size; only PNG textures are sampled (bilinear, repeat), texture rotation and offset are ignored like the exporters |
| Python API | formfabric Python API (headless engine module) | ffb-py | 17 | 09 | landed v0.18.0 (`python/formfabric`: every command as a function, in-process and HTTP backends) |
| Plugins menu | Python plugins (family extension, formsculpt's PARAMS model) | service | 17 | 12 | landed v0.18.0 (PARAMS validation, a sample drape-and-export plugin) |
| (no reference feature) | GoF hand-off to formsculpt and formsurface (family extension) | service, ffb-io | 17 | S3 | deferred, pending owner sign-off: the full hand-off needs the formsculpt owner to agree the contract in docs/integration/gof-handoff.md; a sender that needs no format change landed v0.18.0 |
| Unreal LiveSync, USD with simulation data | Live Link and USD sim export | ffb-io, service | 17 | 09 | deferred, pending owner sign-off: Live Link needs an owner run against Unreal; USD time samples landed v0.15.0 and the Live Link sender (a usda layer rewritten while draping) in v0.18.0 |
| Game-ready pipeline (optimize, seal, rig and weights, UV pack, bake, FBX) | Game-Ready Export | ffb-io, ffb-uv | 14 | 11 | landed v0.15.0 |

## Visualization

| Reference feature | formfabric name | Module | Phase | Ch | Status |
| --- | --- | --- | --- | --- | --- |
| Display toggles and render styles (Alt+1 to Alt+8) | View styles | ui/viewport | 5, 15 | 10 | landed v0.15.0 (view style buttons with Alt+number keys since v0.6.0) |
| Schematic render | Schematic | ui/viewport | 15 | 10 | landed v0.15.0 (SVG and PDF export) |
| Lights, shadows, HDRI, background image | Stage | ui/viewport | 15 | 10 | landed v0.19.0: background image (`stage_set` background {image, fit cover or contain}; local png, jpg, jpeg or webp, drawn as a screen-aligned quad behind the scene, skipped for transparent snapshots) beside lights, shadows, HDRI and ground (v0.15.0) |
| Saved custom views with display state | Saved Views | ui/viewport | 15 | 10 | landed v0.15.0 |
| Snapshot (multi-view, transparent, ppi), video capture, turntable | Capture | ui/viewport, service | 15 | 10 | landed v0.19.0: multi-view snapshot (eight camera views, 2 to 10 views, horizontal or vertical combine, separate files), a ppi field written into the PNG pHYs chunk, and a transparent background for the turntable as GIF (PNG frames are not offered); PNG snapshot, WebM video, GIF and turntable v0.15.0 |
| Strain, stress and pressure maps with units and bins | Fit Maps | ffb-sim, ui/viewport | 15 | 10 | landed v0.15.0 |
| 3D measure (circumference, length) | Measure 3D | ffb-body, ui | 15 | 10 | landed v0.15.0 |

## Beyond scope of v1.0.0 (recorded, not planned)

| Reference feature | Why deferred | Status |
| --- | --- | --- |
| AI Studio (texture, graphic, PBR map, pose generator, pattern drafter) | Needs model hosting and data licensing decisions; a later plan | deferred: model hosting and data licensing undecided |
| Modular block system with structure files and ML labeling | Planned in Phase 16 as Components (the word block is not used) without the ML labeling | landed v0.18.0 for Components (ports, joins, swap and check by port name and length, PATENT_NOTES 2.6); ML labeling deferred: no training data or hosting |
| CONNECT asset marketplace | A service, not an app feature | deferred: a service, not an app feature |
| Reading .zprj, .zpac or other proprietary files | Undocumented formats; interop goes through open formats | deferred: undocumented proprietary formats, by decision |
