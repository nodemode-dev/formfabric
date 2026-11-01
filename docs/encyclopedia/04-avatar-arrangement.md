# Marvelous Designer Encyclopedia: Avatars, Poses and Arrangement

Slice 04 of the formfabric research set. Scope: avatar library, avatar import, Avatar Editor and sizing, avatar properties and collision behaviour, joints and skinning, display modes, poses and motions, arrangement points and bounding volumes, pattern arrangement, reset 2D arrangement, accessories (hair, shoes), tape and tack, morph targets and blend shapes, fitting suit, auto fitting, AI-adjacent conversion features.

Sourcing: all facts come from the official Marvelous Designer Help Center (support.marvelousdesigner.com) and CLO's official CONNECT support site (support-connect.clo-set.com). Wikipedia, forums, Reddit and similar were not used. Where only one official page backs a claim it is marked [single source]. Note that both sites are the same vendor, so "cross-check" below means two distinct official articles agreeing, not an independent third party. Anything that could not be verified is listed in the "Gaps and unverified" section.

Version tags in the manual (ver 2.3.0, 4.2.0, 2024.x, 2025.x, 2026.x) are quoted as the manual states them; they tell a clone which features are legacy versus recent.

---

## 1. Summary for clone builders

1. The avatar is NOT a generic collider. It is a typed object: AVT container (mesh + skeleton rig + size data + pose + motion + accessories + bounding volumes + arrangement points). A clone needs a container format that bundles all of these, plus separate sidecar formats (.avs size, .pos pose, .mtn motion, .arr arrangement points, .pan bounding volumes, .mea measures, .iks IK mapping, .zacs accessories).
2. Garment-avatar contact is governed by a small set of per-avatar surface parameters: Skin Offset (default 3 mm, valid input 0 to 100), Static Friction, Kinetic Friction, optional Soft Body (stiffness 5000 to 10000), mesh subdivision (Divide Mesh 1 to 3). Mesh-vs-mesh collision with a shell offset is the model: the manual explicitly says avatar and garment are both meshes and low offset plus high garment particle distance causes the body to poke through.
3. Arrangement points are a parametric surface atlas over bounding volumes (capsule-like cylinders or cuboids between two joints). Each point is (BV, X, Y, Offset, Wrap Direction). Patterns are placed by clicking a point; the pattern is bent around the BV (spiral wrap if longer than the BV circumference). This is a well-defined feature that is cheap to clone.
4. Two distinct "pose change" mechanisms exist: joint-driven pose files (.pos) with a transition animation, and mesh-morph targets (same topology, Transition Frame Count). Both run simulation during the change so the garment follows.
5. Auto Fitting / Re-Target Draping depends on a Fitting Suit (circumference lines on the avatar, a tight suit pattern draped on it). That is the mechanism by which garments retarget between avatar sizes.

---

## 2. Avatar types, default avatars and library

### 2.1 Avatar formats (the container)

| Extension | Content | Notes |
|---|---|---|
| .avt | Native avatar: model, skeleton rig, sizing, pose, motion, accessories, bounding volumes, arrangement points | Primary format. Source: Avatar Editor article. |
| .avs | Size data only, no model | Must be loaded on the matching avatar; cannot be mixed across avatar versions, ages or genders. Folder of .avs can be dragged onto avatar. |
| .pos | Pose | Joint transform set; also carries hand pose |
| .hpos | Hand pose only (2024.1) | Only works with Female_V2 and Male_V2 avatars [single source] |
| .mtn | Motion (animation) | Produced by Register Pose/Motion |
| .arr | Arrangement points | Can be opened onto an OBJ avatar; "some readjustment may be needed on other avatars" |
| .pan | Bounding volumes | Open/Save in Avatar Editor |
| .mea | Avatar tape measures | Same model and version only |
| .iks | IK joint mapping | Open/Save in IK Joint tab |
| .zacs | Registered accessory (hair, shoes, glasses, earring, hat) | See section 12 |

### 2.2 Default avatars

| Item | Detail | Source |
|---|---|---|
| Downloadable default avatars (FBX) | Male_A (3 MB), Female_A (3 MB), Kid_A (4 MB), Kid_B (4 MB), Kid_C (3 MB) | Default Avatar Download Link article. No baby avatar is listed there. |
| Library default avatars referenced elsewhere | Female_V2, Male_V2, FV2.1_Mia (female), MV2.1_Luka (male), KV1.1G_Melody_(7-20) (girl), KV1.1B_Oliver_(7-20) (boy) | Auto Convert to Avatar and Pose articles |
| Avatar categories in Avatar Editor | Female, Male, Kid, Baby | Shape option table in Avatar Editor article |
| MetaHuman body types | "18 Default MetaHuman Body Types Supported (ver 2024.0.173)" | Article title in Avatar section; body not fetched [single source, title only] |

Library model: avatars are loaded from the Library window or via File > Open/Add > Avatar. Version note: only avatars from "ver. 10 V2 and above" can use the Size Editor.

### 2.3 Avatar right-click menu (3D window, Select/Move tool)

| Item | Effect |
|---|---|
| Select All Faces | Selects every face of the avatar bound to the selected joint's root (4.2.0+) |
| Delete Avatar / Delete All Avatars | Remove one or all |
| Delete Scene and Prop / Delete All Scene and Props | Remove trims/props (2024.0) |
| Move Avatar To Center, To Ground | Per avatar |
| Move All Avatars and Garments To Center, To Ground | Scene-wide |
| Reset Joints | Returns joints to default transforms (2.3.0) |
| Hide Joint / Show All Joints | Joint visibility (2024.2) |
| Change Heels: Up / Down | Presets for heel joint rotation (MD10) |
| Activate / Deactivate Avatar | Deactivated avatar is ignored by garments during simulation (2.3.0). Clone implication: per-avatar collider enable flag. |
| Hide Avatar / Hide All Avatars | Display only |
| Refresh Textures | Reload textures |
| Export to USD Layer | Avatar to USD authoring layer, or new USD with selected trims |

