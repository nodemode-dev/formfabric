# Marvelous Designer (MD): Animation Mode and Animated Cloth

Research slice 08 for the formfabric encyclopedia. Scope: Animation mode, Animation Editor, keyframes, recording and caches, motion import and conversion, wind/pin/tack animation, simulation quality for animation, and animated export (Alembic, MDD, PC2, MC, FBX, USD, glTF, OBJ sequence).

Sourcing: official MD support manual (support.marvelousdesigner.com), official release pages (marvelousdesigner.com). The legacy manual at manual.marvelousdesigner.com blocks fetching for one page (robots) and one page was fetched via search snippet only. No Wikipedia, forums or Reddit used. Claims are marked [single source] when only one official page backs them, and [NOT VERIFIED] when I could not read a page that probably covers the topic.

Conventions: "ver X" tags are MD versions as written in the official docs (legacy numbering 2.x to 12.x, then 2024.0, 2024.1, 2024.2, 2025.0, 2025.1, 2025.2, 2026.0, 2026.1). Latest version found: 2026.1.

---

## 1. Summary (what a clone must replicate)

1. MD animation is a RECORD-AND-CACHE model, not a live-solver timeline. The user sets up a garment in Simulation mode, switches to Animation mode, loads an avatar motion, presses Record, and the solver runs frame by frame while per-vertex positions of every garment mesh are stored as a "cache layer". Playback just scrubs this cache. Everything else (editing, export) operates on cached vertex positions.
2. Since ver 12 / 2025.0 the Animation Editor is a layered timeline: cache layers (recorded garment motion, split into Pattern and Trim types, plus a Softbody cache), and keyframe layers (object, then property layers) for Fabric, Pattern, Avatar, Pin, Tack (2025.2), Wind Controller and Simulation Property. Keyframes use Constant, Linear or Cubic interpolation. Cache layers can be cut, merged, moved, copied, keyframe-deleted and re-keyed (the "fix a bad frame by gizmo then press S" workflow).
3. Motion sources: MD's own MTN joint-motion files (only for MD default-skeleton avatars), FBX with joint animation (Auto Convert to Motion turns any FBX, e.g. Mixamo, into MTN), vertex-cache formats (MC, MCX, PC2, MDD standard and Maya/3ds Max flavors), Alembic, USD (SkelAnimation), glTF. A "Transition Animation" blends from T-pose to the first motion pose.
4. Quality controls are three presets (Animation (Stable), Normal (default), Custom) plus a Simulation Quality value (default 1), a Time Step and substeps (called "Simulations Per Frame", renamed "Number of Simulation" in 2025.0), and a Frame Rate in the animation settings. Official advice: record finals at "Complete" quality (the older name for the high setting), use a low FPS (5) during transition to stabilize.
5. Export of animated garments: Alembic (Ogawa and HDF5), MDD, PC2, Maya MC/MCX, FBX (with mc cache and joint animation), USD (garment mesh animation, avatar skeletal animation, avatar cache), glTF/GLB (2026.0), OBJ sequence. Weld/unweld, thin/thick, unified UV, scale, axis conversion and Entire vs Play region are common options.
6. Recent additions: Keyframe animation overhaul (2025.0), IK joint mapping and IK mode (2025.0), Auto Convert to Motion (2025.0), Clamp Range to Selected Layers (2025.1), Keyable Properties Expansion and Animation Layer Setup (2025.2), Soft Body GPU (2025.2, NVIDIA), Animation Timeline Marker (2026.0), Blend Shape keyframe animation (2026.1). No AI motion generation or motion library feature was found in the official documents read (see section 17).

---

## 2. Mode switching and UI entry points

| Item | Detail | Source |
|---|---|---|
| Mode selector | Top-right of main window; choose ANIMATION (other modes include Simulation). Animation window opens. | Mode: Animation article |
| Main menu | Animation > Record (recording entry); Animation > Animation Editor | Record Animation manual; Keyframe Animation article |
| Animation Editor access points | Main Menu > Animation > Animation Editor; Main Menu > Window > Animation Editor; Bottom Window Center icon "Open Animation Editor"; Side Panel title bar right-click dropdown > Animation Editor | Keyframe Animation (ver 2025.0) |
| Auto Convert entry | Auto Convert to Motion window switches the app to Animation mode automatically so the converted motion can be previewed | Auto Convert to Motion |
| Play Motion | Main Menu > Avatar > Play Motion, also a 2D Toolbar button (plays .pos pose and .mtn motion files) | Play Motion |

Reasonable clone design: a mode switch that changes which dock panels and toolbars are active, with the same 3D viewport shared across modes.

---

## 3. Animation Editor: layout, toolbar, controls

The Animation Editor is a bottom dock with an upper toolbar, a track/layer list at left, a timeline with playhead (called "Current Frame" or "playtime slider"), and footer controls (Animation Max Frame input, Fit Horizontal button).

### 3.1 Upper toolbar (ver 2025.0 article "Play Animation", dated 2025-10-31)

