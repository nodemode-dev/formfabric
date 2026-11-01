# Marvelous Designer: 3D-Window Editing Tools and Newest / Advanced Features (2022 to 2026)

Research slice 11 of 13 for the formfabric encyclopedia. Scope: 3D-window editing tools, modular mode, auto fitting, AI features, EveryWear, retopology, pinching, sculpt, and every per-version feature from 2022 onward.

Research date: 2026-11-01. Latest release found: Marvelous Designer 2026.1 (August 2026).

## 1. Summary

- Marvelous Designer (MD) changed its naming scheme: versions 11 and 12 (12, 12.1, 12.2) ran through 2021 to 2023, then year-based versions began with 2024.0 (Feb 2024), 2024.1, 2024.2, 2025.0 (Apr 2025), 2025.1 (Aug 2025), 2025.2 (Nov 2025), 2026.0 (Apr 2026), 2026.1 (Aug 2026). Cadence is roughly three releases per year.
- The 3D window is built around a small set of modes and a toolbar: five top-level modes (Simulation, Animation, Modular, UV Editor, Sculpt), and a 3D Garment menu containing Select/Move, Fold Arrangement, Pin tools, Flatten, 3D pen (garment and avatar), Edit Line, pinching, Auto Fitting and Re-Drape, retopology tools, and EveryWear polygon optimization.
- AI features are delivered as an opt-in "AI Studio" plug-in (disabled by default, enabled per license in the website account): AI Texture Generator and AI Graphic Generator (2024.1), PBR Map Generator (2024.1), AI Pose Generator and AI Pattern Drafter (2025.1, beta). There is no verified "image-to-garment" end-to-end feature; the closest are Pattern Drafter (measurements, keywords, flat sketch or uploaded image to a T-shirt style pattern, later pants and skirts) and 3D Pencil/3D pen sketch-to-pattern flows.
- Recent trend (2025 to 2026): the tool is moving from pure 2D-pattern-first to also supporting 3D-first drawing (off-avatar 3D pen in 2025.1, 3D Pencil in 2026.0), richer in-viewport deformation (advanced pinching, brush pinching, pinching override, seamline rip), keyframeable cloth parameters (2025.2), and game-ready pipelines (EveryWear: polygon optimization, auto seal, rigging, UV packing, baking, local FBX export).
- Not found in official sources: any feature named "puffer" or "quilting" tool as a dedicated 3D tool. Puffed/inflated looks are covered by the pressure property (keyframeable from 2025.2), shrinkage, and solidify. Quilting questions appear only in CLO user community posts (not used). Marked as unverified below.
- Not found: a tool literally named "Garment to mannequin"; the equivalent workflow is Auto Fitting / Re-Target Draping with Fitting Suit avatars.

## 2. Per-version feature table (2022 onward)

Notes: Version labels are the official ones. "Group" is the official grouping on the New Feature List pages. Source keys are defined in the Sources section.

### 2.1 Marvelous Designer 11 patch (Feb 2022) and 11.x

MD 11 itself shipped in Oct 2021 (outside the 2022 window but listed for context). Official per-version lists for 11.1 and 11.2 could not be fetched (the official "Making Marvelous Designer 11 New Feature List" article exists but was not retrievable; see Gaps).

| Version | Feature | Source |
|---|---|---|
| 11 (Oct 2021, context) | Customizable UI, multiple modes in different 3D windows, drag tools out of menus | S12 |
| 11 (context) | Retopology: ghost points across sewing relationships; patch topology tool (uniform rectangular or triangular patches with set subdivisions) | S12 |
| 11 (context) | Partial freeze: brush to select mesh parts that stay solid during simulation | S12 |
| 11 (context) | In-app store (Windows only); animation editor moved to bottom with frame rate/playback options | S12 |
| 11 patch 6.1.645 (Feb 25 2022) | Export 3D avatar to FBX with skeleton and skinning information | S13 |
| 11 patch (Feb 2022) | UDIM file naming support on texture export | S13 |
| 11 patch (Feb 2022) | Thumbnail previews for physical property presets showing drape under gravity | S13 |
| 11 (Fold Arrangement manual note) | Fold Arrangement symmetric fold application "available in version 11" | S8 |

### 2.2 Marvelous Designer 12 (Nov 2022)

| Group | Feature | Source |
|---|---|---|
| Key | Retopology Improvements (select topology points, snap patches to internal lines/shapes/curves, Cut Topology Faces, ghost topology, drag select) | S10, S11, S14 |
| Key | Animation Editor Improvements (edit keyframes of simulation caches, blend simulations, adjust timing, looping cloth caches) | S10, S11 |
| Key | Styleline Editing (reshape patterns in 3D by editing lines between garment parts; changes propagate to 2D) | S10, S11 |
| Key | Merge Vertex by Proximity at Export | S10, S11 |
| Key | Automatically Create Fitting Suit at Import (custom bipedal avatars) | S10, S11 |
| Key | Spiral Tool (radius, length, width, offset, direction) | S10, S11 |
| Key | Active Tool Hotkey Reference Window | S10 |
| Other | Export Joint Keyframe Animation with FBX | S10 |
| Other | Register Custom OBJ as Zipper Slider and Puller | S10 |
| Other | Assign Different Materials to Zipper Slider, Puller, Stopper and Teeth | S10 |
| Other | Apply Substance Materials to Trims, Buttons and Avatars | S10 |
| Other | Lock Remesh Pattern Selection in 2D Pattern Window | S10 |
| Other | Change Normal Map Blending Method | S10 |
| Additional | Simulation speed and stability improvements; improved accuracy of exported joint transforms; Import FBX as Trim; Load Trim before creating patterns | S10 |
| UI note | "Show X-Ray Joints / Show Avatar Joints (ver. 12)" 3D display option | S20 |

### 2.3 Marvelous Designer 12.1 (May 2023)

| Group | Feature | Source |
|---|---|---|
| Key | USD file compatibility | S10 |
| Key | Omniverse Connector (Beta) | S10 |
| Key | Import Background Image by Drag and Drop | S10 |
| Key | Switch Viewport Layout by Hotkey | S10 |
| Key | Show Statistics (3D window overlay) | S10, S20 |
| Other, File | New Project, New Garment | S10 |
| Other, Library | Show in Explorer | S10 |
| Other, Avatar | Show All Avatars | S10 |
| Other, Animation | Fit the Frame Range; Animation Layers | S10 |
| Other, Retopology | Face-Selection; Show Topology Only (3D) | S10 |
| Other, UI | Window main menu | S10 |