---

## 3. Importing avatars

### 3.1 Supported formats

| Case | Formats | Source |
|---|---|---|
| Avatar without joints | OBJ, FBX, Alembic, OpenCOLLADA | "Can I import my own avatar" |
| Avatar with joints / animation cache | FBX or Alembic (recommended) | Same |
| CLO CONNECT creation guidance | OBJ, FBX, GLB, GLTF; FBX recommended for avatars with joints | CONNECT "Creating Avatar" |
| MetaHuman | DNA (Body + Head .dna files) | MetaHuman DNA Importer |
| Auto-convert inputs | OBJ, AVT, FBX, FBX DAZ Genesis | Auto Convert to Avatar |

The two official sources disagree slightly on the list (the import guidance lists Alembic and OpenCOLLADA; CONNECT adds GLB/GLTF). A clone should accept OBJ, FBX, glTF/GLB, Alembic, Collada, and later USD (export to USD layer is documented).

### 3.2 Open vs Add

| Mode | Behaviour |
|---|---|
| Open (File > Open > Avatar, or Import > FBX with Load Type Open) | Removes all existing objects and leaves only the imported file. Replaces current avatar. |
| Add (File > Add > Avatar, or Import (Add)) | Keeps existing objects and adds the file alongside them. This is how multiple avatars are put in one scene. |

Sources: FBX import article (Open/Add definitions), OBJ Import/Export article ("Import (Add) brings a file in alongside existing content"), Morph Target with AVT Files (File > Open/Add > Avatar).

### 3.3 OBJ import options (File > Import > OBJ)

| Option | Effect | Version |
|---|---|---|
| Load as Avatar | Becomes the collision avatar, replaces original | |
| Load as Trim | Prop/scene. Garments pass through it: collision detection does not work on Trims by default | 3.0.0 |
| Load as Garment | Imports as 3D garment; "Trace 2D Patterns from UV Map" optional | 4.2.0 |
| Load as Morph Target | Morphs the existing object into a second object that has the same mesh structure. Transition Frame Count sets speed: higher is slower | |
| Scale | Free numeric value or unit from dropdown | |
| Axis Conversion | Re-maps each axis direction (used for Y-up/Z-up conversion) | |
| Align Bottom to Ground | Lifts an object below the ground plane | 4.2.0 |
| Add Arrangement Points and BVs | For Load as Avatar: auto-generates bounding volumes and arrangement points that fit the model | 4.2.0 |

Two pages agree on this list (OBJ Import/Export, two article IDs).

### 3.4 FBX import options (File > Import > FBX)

| Option | Effect |
|---|---|
| Load Type | Open or Add |
| Object Type | Avatar (multi-select allowed), Trim (no collision by default; enable collision in Property Editor; Glue icon on gizmo attaches trim to clothing; Scale icon), Scene and Props (collision settings in property window) |
| Auto Create Fitting Suit | Humanoid avatars only |
| Automatically Add Arrangement Points | Humanoid avatars only |
| Camera | Import camera data / camera animation |
| Joint Animation | Import joint animation data |
| Cache Animation (MC) | Import cache animation data |
| Scale | Value or "Measure" |
| Axis Conversion | Axis direction (12.2) |
| Align Bottom to Ground | Translation option |
| Move Garment to Start Position | Moves garment to first frame of imported motion |
| Multiple avatars | One FBX with several avatars can be imported (12.2) |

Export caveat: only joints originally imported from FBX can be exported back to FBX. Export of joint animation writes keys at every frame for the motion duration (File > Export > FBX > Option > Joint Animation; Export All).

### 3.5 Scale units

| Context | Units | Source |
|---|---|---|
| Accessory registration Scale | Auto Scale, mm (default), cm (DAZ Studio), in, ft, m, 8ft (Poser), 8.6ft (Poser) | Register Accessory article |
| Authoring export recommendation | Y-up, millimetre units, transforms zero/one, FBX because "FBX stores unit information, while OBJ does not" | Register Accessory article |
| Avatar Editor display units | Millimeter, Centimeter, Inch, Feet and Inch | Avatar Editor article |
| OBJ/FBX avatar import scale | Free number or unit dropdown | OBJ and FBX articles; the exact dropdown list for avatar import was not shown on the fetched pages |

Clone implication: internal unit should be millimetres (garment particle distance, skin offset and bounding-volume radii are all in mm in the manual).

### 3.6 Authoring requirements from CLO CONNECT (official guidance)

| Requirement | Detail |
|---|---|
| Up axis | Y-up; normals outward |
| Mesh quality | Re-meshed editable geometry; scans only after re-meshing; avoid overlapping/coincident meshes (flicker) |
| Symmetry | Symmetric mesh and joints recommended |
| Polygon budget | Recommended maximum 70,000 triangles (affects speed) |
| Pose | A-pose or T-pose strongly recommended |
| Skeleton | Joint names and structure should match CLO/MD avatar to support Avatar IK |
| Skinning method | Dual Quaternion and Weight Blend skinning "are not compatible" (so linear blend skinning is the supported method) |
| Separate assets | Shoes, hair, earrings as separate assets, not inside the AVT |
| Textures | Enable Embed Media in FBX export |