| Control | Function | Notes |
|---|---|---|
| Record | Records avatar motion together with garment simulation | Press again to stop |
| Start | Jump to beginning of play area | |
| One Frame Back | Step back one frame | |
| Start/Pause | Play the play area or pause | |
| One Frame Forward | Step forward one frame | |
| End | Jump to end of play area | |
| Repeat / Loop | Loop the play area | circular arrow icon |
| Play Speed | 1/4x, 1/2x, 1x, 2x, 4x, 8x | discrete options |
| Merge | Merge selected animation layers into one | two overlapping squares icon |
| Cut | Split layer at current timeline position | scissors icon |
| Delete | Delete layer or keyframe depending on selection | trash icon |
| Create Key | Create keyframe at current position | keyboard shortcut S |
| Simulation Quality / Type | Animation (Stable), Normal (Default), Custom | |
| Simulation Quality / Value | Default 1. Higher value = more detail in recording. No range published. | [single source] |
| Settings (gear) | Pop-up: Frame Rate, Simulation Quality, Play Type, Playback Start/End, Scene Timewarp, Unit | ranges not published |

Note: the keyframe article lists the "Create Keyframe" icon as "I" in one table and shortcut S elsewhere; treat S as the shortcut.

### 3.2 Timeline interactions

| Action | How | Source |
|---|---|---|
| Select a keyframe | LMB click (single); Shift + LMB drag (multiple) | Keyframe and Layer Editing |
| Adjust selection | Drag white handles on either side of selection | same |
| Copy / paste keyframes | Right-click selection > Copy Keyframe; move playhead; right-click > Paste Keyframe. Ctrl+C / Ctrl+V. Pastes start at the current position, in the same layer (cache) or onto layers of same data type (property layers). | same, Keyframe Animation |
| Create keyframe | S | same |
| Delete keyframe | Del | same |
| Fit Horizontal | Button next to Animation Max Frame input at bottom right; shows all layers start to end | same |
| Full layer view | Double-click the animation time slider (ver 2024.0) | same |
| Playback range | Right-click timeline: Set Start Frame, Set End Frame, Clamp Range to Active Frames, Clamp Range to Selected Layers (2025.1) | Keyframe Animation |
| Display range persistence | Display Start/End frame of Animation Editor is saved and restored on restart (12.2) | 12.x feature list |
| Timeline Marker | Animation Timeline Marker added in 2026.0 (details page could not be read) | 2026.0 feature list; [NOT VERIFIED] detail |

Playback range rules: Set Start Frame is disabled if the current frame is past the end frame; Set End Frame is disabled if before the start frame. Clamp Range to Active Frames fits to the first and last keyframes across all layers.

---

## 4. Layers and tracks

### 4.1 Layer taxonomy (Keyframe Animation ver 2025.0)

| Layer type | Content | Behavior |
|---|---|---|
| Object Layer | One per animatable object; shows Start-End keyframe range as a bar | Hover shows handles to change length; child keyframes stretch/shrink proportionally keeping relative spacing |
| Property Layer | Holds real keyframes for a single keyable property; can be grouped | A parent key represents its children; selecting it selects all child keys on that frame; move, copy, delete, interpolation changes apply to all children |
| Cache Layer | Recorded garment motion | Only recorded cache layers can be cut or merged. Types: simulation-recorded caches (split into Pattern layer and Trim layer) and Softbody Cache (avatar motion recorded after Softbody enabled) |

### 4.2 Animation layer operations (Animation Keyframe and Layer Editing, ver 12.1+)

| Operation | How |
|---|---|
| Select layer | Click title (turns lighter grey). Shift-click adds, Ctrl-click removes |
| Rename | Double-click name, or right-click > Rename. Multi-select renames together |
| Reorder | Drag name above or below another layer. "Animation data on a top layer completely blocks layers beneath it" (top layer wins, no blending) |
| Move horizontally by drag | Double-click the layer's interpolation space (solid pink by default); when highlighted red, drag left or right |
| Move by frames | Right-click highlighted layer > Move Layer Horizontally, enter frame count |
| Nudge one frame | Left or Right arrow with layer highlighted |
| Delete layer | Click layer name (turns grey), press Delete |
| Lock | Click leftmost yellow circle ON |
| Hide cached animation | Click center yellow circle OFF |
| Hide keyframe markers | Click rightmost yellow circle OFF |
| Cut | Select layer, move playhead to middle, press scissors; removed portion becomes a new layer |
| Merge | Shift-select 2 or more layers, press Merge; result merges into the top selected layer; layers can be moved first to adjust timing |

### 4.3 Keyframe edit recipes on cache layers (official)

1. Record, stop. Select the garment cache layer.
2. Select a cache keyframe (vertical lines) and press Delete: vertex positions then interpolate linearly across the gap.
3. Place playhead in the gap, optionally enable simulation, select pattern/garment, use the Gizmo to translate/rotate, then press S to create a keyframe. Scrub to verify.
4. For smoother transitions, delete the simulation cache keyframes immediately left and right of the new key.
5. Loop trick (Unreal Engine tips article): copy the first frame and paste it at the end of the animation in the keyframe editor to create a seamless loop. Also the stated use cases: fix isolated simulation issues, blend simulations, adjust timing, create looping cloth caches.