### 2.4 Marvelous Designer 12.2 (late 2023)

| Group | Feature | Source |
|---|---|---|
| Key | Automatic UV packing | S10 |
| Key | Import/Export FBX with multiple avatars | S10 |
| Key | Roll Up | S10 |
| Key | Select all sewn patterns | S10 |
| Key | USD: support unified UV coordinates | S10 |
| Highlighted, 2D | Clone as Pattern (with Sewing); Optimize All Curve Points option dialog | S10 |
| Highlighted, Avatar | Joint Visualization improvement | S10 |
| Highlighted, History | Improved 3D state preview window in History | S10 |
| Highlighted, Auto Fitting | Maintain Texture Size After Auto Fitting; Auto Fitting Improvement | S10 |
| Other | Silent install support; FBX axis conversion; Alembic FPS option removed; Animation Editor display range persisted; default avatars replaced (Mia and Luka); topstitch aligned to pattern outline usability; library multi-download and file extension display; Snapshot dialog layout | S10 |

### 2.5 Marvelous Designer 2024.0 (Feb 28, 2024)

| Group | Feature | Source |
|---|---|---|
| Effortless | Scale Custom Joint Avatar of FBX File | S5 |
| Effortless | Match 2D Pattern Measurements | S5 |
| Effortless | Efficient 2D Pattern Selection | S5 |
| Effortless | Basic Arithmetic Operations in Input Field | S5 |
| Effortless | Thumbnail on Project file icon | S5 |
| Effortless | Save Min-Max Animation Range in zprj | S5 |
| Effortless | Display graphic images outside the pattern | S5 |
| Effortless | Axis Conversion for Alembic Import | S5 |
| Effortless | Run Fit Horizontal with the animation slider | S5 |
| Effortless | Delete the Selected Scene and Prop | S5 |
| All-In-One | Reorganize the Remeshing-related features | S5 |
| All-In-One | Polygon Optimization | S5 |
| All-In-One | Current frame option for Alembic export | S5 |
| All-In-One | USD single/multi object export option | S5 |
| Seamless | LiveSync download menu under Help | S5 |
| Seamless | Include Garment Simulation Data option in USD export | S5 |
| Seamless | 18 default MetaHuman body types in asset library | S5 |
| Seamless | One email account (CLO-SET) sign in | S5 |
| Other | Expiration date display position | S5 |
| Doc note | Wind Controller Setting (above ver. 2024.0) | S20 |

### 2.6 Marvelous Designer 2024.1 (Jun 17, 2024)

| Group | Feature | Source |
|---|---|---|
| Effortless | Show Layer Depth in 2D Window | S5 |
| Effortless | UV Texture Seam Improvements | S5 |
| Effortless | Show/Hide Color menu (Show/Hide 3D Color) | S5, S20 |
| Seamless | UV Packing improvements with new nesting algorithm | S5 |
| Other | Rename Density (g/m2) to Weight (gsm) | S5 |
| Other | Auto Convert to Avatar dialog change for Daz avatars | S5 |
| Other | Improved Opacity Map functions | S5 |
| Other | Minimum Particle Distance for Displacement Map increased | S5 |
| Other | Trim collisions improved | S5 |
| AI (Beta) | AI Texture Generator: keywords to fabric texture images (English prompts only), choose fabric, adjust parameters, preview | S16, S17 |
| AI (Beta) | AI Graphic Generator: keywords to graphics applied to patterns; saved in history; transform/rotate/scale | S16, S17 |
| AI (Beta) | PBR Map Generator: PBR maps from imported texture; normal map with adjustable intensity; displacement, opacity, roughness, metalness saved automatically; works on fabrics, buttons, trims | S16 |
| Other | Trim Window (manage trims like fabrics and buttons; import trim styles and apply/replace in 3D window); not AI | S16 |

The AI features in 2024.1 are listed in the third-party RebusFarm article and the official AI Studio page (S17) that gives the version tags "2024.1". Cross-checked: two sources.

### 2.7 Marvelous Designer 2024.2 (Oct 21, 2024)

| Group | Feature | Source |
|---|---|---|
| Key | Trim Simulation with Collision | S5 |
| Key | Advanced Pinching with Soft Selection | S5, S19 |
| Effortless | [EveryWear] Masking Brush and Thin/Thick Mode | S5 |
| Effortless | Duplicate Pins to Symmetric Pattern | S5, S9 |
| Effortless | Expand Roughness Map Intensity Range | S5 |
| Effortless | [Buttonhole] Z-offset option | S5 |
| Effortless | Smooth Avatar for Template Avatars | S5 |
| Effortless | [USD Layer] Include Garment Simulation Data option | S5 |
| Effortless | [USD] Export Cached Avatar Animation | S5 |
| Effortless | Convert to Segment Point / Curve Point improvement | S5 |
| All-In-One | [EveryWear] Improve Auto Rigging Quality | S5 |
| All-In-One | [EveryWear] Smooth Brush works with multiple joints | S5 |
| All-In-One | Improved Quad Remeshing | S5 |
| Boost Assets | Closed End option for Piping | S5 |
| Seamless | Import FBX as Scene and Props | S5 |
| Seamless | Axis Conversion as USD import/export option | S5 |
| Seamless | Thin/Thick mesh export option via USD Layer | S5 |
| Other | Custom thumbnail when saving ZSE files; [Avatar] Register Pose/Motion | S5 |

### 2.8 Marvelous Designer 2025.0 (Apr 2025)

Key features (S1, S2, S3): UI Improvement (reworked UI), Keyframe Animation (new attributes incl. wind and fabric properties), Quad Mesh Improvement (fewer triangles, cleaner edge flow), IK Joint Mapping (move/rotate joints directly to pose avatars), Side and Back UV Expansion, [Beta] Fur Strand in 3D Window (fur parameters and shape, spline export), Auto Convert to Motion, Modular Library, New Library Window.