---

## 4. Auto Convert to Avatar and related conversion (AI-adjacent)

Main menu > Avatar > Auto Convert to Avatar (2024.0+). This is a geometric fitting/retargeting feature; the manual does not describe it as machine learning, so it is not claimed as AI.

| Step / option | Values |
|---|---|
| Basic 3D Body File | OBJ, AVT, FBX, FBX DAZ Genesis (2024.1). Best with human-shaped A-pose mesh |
| Gender | Female, Male, Girl (7-20), Boy (7-20); auto-filled for DAZ in 2024.1 |
| Converting Option | Automatic (converts body shape), or Custom (Maintain current rig; optionally keep default MD hands and feet) |
| Converted Type | MD Skin Style (original shape, MD appearance, adds joints); Skin Style None; Rigging Only (original shape + appearance, adds joints only); Size Editable (default body shape plus joints so size can be edited) |
| Default skin styles | FV2.1_Mia, MV2.1_Luka, KV1.1G_Melody_(7-20), KV1.1B_Oliver_(7-20) |
| Output | AVT |

Other related items listed in the Avatar section: Auto Convert to Motion (2025.0, page not retrievable), MetaHuman DNA Importer, Blend Shape Avatar (2026.0). No separate "AI body scan to avatar" or "AI fit prediction" feature was found in the avatar/arrangement manual sections. Not verified beyond that.

---

## 5. Avatar Editor (Main Menu > Avatar > Avatar Editor)

Tabs (2025.0 manual): Avatar Size, Measure, Arrangement, Fitting Suit, IK Joint. A Blend Shape tab appears from 2026.0.

### 5.1 Avatar Size tab

| Control | Function |
|---|---|
| Size (Custom / Presets) | Preset or custom measurement set |
| Avatar Name (.avs) | Size file to load |
| Units | mm, cm, inch, ft and inch |
| Set Details (gear) | Activates hidden measurements for editing |
| Show all Measurements | Lists active and hidden measurements; non-enabled ones visible but locked |
| Width / Height | Key measurements; the rest are generated or adjusted to keep proportions |
| Link (default on) | Measurements proportionally linked based on real body scan data |
| Unlink | Key Width/Height edits no longer drive lower measurements |
| Display | Measurements drawn as dotted lines in 3D window; black = editable, grey = not editable |

Gender-specific shape sliders:

| Shape option | Female | Male | Kid | Baby |
|---|---|---|---|---|
| Hand | yes | yes | no | no |
| Head | yes | yes | yes | no |
| Crotch Width | yes | yes | no | no |
| Crotch Volume | yes | yes | no | no |
| Hip Dips | yes | yes | no | no |
| Hip Volume | yes | yes | no | no |
| Breast Shape / Space / Height | yes | no | no | no |

Numeric slider ranges are not stated in the manual pages fetched.

### 5.2 Size Editor (ver 10 and above)

| Item | Detail |
|---|---|
| Path | Main Menu > Avatar > Avatar Editor > Avatar Size |
| Requirement | Avatar from ver 10 V2 or later (.avt) |
| Total Body | Overall body type from height and chest width |
| Height variants | Total Height (ground to top of head, least accurate), HPS Height (ground to neck), Inseam (ground to crotch). Stated accuracy: HPS > Inseam > Total |
| Width | Overall body width |
| Details | Individual measurements; small errors shown in parentheses; the value shown on the avatar is the true one |
| Basic mode | Few measurements available; unassigned ones auto-filled from body scan data |
| Advanced mode | Many measurements available; unassigned fall back to preset defaults (use Basic if many are missing) |
| Show All / Show Editable | Toggle visibility |
| Save/Open | Measurements created in lower versions are incompatible |

Clone implication: a statistical body model (PCA over scan data, "body scan data") is implied by the Link behaviour: set key dimensions and the model predicts the rest. Mechanism not published.

### 5.3 Measure tab (avatar tape)

| Control | Function |
|---|---|
| Tape types | The page says six tape types cover free, circumference, linear and height measurements, but lists only: Basic (behaves like a soft tape) and Surface (follows body surface). The other types were not enumerated. |
| Name | Double-click to rename |
| Delete | Or select with Edit Measurement tool and press Backspace/Delete |
| Fit to Avatar | Refits measures after pose change or loading another avatar's joints |
| Open / Save | .mea |
| Measurement Type | Switch Basic or Surface without recreating |
| Measurement Information | Tape or Circumference |
| Basic Length / Surface Length | Both values displayed |
| Show / Show Length | Visibility toggles |

Separate from this tab, Avatar Tape and Tack on Avatar are garment-to-avatar attachments (section 10).

### 5.4 Blend Shape tab (ver 2026.0, 2026.1)

| Item | Detail |
|---|---|
| Purpose | Mix pre-made body types or expressions at set ratios; one base mesh changes size or expression in real time |
| Requirement | File must contain Blend Shape data and be loaded as Avatar type |
| Deformable Parameters | Creator-defined list grouped by mesh unit; Select Avatar drop-down when multiple blend-shape avatars loaded |
| Weight | 0 to 100 per part |
| Favorite / Show Favorites Only | Filter |
| Reset | Reverts unsaved changes |
| Show Selected Only | Only parameters tied to selected avatar part (2026.1) |
| Add Animation Keyframe | Saves parameter value as a keyframe at current frame (2026.1) |
| Animation I/O | Import/export keyframes as FBX, glTF/GLB, Alembic, USD (2026.1) |

Source: Blend Shape Avatar article only [single source].

### 5.5 IK Joint tab

