# Marvelous Designer: The Simulation System As The User Sees It

Researcher slice 05 of 13. Purpose: feed the formfabric team (open-source clone of Marvelous Designer, "MD") with a verified account of the user-facing simulation system, plus what each feature implies for the solver.

## Summary

Verified, high level:

- MD exposes simulation through a toolbar Simulate toggle, three or four named presets (Normal, Animation/Stable, Fitting, plus a CPU/GPU Processor choice), a global Simulation Properties panel (Object Browser, Scene, Environment), and per-pattern Simulation Properties in the Property Editor (Particle Distance, Layer, Add'l Thickness-Collision, Shrinkage Weft/Warp, Pressure).
- Global solver knobs documented: Time Step, Number of Simulation (substeps per time step), CG Finish Condition (Iteration count or Residual), CG Iteration Count, CG Residual, Self Collision Iteration Count, Air Damping, Gravity (default -9800), CPU core count (max 12), Nonlinear Simulation toggle, and collision toggles (Avatar-Cloth, Self-Collision with Avoidance Stiffness default 0.001, Proximity, Intersection Resolution with Normal/Flipped Normal, Layer Based Collision).
- The manual implies the solver is an implicit (linear system solved by conjugate gradient) integrator with sub-stepping, plus a separate collision stage, particle-based mesh (Particle Distance = mesh edge length), and a layer-ordering mechanism for collision resolution. This is consistent with CLO-associated academic work on semi-implicit integration with reduced artificial damping (Oh, Ahn, Wohn 2006) and multigrid (title only verified).
- Per-state pattern flags: Freeze (rigid-like, still collides), Deactivate (pattern ignored, optionally with sewing), Strengthen (temporary stiffening, orange), Solidify (retain draped shape, Strength parameter), Pins (vertex constraints, optionally attached to avatar), Layer (collision ordering), Shrinkage, Pressure.
- Important gaps (not verifiable from approved sources, see "Unverified" section): numeric ranges and defaults for Time Step, Number of Simulation, CG iteration/residual, Air Damping; the existence of "Hard pin" versus "Soft pin" as named MD features; "Fix" and "Settle" as named MD commands; an explicit Stable/Complete quality list beyond Normal and Complete.

Sourcing note: community/forum posts on support.marvelousdesigner.com and support.clo3d.com were treated as banned and not used, with one exception flagged below (the CLO "6 Things to Know About Particle Distance" post, which is a staff-style help-center post under support.clo3d.com; marked [community-hosted, official domain]). Style3D help pages (a competitor with a similar UI) are used only as a labelled secondary reference and never as the sole source of an MD claim.

## 1. Simulate on/off and the Simulate tool

| Item | Detail | Source |
|---|---|---|
| What it does | Applies gravity to all 3D garments so they fall toward the floor and drape onto avatars and other colliding objects | MD manual "Simulate" |
| Where | Main Menu, 3D Garment, Simulate; or 3D Toolbar, Simulate icon | MD manual |
| Preset picker | Long-press the Simulate icon on the 3D toolbar (since 4.2.0) | MD manual |
| Tool switching | When simulation starts, the active tool switches to Select/Move automatically (since 5.1.0) | MD manual |
| Stop | In 2025.2 and later, simulation can be stopped instantly even under heavy load | MD manual |
| Auto-on interactions | Quick Pinching (key Q) is active by default while simulation runs | MD manual "Quick Pinching" |

Implications for the clone:

- Simulation is a toggled, continuous real-time loop, not a one-shot bake. The UI thread must be able to interrupt the solver at any step (the 2025.2 change suggests earlier versions could hang under load).
- Editing patterns while simulating is tolerated but progress can be lost on edit (a help-center post title references "Simulation progress lost if pattern was edited"; forum, not used as evidence, so treat as [unverified]).
- Interactive manipulation (dragging, pinching, wind) happens inside the same loop as external forces or temporary constraints.

## 2. Simulation presets and quality modes

### 2.1 What the manual states

| Preset | Manual label | Stated purpose | Source |
|---|---|---|---|
| Normal (default) | Normal | For working on a 3D garment; fastest simulation speed | MD manual "Simulate" |
| Animation (Stable) | Animation (Stable) | Higher simulation quality but slower; use when recording animation caches. Added to the Simulation Toolbar in 2024.2 | MD manual "Simulate" |
| Fitting (Accurate Fabric) | Fitting | Most accurate and slowest; fabric stretch is shown more realistically | MD manual "Simulate" |
| GPU | GPU | Listed as a primary preset in the CLO "Edit Simulation Properties" page ("Normal, Fitting, and GPU are the primarily used presets") | CLO manual "Edit Simulation Properties" |

Simulation Quality in Animation mode:

- The Animation mode page states a "Simulation Quality" setting with a "Complete" value that "runs more simulation checks" so avatar and garment are processed more precisely; recommended for stable recordings because recorded motion increases collisions. Cost is implied, not quantified. [single source: MD manual "Mode Animation"]
- The Hi-Res/Low-Res Garment page gives paired presets: Hi-Res uses Simulation Quality = Complete, Low-Res uses Simulation Quality = Normal. Complete gives "the most accurate results for material properties generated in Emulator Mode, but it slows simulation." [single source: CLO manual "Hi-Res / Low-Res Garment"]
- The user's brief names "Normal, Stable/Complete, Fitting, Animation". The verified vocabulary is: presets Normal / Animation (Stable) / Fitting, and Simulation Quality values Normal / Complete. "Stable" is the UI label attached to the Animation preset in 2024.2; "Complete" is a Simulation Quality level. The exact internal mapping from each preset to individual parameters (Time Step, Number of Simulation, CG settings, Nonlinear flag) is NOT published.

### 2.2 Documented relationships between presets and parameters

| Fact | Source |
|---|---|
| Nonlinear Simulation is "selected automatically when a Fitting simulation runs" | CLO manual "Edit Simulation Properties" and MD manual "Simulation Properties" |
| Simulations Per Frame (Simulation Quality) splits each time step into substeps; example: time step 0.03 with 10 simulations gives effective 0.003 | both manuals |
| Complete quality raises simulation checks; Normal is cheaper | MD "Mode Animation", CLO "Hi-Res" |
| Presets can be saved/loaded: folder icon imports, save icon stores custom settings (also Open/Save buttons at top of Property Editor) | both manuals |

Solver implications:

- A preset is just a named bundle: {time step, substeps, CG termination, collision iteration counts, nonlinear flag, thickness/offset}. The clone should model presets as serializable parameter sets and allow user presets (MD does: import/save).
- Fitting implies a nonlinear (likely true stretch/shear material model, possibly Newton or iterated linearization) path, versus Normal being linear/semi-implicit. This is an inference from the "Nonlinear Simulation" toggle; the solver form is not published.
- Complete/Stable implies more substeps and more collision passes, not a different physical model.

## 3. Simulation Properties (global, scene level)

Location: Object Browser, Scene, Environment, Simulation Properties (CLO manual). In the Property Editor the page also offers Import/Save buttons.

| Property | Manual description | Default | Range | Units | Source |
|---|---|---|---|---|---|
| Preset | Choose predefined mode instead of toolbar | Normal | enum | n/a | both |
| Time Step | Time values the engine computes. Smaller = more precise but slower. "Default is recommended" | not published | not published | seconds (inferred from 0.03 example) | both |
| Number of Simulation (Simulations Per Frame / Quality) | Divides each time step into substeps. More = better numeric accuracy and collision stability, much slower | not published (example 10) | not published | count | both |
| CG Finish Condition | Iteration (stop after fixed count; faster, less accurate) or Residual (stop below error threshold; slower, more accurate) | not published | enum of 2 | n/a | both |
| CG Iteration Count | Count used with Iteration condition | not published | not published | count | both |
| CG Residual | Error threshold used with Residual condition | not published | not published | dimensionless | both |
| Self Collision Iteration Count | Number of collision calculations per step; more is more precise, slower. "Use default" | not published | not published | count | both |
| Air Damping | Resistance the garment receives from air | not published | not published | not stated | both |
| Gravity | Main gravity value. Changing it makes results unrealistic unless you want weightless. 0 = no gravity; "positive values produce other effects" | -9800 | not published | not stated (consistent with mm/s^2, so -9800 = -9.8 m/s^2; inference) | MD manual |
| Number of CPU in Use | Physical cores used; MD can use up to 12, auto-detected; a 16-core CPU still uses 12 | auto | up to 12 | cores | both |
| Nonlinear Simulation | Linear vs nonlinear calculation; nonlinear looks more realistic, slower; auto-selected when Fitting runs | off (Normal) | bool | n/a | both |
| Ground | Controls collision and friction with ground | default | n/a | n/a | MD manual |
| Avatar-Cloth Collision Detection | on/off | on (implied, not stated) | bool | n/a | both |
| Self-Collision Detection | on/off, with Avoidance Stiffness | Avoidance Stiffness 0.001000 | not published | n/a | MD manual |
| Proximity Detection | on/off, no further explanation | not published | bool | n/a | MD manual |
| Intersection Resolution | on/off; "Normal / Flipped Normal values" set the direction used to resolve intersecting meshes | not published | bool | n/a | MD manual |
| Layer Based Collision Detection | on/off; "layers and sublayers set collision values"; example: layer 1 sits on top of layers 0 and below | not published | bool | n/a | MD manual |

Solver implications:

- Time Step plus substep count give effective dt = Time Step / N. The clone needs both: a frame-level dt (UI/animation rate) and an internal sub-stepping integer.
- CG Finish Condition confirms the core linear solve is preconditioned or plain conjugate gradient on an implicit system (Ax = b per step). Two termination criteria must be supported: fixed iteration cap and relative/absolute residual.
- "Self Collision Iteration Count" implies an iterative collision response loop separate from CG (e.g., repeated detect-and-resolve passes), with a stiffness-like "Avoidance Stiffness" for a proximity/repulsion force.
- "Proximity Detection" and "Avoidance Stiffness" imply a repulsion-force stage active before actual contact (a thickness-based buffer), versus hard contact constraints.
- "Intersection Resolution" with Normal/Flipped Normal indicates a post-hoc untangling step that pushes interpenetrating vertices out along either mesh normal or its flipped version; it is distinct from continuous collision.
- Core count capped at 12 suggests CPU parallelism saturates (coloring or domain decomposition limits); do not expect scaling beyond ~12 threads.

## 4. CPU versus GPU simulation

| Fact | Detail | Source |
|---|---|---|
| Choice | "Processor" setting offers CPU or GPU | MD manual "Simulate" |
| Speed | GPU "can speed up simulation depending on your computer's specs" (no numbers) | MD manual |
| Parity | Since 2024.2 GPU collision handling matches CPU with minimal differences | MD manual; 80.lv coverage of 2024.2 (independent secondary): "simulations running on the GPU match the collision handling accuracy of those on the CPU" and can handle multiple clothing layers |
| Toolbar | CPU or GPU can be chosen from the simulation toolbar with different presets (2024.2) | MD manual |
| Limits | GPU acceleration for Trim Simulation and Soft Body Simulation is limited in that version | MD manual |
| Trims | Trims (buttons, buckles) can now have collision properties and respond to weight and gravity (2024.2) | 80.lv |

Solver implications:

- The GPU path must reproduce CPU contact behavior including multi-layer collisions; the historic gap was collision fidelity, not stretching.
- GPU solves are limited for rigid-ish extras (trim, soft body), which implies those use a different, CPU-resident dynamics code path.
- A clone should define one solver spec with two backends (CPU reference, GPU accelerated), with CPU as ground truth for regression tests.
- No VRAM, vendor, or minimum hardware limits were found in approved sources. [unverified]

## 5. Pattern-level simulation properties

Location for all: select pattern(s), Property Editor, Simulation Properties.

| Property | What it does | Default | Range | Source |
|---|---|---|---|---|
| Particle Distance | Average distance between points of the pattern mesh; sets mesh size. Affects quality and speed | 20 mm working value | 0.8 to 700.0 | CLO manual "Particle Distance" (range and values) |
| Layer | Draping order of overlapping patterns; 0 = off, order decided automatically | 0 | integers; layers 1 to 23 have distinct colors, repeats from 24; negatives are reverse order; min/max not stated | MD manual "Layer" |
| Add'l Thickness-Collision | Extra collision thickness added to the pattern | 1.5 mm per side (3 mm total) default garment simulation thickness | not stated | MD manual "Adjust Collision Thickness" |
| Shrinkage Weft / Warp | Shrinks or stretches the pattern in 3D only | 100 percent (natural state) | not stated | MD manual "Shrink Pattern" |
| Pressure | Inflates a pattern (pillows, padded jumpers) | 0 (implied) | not stated in MD manual | CLO manual "Pressure" |

### 5.1 Particle Distance (PD)

| Fact | Source |
|---|---|
| PD is "the size of each polygon", smaller PD = more polygons | CLO "6 Things to Know About Particle Distance" [community-hosted, official domain] and CLO manual |
| Range 0.8 to 700.0 | CLO manual |
| Default working value 20 mm; use 20 mm while building and dressing | CLO manual and community post |
| Under 5 mm for finalized garments, or use Hi-Res Garment | CLO manual; Hi-Res page: Hi-Res PD 5 mm, Low-Res 20 mm |
| Hi-Res/Low-Res tolerance: applies to patterns within the 5 to 20 mm range; small patterns below 5 mm keep their finer spacing for stability | CLO Hi-Res page |
| At very large PD (example 100 mm) pieces can pass through the avatar or each other | community post [single source] |
| Change at any time via Transform Pattern then Property Editor | community post, CLO manual |
| Apply: select pattern(s), type value, press Enter | CLO manual |

Hi-Res / Low-Res Garment bundle (Main Menu, 3D, Hi-Res Garment / Low-Res Garment) [single source: CLO manual]:

| Setting | Hi-Res | Low-Res |
|---|---|---|
| Garment Particle Distance | 5 mm | 20 mm |
| Add'l Thickness (Collision) | 1 mm | 2.5 mm |
| Avatar Skin Offset | 0 mm | 3 mm |
| Simulation Quality | Complete | Normal |

Implication: Hi-Res reduces PD, collision thickness, and skin offset together, so the clone needs a single "resolution preset" that retargets mesh density and collision margins together. Note the smaller PD requires a proportionally smaller collision margin: thickness (1 mm) is below PD (5 mm) in Hi-Res, and 2.5 mm below 20 mm in Low-Res.

Solver implications:

- Per-pattern edge length, not a global mesh size: each 2D pattern is triangulated (likely regular-ish triangle mesh) with target edge length PD. Seams need to stitch meshes with different PDs (T-junction/mismatch handling), since patterns with different PD are sewn together.
- Remeshing happens on PD change without losing draped state (the user can change PD anytime during a session). The clone needs to resample the current 3D state onto a new 2D-parameterized mesh (interpolate positions and velocities).
- Cost scales roughly with vertex count, so PD halving = about 4x vertices.

### 5.2 Thickness: collision versus rendering

| Fact | Detail | Source |
|---|---|---|
| Simulation (collision) thickness | Invisible property that sets the collision limit "for Garments to result smooth Simulation" | MD manual "Adjust Collision Thickness" |
| Default | 1.5 mm per side, 3 mm total | same |
| Rendering thickness | Separate visual value; since 2026.1 the collision-thickness direction matches the visible rendering thickness | same |
| Field | Add'l Thickness-Collision in Simulation Properties; applied immediately | same |
| Guidance | Lower for form-fitting, higher for padded/bulky garments | same |
| Avatar side | Avatar Skin Offset default 3 mm; adjust together with thickness for form-fitting fit measurements | same |
| Style3D equivalent (secondary) | Extra Rendering Thickness (visual, requires Show Fabric Thickness) versus Extra Collision Thickness (enlarges collision volume) | Style3D help [competitor, secondary] |

Implications: collision contact distance between two cloth surfaces is approximately the sum of their per-side thicknesses; cloth versus avatar is cloth thickness plus avatar skin offset. Rendering thickness is a display-only offset (shell extrusion), and since 2026.1 the collision offset follows the surface side that matches it. [single source for the 2026.1 direction change]

### 5.3 Layer property

| Fact | Detail |
|---|---|
| Purpose | Controls which overlapping layers simulate inward or outward; fixes inward-simulated pockets and buttons; enables outerwear over dresses/shirts |
| Operation | Select pattern (3D Select/Move or 2D), Property Editor, Simulation Properties, Layer, type value, click Simulate |
| Visual | Patterns with layer values show a blue outline; Show Pattern Layer (2D Information) colors by layer (2024.1); layer 0 always light gray |
| Best practice | When draping order is correct, set layer back to 0 for more stable simulation; draping does not change |
| Mixed selection | If Layer shows red with value 0, selected patterns have mixed values |

Solver implications: Layer is an ordering constraint for collision resolution (which side each cloth is allowed to be on), used to break symmetry in initial interpenetration. When all layers are 0 the solver relies on the automatic arrangement order. The advice to reset to 0 for stability implies layer-based collision is more restrictive or costlier than free collision. The global "Layer Based Collision Detection" toggle (Section 3) activates this behavior with layers and sublayers.

### 5.4 Shrinkage

| Fact | Detail |
|---|---|
| Parameter | Shrinkage Weft / Warp, per direction |
| Default | 100 percent (no shrink or stretch) |
| Effect | Pattern shrinks or stretches in 3D; the 2D pattern is not changed |
| Uses | Wash shrinkage; temporarily shrinking an inner layer so outer layers drape easily |
| Range | Not stated |
| Related | Steam brush (Materials, Steam) has Shrinkage, Size, Hardness, Mode Add/Remove (painted shrinkage per region); defaults and ranges not given [single source: MD manual "Steam"] |

Solver implications: rest-length scaling of the warp and weft edges of the stretch model (rest state change), independent of the 2D pattern geometry. Needs anisotropic rest metric per triangle in the material frame (warp/weft aligned). Steam implies a spatially varying rest-metric field painted on the mesh.

### 5.5 Pressure

| Fact | Detail |
|---|---|
| Purpose | Inflate patterns: padded jumpers, balloons, pillows, cushions |
| Sign | Positive pushes against the back of the fabric; negative applies to the front |
| Pairing | For paired (outside and inside) patterns the absolute values must match, e.g. 10 and -10 |
| Result | Patterns inflate when simulation is on |
| Range | MD sources give only examples; Style3D (competitor, secondary) shows -100 to 100 |

Solver implications: per-face force along the normal proportional to pressure times area (or a volume-preserving constraint on a closed surface). The need for a mirrored pair implies the "inside" and "outside" faces are separate pattern meshes forming a closed bag, with pressure acting outward on both.

## 6. Freeze, Deactivate, Strengthen, Solidify (pattern state flags)

| Command | Behavior | Where | Visual | Source |
|---|---|---|---|---|
| Freeze / Unfreeze | Fastens patterns so they behave like solid objects; frozen patterns still collide with other objects; recommended to drape and freeze inner layers first, then drape outer wear (less instability) | 3D: Select/Move, right-click garment. 2D: Transform Pattern, right-click pattern | Freeze color; Show/Hide 3D Color toggles | MD manual "Freeze Unfreeze" |
| Deactivate (Pattern Only) / Activate | Pattern treated as nonexistent in 3D; others simulate as if absent; use to keep fine details (pleats, slits) from collapsing; also speeds simulation | same menus | n/a | MD manual |
| Deactivate (Pattern and Sewing) / Activate | Variant that also affects sewing; exact semantic difference not described on the Pattern Only page | | | CLO manual title only [unverified detail] |
| Deactivate / Activate Avatar | Garments do not recognize the avatar during simulation | 3D Select/Move, right-click avatar | | MD manual |
| Trim simulation status (2025.0) | Freeze, Activate, Deactivate on 3D trims via context menu, Simulation submenu; Show All Trim Status Color added in 2025.1 | 3D trim context menu | status colors | MD "New in 2025.0" |
| Strengthen / Unstrengthen | Temporary hardened state: all stiffness properties increased, clean folds at fold angles applied; patterns turn orange | 3D and 2D context menu; Ctrl+H (Windows) or Ctrl+L (Mac) | orange | MD manual "Strengthen Unstrengthen" |
| Solidify | Maintains the desired draped state per pattern (example: jeans knee creases); Property Editor, Bond/Skive, Solidify, Strength: higher = better shape retention | 3D right-click Solidify | solidify color | MD manual "Solidify" (Ver 2.5.0) |

Notes:

- The user brief lists "Strengthen (increase resolution)". The manual describes Strengthen as a stiffness boost (hardened state), not a resolution increase. Resolution increase is the Particle Distance / Hi-Res Garment mechanism. This is a correction to the brief.
- Strengthen parameters beyond on/off are not exposed in the manual page. [unverified: whether an internal stiffness multiplier exists]

Solver implications:

- Freeze: vertices become kinematic (infinite mass) or are rigidly attached to a frozen reference configuration, yet remain in collision. Frozen patterns can be moved only as rigid bodies.
- Deactivate: remove vertices and constraints from solve and from collision acceleration structures entirely; this is both a stability tool and a speedup. Pattern and Sewing variant also removes seam constraints.
- Strengthen: temporary scaling of stretch, shear, and bend stiffness plus using fold-angle rest state for crisp folds; reversible on Unstrengthen.
- Solidify: a per-pattern shape-memory constraint, with Strength = stiffness of a restoring force toward the captured draped configuration (rest shape update).

## 7. Pins

### 7.1 Pin tools and operations (verified)

| Operation | Detail | Source |
|---|---|---|
| Tools | Pin (Box) and Pin (Lasso): Main Menu, 3D Garment; or hold the 3D Toolbar button more than 2 seconds. Garment mesh turns red in both 3D and 2D windows | MD manual "Pins" and "Create Pin" |
| Single vertex | Click a vertex; with Select/Move hold W and click a point | same |
| Area | Click and drag over area to pin vertices | same |
| Segment | Double-click a pattern outline; in 2D adjacent vertices with angle difference under 20 degrees also get pins | same |
| Whole pattern | Double-click a pattern | same |
| Internal line or shape | Double-click (ver 4.0.0) | same |
| Select all pins on pattern | Double-click an existing pin (2025.2) | MD "Pins" |
| 2D pinning | Shift+Ctrl+drag on a pattern adds pinned areas; Ctrl+drag removes pins | same |
| Moving | Drag the pinned mesh in 3D or use the Gizmo; pins work like Freeze; useful for turning collars, lapels, cuffs | same |
| Delete | Edit, context menu, 3D Garment: delete all or selected-pattern pins; right-click pin; W-select pin | same |
| Symmetry | "Duplicate to Symmetric Pattern" copies pins to mirrored pattern (2024.2); error for patterns without symmetry | same |
| Attach to avatar | Right-click pin: Attach Pin to Avatar / Attach All Pins to Avatar; fixes to the nearest avatar part, keeps garment from sliding in simulation or animation recording; attached pins cannot be selected or moved; release via Detach from Avatar | same |

### 7.2 Hard pin versus Soft pin

Not verified. The approved sources fetched describe a single "Pin" concept that "works like Freeze" (positional constraint), plus separate "Quick Pinching" and "Advanced Pinching" (temporary mouse-driven pulls, Section 9). No MD page using the terms "Hard Pin" or "Soft Pin" was found, and the 2026.1 notes list Brush Pinching only. Treat "hard/soft pin" as [unverified]. Reasonable inference for the clone: hard pin = Dirichlet constraint (vertex fixed to target), soft pin = stiff spring to a target; MD's attach-to-avatar pin is the closest to a moving-target constraint.

Solver implications:

- Pins are vertex position constraints on the cloth mesh; in an implicit CG solve they are handled by filtering (zeroing DOFs) or by large stiffness springs. Moving the pinned region by Gizmo means constraint targets are time-varying and need velocity update to avoid snap.
- Pin attached to avatar requires tracking the target avatar skinned vertex or bone (nearest part), updated every animation frame.
- The 20 degree rule is a 2D-side selection heuristic for segment chaining along a pattern outline, easy to clone.

## 8. Collision settings: avatars, objects, ground, trims

| Setting | Detail | Source |
|---|---|---|
| Avatar-Cloth Collision Detection | toggle | MD Simulation Properties |
| Avatar Skin Offset | default 3 mm (0 mm in Hi-Res) | MD thickness page, CLO Hi-Res page |
| Garment collision thickness | 1.5 mm per side by default (3 mm total); Hi-Res add'l thickness 1 mm, Low-Res 2.5 mm | same |
| Ground | collision and friction with ground; default recommended | MD Simulation Properties |
| Deactivate Avatar | garments ignore the avatar | MD manual |
| Trims | 2024.2: collision properties on buttons and buckles; 2025.0: Freeze/Activate/Deactivate simulation status for trims | 80.lv; MD "New in 2025.0" |
| Per-pattern frozen collision | Frozen patterns collide, deactivated do not | MD manual |
| Friction | Garment-avatar and garment-ground friction exist; per-fabric friction is part of the fabric Physical Property Preset (not fetched in this slice) | [partially unverified] |

Solver implications:

- Avatar collision uses the avatar surface mesh (skinned when animating) plus an offset, so a signed-distance or BVH proximity query with thickness margin is required.
- Skin Offset is a separate scalar from garment thickness; contact distance = garment half thickness + skin offset (inference).

## 9. Drag behavior while simulating, Quick and Advanced Pinching

| Fact | Detail | Source |
|---|---|---|
| Select/Move with Simulate on | Click and drag part of the 3D garment; it moves naturally, for fine-tuning | MD "Select Move 3D" |
| Select/Move with Simulate off | Dragging switches to pattern arrangement; garment moves as patterns | same |
| Quick Pinching | Pulls the mouse-picked point as a single point; automatically active when simulation runs; key Q; 3D Toolbar drop-down (2025.0); no parameters | MD "Quick Pinching" |
| Advanced Pinching | Soft-selection style pinch with falloff shape, distance, power; 2025.0 under the Pinching drop-down in the 3D Toolbar; 2024.2 and earlier via 3D background context menu, Shift+E | MD "Quick Pinching" |
| Brush Pinching (2026.1) | Deforms garment surface along a brush stroke. Brush options: Size, Focal Radius, Lazy Distance (stroke trail length), Display Weights. Pinching options: Strength, Push Inward, Average Radius (reference area for averaging normals to stabilize pinch direction) | MD "New in 2026.1" |

Solver implications:

- Dragging while simulating is a temporary external force or soft positional constraint on the picked vertex (or weighted vertex set) that tracks the cursor in a camera-aligned plane. Release removes it, and the cloth continues to settle.
- Advanced Pinching's falloff (shape, distance, power) suggests a weight field w(d) over geodesic or Euclidean distance applied to constraint targets, so a clone needs a configurable falloff kernel.
- Brush Pinching accumulates stroke path points as targets over time (lazy distance acts as stroke smoothing).

## 10. Wind controller

Location: Main Menu, Display, Environment, Show Wind Controller (or 3D Window View icon, Environment). A cone-shaped controller appears. Click it to get a gizmo and Property Editor values. Tick Activate under Wind then press Simulation.

| Property | Description | Range / default | Source |
|---|---|---|---|
| Name | Rename the wind | free text | MD wind pages |
| Activate | On/off | off until ticked | same |
| Unlimited Bound | On by default; wind affects the whole space based on position and direction | default on | MD "Wind Controller" |
| Type | Spherical or Planar | enum | both pages |
| Strength | Intensity | not stated | both |
| Decay | Range; the wind follows an arc and weakens with distance; 0 = straight line indefinitely | 0 to 100 | both |
| Frequency (sec) | Cycle of strengthening and weakening; 0 = steady | not stated | both |
| Turbulence | Irregularity of intensity | 0 to 100 | both |
| Rotation Key Activate | Allows animation keys to set wind rotation | bool | MD "Wind Controller" |
| Translation X / Y / Z | Keyable position values | | same |
| Multiple winds | Scene Browser, Wind group, right-click, Create Wind or Duplicate | | same |

Two independent MD manual pages agree on the table (the two pages differ only in menu path wording).

Solver implications: external aerodynamic force per triangle, typically proportional to (wind velocity relative to cloth velocity) projected on the face normal times area (inference), with wind field = direction, strength, spatial decay (spherical: radial from source; planar: along axis), temporal modulation (frequency, sin-like) and noise (turbulence). Keyframeable transform means wind is animated like any scene object.

## 11. Gravity and environment changes

- Gravity default -9800, editable in global Simulation Properties; 0 gives weightless; positive values "produce other effects" (flip). Keep default for realism. [single source: MD manual]
- Air Damping is global; no value published.
- Practical for the clone: gravity vector as scene parameter, with direction (assumed along -Y or -Z per the app's world axis) and magnitude; allow change at runtime without restarting the solver.

## 12. Intersection resolution

| Item | Detail | Source |
|---|---|---|
| Global toggle | Intersection Resolution on/off in Simulation Properties; Normal / Flipped Normal values set the direction for fixing intersecting meshes | MD Simulation Properties |
| Related | Layer property (ordering), Layer Based Collision Detection, Self-Collision Detection with Avoidance Stiffness 0.001, Proximity Detection | same, plus MD "Layer" |
| Manual stability advice | Freeze inner layers before draping outer layers; set Layer back to 0 after draping; deactivate fine-detail patterns; use Hi-Res (smaller PD) for final | MD manuals |

No dedicated "intersection resolution tool" page (brush-like command) was found; the evidence only shows a solver option. [unverified whether a separate interactive tool exists]

Solver implications: cloth-cloth interpenetration is untangled after the fact by moving vertices along the reference normal or its flip, rather than relying purely on continuous collision. A clone should combine (a) repulsion/proximity forces, (b) contact constraints, (c) an untangling pass guided by layer order or user-chosen normal direction.

## 13. Solid objects and rigid behavior

| Feature | Evidence |
|---|---|
| Freeze (patterns as solid objects) | MD manual: "behave like solid objects during simulation", still collide |
| Trim simulation | 2024.2 and 2025.0: trims collide with garments, react to weight and gravity, have Freeze/Activate/Deactivate states; GPU acceleration for them limited |
| Soft Body Simulation | Mentioned by the MD manual as a GPU-limited feature in 2024.2; its dedicated page was not located [unverified detail] (Style3D has a "Soft Body" page, competitor, not fetched) |
| Avatar | Treated as collider (static or animated), can be deactivated |

Implications: three tiers of dynamics: cloth (deformable mesh), kinematic (frozen, pinned, avatar), and trim/soft-body rigid-ish objects with their own dynamics integrated into the same contact system. No general rigid-body engine is documented.

## 14. Animation-time (stable) simulation and morph stability

- Animation mode: record garment animation, switch mode from the top-right control, Animation window; Simulation Quality Complete recommended for stable recording (more checks, more accurate avatar/garment processing). [MD "Mode Animation"]
- Animation (Stable) preset added 2024.2 so you can run simulations before recording an animation cache. [MD "Simulate"]
- Attaching pins to the avatar is recommended to prevent sliding during animation recording. [MD "Pins"]
- The Fitting/Complete family trades speed for accuracy; the cache is recorded per frame, so stability (no popping) outranks speed.
- Morph stability: not covered by the pages fetched. [unverified]

Solver implications: for animation the avatar moves quickly between frames, so more substeps and stricter collision passes are needed to avoid tunneling (hence Complete). Per-frame cache means the clone needs an offline mode with deterministic stepping and a vertex-cache format.

## 15. Fix and Settle, and other commands named in the brief

| Name in brief | Finding |
|---|---|
| Fix | No MD manual page named "Fix" found in simulation context. The closest verified equivalents: Freeze, Pin, Attach Pin to Avatar, Solidify. [unverified] |
| Settle | No MD feature called "Settle" found. A draped garment settles on its own when Simulate runs. [unverified] |
| Hard / Soft pin | See Section 7.2. [unverified] |
| Strengthen as "increase resolution" | Incorrect; Strengthen is temporary stiffening (Section 6). Resolution is via Particle Distance / Hi-Res Garment (Section 5.1). |

## 16. What the academic literature supports about the solver

CLO's own technical publications are not enumerated by CLO; the following were verified as peer-reviewed or archival entries.

| Paper | Venue | What it says | Attribution caveat |
|---|---|---|---|
| Low damped cloth simulation, Oh, Ahn, Wohn | The Visual Computer 22(2), pp. 70 to 79, 2006 | New implicit integration method. Semi-implicit integration adds artificial internal damping through linearization; the authors compute the artificial damping only for pure internal deformation (not rigid rotation) and incorporate it stably; reproduces a range of materials with fewer damping artifacts, including real time | Authors per a library index (ixdf author page for Seungwoo Oh, KAIST repository entry lacks author list); link between this author and CLO is NOT stated on that page |
| A physically faithful multigrid method for fast cloth simulation, Oh, Noh, Wohn | Journal of Visualization and Computer Animation (Computer Animation and Virtual Worlds), 2008 | Title and venue verified from the ixdf author listing; the abstract could not be retrieved | Same caveat |
| Constrainable Multigrid for Cloth, Jeon, Choi, Kim, Choi, Ko | Computer Graphics Forum, 2013 | Multigrid handling point and sliding constraints via soft-constraint formulation; solves the linear system 4 to 9 times faster than modified preconditioned CG without quality loss; applicable to cloth simulators using implicit time integration | Authors are from Hyeong-Seok Ko's group, not stated as CLO staff |

The brief states CLO's founder authored cloth papers on multigrid and implicit integration. The sources above confirm a Seungwoo Oh with exactly those two topics, but no approved page ties him to CLO, so treat the authorship link as plausible and unverified. The SIGGRAPH History Archives entry for CLO Virtual Fashion only confirms exhibition at SIGGRAPH 2011, 2019, 2023, 2024 and "state-of-the-art 3D Cloth Simulation Algorithm" with no named founder. A Justia patent list for CLO (six items) covers draping simulation/automatic sewing, digital clothing creation, draping transfer between avatars, and pattern transfer, none about the solver internals.

What the manual implies, combined with the literature:

| Manual observation | Literature-consistent reading |
|---|---|
| CG Finish Condition (Iteration or Residual) | Implicit backward-Euler or Newton step solved by (preconditioned) conjugate gradient |
| Time Step and Number of Simulation | Fixed dt with sub-stepping |
| Nonlinear Simulation (auto on in Fitting) | Switch from linearized to nonlinear force model, slower, more accurate stretch |
| Low Air Damping default and no generic damping slider | Low-damped implicit integration (Oh et al. 2006) avoids global numerical damping, which fits |
| Core cap 12 | Typically CG matrix-vector products and collision pass parallelized |
| Multigrid | Plausible (title verified for Oh et al. 2008), but not stated by the MD manual; do not claim MD uses it |

## 17. Recommendations for formfabric (derived)

1. Parameter model: expose Time Step, Number of Simulation (substeps), CG Finish Condition (iteration / residual) with CG Iteration Count and CG Residual, Self Collision Iteration Count, Air Damping, Gravity (default -9800 in mm/s^2), Nonlinear toggle, CPU threads, collision toggles, Avoidance Stiffness (default 0.001), Intersection Resolution (Normal / Flipped Normal), Layer Based Collision.
2. Presets: Normal, Animation (Stable), Fitting, plus a Processor flag (CPU/GPU). Presets are serializable files. Choose own numeric defaults; MD does not publish them.
3. Per-pattern: Particle Distance (0.8 to 700, default 20 mm, Hi-Res 5 mm), Layer (integer, 0 = off), Add'l Thickness-Collision (default 1.5 mm/side), Shrinkage Weft/Warp (100 percent default), Pressure (signed, mirrored pairs), plus state flags Freeze, Deactivate (pattern / pattern and sewing), Strengthen, Solidify (Strength).
4. Interaction: Quick Pinching (Q) on while simulating; Advanced Pinching with falloff; Brush Pinching; pins (box, lasso, segment, pattern, internal line) with attach-to-avatar and symmetric duplication.
5. Environment: Wind controller (Spherical/Planar, Strength, Decay 0 to 100, Frequency, Turbulence 0 to 100, Unlimited Bound, keyable transform), multiple winds.
6. Tooling for stability: Freeze-inner-then-drape workflow, Layer reset to 0, Hi-Res switch, Deactivate for fragile details.
7. Interruptibility: the solver step loop must be cancel-safe (MD 2025.2 added instant stop).

## 18. Unverified or missing items

| Item | Status |
|---|---|
| Numeric defaults/ranges for Time Step, Number of Simulation, CG Iteration, CG Residual, Self Collision Iteration, Air Damping | Not published on fetched pages |
| Exact preset-to-parameter mapping | Not published |
| Hard pin / Soft pin | No approved source found |
| Fix / Settle commands | No approved source found |
| Dedicated intersection resolution tool | Only global option found |
| Deactivate (Pattern and Sewing) semantics | Title only |
| GPU limits (VRAM, vendor, layers) | Only trim and soft body limited in 2024.2 |
| Soft body page | Not located |
| Pressure range in MD | Style3D shows -100 to 100 (secondary) |
| Seungwoo Oh to CLO link | Not confirmed |
| Morph/animation stability beyond Complete quality | Not covered |

## Sources

Fetched (primary, MD/CLO official manual):

- https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321/ (Simulation Properties)
- https://support.clo3d.com/hc/en-us/articles/30201166554649-Edit-Simulation-Properties
- https://support.marvelousdesigner.com/hc/en-us/articles/47358232630937 (Simulate)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358375599769-Mode
- https://support.marvelousdesigner.com/hc/en-us/articles/47358202260761--Mode-Animation
- https://support.marvelousdesigner.com/hc/en-us/articles/47358315135897-Freeze-Unfreeze
- https://support.marvelousdesigner.com/hc/en-us/articles/47358262829849-Deactivate-Pattern-Only-Activate
- https://support.marvelousdesigner.com/hc/en-us/articles/47358400540697-Deactivate-Activate-Avatar
- https://support.marvelousdesigner.com/hc/en-us/articles/47358253657497-Strengthen-Unstrengthen
- https://support.marvelousdesigner.com/hc/en-us/articles/47358310357529-Solidify
- https://support.marvelousdesigner.com/hc/en-us/articles/47358146159769 (Pins)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358422800537-Create-Pin
- https://support.marvelousdesigner.com/hc/en-us/articles/47358163705369 (Quick Pinching)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358168697497-Select-Move-3D
- https://support.marvelousdesigner.com/hc/en-us/articles/47358288780441 (Layer)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358430412441-FABRIC-PHYSICAL-PROPERTIES-Adjust-Collision-Thickness
- https://support.marvelousdesigner.com/hc/en-us/articles/47358324821785-Shrink-Pattern
- https://support.marvelousdesigner.com/hc/en-us/articles/47358436596633 (Steam)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358383927065-Wind-Controller-Setting
- https://support.marvelousdesigner.com/hc/en-us/articles/47358369726233 (Wind effect)
- https://support.marvelousdesigner.com/hc/en-us/articles/48955485213209 (New in MD 2025.0, Trim Simulation)
- https://support.marvelousdesigner.com/hc/en-us/articles/59781111736345 (New in 2026.1, Brush Pinching)
- https://support.clo3d.com/hc/en-us/articles/115000414447 (Particle Distance)
- https://support.clo3d.com/hc/en-us/articles/115012381348-Pressure
- https://support.clo3d.com/hc/en-us/articles/115000430668 (Hi-Res / Low-Res Garment)
- https://www.marvelousdesigner.com/support/news/view/3e260cf8818a4ebf8d0639f1c610ab79 (2024.2 news; body missing, no simulation details extracted)

Fetched (secondary, independent):

- https://80.lv/articles/marvelous-designer-2024-2-now-available (GPU collision parity, trim collision)
- https://help.style3d.com/studio/en/c74f/3365/1c36/f135 (Style3D competitor, labelled secondary only)
- https://support.clo3d.com/hc/ko/community/posts/360002664467-6-Things-to-Know-About-Particle-Distance?page=1 (community-hosted post on official CLO domain, used only for PD facts that the manual also supports, except the 100 mm pass-through note which is single source)

Fetched (academic and archival):

- https://koasas.kaist.ac.kr/handle/10203/90854 (Low damped cloth simulation abstract)
- https://ixdf.org/literature/author/seungwoo-oh (author publication listing)
- https://diglib.eg.org/handle/10.1111/v32i7pp031-039 (Constrainable Multigrid for Cloth)
- https://history.siggraph.org/?p=182606 (CLO Virtual Fashion SIGGRAPH archive entry)
- https://patents.justia.com/assignee/clo-virtual-fashion (CLO patent list)

Attempted but unusable: cgchannel.com 2024.2 article (robots disallowed); koasas.kaist.ac.kr/handle/10203/20327 and zendy.io for the multigrid abstract (no content returned).