| Group | Feature | Source |
|---|---|---|
| Effortless | Schematic Render | S1, S2 |
| Effortless | Auto Sewing | S1, S2 |
| Effortless | Blending Graphics with Fabric | S1, S2 |
| Effortless | Garment Fit Properties | S1, S20 |
| Effortless | Object Browser (all style windows and scene window included later) | S1 |
| Effortless | PBR map improvements; Smooth corners for topstitch; Offset internal line improvements; Avatar editor improvements; Custom UI layout; IK mode for avatar joints; Auto fitting improvements; Load all texture maps | S1 |
| Seamless | Color switch for graphics/textures; FBX export with material names; Polygon optimization for topstitch; USDZ file support | S1, S2 |
| Other | UV Editor anti-alias lines; Show Stress Map option in fit maps; many new seamline property types; custom OBJ topstitch assets; physical property creator info; more material options for seam taping; texture map open/add in another app; EveryWear moved to Connect menu; Auto Convert to Avatar moved into avatar editor; object property editor changes incl. eye control; new 2D garment rendering styles; Advanced pinching; open/close any window from menu/title bar; 2D snapshots; 3D multiview snapshots; background aspect ratio toggle; individual curve line and point editing range; Tack for attaching trims across multiple vertices and patterns | S1 |
| Other (press) | Auto Seal; new Sculpt Tool; animation with trim simulation; USDZ export; OBJ zipper teeth presets | S3 [single source for "new Sculpt Tool"] |
| Doc note | Quick Pinching and Advanced Pinching documented as "Ver. 2025.0" with a new 3D toolbar drop-down | S19 |
| Doc note | Re-Target Draping / Re-Drape 3D Arrangement documented as "ver. 2025.0" | S18 |

### 2.9 Marvelous Designer 2025.1 (Aug 2025)

| Group | Feature | Source |
|---|---|---|
| Key | Off-Avatar 3D Pen Drawing: draw curves on avatar, in empty 3D space, or on a plane; convert to editable 2D patterns | S2, S4, S6 |
| Key | Keep Topology after Auto Fitting: preserves quad structure so UVs, rigging, blend shapes, vertex colors are reusable | S2, S4, S6 |
| Key | Pattern Drafter (Beta): shirt pattern from measurements, keywords, or flat sketches; General/Simple/Custom measurement views; AI Pattern Drafter extension from text prompts or schematic sketches, T-shirts only at launch | S2, S4, S6, S21 |
| Key | Soft Body for Custom Avatars (imported rigs and meshes; non-skeletal avatars) | S2, S4, S6 |
| Key | Denim Wet Wash (strength, direction, masking; blend multiple washes; presets Rinse, Enzyme, Stone, Acid, Bleach, Ozone) | S2, S4, S6 |
| Key | AI Pose Generator (Beta): poses from text prompt or full-body reference photo; outputs .pos files; service may be temporarily suspended | S2, S4, S6, S17 |
| Effortless | Zipper style customization; grouping of internal lines and shapes; trim mirror paste; blend print with fabric; avatar wireframe display | S2, S4 |
| Boost Assets | AI Studio icon in top-right corner | S2, S4 |
| Other | Import/export presets; show/hide scene and props; enhanced masking selection; partial auto rigging; clamp range to selected animation layers; schematic render improvements; warning dialog when starting sculpt simulation; flattening multiple areas into a single merged pattern; selective saving of individual garments | S2, S4 |

### 2.10 Marvelous Designer 2025.2 (Nov 17, 2025)

| Group | Feature | Source |
|---|---|---|
| Key | Convert Patterns to Trims and Accessories (context menu; trims treated as rigid geometry) | S7, S22, S23 |
| Key | MetaHuman DNA Importer (native DNA, proportions/mesh/rig unchanged; outfits adapt when body changes) | S7, S22, S23 |
| Key | Keyable Properties Expansion: shrinkage, pressure, solidify, tack strength, trim weight | S7, S22, S23 |
| Key | EveryWear Export with FBX (local export without CONNECT upload) | S7, S22, S23 |
| Key | Pleat Creation Tool: knife, box, accordion; number, depth, interval, fold angle; non-destructive | S7, S22, S23 |
| Effortless | Zipper Improvement; Add Pants and Skirts in Pattern Drafter; Soft Body Simulation in GPU; Animation Layer Setup; Instant Simulation Stop; AI Feature Activation; Pin Group Selection (double-click selects all pins on pattern); UI Color Customization; Add Edit Zipper Tool | S7 |
| All-In-One | [EveryWear] Target Joint Edit Mode; Mirror Weights; Switch to Bind Pose | S7 |
| Boost Assets | Hat as an Accessory category | S7 |
| Seamless | Select All with Same Property in UV Editor | S7 |
| Other | Library docked widget for universal browser; Seamline Sync during Export; Script menu renamed Plugins; EveryWear labeled Beta and a Plug-In; Wind Gizmo rotation fix; Style Configurator for Kid Avatars; Preferences reset dialog; default topstitch thread thickness unit mm; 255 char file name limit removed; AutoLogout improvement; Graphic placement by measurement | S7 |

### 2.11 Marvelous Designer 2026.0 (Apr 2026)

The official page lists names only (S24). Descriptions below come from DigitalProduction (S25) where available.

| Group | Feature | Source |
|---|---|---|
| Key | Lacing Tool: click eyelet holes in sequence; generates an editable lace pattern in 2D; reverse cord direction per hole; curvature slider; lace is a normal pattern (thickness, material, physical properties) | S24, S25 |
| Key | 3D Pencil: sketch on avatar in locked front or back view; eraser mode; convert to 3D pen result then edit; front and back sketches sew together on flatten | S24, S25 |
| Key | Preserve 3D Garment Shape on 2D Scaling [no description found] | S24 |
| Key | [EveryWear] Template-Based Rigging (Beta) [no description found] | S24 |
| Key | Toon Shader / Express Cartoon: Material type Toon for fabrics, 3D objects, avatars; base color, opacity, shadows (shade color/map, hardness, shift), emission, MatCap, rim light (color, fresnel power, lift), outlines (mode, mask map, color, width in mm) | S24, S25 |
| Effortless | GPU-Accelerated Trim Simulation | S24 |
| Effortless | Isolate Selection (documented in the 3D Window Environment section) | S24, S20 |
| Effortless | Animation Timeline Marker; Steam Eraser Brush; Maintain Graphics and Seam Lines on Fur; Pattern Archive; Graphic On/Off Toggle; Edit Mode in Pattern Drafter; Display Garment Fit Maps on 2D Patterns; Open Selected Material from ZFAB and Swap Front/Back; Symmetric Line Selection in Half-Symmetry Pattern Trace; Select Desired Point in Smooth Curve Tool; Zipper Usability Improvements; Optimize Flattened Pattern Outline Points; Real-time arithmetic input updates; Download folder path for CONNECT; Preview in Docked Library; Fabric-Aware Strain Maps; Unlink Internal Line Symmetry | S24 [names only] |
| Export | glTF and VRM support; Play region selection for FBX/glTF export; Bake keyframes for joint animation export; FBX import Open/Add with scale, axis, joint and cache animation toggles; trim import with collide toggle and glue; FBX export of single/multiple objects, weld/un-weld, thin/thick, combined UVs and textures; Weld Turned Sewing Lines | S25 [single source] |