| Item | Detail |
|---|---|
| Purpose | Map arbitrary rig joints to the MD joint set so poses and motions transfer |
| Required joints | Shoulders, elbows, wrists, pelvis, knees, ankles must all be mapped; names unique in FBX |
| Limitation | Quadrupeds and creatures may not map |
| Open / Save | .iks |
| Auto Mapping | Works if names match MD naming, or avatar comes from Daz 3D, Adobe Mixamo, Reallusion Character Creator, Unreal MetaHuman |
| Set T-Pose | Puts avatar in T-pose; adjust joints with gizmo so poses apply correctly |
| Reset Pose | Active after T-pose set |

---

## 6. Avatar properties and the avatar as collider

Location: 3D Toolbar > Select/Move > select avatar > Property Editor.

### 6.1 Property table (2025.x)

| Group | Property | Detail |
|---|---|---|
| Information | Name, Size | Size shows preset chosen in Avatar Editor |
| Body Styling (2025.0) | Skin Style | Presets or Customize via Style Configurator: hair, skin, underwear, eyebrows, eyes, lips colours |
| Eye Control (2025.0) | Horizontal, Vertical | Eye direction |
| Surface | Skin Offset | Default 3 mm; input accepted 0 to 100 (unit not stated on that page) |
| Surface | Static Friction (coefficient) | Applies when garment is stationary on avatar |
| Surface | Kinetic Friction (coefficient) | Applies when garment is moving |
| Surface | Soft Body Simulation (2024.0+) | Deformable body, skin elasticity. Indicator button in 3D window when on. |
| Surface | Soft Body Stiffness | 5,000 to 10,000; lower is smoother |
| Surface | Divide Mesh (2025.0) | Subdivides avatar mesh, 1 to 3; denser slows sim |
| Material (selected) | Material | Per avatar part |
| Miscellaneous | Mesh Type, Element Count | Read-only info |
| Scale | Scale values | Typed in Property window or via scale gizmo (OBJ avatar from ver 10, FBX avatar from 2024.0) |

2024.2 variant: Smooth toggle with Division Level 1 to 5. The separate "Avatar Smooth and Division (4.2.0)" article says Division Level 1 to 3. The two official pages differ (1 to 5 in the 2024.2 property table versus 1 to 3 in the 4.2.0 article); the current 2025 property says Divide Mesh 1 to 3. Use 1 to 3.

Friction value defaults and numeric ranges are NOT given in the manual.

### 6.2 Skin offset semantics

| Statement | Source |
|---|---|
| Distance between avatar body and garment | Skin Offset article |
| Default 3 mm "suits garment production" | Skin Offset article, Avatar Properties |
| Avatars and garments are both mesh; low offset with high Particle Distance lets the body show through | Skin Offset article |
| Lowering Skin Offset should be paired with lowering garment Particle Distance (prevents penetration, slows sim) | Skin Offset article |
| Workflow: leave default while building; lower both for fit check or final render | Skin Offset article |

Collider model implied for a clone: triangle-mesh collider with a signed-distance shell at skin_offset (3 mm default), plus Coulomb friction with distinct static and kinetic coefficients per avatar, and garment-to-garment friction separate (the friction article notes garment-garment friction can steer draping direction).

### 6.3 Soft body

| Item | Detail |
|---|---|
| Version | 2024.0 |
| Recording | Animations with soft body on are recorded separately and exportable to Alembic cache; other formats export only the current frame |
| Non-rigged avatar-type objects | Simulatable from 2025.1 |
| GPU | NVIDIA GPU support from 2025.2 |

### 6.4 Avatar materials

Diffuse Color, Diffuse Map, Ambient Color/Map, Specular Color/Map, Shininess, Emission Color, Opacity. Saved through File > Save As > Avatar (2.5.0). These are render-only; no physical parameters in materials.

### 6.5 Trims versus avatars (collision difference)

| Object type | Collides with garment? |
|---|---|
| Avatar | Yes |
| Trim (OBJ/FBX) | No by default ("garments pass through"); collision can be enabled in Property Editor |
| Scene and Props (FBX) | Collision settings in property window |
| Deactivated avatar | Ignored by garments |

---

## 7. Joints, skinning, display modes

### 7.1 Display options

| Control | Path / shortcut | Effect |
|---|---|---|
| Show X-Ray Joints / Show Avatar Joints | Main Menu > Display > Avatar; 3D window Avatar Display hover; Shift+X (version 12) | Draws joint arrangement. Purple pelvis point at centre. |
| Joint display size | Pop-up box when Show Avatar Joints is on (2024.2); reset button top right | Numeric size |
| Child group display | Selecting a joint shows its child group (2.3.0) | |
| Show Arrangement Points | Display > Avatar | Draws points on the bounding volumes |
| Hide Joint / Show All Joints | Right-click joint menu (2024.2) | Per-joint visibility |
| X-ray in the Bounding Volume editor | Joint Name 0 shown as X-ray, Joint Name 1 shown as avatar joint | |

### 7.2 Using joints

| Operation | Detail |
|---|---|
| Adjust Avatar Location | With X-ray joints on, select purple pelvis point and drag or use gizmo |
| Adjust Avatar Pose | Move/rotate joints with the gizmo; undo supported; page itself was not retrievable (details [unverified]) |
| Reset Joints | Right-click menu |
| Change Heels Up/Down | Presets on heel joints |
| Pose by keyframes | After DNA import: show avatar joint and set keyframes on joints |

### 7.3 Skinning and skin weights