Clone implication: cache layer = array of frames of vertex arrays with a sparse "key" flag per frame; non-key frames between keys are linearly interpolated; the solver can be re-run from any frame to overwrite.

---

## 5. Keyframe animation system (ver 2025.0, expanded 2025.2)

### 5.1 Workflow

1. Open Animation Editor. Click Add Animation (top left) to see animatable objects; select one to create an empty layer. Or toolbar "Create All Object Layers" to create layers for all objects in the current world.
2. Expand layer, set a keyable property value, click Insert Key (same as the Insert Key button in the Property Editor).
3. Move Current Frame, change value, Insert Key again. Play to see interpolation.

### 5.2 Key creation modes

| Mode | Behavior |
|---|---|
| Manual | Change value, click Insert Keyframe; keys the current value at current frame |
| Auto Keying | Enable Auto Keying, set the first key manually; afterwards edits to that property create keys automatically at the current frame (first key must always be manual) |
| Batch | Add Keyframe for Selected Layers; Add Keyframe for All Layers; Create All Object Layers (toolbar) |

### 5.3 Interpolation

| Type | Behavior |
|---|---|
| Constant | Holds current value until just before next key |
| Linear | Straight line, constant rate |
| Cubic | Curved, smoother; visible with 3 or more keys |

Set by selecting keys, right-click, choose type. Changing a parent key's interpolation updates all child keys. No tangent/bezier handle editing or curve editor (graph editor) was described in the pages read.

### 5.4 Keyable object categories (properties lists not reproduced in the fetched text)

| Object | Notes |
|---|---|
| Fabric | Back or side materials set to "Use Same Material as Front" do not appear in the layer |
| Pattern | Patterns can transition between assigned fabrics; property values interpolated |
| Avatar | Joints editable for dynamic motion (joint keyframes); IK mode (2025.0) |
| Pin | Pin properties |
| Tack | Added 2025.2 |
| Wind Controller | Strength, etc., plus position/rotation (see section 9) |
| Simulation Property | Simulation properties keyed over time |
| Cache | Recorded caches (see 4.1) |
| Blend Shape | Added 2026.1 (see section 13) |

The exact property lists per category exist in the official article but were omitted by the fetch tool; a clone should treat "any numeric/vector/color/boolean property exposed in the property editor" as the design target. [NOT VERIFIED] per-property list.

Version trail: Keyframe Animation + UI overhaul 2025.0; Clamp Range to Selected Layers 2025.1; Keyable Properties Expansion and Animation Layer Setup 2025.2; Timeline Marker 2026.0; Blend Shape Animation 2026.1.

### 5.5 Avatar IK (2025.0)

"IK Joint Mapping" and "IK Mode for Avatar Joint" were introduced in 2025.0 (feature list). The dedicated article "Avatar IK Joint Mapping (2025.0)" could not be fetched. [NOT VERIFIED] mapping UI, chain definitions, solver type.

---

## 6. Recording simulation (the core animation workflow)

### 6.1 Official steps (Record Animation manual, legacy; consistent with newer pages)

1. Switch to Animation mode (upper-left Animation tool / top-right mode selector).
2. Set Simulation Quality to Complete (pop-up asks to switch Simulation Mode to Complete).
3. Load the motion file. Two blue bars appear on the timeline: the motion, and a shorter transition animation.
4. Click Record. Simulation turns on automatically and the motion bar turns red as it records.
5. Recording stops automatically when the animation ends; to stop early click Pause again.

From ver 2.5.0 the user can re-drape with the Simulation button without recording.

### 6.2 Official best practices (ANIMATION MODE: Garment Animation Making)

| Tip | Value |
|---|---|
| Lighten garment | Inner hidden layers particle distance 20 mm; outer layers 10 mm, or 5 mm where wrinkles need detail; if many layers at 5 mm use 7 mm |
| Delete invisible/unneeded layers | Yes |
| Fix garment to avatar | Avatar Tape or Tack on Avatar at shoulders and sides; Tack on overlapping layers or dangling parts |
| Slow transition for heavy garments | Increase transition animation time; entering 5 FPS in the animation dialog stabilizes sim and reduces collisions |
| Simulation quality | Complete for first recording; Normal allowed for middle section; Complete again at end of transition |
| Re-check draping | Return to Simulation mode after recording to verify stability; adjust with Select/Move tool, then return |
| Final | Record final animation at Complete (Complete and Normal differ visibly in realism) |

Also: enabling "Avatar-Garment Collision Detection (Vertex-Triangle)" in Simulation Properties (right-click 3D Garment window background > Set Simulation Properties) gives more accurate collisions. Motion data increases garment collision during recording, which is why Complete helps.

### 6.3 Pre-roll / start pose blending (Transition Animation)