### 2.12 Marvelous Designer 2026.1 (Aug 2026)

| Feature | Detail | Source |
|---|---|---|
| Seamline Ripping Tool [Beta] | Seam lines tear when simulated force exceeds a resistance; adjustable fraying; fray disabled on Mac; cannot yet be exported as mesh | S26, S27 |
| Pinching Simulation Override | While pinching, your own fabric/simulation settings replace global values; Override Strength blend; Update and Reset Rest Shape | S26, S27, S28 |
| Brush Pinching | New pinch type: push/pull fabric along a stroke path; Size, Focal Radius, Lazy Distance, Display Weights, Strength, Push Inward, Average Radius | S26, S27, S28 |
| Blend Shape Animation | Keyframe avatar blend shapes inside MD; import/export via FBX, glTF/GLB, Alembic, USD | S26, S27 |
| Camera interchange | Import/export cameras via FBX and glTF/GLB; one Main Camera at a time | S27 [single source] |
| Variable Fold Angle | Fold angle transitions along an internal line from a chosen value to 180 degrees | S27 [single source] |
| Zipper Extend | Adjust length and direction of zipper chain | S27 [single source] |
| Autosave settings | Interval, max file count, retention, file type, storage path | S27 [single source] |
| Toolbar customization | Reorder, hide, regroup tools; layouts saved as JSON; tools cannot move between 2D and 3D toolbars | S27 [single source] |

## 3. Tool sections

### 3.1 Mode system (3D window context)

Source: S29. Five modes switched with a button at the top right:
1. Simulation Mode: create/edit patterns in 2D, drape/animate garments in 3D.
2. Animation Mode: record, play, edit garment animation.
3. Modular Mode: mix and match garment components (sleeves, collars) as blocks.
4. UV Editor Mode (introduced in MD 7.5 per the page).
5. Sculpt Mode (introduced in MD 8 per the page).

Clone note: a single viewport with a mode switch that swaps toolbars and side panels is the model to copy.

### 3.2 Select/Move (3D)

Source: S30.
- Path: Main Menu, 3D Garment, Select/Move; or 3D Toolbar. Default hotkey Q (S19).
- Behavior depends on simulation state. Simulation active: click-drag on garment pulls it as a physical, simulated drag ("moves naturally"). Simulation inactive: click-drag switches to pattern arrangement, moving the garment as rigid patterns.
- Pinning shortcut: with Select/Move, hold W and click to pin a point (requires English input language) (S9).
- Move with Arrow Keys (3D) exists as a separate manual page (title seen in search, not fetched).

Clone note: the same drag gesture must map to a spring/constraint pull when the solver runs and to a rigid transform of the 3D pattern when it is paused.

### 3.3 Transform in 3D and gizmo

Not verified. The manual's 3D Garment menu has an arrangement/gizmo system (Fold Arrangement uses a gizmo; Pin tool uses a Gizmo to move pinned mesh, S9). A dedicated page for "Transform Pattern (3D)" was not retrieved. Treat as a gap.

### 3.4 Fold Arrangement

Source: S8.
- Path: Main Menu, 3D Garment, Fold Arrangement; or 3D Toolbar.
- Purpose: fold seam allowances, collars, cuffs before simulation for stability.
- Shows all internal lines and shapes on the 3D garment; picking one shows a folding gizmo with red and green axes rotating along a blue circle.
- Symmetric patterns fold symmetrically (noted from version 11).
- Can also fold after simulating.
- 2026.1 adds Variable Fold Angle along an internal line (S27, single source).
- Clone note: fold is an internal-line hinge rotation: rotate the mesh strip on one side of the line about the line axis by an angle, then let simulation resolve.

### 3.5 Pin tools

Source: S9.
- Pin (Box) and Pin (Lasso) tools: garment mesh turns red; click a vertex or drag an area to pin its vertices. Shift+Ctrl drag adds, Ctrl drag removes.
- Pins can be created on segments, whole patterns, or internal lines (ver 4.0.0).
- Pinned mesh is moved by drag or gizmo.
- Delete selected, all, or all on selected patterns.
- Duplicate Pins to Symmetric Pattern (2024.2).
- Attach/Detach Pins to Avatar: attaches to nearest avatar part; attached pins cannot be selected or moved.
- Double-click selects all pins on a pattern (documented as 2025.2, "Pin Group Selection").
- Pins can be keyframed in the animation timeline? Not verified.

### 3.6 Pinching (Quick, Advanced, Brush, Override)

Sources: S19 (Quick/Advanced), S28 (Brush), S26/S27 (Override).
- Quick Pinching: pulls a single picked point. Runs while simulation is active; Q is default when simulation starts. Menu: 3D Toolbar drop-down (2025.0).
- Advanced Pinching: in 2025.0 under the Pinching drop-down; in 2024.2 and earlier via right-click 3D background "Advanced Pinch" or Shift+E. Dialog options:
  - Range: graph of Falloff Shape x Distance x Power (read-only).
  - Falloff Shape: Bump or Linear.
  - Falloff Distance: 1 to 500 mm.
  - Falloff Power: -2.0 to +2.0.
  - Distance Measure: Surface (independent layer only) or Straight (all patterns within distance).
  - Distance Indicator (two concentric circles), Limit to Selected Patterns, Display Weights.