| Fact | Source |
|---|---|
| Rigged avatars use joints; FBX is the recommended format | Import and CONNECT |
| Dual Quaternion and Weight Blend skinning are not compatible | CONNECT Creating Avatar |
| Skin weights specifics (editing, painting, limits) are not documented in the fetched pages. A search for MD skin-weight editing returned no official article. No in-app weight painting was found. | Gap |

Clone implication: import skin weights from FBX/glTF, evaluate linear blend skinning, and expose a joint gizmo. In-app weight editing is not evidenced in MD.

---

## 8. Pose library, poses and motions

### 8.1 Open / save pose

| Item | Detail |
|---|---|
| Open Pose | File > Open > Pose (Ctrl+Shift+P). Turns Simulation on automatically; avatar transitions from current pose to new pose; garments simulate through the change |
| Pause/resume | Click Play Motion or Simulation tool to pause; again to resume |
| Save Pose | File > Save As > Pose creates .pos |
| Undo/Redo | Ctrl+Z, Ctrl+Y (5.0.0) |
| Library | Poses and hand poses (.hpos) are downloadable from the Library (hand poses 2024.1) |
| Play Motion | Main Menu > Avatar > Play Motion or 2D toolbar. Pose files auto-activate Play Motion; motion files (.mtn) need the tool selected. Transition Animation of v10 default avatars is not compatible with earlier versions. |

### 8.2 Open Pose dialog (2024.1)

| Option | Values / meaning |
|---|---|
| Load pose type | Avatar Size + (applies size, pose, motion data, joint positions) or Joint Rotation Only (applies joint rotation, keeps avatar size). The article's labels (Pose Only / Pose and Joint Translation) and descriptions are inconsistent; implement as two modes: copy joint translations too, or rotations only. |
| Partial Pose | Full Body, Both Hands, Left Hand, Right Hand |
| Maintain Current Shoe Pose | Keep shoe/heel pose |
| Align Bottom to Ground | Re-ground after pose |
| Reset / Apply / Cancel | Dialog buttons |

### 8.3 Register Pose/Motion (Main Menu > Avatar > Register Pose/Motion)

| Item | Detail |
|---|---|
| Input | FBX with animation |
| Output | .pos (pose) or .mtn (motion) |
| Warning | All avatars and garments in the 3D window are deleted |
| Gender target | Female, Male, Kid |
| Shoe type (Female only) | Heels or Flat (affects heel joint rotation) |
| Motion region | Entire, Play Only, Current Frame |

### 8.4 Other pose-change methods

| Method | How it works | Source |
|---|---|---|
| Change Pose by Collada joint information | Export T-pose and final pose avatars as Collada; save the final pose as .pos from the posed file; dress the T-pose avatar; Open Pose to transition; then Import OBJ of the final pose to verify the garment is under the surface; drag garment parts out of avatar while simulating. Stable even for large pose differences because joints move continuously. | Collada pose article |
| Change Pose by OBJ Morphing | Mesh-morph between two same-topology OBJ avatars | Page not retrievable; behaviour inferred from OBJ import article (Load as Morph Target) |
| Morph Target with AVT | File > Open or Add > Avatar with Load Type "Load as Morph Target"; the first avatar adopts the second's pose or shape; fails with a warning if incompatible | Morph Target with AVT Files |
| Blend Shape | See 5.4 | |

---

## 9. Morph during simulation and timed morph

| Feature | Detail |
|---|---|
| OBJ Load as Morph Target | Target must have same mesh structure; Transition Frame Count sets duration; higher means slower morph. Garment simulates during the transition. This is the "timed morph". |
| AVT Morph Target | Same Load Type but for AVT; no frame count documented in the article |
| Pose transition | Transition Animation for pose files; frame rate in the animation dialog (5 FPS suggested for heavy garments to slow avatar motion and reduce collisions) |
| Animation Mode tips | Increase transition time for heavy garments; particle distance 20 mm for hidden inner layers, 10 mm outer, 5 mm where fine wrinkles matter (7 mm if many 5 mm layers); first recording at Complete quality; Normal allowed for mid-section only; return to Complete at the end of transition; attach garment with Avatar Tape or Tack on Avatar to avoid swinging |
| Joint animation import | FBX import with Joint Animation checked; garment created at a chosen frame; simulate; export FBX with Export All; joint animation exported keyed at every frame |
| Blend shape keyframes | 2026.1: Add Animation Keyframe per parameter |

The animation keyframe/layer editing articles were not fetched, so keyframe curve interpolation is [unverified].

---

## 10. Attaching garment to avatar: Tack, Pin, Avatar Tape

| Feature | Path | Operation |
|---|---|---|
| Tack on Avatar | Main Menu > Sewing > Tack > Tack on Avatar; 3D toolbar | Click a point on the 3D garment (garment goes semi-transparent, dotted line follows cursor), then click a point on the avatar. Same tack is created on the 2D pattern. Simulate keeps both points fixed. |
| Line Tack on Avatar | Search result title only | Line variant; details not fetched |
| Attach Pin to / Detach Pin from Avatar | Search result title only | Details not fetched |
| Avatar Tape | Referenced in Animation Mode article | Used to attach shoulders/sides; details not fetched |

Clone implication: tacks are point constraints between a garment particle and a barycentric point on the avatar surface (follows skinning).

---

## 11. Bounding volumes and arrangement points

### 11.1 Concept

Bounding Volumes (BV) are shapes (cylinder or cuboid) stretched between two joints. Arrangement Points lie on BV surfaces. Patterns click-placed on a point are bent around the BV. The default MD set is required for Auto Sewing: "Only Marvelous Designer's default bounding volumes work. Custom bounding volumes and arrangement points won't work" with Auto Sewing (Auto Sewing page, [single source]).