| Aspect | Detail |
|---|---|
| What | A generated staged animation from the T-pose (or avatar's current pose) to the motion's first pose, created when loading an MTN or importing a cache, if the current pose differs from the first frame |
| Prompt | When loading a motion file, a dialog asks whether to create a transition; choose Yes when poses differ |
| Appearance | Shorter second blue bar next to the motion bar on the timeline |
| Duration tuning | Increase transition time for heavy garments (garment tips article); 5 FPS during transition suggested |
| Where available | Joint Motion open (MTN) as the "Transition Animation" option; Animation Import (cache) option "Transition Animation" |
| Compatibility | Transition Animation for default avatars of ver 10 is not compatible with earlier versions |
| Related option | "Garment Position: Move Avatar & Garment to Start Position" (ver 2.5.0) teleports avatar and garment to the first frame location |

No explicit separate "pre-roll frames" parameter was found. The transition bar serves as the pre-roll. Exact duration controls: [NOT VERIFIED].

### 6.4 Animation settings pop-up

Fields: Frame Rate, Simulation Quality, Play Type, Playback Start/End, Scene Timewarp, Unit. The official page gives names only, no ranges or defaults [single source, not verified numerically]. Observed hints: 5 FPS usable during transition; Alembic FPS was an export option below ver 12.2 and was removed in 12.2 (export follows scene rate).

### 6.5 Auto Cache

I found no official page describing a feature named "Auto Cache" in MD. Recording is explicit (Record button), and recording produces the cache layer. Treat as not existing. [NOT VERIFIED / likely absent]

---

## 7. Simulation settings relevant to animation

Source: Simulation Properties article, UI Changes (Ver 2025.0).

| Property | Meaning | Published value |
|---|---|---|
| Presets | Normal (default), Animation (Stable), Fitting (accurate fabric) | |
| Time Step | Simulation time per solver step; smaller = refined but slower | recommend default; example 0.03 |
| Simulations Per Frame / "Number of Simulation" (renamed from "Simulation Quality" in 2025.0) | Divides each time step into substeps; example 0.03 / 10 = 0.003 | no range published |
| Toolbar Simulation Quality value | In Animation Editor; default 1 | no range |
| CG Finish Condition | CG Iteration Count (fast, fixed iterations) or CG Residual (stop below error threshold) | no defaults |
| Self Collision Iteration Count | Number of cloth collision passes | default kept |
| Air Damping | Resistance from air; raise for floaty motion | no default published |
| Gravity | Default -9800 (mm/s^2 scale; units imply 9.8 m/s^2 downward); 0 for weightless | -9800 |
| Number of CPU in Use | Max 12, auto-detected | |
| Nonlinear Simulation | More accurate, slower; used by Fitting | |
| Collision toggles | Ground; Avatar-Cloth Collision Detection; Self-Collision (Avoidance Stiffness default 0.001000); Proximity Detection | |
| Collision Resolution | Intersection Resolution (normal or flipped normal); Layer Based Collision Detection | |
| Skin Offset (avatar) | Default 3 mm | |
| Collision thickness | Default 2.5 mm (UE tips raised to 5 mm to separate layers) | |
| Presets save/load | Open and Save buttons at top of Property Editor | |

Wind/gravity/damping can be keyed over time as "Simulation Property" keyframes (2025.0).

2025.0 also added "Animation with Trim Simulation" (trims simulate during animation); 2025.2 "Instant Simulation Stop"; 2026.0 "GPU-Accelerated Trim Simulation" and "Fabric-Aware Strain Maps"; 12.0 "Simulation Speed & Stability Improvements".

---

## 8. Avatar motion import and retargeting

### 8.1 Formats and entry points

| Format | Menu | Notes |
|---|---|---|
| MTN (MD joint motion) | File > Open > Joint Motion; save with File > Save As > Joint Motion | Only default MD-skeleton avatars or avatars imported as COLLADA with motion data (Open COLLADA); MTN cannot apply to imported USD avatars |
| POS (pose) | Avatar > Play Motion | Pose files; Play Motion tool auto-activates |
| FBX (joint animation) | File > Import > FBX, enable Joint Animation | Rigged FBX imported as Avatar; or convert via Auto Convert to Motion |
| Auto Convert to Motion (2025.0) | Avatar > Auto Convert to Motion | FBX from Mixamo or custom to MTN |
| Alembic (.abc) | File > Import > Alembic | Cache, animation and normals (ver 2.4.0) |
| Maya Cache .mc, .mcx | File > Import > Animation | |
| Point Cache 2 (PC2) | same | FPS field appears only for PC2 |
| MDD Standard; MDD Maya/3ds Max | same | |
| Cache (Maya/3ds Max) | same | listed; workflow not described |
| COLLADA | File > Import > COLLADA | Avatar plus motion |
| USD | File > Import > USD | SkelAnimation; if several loaded, uppermost plays |
| glTF/GLB (2026.0) | File > Import | Animation not in import settings; export only |
| BVH | No official page found | [NOT VERIFIED] BVH import not documented in pages read |

### 8.2 Auto Convert to Motion (ver 2025.0)

1. Avatar > Auto Convert to Motion; window opens and mode flips to Animation.
2. Select gender of target avatar, open Motion File (FBX), click Convert. Mixamo FBX or custom animation works.
3. Play in the Animation Editor to preview.
4. Save as MTN.

Rules: error if avatars already exist in 3D window, remove them first. Converted motion works with all MD avatars. To move avatar and garment to the first frame, load the saved MTN. Only parameters: Gender and Motion File. This is MD's retargeting step: an arbitrary FBX skeleton is mapped onto the MD default male/female skeleton and saved as MTN (mapping algorithm not documented). [single source]

### 8.3 Joint Motion open options (MTN)

| Option | Effect |
|---|---|
| Joint Information (Rotation and Translation) | Applies avatar height, pose, joint rotations and positions |
| Joint Rotation Only | Applies rotation only; keeps current avatar size (the retarget-friendly option) |
| Garment Position (Move Avatar and Garment to Start Position) | ver 2.5.0 |
| Transition Animation | T-pose to first motion pose |

### 8.4 FBX Import options (Import FBX window)

| Option | Values / effect |
|---|---|
| Load Type | Open (removes all existing objects) or Add |
| Auto Create Fitting Suit | humanoid avatars only |
| Object Type | Avatar (multi-select allowed), Trim (no collision with clothing unless Collision checked; Glue and Scale gizmo icons), Scene and Props (collision settings in Property window Collision menu) |
| Object Camera | import camera data and camera animation |
| Joint Animation | import joint animation |
| Cache Animation (mc) | import cache animation |
| Automatically Add Arrangement Points | humanoid only |
| Scale | numeric or Measure |
| Axis Conversion | ver 12.2 |
| Translation: Align Bottom to Ground | lifts below-ground objects |
| Option: Move Garment to Start Position | moves garment to first frame of imported motion |

Multiple avatars can be imported and exported at once (12.2). FBX SDK 2020 (ver 10).

### 8.5 Animation Import (cache) options

| Control | Detail |
|---|---|
| Object | apply to individual object or Select All Avatars |
| FPS | PC2 only |
| Scale | no published range |
| Axis Conversion | Y-up default |
| Move Avatar and Garment to Start Position | ver 2.5.0 |
| Transition Animation | gradual T-pose to first pose if poses differ |

### 8.6 Alembic import options

Load Type (Open or Add), Scale (value or preset), Axis Conversion (2024.0), Move Garment to Start Position, Move Avatar and Garment to Start Position.

### 8.7 Mixamo workflow (composed from the official pages above)

1. Download animated FBX from Mixamo (Mixamo itself not an MD page; the MD page names it as a valid source).
2. In MD: Avatar > Auto Convert to Motion, choose gender, convert, save MTN.
3. Load default avatar, Open Joint Motion, choose Joint Rotation Only and Transition Animation.
4. Dress garment in T-pose, switch to Animation, quality Complete, Record.

Alternative for non-default or custom rigs: import FBX as Avatar with Joint Animation checked (the rig is used directly, no retarget needed, but MTN is unusable with it). Register Pose/Motion for custom avatars exists as an article ("Avatar: Register Pose/Motion", not fetched). [NOT VERIFIED] details.

---

## 9. Wind controller animation

Sources: ANIMATION MODE: Wind Animation (posted 2025-05-29, labelled for MD 7+), Wind Controller Setting (Above ver 2024.0).

### 9.1 Parameters

| Parameter | Range / default | Meaning |
|---|---|---|
| Name | text | rename |
| Wind Activate | on/off | |
| Unlimited Bound | on by default | influences whole space from wind position and direction |
| Type | Spherical or Planar | Spherical blows outward from controller in all directions; Planar blows from set direction regardless of location |
| Strength | no range given; docs example 500 to 1000 | intensity |
| Decay | 0 to 100 | higher = weakens along an arc; 0 = straight and endless |
| Frequency (sec) | no range; 0 = steady | cycle seconds for strengthening and weakening |
| Turbulence | 0 to 100 | irregularity of intensity |
| Rotation Key Activate | flag | lets animation keys set rotation |
| Translation X, Y, Z | | each can be keyed per axis |

### 9.2 Keying workflow (legacy method, still valid)

1. Display > Environment > Show Wind Controller.
2. Select the controller, move Playhead to target frame FIRST (moving afterwards would move the controller along interpolated values).
3. Set position and rotation with Gizmo and enter property values.
4. Right-click the Playhead > Add key (menu appears only when wind controller selected).
5. Values interpolate gradually between keys. A last key with no following key holds its values to the end of the range.
6. Wind is applied when the animation is recorded (it is a solver input, not baked until Record).
7. Delete keys: select controller, right-click key > Delete Key; for ranges, put playhead at start, Shift-click the end, right-click > Delete Key.

In 2025.0+ wind is keyable via the generic property layer system (Add Animation > Wind Controller > Insert Key).

---

## 10. Pins, tacks, freeze, trims in animation

| Feature | What is documented | Gap |
|---|---|---|
| Pin | Create (click vertex, drag area, double-click outline segment), delete (W key), mirror (ver 2024.2), Attach Pin to Avatar (cannot select/move until detached). Pin is a keyable object in the 2025.0 keyframe system, so pin position can be animated and the garment pulled by it | Pin property list, stiffness ranges not read; [NOT VERIFIED] |
| Tack | Keyable from 2025.2. Tack on Avatar recommended in garment tips to keep layers from swinging | Tack properties not read |
| Avatar Tape | Garment-to-avatar attachment recommended for animation | |
| Freeze | Frozen patterns behave as solid objects and still collide with other garments (unlike deactivated patterns). Right-click Freeze/Unfreeze in 3D or 2D. Freeze color toggle (2024.1). | The pages read do not mention animation behavior (is frozen state keyable, does a frozen pattern follow avatar?). [NOT VERIFIED] |
| Trim animation | "Animation with Trim Simulation" 2025.0; Trims imported via FBX/USD as Trim object; cache layer type Trim | |
| Gravity animation | Simulation Property is keyable (2025.0); gravity default -9800 | exact keyable list not read |

Known user issue (title seen only): "Some parts of the garment don't follow the avatar animation" (community post; page failed to load). Not used as a source.

---

## 11. Soft body (avatar skin elasticity)

| Item | Detail |
|---|---|
| Introduced | ver 2024.0 |
| Purpose | Deformable-body simulation to express skin elasticity |
| Indicator | Button appears at bottom-right of 3D window while active |
| Non-rigged avatar support | 2025.1 ("Soft Body for Custom Avatars"; non-rigged Avatar-type objects) |
| GPU | 2025.2 NVIDIA GPU soft body simulation |
| Soft Body Stiffness | 5,000 to 10,000; smaller = smoother |
| Animation recording | Default Adult Avatar animations with Soft Body on are recorded separately (Softbody Cache layer) and can export to Alembic. Other export formats capture only the current frame |
| Related | Divide Mesh 1 to 3 (2025.0); Smooth Avatar Division Level 1 to 5 (2024.2) |

Clone note: softbody cache is a second cache layer type (avatar vertex positions), exported via Alembic (and USD avatar Cache, 2024.2; glTF avatar cache animation, 2026.0).

---

## 12. Exporting animated garments

### 12.1 Alembic (File > Export > Alembic (HDF5) / Alembic (OGAWA))

Alembic is supported since 2.4.0; Ogawa/HDF5 choice in ver 6.1.

| Option | Detail |
|---|---|
| File Type | HDF5 or Ogawa (6.1) |
| Object: Unweld | exports vertices of sewn parts as multiple objects; quad thickness from 2.3.0 |
| Object: Weld | merges sewn vertices to one point |
| Weld Selected Pattern | weld by sewing lines between selected patterns (2.3.0) |
| Thin / Thick | thickness excluded or exported |
| Merge Vertex by Proximity | merge across patterns (12.0) |
| Weld Turned Sewing Lines | 2026.0 |
| Unified UV Coordinates | fit all pattern textures to 0 to 1 UV |
| Include Avatar Shape | save avatar in file |
| FPS | below 12.2 only; removed in 12.2 |
| Basic (scale) | units or percent |
| Animation | Entire Region, Play Region Only, Current Frame (2024.0) |
| Color | Strain or Stress exported as vertex color |

Alembic with Maya/Unreal: Unreal tip article does not mention Alembic; USD route with "Include garment simulation data" is recommended for Chaos Cloth.

### 12.2 Vertex cache formats (File > Export > Animation)

Formats: Maya Cache .mc, Maya Cache .mcx, Point Cache 2, MDD Standard, MDD Maya/3ds Max. "Only garment cache data can be exported."

| Option | Detail |
|---|---|
| Object | Single or Multiple objects |
| Weld/UnWeld/Selected Pattern | as above |
| Thin/Thick | as above |
| Combined UV | Unified Texture (3.1.0) |
| Image File | path/name, suffix _0, _1 auto-added |
| Texture Size | 100 to 10000 px, fixed ratio |
| Fill Texture Seams | 0 to 15 mm |
| Scale, Axis Conversion | as import |
| Animation | Entire Area or Play Area Only |
| Save With Texture Files (.zip) | packages obj, mtl, textures |
| Save with Meta Data (.xml) | pattern mesh, physical properties, sewing (2.3.0) |
| Diffuse Color Combined on Texture | 2.2.0 |
| Exclude Ambient Color (for Modo) | 2.2.0 |

### 12.3 OBJ sequence (File > Export > Animation > OBJ Sequence)

One OBJ per frame (1 s at 30 fps gives 30 files). Options: object selection dropdowns (2.2.0), Single/Multiple, Unweld/Weld, Thin/Thick, Combined UV (3.1.0), Texture Size 100 to 10,000 px, Fill Texture Seams 0 to 15 mm, Include Internal Shape (2.3.0), Scale, Axis Conversion, Entire or Play Area, Input Frame range (5.1.4), Save with Bone Information (JSON) for selected frames.

### 12.4 FBX (File > Export > FBX)

| Option | Detail |
|---|---|
| Object | Select an Avatar (6.1) / Select All Avatars (12.2) |
| Export Mode | Single or Multiple objects |
| Welding | Un-Weld, Weld, Weld Selected Pattern, Weld Turned Sewing Lines (2026.0) |
| Thickness | Thin / Thick (needs quadrangular mesh); per-side textures (5.0.0); Unified Texture (3.1.0) |
| Option: Joint Animation | Re-saves joint animation originally imported with the FBX (changed 2026.0); exported with a key at EVERY frame for the full motion length; only joints originally imported as FBX can be exported |
| Option: Cache Animation (MC) | Saves garment cache (recorded animation) as mc inside FBX |
| Scale, Axis Conversion | |
| Animation | Entire Region (all keys) or Play Region Only |
| File Type | Binary or ASCII |
| Texture paths | Relative (names only) or Absolute; Save With Texture Files (.zip) |
| Materials | material names exported (2025.0) |

Workflow for garment plus animated avatar: import rigged animated FBX as Avatar with Joint Animation, drape garment on a matching frame, Record, File > Export > FBX with Export All and Joint Animation, open in DCC and expect per-frame joint keys [single source for exact steps; matches option list].

12.0 added "Export Joint Keyframe Animation with FBX" and improved accuracy of exported joint transforms.

### 12.5 USD (File > Export > USD; ver 12.1 base)

| Option | Detail |
|---|---|
| Single/Multiple Object | 2024.0 |
| Thin/Thick, Unified UV, Image Size, Fill Texture Seams | |
| Include Garment Simulation Data | for solver set-up in engines (Chaos Cloth) |
| Garment Animation | 2024.1; pattern cache animation exported as mesh animation (no separate animation prim) |
| Avatar Animation: Joint | 2024.1; skeletal animation (SkelAnimation under SkelRoot) |
| Cache | 2024.2; avatar cache animation |
| Basic Scale (unit or %), Axis Conversion (Up Axis) | 2024.2 |
| USD Layer window | Create / Import / Clear; Authoring Layer; Export to USD Layer; Export USDZ from Layer (read-only); USDZ file type support added 2025.0 |

Notes: avoid renaming the Animation Layer; to export avatar and patterns together select avatar first. Project .zprj stores USD layer info.

### 12.6 glTF / GLB (ver 2026.0)

Export options: Garment animation, Avatar joint animation, Bake keyframes (key on every frame), Avatar cache animation, Animation range (Entire Region or Play Region Only). Blend shape animation importable/exportable as FBX, glTF/GLB, Alembic or USD (Blend Shape Avatar article).

### 12.7 Cross-format matrix (from the above)

| Capability | Alembic | MDD/PC2/MC | OBJ seq | FBX | USD | glTF |
|---|---|---|---|---|---|---|
| Garment vertex animation | yes | yes | yes | yes (mc cache) | yes (mesh anim) | yes |
| Avatar skeletal animation | no (avatar shape option) | no | bone JSON | yes (joint keys) | yes (SkelAnimation) | yes |
| Avatar cache (softbody) | yes | no | no | n/a | yes (2024.2) | yes (2026.0) |
| Blend shape animation | yes | n/a | n/a | yes | yes | yes |
| Frame range choice | entire/play/current | entire/play | entire/play/input range | entire/play | n/s | entire/play |

---

## 13. Avatar morph and blend shape animation

| Item | Detail |
|---|---|
| Blend Shape Avatar (2026.0) | Main Menu > Avatar > Avatar Editor > Blend Shape tab. Avatar file must contain Blend Shape data and be loaded as Avatar type |
| Parameter weight | 0 to 100 per parameter, grouped by mesh unit |
| UI | Select Avatar (only if several blend shape avatars), Favorites, Show Favorites Only, Reset, Show Selected Only (2026.1) |
| Animation (2026.1) | "Add Animation Keyframe" button beside each parameter stores value at Current Frame; keyed in the Animation Editor; cuts down workflows that required morph targets |
| Interchange | FBX, glTF/GLB, Alembic, USD |
| Older approach | Size/pose variation via avatar sizing (parametric) and Alembic/MDD vertex-cache avatars; per-frame morph via imported cache animation |

---

## 14. Playback speed, loop, frame rate

| Control | Values | Note |
|---|---|---|
| Play speed | 1/4x to 8x (1/4, 1/2, 1, 2, 4, 8) | toolbar |
| Loop | on/off | toolbar |
| Frame Rate | in animation settings; units/range not published | 5 FPS used for transition stabilization |
| Scene Timewarp | in settings; semantics not documented | [NOT VERIFIED] |
| Play Type | in settings; semantics not documented | [NOT VERIFIED] |
| Playback Start/End | play area; right-click timeline set start/end | |
| Animation Max Frame | numeric input at timeline bottom right | |

Frame rate must match the target DCC; since 12.2 Alembic export has no FPS option (uses scene rate). PC2 import has an FPS field because the format does not store rate in the way MC/MDD do.

---

## 15. Crop / trim timeline

Official trimming tools: Set Start/End Frame, Clamp Range to Active Frames, Clamp Range to Selected Layers (2025.1), Cut and Merge for cache layers, Delete layer, Move layer. Export has Play Region Only to export just the trimmed range. A destructive "crop cache to range" command was not documented in pages read.

---

## 16. File and project persistence

| Item | Detail |
|---|---|
| .zprj | Project file; stores USD layer info (cache stored in project, [NOT VERIFIED] whether cache embeds) |
| .mtn | Joint motion |
| .pos | Pose |
| .avt | Avatar (Open/Save article exists, not read) |
| Presets | Simulation presets load/save in Property Editor; Import and Export Presets (2025.1) |

---

## 17. AI and motion-library features

- No AI motion generation, text-to-motion or built-in motion library was found in the official documents read (feature lists 12.x, 2025.x, 2026.0, 2026.1 snippets). Related automation: Auto Convert to Motion (retarget FBX to MTN), IK Joint Mapping and IK Mode (2025.0), EveryWear template-based rigging (Beta, 2026.0) which targets game rigging not motion.
- Default avatars ship with built-in poses and motions (Play Motion plays pose/motion files); the contents of a stock motion library were not documented in pages read. [NOT VERIFIED]
- 2026.1 features beyond animation: Seamline Ripping Tool (Beta, fabric tears during simulation with adjustable fraying), Pinching Simulation Override, Brush Pinching. Relevant for animated simulation because ripping happens during solve.

---

## 18. Gaps and recommendations for formfabric

Verified design to copy:
1. Cache-layer data model with sparse keyframes, linear fill between keys, solver re-run from a frame, cut/merge/move/lock layers, top layer overrides lower layers.
2. Property-layer keyframes (Constant/Linear/Cubic, parent/child grouped keys, auto key, batch key) driving fabric, pattern, avatar joints, pin, tack, wind, solver properties and blend shapes.
3. Record pipeline: load motion, transition (T-pose blend), solver at high quality with substeps, per-frame vertex capture.
4. Wind controller with Spherical/Planar, Strength, Decay 0 to 100, Frequency sec, Turbulence 0 to 100, keyed position and rotation.
5. Import: FBX, Alembic, MDD, PC2, MC, USD, glTF; MTN-equivalent motion format; auto convert (retarget) from foreign skeleton to a canonical skeleton.
6. Export matrix of section 12.7 with weld/thickness/UV/scale/axis/range options.

Unverified items the next researcher should chase (pages found but not readable, or not found):
- Per-property keyable lists (Keyframe Animation ver 2025.0 table).
- Avatar IK Joint Mapping page (fetch failed), Animation Marker page (fetch failed), 2026.1 New Feature List page (fetch failed).
- Settings pop-up ranges (Frame Rate, Scene Timewarp, Play Type, Unit, Simulation Quality value).
- Behavior of Freeze, Pin stiffness, Pin keyframe properties during cache recording.
- BVH support (no evidence found).
- Whether "pre-roll" exists as a user parameter (only transition animation documented).
- "Auto Cache" (no evidence).
- Motion library / AI motion (no evidence).

---

## Sources (URLs fetched)

Official MD support manual (support.marvelousdesigner.com/hc/en-us/articles/...):
- 47358333269529 Animation Keyframe and Layer Editing (ver. 12)
- 47358157019161 Keyframe Animation (ver. 2025.0)
- 47358280523033 ANIMATION MODE: Play Animation
- 47358202260761 [Mode] Animation
- 47358255571609 ANIMATION MODE: Garment Animation Making
- 47358326766233 ANIMATION MODE: Wind Animation
- 47358383927065 Wind Controller Setting
- 47358369726233 Wind Controller Setting (Above ver. 2024.0)
- 47358158495129 Auto Convert to Motion (Ver. 2025.0)
- 47358232885017 3D File (FBX) Import/Export
- 47358253053081 Alembic Import/Export
- 47358364600601 Animation Import
- 47358355178393 Cache Animation Export
- 47358384957209 OBJ Sequence Export
- 47358363172249 Motion (*.mtn) Open/Save
- 47358474847897 Play Motion
- 47358343098777 USD file Compatibility (ver. 12.1)
- 55686813557913 glTF 2.0 (glTF/GLB) File (ver 2026.0)
- 55654243482649 Blend Shape Avatar
- 47358125463321 Simulation Properties
- 47358263976729 Avatar Properties
- 47358126161177 UI Changes (Ver 2025.0)
- 47358220563737 FILE: Export Joint Keyframe Animation with FBX
- 47358315135897 Freeze/Unfreeze
- 47358146159769 Pins
- 47358145573401 Tips and Tricks: Marvelous Designer and Unreal Engine
- 47358120307353 MD 2025.0/2025.1/2025.2 New Feature List
- 55837641308313 MD 2026.0 New Feature List
- 47358241113625 MD 12/12.1/12.2 New Feature List

Official site:
- https://www.marvelousdesigner.com/products/release
- https://www.marvelousdesigner.com/support/news/view/448aa11bc06d4b91a63668ca1d12a96e (2026.1 release news)
- http://manual.marvelousdesigner.com/display/MD4M/Record+Animation (legacy official manual)
- http://manual.marvelousdesigner.com/display/MD4M/Cache+Animation (legacy official manual)

Fetch attempts that failed (not used as evidence): Avatar IK Joint Mapping (2025.0) article 47358133151385; Animation Marker article 56122489802009; 2026.1 New Feature List article 59743730927129; legacy manual Play and Edit Animation (robots disallowed); community post 48272209131801.