- Brush Pinching (2026.1): with Simulate active, press and hold the Quick Pinching tool over 2 seconds, choose Brush Pinching. Drag to apply pinching along the accumulated stroke path. Settings: Size, Focal Radius, Lazy Distance (stroke trail length), Display Weights, Strength, Push Inward, Average Radius (averaging neighbor normals to stabilize direction).
- Pinching Simulation Override (2026.1): temporarily swaps selected global/fabric settings with your own while pinching, blended by Override Strength; Update and Reset Rest Shape (S27).
- Clone note: pinching = soft-selected vertex set with falloff weights driven by a target position constraint. Brush pinching = accumulated stroke path with lazy-follow smoothing producing a moving target.

### 3.7 3D Pen (garment) and Edit 3D Pen / Edit Line

Sources: S31 (edit), S32 (flatten), S2/S4 (off-avatar), S25 (3D Pencil).
- Edit 3D Pen (Garment) (named Edit Line (3D Pattern) in ver 5.0.0): click a segment (turns yellow), drag either endpoint; Delete key or right-click Delete removes. Path: 3D Garment menu or 3D Toolbar.
- Flatten (S32): extracts patterns from lines/shapes drawn on the avatar. Only closed shapes become patterns. Hovering highlights the area. Shift+click multi-select. Enter = Flatten (Separate): each area becomes its own pattern with default fabric, sewing created automatically between adjacent patterns, lines remain on avatar. Shift+Enter = Flatten (Merge): adjacent surfaces combine into one pattern. 2025.1 adds flattening multiple areas into a merged pattern (S2).
- Off-Avatar 3D Pen (2025.1): draw straight or curved connected lines on the avatar, in empty 3D space, or on a plane, then flatten (S2, S4, S6).
- 3D Pencil (2026.0): freehand sketch tool. Sketch dialog, front/back locked view with zoom allowed, eraser with size, convert to 3D pen result, auto-switch to edit tool, front and back sketches auto-sew on flatten (S25).
- Auto Seal in EveryWear polygon optimization uses a "3D Pen Mode" closed curve (S33).

Clone note: a surface-snapped polyline/curve tool on avatar mesh (geodesic snapping), a free-space and plane mode, closed-loop detection into regions, and a flatten solver (3D surface patch to 2D pattern) are required.

### 3.8 Sculpt mode (brush tools on cloth)

Source: S34 (official manual page). Accessed by the mode switch; Brush toolbar appears top-left; hotkey B opens brush type and settings; garment displays gray; Material Type changes color.
- Brush types: Wrinkle (ver 10), Release (ver 10, Shift+drag to remove wrinkles), Sculpt (emboss; Invert engraves), Smooth (Shift shortcut), Grab, Stamp (image stamp), Pinch.
- Properties by brush: Simulation Preset (Wrinkle, Release); Strength (Sculpt, Smooth, Grab, Stamp, Pinch); Size (all); Focal (coverage like hardness, all); Spacing (Sculpt); Depth (Sculpt, Smooth, Grab, Pinch); Invert (Alt; Sculpt, Stamp); Image (Sculpt, Grab, Stamp); Realistic and Layer (Wrinkle, Release, ver 10).
- Image brushes: default images or custom monotone images (white areas emboss); Focal unavailable with image brushes; Stamp drag sets size and angle.
- View options: Double Side Display; Show/Hide 3D Pattern with Shift+Q (isolate or swap patterns); rendering style Internal Lines, Monochrome Surface (Alt+3), Wireframe on Surface (Alt+6).
- 2025.1 added a warning dialog when starting sculpt simulation (S2, S4). Press coverage of 2025.0 mentions a "new Sculpt Tool" (S3) without detail; unverified further.
- Steam Eraser Brush (2026.0, name only, S24).
- Partial Freeze (MD 11): brush to select mesh parts that stay solid during simulation (S12).

Clone note: sculpt mode edits displacement/offset on the cloth mesh outside the solver (or as a rest-shape modifier); wrinkle/release brushes run local simulation with a preset (S34 says Simulation Preset applies to Wrinkle and Release).

### 3.9 Modular mode, Modular Configurator, Modular Library, Blocks

Sources: S35 (library), S36 (configurator window), S29 (mode list). Introduced as a Key Feature "Modular Library" in 2025.0 (S2); the Modular Mode itself appears in the older mode list (S29) so mode predates 2025.0; the 2025.0 library is a new organization and creation layer. [exact first version of Modular Mode not verified]

Library structure (S35): Group, Category, optional Line, Style, Block.
- Location: Main Menu, Window, Modular Library; or right-click Side Panel title bar.
- Group and Category are folders on disk (Add Group chooses a folder; Add Category names a subfolder).
- Line (optional tier): groups multiple Styles; "Add Line with Same Style" duplicates all Styles and Blocks of a Line.
- Style: represents a fit type; all blocks linked to a style should fit together. Thumbnail auto-captured from 3D or custom image.
- Block: a set of one or more patterns forming a component. Block Type plus Tag (tag distinguishes multiple blocks of same type; same type and tag overwrites). Blocks created in the Modular Creator by selecting patterns in 2D (Shift multi-select), Enter, then name and thumbnail.
- Right-click block: Delete, Share Selected Block (shared across styles, edits apply to all instances), Change Thumbnail, Replace with New Block.
- Auto Modular Labeling: identifies which block each pattern belongs to by analyzing pattern shape, "based on learning from many pattern datasets"; colors by block type; manual fix via Edit Block; then Save Block, then Add and check sewing. This is an ML classifier feature [single source: S35].
- Auto Block Creation: Modular Configurator option to add all blocks and assign patterns automatically.
- Modular Library options (Preferences, Other): Default Folder; Protect Modular Library to Restrict Editing; Prevent Blocks Mix and Match across Different Styles (warns); Notch Sewing (Modular Sewing tags follow notch placement to save sewing relationships).

Modular Configurator window (S36), used in Modular Mode:
1. Open Modular Configurator from vertical tab menu at left.
2. Choose category in horizontal tab bar.
3. Pick garment folder (shows Modular Structure files).
4. Select a Modular Structure; window shows only compatible Blocks.
5. Select a Block; current blocks are swapped for it.
6. Adjust design and finish.