### 11.2 Bounding Volume parameters (Avatar Editor > Arrangement)

| Parameter | Meaning |
|---|---|
| Name | Double-click to rename; new volume is named "PAN" (or "Pan") |
| Joint Name 0 | Start joint (displayed as X-ray joint) |
| Joint Name 1 | End joint (displayed as avatar joint). If the order is reversed the left and right sides flip on arrangement. |
| Shape (MD10) | Cylinder or Cuboid |
| Height | Volume height |
| Length X / Radius X (mm) | Horizontal radius, adjusts circumference left/right |
| Length Y / Radius Y (mm) | Vertical radius, adjusts circumference top/bottom |
| Round Corner (MD10) | Corner radius; Cuboid only |
| Hands/Feet | Extends BV length to hands or feet; toggling refreshes the whole volume |
| Add / Delete | List management |
| Reset Bounding Volumes (4.2.0) | Back to original positions |
| Fit to Avatar (All) | Refits BVs to the current avatar joints after pose change or loading another avatar's joints |
| Open / Save | .pan |

The Avatar Editor article labels radius fields "Radius X (mm)" and "Radius Y (mm)"; the BV article names them Length X and Length Y. Both describe the same X/Y extents (two official articles).

### 11.3 Arrangement point list (Avatar Editor > Arrangement)

| Control | Function |
|---|---|
| Name | Rename. Suffix _L and _R (for example NAME_1_L, NAME_1_R) keeps pairs symmetrical. All points except those on the avatar body are symmetrical by default. |
| Add | Creates an unassigned point named "Arrangement Point"; must then set BV and X/Y |
| Delete | Remove |
| Open / Save | .arr; can apply to an OBJ avatar, may need readjusting on others |
| Arrangement BV | Which BV holds the point |
| X | Left/right on BV |
| Y | Up/down on BV |
| Offset | Distance from avatar surface |
| Wrap Direction | Up or down |

Binding: a point is bound to one BV, a BV is bound to two joints, so points follow joint motion (skeleton-bound). This chain is the whole binding model.

### 11.4 Display and generation

| Item | Detail |
|---|---|
| Show | Display > Avatar > Show Arrangement Points (points are generated from the BV and follow its curves) |
| Auto generation | "Automatically Add Arrangement Points" on FBX/DNA import (humanoid only), "Add Arrangement Points and BVs" on OBJ avatar import (4.2.0), or auto from MD default avatars |
| Manual | Avatar Editor > Arrangement |
| Multi-avatar | Points belong to each avatar [partial evidence from a CLO support thread title; not fetched, [single source]] |

---

## 12. Arranging patterns onto points

### 12.1 Operation

| Step | Detail |
|---|---|
| 1 | Enable Show Arrangement Points |
| 2 | Select a pattern in 2D or 3D; hover a point for a placement preview (2.4.0) |
| 3 | Click a point: the pattern centre is placed at that location |
| 4 | If pattern and points both have symmetry, placement is mirrored (3.0.0) |
| 5 | Fine adjust with Gizmo or Property Editor |

### 12.2 Wrap behaviour

| Item | Detail |
|---|---|
| Spiral wrap | Patterns longer than the BV circumference (cuffs) are arranged in a spiral |
| Flip Wrap Direction | Right-click pattern in 3D window to reverse the spiral |
| Wrap Direction (per point) | Up or Down set in the arrangement point |

### 12.3 Arrangement property (Property Editor > Arrangement)

Visible only for patterns placed with arrangement points. Path: Select/Move > select 3D pattern > Property Editor > Arrangement.

| Property | Effect |
|---|---|
| Point Name | Read-only name of the underlying arrangement point |
| Shape Style | Toggles the BV between flat and curved |
| Position X | Horizontal across the BV |
| Position Y | Vertical across the BV |
| Offset | Gap and curvature between BV and pattern |
| Orientation | Rotates pattern direction relative to BV |

Numeric ranges are not given.

### 12.4 Auto Sewing relies on arrangement

Use points correctly: collar and band use neck points; body uses body points; sleeves use arm points (cuffs arm or wrist); bottoms use leg points (wide pants: front and back pieces each on their own point). Auto Sewing path: Main Menu > Sewing > Auto Sewing. Garment type Top, Pants, Skirt; front type Openable, Partially Openable, Closed; for Closed tops the collar seam Auto, Center Front, or Back Left. Existing sewing is kept. Sewing lines need segment points. [single source]

### 12.5 Reset 2D Arrangement and related

| Item | Detail |
|---|---|
| Reset 2D Arrangement | Spreads and arranges all or selected 3D patterns identical to the 2D window state. For selected: select patterns (Shift for multiple), right-click in Select/Move, choose reset option. For all: 3D Garment menu or 3D toolbar. |
| Related | Reset 3D Arrangement, Symmetrical 3D State (2024.0) |

---

## 13. Fitting Suit, Auto Fitting, Re-Target Draping

### 13.1 Fitting Suit (Avatar Editor tab)

| Control | Function |
|---|---|
| Draw circumference lines | At indicated positions on the avatar; the suit pattern shows live |
| Create Symmetric Tape | Toggle symmetric line creation |
| Auto Create Fitting Suit | Automatic; draw by hand if poor |
| Drape Fitting Suit | Drapes the suit on the avatar |
| Delete / Reset | Delete one line / delete all lines |
| Save | .avt |

### 13.2 Auto Fitting