Also "Edit Block Folders in the Modular Configurator Window" exists as a separate page (title only seen).

Clone note: blocks are named sets of pattern pieces plus sewing tags (type, tag, notch-based sewing identity). Swapping a block removes the old pieces of that type and inserts new ones, re-creating sewing by tag. A "structure" file defines which block types and sewing relations a style needs.

### 3.10 Auto Fitting, Re-Target Draping, garment conversion between avatars

Source: S18 (official), plus S2/S4 (topology option), S10 (12.2 improvements).
- Auto Fitting: resizes a garment to fit a custom avatar and simulates on it. Path: 2D Pattern, Fitting, Auto Fitting; or 2D toolbar.
- Re-Target Draping / Re-Drape 3D Arrangement (ver 2025.0): simulate on the avatar keeping original size. Path: 3D Garment menu or 3D toolbar.
- Requirements: avatar with Fitting Suit data; garment with Fitting Suit data (simulated in MD 9.5 or later, or on a Fitting Suit avatar). MD 12 added automatic fitting suit creation at import for custom bipedal avatars.
- Options: Maintain Pattern Curvature (%); Maintain Graphic Size / Maintain Texture Size (texture option added in 12.2); Maintain Topology (2025.1: ON preserves mesh structure but all pattern meshes must be synchronized first; UV may need readjusting; OFF lets remeshing happen).
- Tips: raise particle distance for speed; reset garment properties to default.
- The "Fitting Suit" is a reference mapping between avatar bodies: a standardized body-near mesh used to transfer the garment between avatars of different size. [mechanism inferred from requirements, not documented in detail]

Clone note: to implement garment-to-avatar conversion you need (a) a correspondence between source and target body (fitting suit), (b) pattern resizing by measured body deltas with curvature preservation parameter, (c) re-simulation, (d) optional mesh topology retention for rigging reuse.

### 3.11 EveryWear (game-ready pipeline) and polygon optimization

Sources: S33 (official CLO-SET EveryWear game-ready guide), S5, S7, S1.
- EveryWear is an add-on pipeline converting garments to game assets; moved to Connect menu in 2025.0; labeled Beta and a Plug-In in 2025.2; local FBX export from 2025.2.
- Pipeline from S33: (1) Polygon optimization via Garment Polygons tab: target count and Optimize, or by particle distance; patterns show "Quad (Optimized)". (2) Accessories optimized separately; Billboard vs Mesh for buttons/zippers/trims. (3) Auto Seal: closes hems and cuffs using guide curve in Disk Mode (three points) or 3D Pen Mode (closed curve); set rendering style Thick and enable Back Face to view. (4) Masking Brush: mask hidden geometry (inner pocket linings, areas covered by avatar). (5) Rigging: Apply (Max Influence Joints optional); rig final design only, since changing design or simulating deletes rig data. (6) Skin weights brush: focus on collar, armpit, pants, skirt, wrist, ankle, elbow, knee. (7) Skinning test with .mtn joint motion and Animation Editor. (8) UV packing: image size 2048, seam size 8, Pack UVs, 0 to 1 single map. (9) Texture baking: Detail Normal Map and Ambient Occlusion Map with Preview and Flip Normal. (10) Default pose: a .pos file for the target game avatar.
- Version history: Masking Brush and Thin/Thick Mode (2024.2); Improve Auto Rigging Quality; Smooth Brush works with multiple joints (2024.2); Auto Seal (2025.0 press, S3); partial auto rigging (2025.1); Target Joint Edit Mode, Mirror Weights, Switch to Bind Pose (2025.2); Template-Based Rigging Beta (2026.0).
- Polygon Optimization as a general feature: 2024.0 All-In-One; topstitch polygon optimization 2025.0; quad mesh improvement 2025.0 and improved quad remeshing 2024.2.

Clone note: needed are quad-dominant remeshing with target polygon count, hem sealing (surface thickening/capping), automatic skinning from body weights, weight painting, UV pack + bake of normal/AO from high-poly.

### 3.12 Retopology mode tools

Sources: S14 (MD 12 retopology improvements), S12 (MD 11), S10 (12.1).
- Create Topology tool (3D/2D toolbar, Topology Tools): draw points/edges to cut faces; faces with more than 4 sides highlighted orange; snap to internal lines/shapes and boundaries.
- Edit Topology tool: Ghost Topology (mirror points across sewing relationship), Drag Select, Shift add, Ctrl deselect.
- Patch Topology tool (2D toolbar): uniform rectangular or triangular patches; snaps to internal lines and curve points (convex and concave); validity check flags overlapping patches.
- 12.1: Face-Selection and Show Topology Only (3D).
- Related: Lock Remesh Pattern Selection (12), Reorganize Remeshing features (2024.0), Improved Quad Remeshing (2024.2), Quad Mesh Improvement (2025.0), Maintain Topology in Auto Fitting (2025.1), Optimize Flattened Pattern Outline Points (2026.0).

### 3.13 Solidify, shrinkage, pressure, tack, weld

- Solidify (S37): holds a pattern piece in its draped shape during simulation (example: creases behind jeans knee). Apply via right-click on 3D pattern selected with Select/Move, or Property Editor, Bond/Skive, Solidify with Strength slider. Show/Hide 3D Color toggles the overlay color. Doc marks Ver 2.5.0 as origin. Keyable (animatable) from 2025.2.
- Pressure, Shrinkage, Tack strength, Trim weight: keyframeable from 2025.2 (S7, S22, S23). Pressure is the inflate control; no separate "puffer" tool was found in official docs. Treat puffer/quilting as achievable by: pressure + shrinkage + internal-line stitched seams with small particle distance [inference, unverified].
- Tack (2025.0): attaches trims across multiple vertices and patterns (S1).
- Weld: appears as export option (Weld/Un-weld, Weld Turned Sewing Lines in 2026.0 FBX export, Merge Vertex by Proximity at export in 12). No interactive "Weld" tool in 3D window was found. Bond/Skive (Show/Hide Bond/Skive and Skive pages listed in 3D Window Environment, S20) are sewing-edge treatments, not weld.
- Garment thickness: rendering thickness vs simulation thickness; Thin/Thick export options (2024.2, 2026.0). "Solidify" is not thickness generation; thickness is a fabric property (not fully documented in what I fetched).