| Item | Detail |
|---|---|
| Path | Main Menu > 2D Pattern > Fitting > Auto Fitting; 2D toolbar |
| Needs | Avatar with Fitting Suit completed; garment with Fitting Suit data (garment simulated on such an avatar, or on an avatar with Fitting Suit data, MD 9.5+) |
| Result | Garment size is automatically adjusted for the avatar and simulated on it |

### 13.3 Re-Target Draping / Re-Drape 3D Arrangement (2025.0)

Path: Main Menu > 3D Garment, or 3D toolbar. Keeps garment size, UVs and textures; re-simulates on the new avatar.

| Option | Setting |
|---|---|
| Maintain Pattern Curvature (%) | Higher: only pattern size changes, curvature is kept. Lower: both size and curvature change |
| Maintain Graphic Size | On: graphics keep size. Off: they scale with pattern |
| Maintain Texture Size | On or off, same logic |
| Maintain Topology | On: keeps topology, patterns must be fully synchronised (run simulation first) or they are remeshed; UVs may need readjustment. Off: remeshed automatically |

Speed tips: raise particle distance of the garment before running; reset garment properties to default.

---

## 14. Hair, shoes and accessories

Path: Main Menu > Avatar > Register Accessory (ver 2025.0). Output .zacs.

| Item | Detail |
|---|---|
| Types | Hair, Shoes, Glasses, Earrings, Hat |
| Mesh layer names required in source file | hair; shoes_r, shoes_l; glasses_r, glasses_l; earring_r, earring_l; hat |
| Rigging Type | Standard (default MD avatar types) or Rigged (joints matching avatar names merge) |
| Scale | Auto Scale, mm default, cm, in, ft, m, 8ft, 8.6ft |
| Axis Conversion | Y-up default; Invert checkbox |
| Mirror Creation | Shoes and earrings: auto-create mirrored object across avatar centre axis |
| Patterns | Materials to register |
| Thumbnail | Auto or custom |
| Modelling | Add support edges at sharp edges; use a transparent flat dummy mesh for complex contact areas (laces, heels), with smaller consistent spacing than the shoe mesh; heel needs its own material, ordered at the bottom |
| Export | Y-up, millimetres, transforms identity, FBX |
| Shoe height | Not documented in this article. Only Change Heels Up/Down presets (heel joint rotation) and "Shoe Type Heels or Flat" for pose registration were found. A numeric "shoe height" parameter was not located [unverified]. |
| Avatar shoe | Avatars carry shoes as accessories; Open Pose has "Maintain Current Shoe Pose" |

Hair is a collision mesh only; MD does not simulate hair strands per the pages found (no hair simulation described).

---

## 15. Mannequins, hangers

No official page for a hanger or mannequin feature was found in the avatar, arrangement or search results beyond generic avatar import. Mannequins in MD are ordinary avatars (OBJ/FBX) imported with Load as Avatar, and hangers are props imported as Trim or Scene and Props with collision enabled. This is inference, not a documented feature [unverified]. The Avatar Editor, Fitting Suit and Size tabs apply to MD-format humanoid avatars; a non-humanoid mannequin lacks Auto Sewing support because of the humanoid-only arrangement point generation.

---

## 16. How the simulation treats the avatar (collision summary)

| Aspect | Documented behaviour |
|---|---|
| Collider geometry | Avatar mesh (triangles). Subdivision (Divide Mesh 1 to 3) increases contact fidelity at CPU cost. |
| Contact thickness | Skin Offset (default 3 mm) added to garment-avatar distance |
| Friction | Static and Kinetic coefficients per avatar; garment-garment friction is separate |
| Penetration | Occurs when garment particle distance is large relative to Skin Offset; reduce both together |
| Moving collider | Joint driven (skinned) pose transitions, mesh morphs, imported cache animation (Alembic MC) and joint animation; garment follows via simulation, not baked |
| Soft body | Optional deformable skin, stiffness 5000 to 10000, GPU from 2025.2 |
| Disabled colliders | Deactivated avatars are ignored |
| Trims | Not colliders unless enabled |
| Quality | Animation recording quality Complete vs Normal differ clearly |
| Fixing garments | Tack, Avatar Tape and Pins attach to avatar surface |
| Penetration fix workflow | After a pose change, drag garment parts out from under the avatar with left mouse while simulation runs |

---

## 17. Clone requirements checklist (derived)

| Priority | Requirement | Notes |
|---|---|---|
| P0 | Avatar container with mesh, skeleton, skin weights, size, BVs, arrangement points | Mirrors AVT contents |
| P0 | OBJ/FBX/glTF/Alembic import with Open vs Add, scale, axis conversion, align bottom to ground, load as avatar/trim/morph target | Section 3 |
| P0 | Skin offset, static/kinetic friction per avatar; collision shell | Section 6 |
| P0 | Arrangement points on BVs with X, Y, Offset, Orientation, wrap, flip, symmetry | Sections 11, 12 |
| P1 | Pose files with transition animation and partial (hand) pose, joint rotation only mode | Section 8 |
| P1 | Morph target with transition frame count | Section 9 |
| P1 | Size editor with linked statistical body model and measure tapes (basic, surface) | Section 5 |
| P1 | Auto Create Fitting Suit, Auto Fitting, Re-Target Draping | Section 13 |
| P2 | Auto convert to avatar (rigging only, skin style) | Section 4 |
| P2 | IK mapping with auto-detect for Mixamo, Daz, MetaHuman, Character Creator naming | Section 5.5 |
| P2 | Accessory registration (.zacs) with naming conventions | Section 14 |
| P2 | Soft body avatar, subdivide avatar | Section 6 |
| P3 | Blend shape avatars with weight 0 to 100 and keyframes | Section 5.4 |
| P3 | MetaHuman DNA import | Section 3 |

---

## 18. Gaps and unverified

| Topic | Status |
|---|---|
| Numeric ranges for Avatar Size sliders and friction coefficients | Not stated in fetched pages |
| Full list of the six avatar tape types | Only Basic and Surface named |
| Skin weight editing in app | No official article located; likely absent |
| Adjust Avatar Pose article, Change Pose by OBJ Morphing, Avatar IK Joint Mapping 2025, Auto Convert to Motion, Simulation article | Pages returned fetch errors; content not read |
| Shoe height numeric parameter | Not found |
| Hanger/mannequin as a feature | Not found |
| AI body/fit features | None found in the avatar/arrangement manual sections; Auto Convert and Auto Fitting are described algorithmically |
| 18 MetaHuman body types, Avatar Tape, Line Tack, Pin to avatar | Only titles seen, bodies not fetched |
| Whether some version tags have shifted (Division Level 1 to 5 vs 1 to 3) | Conflicting official pages, noted in 6.1 |
| Auto Sewing restriction to default BVs | Single source |
| Blend Shape details | Single source |

---

## Sources

All fetched during this research (official only):

- https://support.marvelousdesigner.com/hc/en-us/articles/47358128221337 (Avatar Editor / Avatar Settings, ver 2025.0)
- https://support.marvelousdesigner.com/hc/en-us/sections/51986513612313-Avatar (Avatar section index, 29 articles)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358263976729-Avatar-Properties
- https://support.marvelousdesigner.com/hc/en-us/articles/47358339835929-Avatar-Skin-Offset-Setting
- https://support.marvelousdesigner.com/hc/en-us/articles/47358326952601-Avatar-Friction-Setting
- https://support.marvelousdesigner.com/hc/en-us/articles/47358298797849-Avatar-Smooth-and-Division-ver-4-2-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358328615705-Avatar-Materials
- https://support.marvelousdesigner.com/hc/en-us/articles/47358293035289--Property-Scale-Avatars
- https://support.marvelousdesigner.com/hc/en-us/articles/47358350508697 (Open/Save Pose)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358136418073-Avatar-Register-Pose-Motion
- https://support.marvelousdesigner.com/hc/en-us/articles/47358474847897-Play-Motion
- https://support.marvelousdesigner.com/hc/en-us/articles/47358445646489-Change-Pose-by-Collada-s-Joint-Information
- https://support.marvelousdesigner.com/hc/en-us/articles/47358431264537-Morph-Target-with-AVT-Files
- https://support.marvelousdesigner.com/hc/en-us/articles/55654243482649-Blend-Shape-Avatar
- https://support.marvelousdesigner.com/hc/en-us/articles/51752244831897-MetaHuman-DNA-Importer
- https://support.marvelousdesigner.com/hc/en-us/articles/47358210639897-Auto-Convert-to-Avatar-Ver-2024-0-and-above
- https://support.marvelousdesigner.com/hc/en-us/articles/47358210334745-Can-I-import-my-own-avatar
- https://support.marvelousdesigner.com/hc/en-us/articles/360037330751 (OBJ Import/Export)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358223288601 (OBJ Import/Export, second copy)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358232885017 (FBX import)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358220563737-FILE-Export-Joint-Keyframe-Animation-with-FBX
- https://support.marvelousdesigner.com/hc/en-us/articles/47358177625881-Default-Avatar-Download-Link
- https://support.marvelousdesigner.com/hc/en-us/articles/47358221206297--Size-Editor-Adjust-Avatar-Size-ver-10
- https://support.marvelousdesigner.com/hc/en-us/articles/47358347146649-Avatar-Joints-Right-Click-Menu
- https://support.marvelousdesigner.com/hc/en-us/articles/47358360352153-Avatar-Right-Click-Pop-Up-Menu
- https://support.marvelousdesigner.com/hc/en-us/articles/47358400540697-Deactivate-Activate-Avatar
- https://support.marvelousdesigner.com/hc/en-us/articles/47358402406425-Adjust-Avatar-Location
- https://support.marvelousdesigner.com/hc/en-us/articles/47358215209881 (Show X-Ray Joints)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358365447705-Add-Bounding-Voulme
- https://support.marvelousdesigner.com/hc/en-us/articles/47358360672025-Rearrange-Bounding-Volume-Fit-to-Avatar
- https://support.marvelousdesigner.com/hc/en-us/articles/47358262924185-Arrange-Pattern-with-Arrangement-Points-Flip-Wrap-Direction
- https://support.marvelousdesigner.com/hc/en-us/articles/47358278939417-Arrangement-Property-Setting
- https://support.marvelousdesigner.com/hc/en-us/articles/47358281785369-5-Arrangement
- https://support.marvelousdesigner.com/hc/en-us/articles/47358149073305 (Auto Sewing)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358384849177-Reset-2D-Arrangement
- https://support.marvelousdesigner.com/hc/en-us/articles/47358335130649 (Auto Fitting and Re-Target Draping)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358145246361 (Register Accessory)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358255571609-ANIMATION-MODE-Garment-Animation-Making
- https://support.marvelousdesigner.com/hc/en-us/articles/47358249329305-Tack-on-Avatar
- https://support-connect.clo-set.com/hc/en-us/articles/45304313038233-Creating-Avatar (CLO CONNECT)

Fetch failures (not used for content): Change-Pose-by-OBJ-Morphing, Avatar-IK-Joint-Mapping-2025-0, Auto-Convert-to-Motion, Adjust-Avatar-Pose, Simulation.