### 3.14 Pleats, Lacing, Trim conversion (3D-driven authoring tools)

- Pleat Creation Tool (2025.2): knife, box, accordion pleats; number, depth, interval, fold angle with live preview; non-destructive (S22, S23).
- Lacing Tool (2026.0): see 2.11; creates a lace cord pattern routed through eyelets (S25).
- Convert Patterns to Trims/Accessories (2025.2): context menu action; rigid geometry (S22, S23).
- Trim Simulation with Collision (2024.2), Animation with trim simulation (2025.0), GPU trim simulation (2026.0).
- Soft Body: custom avatars (2025.1); GPU soft body (2025.2).

### 3.15 AI features

Sources: S16, S17, S2, S4, S21, S6.

| Feature | Version | Input | Output | Notes |
|---|---|---|---|---|
| AI Texture Generator | 2024.1 | Keywords (English), chosen fabric, parameters | Fabric texture images, compare and choose | Beta; AI Studio plug-in |
| AI Graphic Generator | 2024.1 | Keywords | Graphics applied to patterns; history; transform/rotate/scale | Beta |
| PBR Map Generator | 2024.1 | Imported texture image | Normal (adjustable intensity), displacement, opacity, roughness, metalness maps auto-applied | Works on fabrics, buttons, trims |
| AI Pose Generator | 2025.1 Beta | Text prompt or full-body photo | .pos avatar pose files | May be temporarily suspended |
| AI Pattern Drafter | 2025.1 Beta | Keywords, schematic sketch or uploaded image | Measurements plus basic pattern; selectable history | T-shirt only at launch; pants and skirts added 2025.2 (non-AI Pattern Drafter list, S7) |
| Pattern Drafter (non-AI) | 2025.1 Beta; Edit Mode 2026.0 | Garment type and measurements | Pattern | General/Simple/Custom measurement views; dropdowns for measure locations; symmetry or half symmetry |
| Auto Modular Labeling | 2025.0 era | Pattern shapes | Block labels | ML trained on pattern datasets [single source S35] |
| Auto Sewing | 2025.0 | Patterns | Sewing lines | Described by name only [no detail found] |

- Activation: AI Studio plug-in is disabled by default; personal users toggle in website My Account; enterprise admins toggle per End-User License ID (S17). "AI Feature Activation" listed in 2025.2 (S7).
- Image-to-garment: not found as an official feature. Do not claim it.
- AI texture/pose features are cloud services; a clone would need pluggable model backends (image diffusion for textures, pose-from-text, LLM-to-measurement mapping).

### 3.16 Avatar pose-matching, IK, motion (3D editing relevant)

- IK Joint Mapping (2025.0): move/rotate joints directly; IK mode for avatar joints (S1, S3).
- Auto Convert to Motion (2025.0, name only).
- AI Pose Generator (2025.1).
- Register Pose/Motion (2024.2).
- Blend Shape Animation (2026.1).
- This topic overlaps other researchers' avatar and animation slices; only mapped here for completeness.

### 3.17 Display and inspection tools in 3D window

From the official 3D Window Environment section listing (S20): Strain Map (Fit Maps), Isolate Selection, Garment Fit Properties (2025.0), Quick/Advanced Pinching, 3D Window Display Icons, Schematic Render (2025.0), Show X-Ray Joints / Avatar Joints (12), Show Statistics (12.1), Show/Hide Bond/Skive, Show Mesh, Garment Fit Maps, Show 3D Grid, Show 3D Shadow, Show/Hide 3D Color (2024.1), Skive, Show Camera, Camera Setting, Wind Controller Setting (updated above 2024.0), Ground Setting, Complete Nonlinear Simulation, Zoom Extents All (3D). 2026.0 adds Fabric-Aware Strain Maps (S24) and Display Garment Fit Maps on 2D Patterns.

## 4. Features not found in official sources (explicit gaps)

1. Official 11.1 and 11.2 feature lists: the "Making Marvelous Designer 11 New Feature List" article was not retrievable; only the 6.1.645 patch details (S13) were found.
2. Dedicated documentation for: Transform (3D), Garment to Mannequin, Weld tool, Puffer/Quilting tool, Image-to-Garment, and "Sculpt/cloth stamps" beyond the Stamp brush in Sculpt mode (S34). If they exist in a release, they are not in the sources fetched.
3. Descriptions for 2026.0 items other than Lacing, 3D Pencil, Toon, and export items; the official page lists names only (S24). Template-Based Rigging and Preserve 3D Garment Shape on 2D Scaling have no description.
4. Auto Sewing (2025.0) and Auto Seal specifics: names only from official list.
5. The press-only claim "new Sculpt Tool" in 2025.0 (S3) conflicts with Sculpt mode existing since MD 8 (S29); likely a refresh of the Sculpt mode or new brushes, not verified.
6. 2025.0 new-feature press (S3) says nine headline features but lists seven; official list (S2) shows nine key features; trust S2.
7. cgchannel.com articles (2025.1, 2025.2, 2026.0, 2026.1) block fetching by robots rules; they appear in search results but contents were not read.

## 5. Implementation priorities for the formfabric clone (derived from this slice)

Priority A (core 3D editing, in every release):
1. Dual-behavior Select/Move (physics pull vs rigid pattern arrangement).
2. Pin tools (box/lasso, attach to avatar, symmetry duplicate) as simulation constraints.
3. Pinching family: single-point, falloff-weighted soft selection (Bump/Linear, 1 to 500 mm, power -2 to 2, surface vs straight distance), brush stroke with lazy trail, simulation parameter override with blend strength.
4. Fold Arrangement via internal-line hinge gizmo.
5. 3D pen on avatar (surface snapped), edit points, Flatten (separate and merge) with auto-sewing between neighbors.
6. Auto Fitting and Re-Drape with fitting-suit correspondence and maintain-topology option.

Priority B (production features):
7. Modular system: Group/Category/Line/Style/Block data model with type and tag; structure files; block swap; notch-based sewing tags; auto labeling via classifier.
8. Sculpt mode: brush set (Wrinkle, Release, Sculpt, Smooth, Grab, Stamp, Pinch) with strength, size, focal, spacing, depth, invert, image stamps.
9. Retopology: create/edit/patch topology tools with ghost points across sewing relationships.
10. Solidify and keyframeable shrinkage, pressure, tack, trim weight.
11. EveryWear equivalent: polygon optimization, auto seal, masking, auto rigging and weight painting, UV pack and bake, local FBX export, glTF/VRM.

Priority C (newest):
12. Pleat tool, lacing tool, pattern-to-trim conversion, seamline rip with fray, 3D Pencil, off-avatar 3D pen, toon shader, blend shape animation, camera interchange.
13. AI plug-in layer with opt-in activation: texture, graphic, PBR map, pose, pattern drafter.

## 6. Sources

Official CLO / Marvelous Designer sources (primary):
- S1: https://support.marvelousdesigner.com/hc/en-us/articles/47358120307353 (2025.0, 2025.1 New Feature Overview)
- S2: https://support.marvelousdesigner.com/hc/en-us/articles/47358120307353-Marvelous-Designer-2025-0-2025-1-2025-2-New-Feature-List (2025.0 to 2025.2 list)
- S5: https://support.marvelousdesigner.com/hc/en-us/articles/47358170563481-Marvelous-Designer-2024-0-2024-1-2024-2-New-Feature-Lists
- S10: https://support.marvelousdesigner.com/hc/en-us/articles/47358241113625-Marvelous-Designer-12-12-1-12-2-New-Feature-List
- S11: https://support.marvelousdesigner.com/hc/en-us/articles/47358183558169-What-s-NEW (Marvelous Designer 12 New Feature List)
- S4: https://www.marvelousdesigner.com/support/news/view/f4db0954ebea41aab139ab52bc7ccc37 (2025.1 official news)
- S7: https://support.marvelousdesigner.com/hc/en-us/articles/47358120307353-Marvelous-Designer-2025-0-2025-1-2025-2-New-Feature-List (same as S2, 2025.2 groups)
- S8: https://support.marvelousdesigner.com/hc/en-us/articles/47358260153881 (Fold Arrangement)
- S9: https://support.marvelousdesigner.com/hc/en-us/articles/47358146159769 (Pin functions)
- S14: https://support.marvelousdesigner.com/hc/en-us/articles/47358176294809-Marvelous-Designer-12-Retopology-Improvements
- S17: https://support.marvelousdesigner.com/hc/en-us/articles/49694260744089-AI-Studio-Plug-In-Activation
- S18: https://support.marvelousdesigner.com/hc/en-us/articles/47358335130649-Auto-Fitting
- S19: https://support.marvelousdesigner.com/hc/en-us/articles/47358163705369-Quick-Pinching-Advanced-Pinching-with-Soft-Selection-Ver-2025-0
- S20: https://support.marvelousdesigner.com/hc/en-us/sections/51986433234585-3D-Window-Environment
- S21: https://support.marvelousdesigner.com/hc/en-us/articles/48920347302297 (New in 2025.1, Pattern Drafter)
- S24: https://support.marvelousdesigner.com/hc/en-us/articles/55837641308313 (Marvelous Designer 2026.0)
- S26: https://www.marvelousdesigner.com/support/news/view/448aa11bc06d4b91a63668ca1d12a96e (2026.1 official news)
- S28: https://support.marvelousdesigner.com/hc/en-us/articles/59781111736345 (New in 2026.1, Brush Pinching)
- S29: https://support.marvelousdesigner.com/hc/en-us/articles/47358375599769-Mode
- S30: https://support.marvelousdesigner.com/hc/en-us/articles/47358168697497-Select-Move-3D
- S31: https://support.marvelousdesigner.com/hc/en-us/articles/47358212337305-Edit-Line-3D-Pattern-Edit-3D-Pen-Garment
- S32: https://support.marvelousdesigner.com/hc/en-us/articles/47358340247193-Flatten
- S34: https://support.marvelousdesigner.com/hc/en-us/articles/47358335421593--Mode-SCULPT
- S35: https://support.marvelousdesigner.com/hc/en-us/articles/47358157799961 (Modular Library)
- S36: https://support.marvelousdesigner.com/hc/en-us/articles/47358417043865-MODULAR-MODE-Select-a-Block-in-the-Modular-Configurator-Window
- S37: https://support.marvelousdesigner.com/hc/en-us/articles/47358310357529-Solidify
- S33: https://support-connect.clo-set.com/hc/en-us/articles/51485211774105-Convert-to-Game-Ready-Asset-using-Everywear (official CLO-SET support)
- Search-result-only official pages (titles seen, not fetched): 47358216100121 Move with Arrow Keys 3D; 47358422800537 Create Pin; 47358416403097 Edit Block Folders in the Modular Configurator Window; 47358257345305 Does Marvelous Designer support the quad mesh.

Independent trade press (secondary, used for cross-checks):
- S3: https://digitalproduction.com/2025/05/01/marvelous-2025-now-with-extra-fur-fewer-triangles
- S6: https://digitalproduction.com/2025/08/21/marvelous-designer-2025-1-draw-wash-repeat/
- S22: https://digitalproduction.com/2025/11/18/marvelous-designer-2025-2-metahumans-pleats-keyframes/
- S23: https://cgpress.org/archives/marvelous-designer-2025-2-introduces-improved-animation-controls-trim-conversion-fbx-export-metahuman-support-and-a-new-pleat-tool.html
- S25: https://digitalproduction.com/2026/04/15/marvelous-designer-2026-0-adds-3d-pencil-and-lacing/ (note: the page text retrieved by the fetch summarizer contained the 2026.0 feature details; a later fetch of the same page for other topics returned only pricing, so treat S25 details as single-source)
- S27: https://digitalproduction.com/2026/08/25/marvelous-designer-2026-1-rips-seams/
- S12: https://digitalproduction.com/2021/10/28/marvelous-designer-11-veroeffentlicht/
- S13: https://www.cgchannel.com/?p=130909 (Feb 25 2022 patch)
- S15: https://80.lv/articles/marvelous-designer-12-launched
- S16: https://rebusfarm.net/news/marvelous-designer-2024-1-unveiling-cutting-edge-ai-features

Fetch failures: support.marvelousdesigner.com Release Notes section index (client error); cgchannel.com 2026.1 article (robots disallowed); marvelousdesigner.com 2025.2 news page (content not rendered).
