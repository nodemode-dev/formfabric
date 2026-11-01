# Marvelous Designer (MD): Rendering, Visualization and Analysis Displays

Research slice 10 of 13 for the formfabric encyclopedia. Scope: real-time viewport, render styles, lighting and environment, built-in render, snapshot and video capture, cameras, fit-analysis maps (strain, stress, fit, pressure), contact display, show/hide toggles, measure tools in 3D.

Research was capped at about 45 tool calls. Only support.marvelousdesigner.com, marvelousdesigner.com, the official MD User Guide 2024 PDF (hosted at s3.marvelousdesigner.com) and official CLO Virtual Fashion pages (support.clo3d.com) were used. Wikipedia, wikis, forums and Reddit were not used. Tags used below:

- [verified] : stated in an official MD page I fetched.
- [single source] : only one approved page states it.
- [CLO only] : documented for CLO (sibling product by the same company), NOT confirmed for MD. Do not assume MD has it.
- [unverified] : could not be confirmed; listed as a gap.

## 1. Summary

1. MD's visualization is a real-time OpenGL-style viewport (the manual never names the graphics API) with a vertical toggle toolbar in the 3D window. Display is controlled by roughly 40 independent show/hide toggles grouped as Garment, Trims, Garment Rendering Style, Garment Rendering Option, Fit Maps, Avatar, Avatar Rendering Style and Environment. [verified, 3D Window Display Icons]
2. I could find NO documentation of a separate offline ray-traced Render window with render properties (image size, samples, denoise, IBL, turntable render) for MD. CLO has one (V-Ray based, with a Render toolbar), and MD's own 2024 User Guide still contains orphan references to an "Interactive Render window" and "Render Properties", but no MD chapter describes them. An MD support community post titled "Why does Clo3d has Render, but Marvelous Designer has Closet?" exists, which suggests MD does not ship the CLO render, but I could not fetch its body. Treat "MD has a built-in V-Ray style renderer" as NOT verified. [unverified]
3. MD's replacement for offline render is a viewport "Quality Render" toggle (since Ver. 10, "adds depth to shadows and highlights"), a viewport "Schematic Render" (Alt+0, Ver. 2025.0), a 3D Snapshot (F10, multi-view capable, transparent background, optional HTML), and a Video Capture of simulation animation (Ver. 2024.0). Final-quality images are expected to come from external renderers (Marmoset, Unreal, Blender, etc.) via FBX, OBJ, Alembic, USD export. [verified for existence; internals single source]
4. Analysis maps are well documented: Strain Map (percent of stretch, Global or Fabric modes, 8 color ranges), Stress Map (kPa, gf/cm2 or mmHg, weft/warp/shear), Pressure Map (garment pressure on avatar, same units), Fit Map (three tightness categories, older versions only), and Show Pressure Points (contact points). A single "Garment Fit Properties" dialog configures range count, colors, opacity and graph type for all of them. [verified]
5. Measure tools: Avatar tapes (circumference basic and surface, tape basic and surface, linear, height), Garment measures (linear, circumference parallel to floor), Attach-to-Avatar-Tape workflow. Garment measurements can be baked into 3D snapshots since 4.1.0. [verified]

## 2. What MD is NOT (important for clone scoping)

| Claim | Status | Evidence |
|---|---|---|
| MD has a separate Render window with Render Properties, Light Properties, Image/Video properties | Not verified for MD. Documented for CLO (V-Ray). | CLO pages exist at support.clo3d.com under "V-Ray Render ...". No MD manual article found. MD User Guide 2024 mentions "Interactive Render window" only in passing (fabric Type presets "Render Only", Displacement Map "only visible in Interactive Render window", and "Seam Puckering Intensity in Render Properties" with a pointer to "Chapter 6 Presentation", which in the 2024 guide contains only Quality Render, Animation, Omniverse, Retopology, Fitting Suit, Modular, Wind, Statistics). |
| MD has real-time ray tracing | Not found in any fetched source | The 2026.1 release page and 2026.0 news page list no ray tracing. |
| MD has HDRI / IBL image-based lighting | Not found for MD | Only a "Show Light" controller and a lighting mention are documented (Section 5). |
| MD has depth of field, denoise, sample counts | Not found for MD | These appear only in CLO V-Ray docs (Noise Threshold). |

Implication for formfabric: the minimum viable "MD parity" is the viewport plus snapshot plus video capture plus analysis maps. A path-traced renderer is an optional extra, with CLO's render properties (Section 8) as a ready-made parameter reference.

## 3. Real-time viewport: display icons and shortcuts

Source: "3D Window Display Icons" (Marvelous Designer help center, titled "3D Render Style"). All entries [verified], single page, version tags as the page gives them.

### 3.1 Render modes

| Item | Shortcut | Version | Behavior |
|---|---|---|---|
| Quality Render (3D Window) | none listed | Ver. 10 | Toggle that improves render quality; slower, toggle as needed. 2024 User Guide: "Heightened, more realistic visualization of the garment in the 3D window; adds depth to shadows and highlights." Icon in the 3D window. |
| Schematic Render | Alt+0 | 2025.0 | Renders 3D garments in a schematic (technical drawing like) format. Details of line weights or colors not documented. |

### 3.2 Garment rendering style (mutually exclusive set)

| Style | Shortcut | Version | Behavior |
|---|---|---|---|
| Thick Textured Surface | Alt+1 | older | Fabric with thickness. User Guide: no shading on back face; fabric thickness and render thickness are expressed. |
| Textured Surface | Alt+2 | older | Fabric without thickness. Back face shaded with a dark gray overlay; thickness and render thickness not expressed. |
| Thick Monochrome Surface | Alt+3 | 2025.0 | Thick fabric without texture images. |
| Monochrome Surface | Alt+3 or Alt+4 (page lists both, conflicting) | 2025.0 | Fabric without texture images. |
| Translucent Surface | Alt+4 (Ver. 7.1.0) or Alt+5 (Ver. 2025.0) | 7.1.0 | Applies transparency to all fabric. Shortcut changed by version. |
| Opaque Surface | none | 5.1.4 | Listed alongside Translucent. |
| Thick Textured Surface (Back) | none | 2025.0 | Both sides in original color with thickness. |
| Random Color Surface | none | 5.0.0 | Colors each pattern a random color to tell pieces apart. |
| Wireframe on Monochrome (Thick) | none | 2024.2 | Thick untextured fabric plus pattern wireframe overlay. |

### 3.3 Garment rendering options (independent toggles)

| Option | Version | Behavior |
|---|---|---|
| Show Wireframe (formerly "Mesh", renamed in 2025.0) | old | Shows the mesh edge structure (the simulation triangle mesh). |
| Show Fur Strand | 2025.0 | Renders individual fur filaments. |
| Random Fur Strand Color | 2025.0 | Auto-varies strand colors. |

### 3.4 Additional Thickness Rendering

User Guide 2024 [verified]: a property that adds visual thickness to the fabric WITHOUT affecting physical properties. Default 0. Only visible in the Thick Textured Surface view. Separate from collision thickness (invisible) and simulation thickness. A 2026.1 release item, "Align Collision Direction with Render Direction", syncs collision thickness direction with render thickness direction. [single source for 2026.1: release page]. A patch note for 2026.1.237 (10/14/2026) fixes side faces not rendered in the 3D window when using garment thickness. [single source]

### 3.5 Garment display toggles

| Toggle | Shortcut | Version | Notes |
|---|---|---|---|
| Show Garment | Shift+C | old | Shows or hides all garments. |
| Show Archived Pattern | | 2026.0 | Shows or hides archived patterns. |
| Show Seamlines (formerly Show 3D Seamlines) | Shift+S | 2025.0 rename | Seam lines in 3D. |
| Show Internal Lines | | | Internal lines in 3D. |
| Show Graphics | | 2026.0 | Graphics on patterns. |
| Show 3D Pen (Garment) (formerly Show Line (3D Pattern)) | | 2025.0 | 3D line tools output. |
| Show Threads | | 4.2.0 | Sewing lines (threads) connecting patterns. This is the "sewing line display". |
| Show Pins | | | Pins. |
| Show Subdivisions | | 5.1.0 | Subdivision state. |
| Show Garment Measurements | | | Garment measure tape annotations. |
| Show Topology | | 5.1.0 | Retopology mesh. |
| Hide 3D Pattern | Shift+Q | | Right-click a pattern then Hide 3D Pattern, hides pieces in 3D (User Guide 2024). |
| Deactivate state colors | | | Deactivated patterns render transparent purple. |
| Show/Hide Color (3D background menu) | | 2024.1 | Toggles state colors for Freeze, Solidify, Strengthen, Layer, Subdivide; patterns and meshes separately; individual or all. [verified, single page] |

### 3.6 Trims display

| Toggle | Version |
|---|---|
| Show Button (buttons and buttonholes) | |
| Show Piping | |
| Show Bond/Skive | 2.3.0 |
| Show Puckering | 2024.0 |
| Show All Trims | |

### 3.7 Avatar display and avatar rendering style

| Toggle or style | Shortcut | Version | Notes |
|---|---|---|---|
| Show Avatar | Shift+A | | Show or hide avatars. |
| Show Arrangement Points | Shift+F | | Arrangement points. |
| Show Bounding Volumes | | | Avatar bounding volumes. |
| Show Avatar Joints (formerly Show X-Ray Joints) | Shift+X | 2025.0 | Joint display through the body (this is the only "X-ray" feature found; no general X-ray garment mode documented). |
| Show Avatar Measurements | | | Avatar tapes. |
| Show 3D Pen (Avatar) (formerly Show Line (Avatar)) | | 2025.0 | |
| Show Avatar Fitting Suit | | | Fitting suit volume. |
| Show Scene and Props | Shift+P | 2025.1 | |
| Avatar Textured Surface | | | Avatar with texture images. |
| Avatar Monochrome Surface | | | |
| Avatar Translucent Surface | | 2025.1 | |
| Avatar Show Wireframe | | 2025.1 | |

### 3.8 Environment display

| Toggle | Version | Notes |
|---|---|---|
| Show Light (formerly Show Light Controller) | 2025.0 rename | Shows the light controller gizmo in 3D. |
| Show Wind Controller | | Wind gizmo. Wind properties: strength, frequency, turbulence, animatable; 2024.0 added multiple wind controllers with an "Unlimited Bound" option limiting effect to an area. |
| Show 3D Shadow | | Real-time shadow on or off. |
| Show 3D Grid | | Floor grid. |
| Show Statistics | | Overlay of Vertices, Edges, Triangles, Faces, Patterns, Trims, Sewings, CPUs, FPS (User Guide 2024). |

### 3.9 Gaps in the display-icon documentation

- "Show pattern outlines" in 3D as a distinct toggle: not listed. Closest items are Show Seamlines, Show Internal Lines, Show Threads, Wireframe on Monochrome (Thick). [unverified]
- A general X-ray / see-through-garment mode: not listed other than Translucent Surface and joint X-ray. [unverified]
- "Toon Shader" was added in 2026.0 to preview toon-style garments, supporting VRM MToon material. [single source: 2026.0 news page]

## 4. Graphic options (user settings for viewport quality)

Source: support.marvelousdesigner.com article 47358263791769 (User Settings, Graphic Options). [verified, single source]

Path: Main Menu, Settings, Preferences, Graphic Options (Ver 11 and 2025.0 wording); older "User Settings".

| Setting | Type | Effect |
|---|---|---|
| Show Shadow | checkbox | Default for avatar shadow display. |
| Use Shader | checkbox | Turns shader on or off; shader smooths shading on objects. |
| Use Normal Map | checkbox | Uses normal maps; affects how seamlines and fabric texture look in 3D. |
| Antialiasing | level | Smoother outlines with higher values, slower rendering. Allowed values not documented. |
| Use VBOs | checkbox | Vertex buffer objects for faster rendering. Confirms the viewport is a hardware accelerated OpenGL class pipeline. |
| Image Resolution Setting | resolution cap, 2025.0 only | Maximum image resolution for rendering; larger images are downscaled only during rendering; more memory at higher values. |

Changes become the defaults on restart.

## 5. Lighting, shadows, environment, background

What the official sources confirm:

| Item | Status | Detail |
|---|---|---|
| Light controller in the 3D window | [verified] | "Show Light" toggle displays the controller (Environment Display group). |
| Light intensity | [single source, magazine] | MD Magazine issue 2 (March 2024) interview screenshot caption: "Lower the intensity to 0.20" next to "Show Light Controller". Implies an intensity float property with a fine granularity; range unknown. |
| Shadows | [verified] | Show 3D Shadow toggle; Show Shadow default in Graphic Options. Shadow is for avatar per Graphic Options text; garment shadow behavior not specified. |
| Background color / environment map / HDRI for the viewport | [unverified] | No MD article found. The fetch of "3D Window Environment" section index returned only the section name. |
| Light properties in Property Editor | [unverified for MD] | Not found for MD. |
| Quality Render | [verified] | Adds depth to shadows and highlights; ON/OFF image comparison in User Guide. |

CLO reference only [CLO only]: Light Properties (V-Ray Render) has Environment Map (six default maps, image based) and Light Intensity (default 1.00, range 0.00 to 4.00), with Save/Open of a light configuration. This is the closest documented model of IBL controls from the same vendor, and is a sensible schema to copy if formfabric adds environment lighting.

## 6. Cameras and views

### 6.1 Default camera (Property Editor) [verified, single article: Camera Setting]

Access: Main Menu, Preferences, Camera Properties, Default Camera (Ver 11: Settings/Preferences, Preferences, Camera); or right-click the 3D window, Camera Properties, Default Camera; or Object Browser, Scene, Root, 3D Simulation, Default Camera.

| Parameter | Meaning | Range or default |
|---|---|---|
| Field of View | Lens view range | not documented |
| Type | Spherical (object-centered) or Absolute (X, Y centered) | enum |
| Clip Plane Near | Nearest capture distance | not documented |
| Clip Plane Far | Farthest capture distance | not documented |
| Spherical: Distance | Zoom in or out | not documented |
| Spherical: Horizontal Angle | Orbit horizontally | not documented |
| Spherical: Vertical Angle | Orbit vertically | not documented |
| Absolute: Horizontal Shift | Move objects horizontally | not documented |
| Absolute: Vertical Shift | Move objects vertically | not documented |

Not documented: orthographic vs perspective projection toggle, per-camera lens in mm. A 3D snapshot dialog exposes preset camera names (Section 7.1). Related articles named but not fetched: "3D Garment View Controls", "Show Camera".

### 6.2 Custom View (saved cameras) [verified, single article]

Access: Main Menu, Display, Viewpoint, Custom View; or right-click the 3D window, Custom View.

| Operation | How |
|---|---|
| Create | Rotate 3D window, open Custom View window, click the camera icon at top right; the current view is appended to the list. |
| Recall | Double-click a list entry; 3D window rotates to that view. |
| Open or Save All | Buttons to load or save the whole group of custom views as a file. |
| Rename | Right-click, Rename. |
| Delete | Right-click, Delete. |
| Delete All, Overwrite | Right-click menu, since Ver. 4.1.0. |

2026.1 added "Display State for Custom View" (saves 3D display states with a custom view so scene states stay consistent across the rendering pipeline) and "Camera Import and Export" via FBX and glTF/GLB. [single source: release page]. Clone implication: a saved view = camera transform plus (from 2026.1) a snapshot of display toggles.

### 6.3 View controls

Standard orbit, pan, zoom with mouse. The only manual page found is a video tutorial landing page ("2. View Control") with no text; shortcuts for numeric preset views were not retrieved. [unverified]

## 7. Snapshot, Video Capture, Turntable

### 7.1 3D Snapshot [verified, article "3D Snapshot"]

Path: Main Menu, File, Snapshot, 3D Window (F10). Flow: Save File dialog (name and location), then the Snapshot window with options, then Save, Cancel or Reset.

Single view options:

| Option | Detail |
|---|---|
| Image Size Preset | Preset list or custom |
| Orientation | Portrait or Landscape |
| Width, Height | Pixels or chosen unit |
| Unit | Measurement unit |
| Resolution (ppi) | Auto adjusts to paper size since 3.2.0 for print |
| Transparent Background | Alpha channel output |
| Save HTML File | Also writes an HTML file of the snapshot (MD10 and later) |
| Reset | Program defaults |

Multi-view options:

| Option | Detail |
|---|---|
| Camera views | Front, 3/4 Left, Right, Left, Back, 3/4 Right, Top, Bottom, Custom (hover preview, click camera icon to capture a custom view) |
| Layout View Count | 2 to 10 |
| Layout | Horizontal or Vertical when combined |
| Save Separate Images | Each view as its own file |
| Preview | Side by side, per image orientation, refresh, open or save a settings file (.mvs) |
| Transparent Background, Save HTML File | as single view |

Since 4.1.0, garment measurements are included in snapshots if Show Garment Measurements is on. This is the tech-pack screenshot route: multi-view orthographic style sheet (Front, Back, Left, Right) with measurement overlays and a transparent background. Image file formats are not listed in the article. [unverified]

Python scripting [single source, official support community reply]: `mdsa.set_capture_screen_width_and_height(w, h)` and `mdsa.capture_screen(path)` take a 3D view capture at a chosen pixel size. A feature request for frame-sequence snapshot was answered "not available at this time" (community thread; vendor staff response).

### 7.2 Video Capture [partially verified]

- Introduced in 2024.0: "Record a video of the animation seen in the 3D window" (User Guide 2024) and "Exporting video of your simulated garments in motion is now possible" (MD Magazine 2024). [two official sources agree]
- A manual article "Video Capture (ver. 2024.0)" exists (linked from Animation Changes page) but I could not retrieve it. Codec, container, resolution, FPS options for MD: [unverified].
- Animation timeline context from User Guide: Recording on/off, skip to start or end, Loop, Merge tracks, Cut Anim Layer, Delete keyframe, Playback speed, Create Keyframe, Time Slider with animation layers; Quick Menu: Play Type (Frame stepping or Real Time), Current, Start, End, Scene Time Warp, Display Unit, FPS.

### 7.3 Turntable

No MD turntable article found. [unverified for MD]. CLO reference [CLO only]: Image/Video Render Properties offers Turntable image set with Image Count, Start and End Frame, Direction (Clockwise or Counterclockwise), and Video with Total Duration, Frame Duration, Entire Region or Play Region, FPS; GIF turntables allow transparent background; video codec MP4 or AVI on Windows, MOV on Mac. A CLO video-capture help page mentions Lagarith Lossless and Xvid MPEG-4 codecs and a Rotation Axis option for turntable capture (user comment on that page, weak evidence).

## 8. CLO built-in render (reference model, [CLO only])

Included because MD's 2024 guide has residual references and because the team may want a path-traced mode. None of this is confirmed for MD.

| Group | Parameter | Values |
|---|---|---|
| Render Properties | Engine | CPU, GPU (CUDA), GPU (OpenCL); engine option absent on Mac |
| | Noise Threshold | default 0.01, range 0.001 to 0.015; higher is noisier |
| | Max Render Time (min) | caps per image time; defaults render about 2 minutes on average |
| Light Properties | Environment Map | 6 defaults, image based |
| | Light Intensity | default 1.00, range 0.00 to 4.00 |
| | Save/Open | persisted light setup |
| Image/Video Properties | Media type | Image, Turntable image set, Video |
| | Viewpoint | Current View or a saved Custom View |
| | Colorways, Grading Sizes | all or selected |
| | Image Size, Orientation, Width, Height, Maintain Ratio, Resolution ppi | presets or custom |
| | Background | solid color, texture image, or transparent |
| | File format | JPG, PNG |
| | Panoramic Custom Views, Panoramic Colorways | combine renders into one sheet |
| | Video codec | MP4 or AVI (Windows), MOV (Mac) |
| MD tie-in | Fabric Type "Render Only" presets, Displacement Map, Seam Puckering Intensity | MD User Guide says these appear only in the Interactive Render window or Render Properties |

CLO documents the engine as V-Ray (page titles "V-Ray Render ..."). Max sample counts, denoiser, DOF were not on the fetched pages.

## 9. Analysis displays: Garment Fit Maps

Access: 3D window vertical toggle toolbar, hover (or long press) the Garment Fit Maps icon (the page says 5th icon for MD, upper left in newer builds), pick a map. Only one map is active at a time. Click a point on the garment with a map on and its value is shown at the picking point. [verified, multiple sources]

### 9.1 Map catalog

| Map | Shortcut | Availability | Meaning | Default scale |
|---|---|---|---|---|
| Stress Map | Alt+6 | "below Ver. 12" in the display-icons page; present in 2025.0 Garment Fit Properties | External stress that distorts the garment, force per unit area along the fabric grain | Eight colors; red 100 kPa strongest, blue 0.00 kPa none (older docs; CLO page says the same) |
| Strain Map | Alt+7 | all versions | Percentage of fabric stretch against non-stretched state | Eight colors; blue 100% (no stretch), red 120% default upper limit; User Guide notes many fabrics exceed 120% so the scale should be adjusted per fabric |
| Fit Map | Alt+8 | "below Ver. 12" | Tightness categories | Red Can't Wear, Orange Very Tight, Yellow Tight (only these three categories documented; no color for loose) |
| Pressure Map (renamed from Show Pressure Points in 2025.0) | none | 2025.0 onward | Pressure the garment exerts on the avatar | Units kPa, gf/cm2 or mmHg |
| Show Pressure Points | none | up to 2024.2 | Toggle showing contact points between garment and avatar | no scale; just a contact-point display |

Contact point display: "Show Pressure Points" is the contact-point overlay (shows where avatar touches garment). In 2025.0 it was superseded by the Pressure Map. [verified]

### 9.2 Garment Fit Properties dialog (ver. 2025.0) [verified, single article]

Path: Main Menu, Settings, Garment Fit Properties.

| Setting | Applies to | Values |
|---|---|---|
| Range Count | all three maps | 8 or 4 color bands |
| Color | each map separately | per band color editable |
| Opacity | each map separately | adjustable (range not stated) |
| Graph Type | all | Gradation, Section, Mesh, Face, Point |
| Range units | Stress, Pressure | kPa, gf/cm2, mmHg |
| Range unit | Strain | % |
| Show | Stress and Strain | Weft, Warp, Shear (any combination) |

Graph types: Gradation blends smoothly; Section uses discrete bands; Mesh colors the mesh lines; Face colors garment surfaces; Point colors mesh points.

Usage guidance (same article): raise the strain percentage ceiling for very stretchy fabric, lower for non-stretch; red areas in Strain Map are not necessarily critical, confirm with Stress Map; test several poses (standing, sitting, running). Pre-checks: right fabric applied, avatar size matches measurement chart (Avatar Editor, Avatar Size), pose chosen, Simulation Mode set to Fitting (Accurate Fabric) or Animation (Stable), re-simulate after edits.

### 9.3 Strain Map detail (2026 article) [verified, single source, dated March 31, 2026]

| Mode | Definition |
|---|---|
| Global | Absolute strain against a fixed standard, fabric independent. Shows how much the pattern itself deformed. |
| Global Range | Span from 100% base to a user-defined Maximum split into 8 segments; each segment color editable via color chip. |
| Fabric | Strain measured against each fabric's own tensile limit (fit assessment reflecting material). |
| Fabric Range | Span from 100% to each fabric's tensile limit split into 8 segments with customizable colors. |
| Stretch limit | Auto-computed from the fabric's tensile properties (point where fabric can no longer structurally expand under consistently strong force, "too uncomfortable to wear"); can be typed in manually. |

Point readouts when you click the garment:

| Field | Meaning |
|---|---|
| Dominant Direction | Fabric direction (warp, weft or bias) closest to its stretch limit |
| Actual Stretch | Absolute stretch amount in that direction |
| Force | Percent of stretch limit reached; 100% is the limit, above 100% exceeded |

Clone implications: per-triangle strain tensor in material frame (weft, warp, shear), normalized by a per-fabric limit; an 8-bin (or 4-bin) legend; picking readout. Stress and pressure need a constitutive model (kPa from the fabric's stiffness parameters) and avatar contact forces respectively.

## 10. Measure tools in 3D

Source: MD User Guide 2024 chapter 4 "Measurement Tools" and the display-icons article. [verified, single primary source plus the manual section listing "Measure (16 articles)"]

### 10.1 Avatar tapes

| Tool | Behavior |
|---|---|
| Edit Measure (Avatar) | Select and edit existing measures and tapes; right-click for menu |
| Basic Circumference Measure | Tape around the avatar (waist, arm, leg, neck) |
| Surface Circumference Measure | Tape around the exact contour of the avatar surface |
| Basic Tape Measure | Linear length along the avatar (arm length, back length, inseam) |
| Surface Tape Measure | Length over and around the exact contour |
| Linear Tape Measure | Straight line through the avatar between two end points; does not conform to the surface |
| Height Measure | From the floor to a point (floor to HPS, HPS to waist) |
| Attach to Measure (Avatar) | Constrain a garment edge or seam to an avatar tape (below) |

Attach workflow: place a tape; choose Attach to Measure (Avatar); click the seam or edge (blue highlight); garment turns transparent and segment shows red; click the tape to attach; simulate to pull the garment to the tape. Edges with several segments need each segment paired; the circumference is evenly distributed (example: a waistband-less pant top edge needs 4 segments). Detach: right-click then Detach from Avatar Measure, or Detach All From Avatar Measure.

### 10.2 Garment measures

| Tool | Behavior |
|---|---|
| Edit Garment Measure | Select and delete garment measures |
| Linear Garment Measure | Straight line through the garment between two points |
| Circumference Garment Measure | Tape around a 3D garment parallel to the floor |

Display toggles: Show Garment Measurements, Show Avatar Measurements. Garment measurements appear in 3D snapshots (4.1.0+).

## 11. Sewing line and fit-checking displays

| Feature | Detail | Source |
|---|---|---|
| Show Threads (4.2.0) | Draws sewing lines connecting patterns | display icons |
| Show Seamlines (Shift+S) | Seam lines drawn on 3D garment | display icons |
| Sewing mistake check | User Guide chapter 1 references highlighting seam segments to catch mistakes | User Guide (limited text) |
| Statistics overlay | Vertices, Edges, Triangles, Faces, Patterns, Trims, Sewings, CPUs, FPS | User Guide |
| State colors | Freeze, Solidify, Strengthen, Layer, Subdivide colors on patterns and meshes; show or hide | Show/Hide 3D Color |

## 12. Tech pack screenshot workflow (inferred from verified parts)

1. Set Monochrome or Textured style, turn on Show Garment Measurements and Show Seamlines.
2. File, Snapshot, 3D Window (F10), Multi-View with Front, Back, Left, Right or 3/4 views, Transparent Background on, ppi per print size.
3. Optional Save HTML File and Save Separate Images.
4. For fit comparison sheets, enable Strain Map or Pressure Map first; the map color is part of the viewport and therefore captured. (Capture of maps in snapshots is not explicitly documented; the user-visible 3D window is what is saved. [unverified])

## 13. Build checklist for formfabric

| Priority | Feature | Spec to copy |
|---|---|---|
| P0 | Style toggles: textured, thick textured, monochrome, translucent, random color, wireframe overlay | Section 3.2 |
| P0 | Thickness render separate from physics thickness (default 0) | Section 3.4 |
| P0 | Strain, Stress, Pressure maps with 8 or 4 bins, graph types, units, weft/warp/shear filter, click-to-probe | Section 9 |
| P0 | Snapshot with presets, transparent bg, ppi, multi-view 2 to 10 | Section 7.1 |
| P1 | Saved camera list with display state | Section 6.2 |
| P1 | Seam/thread/internal line/pin/trim/avatar show-hide toggle set with shortcuts | Section 3 |
| P1 | Measure tapes and attach-to-tape constraint | Section 10 |
| P1 | Default camera: FOV, spherical vs absolute, near and far clip | Section 6.1 |
| P2 | Video capture of recorded simulation | Section 7.2 |
| P2 | Quality Render (better shadows and highlights; likely SSAO or PCF shadows) and Schematic Render | Section 3.1 |
| P3 | Offline path tracer with IBL, noise threshold, max time, turntable | Section 8 (CLO) |

## 14. Open questions (not verifiable within budget)

- Does current MD (2025.0 to 2026.1) ship any offline Render window? Contradictory residue in docs; the community thread body could not be fetched.
- MD Video Capture article body (codecs, resolution, FPS).
- Light controller parameters (direction, color, ambient) and any viewport environment map or background color setting.
- Value ranges for Antialiasing, Opacity, FOV and clip planes.
- Numeric view-preset shortcuts and a proper camera orthographic mode.
- Exact definition and color of the Pressure Map scale defaults.
- The Fit Map is tagged "below Ver. 12" so it may have been dropped in newer versions in favor of Strain, Stress and Pressure maps with Garment Fit Properties; not confirmed in release notes.

## Sources (URLs fetched)

Primary MD help center and site:
- https://support.marvelousdesigner.com/hc/en-us/articles/47358200025497 (3D Window Display Icons / 3D Render Style)
- https://support.marvelousdesigner.com/hc/ja/articles/47358286228505-Garment-Fit-Maps
- https://support.marvelousdesigner.com/hc/en-us/articles/47358145163033-Garment-Fit-Properties-ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/56518392287513--Fit-Maps-Strain-Map
- https://support.marvelousdesigner.com/hc/en-us/articles/47358354219033-3D-Snapshot
- https://support.marvelousdesigner.com/hc/en-us/articles/47358382562713-Camera-Setting
- https://support.marvelousdesigner.com/hc/en-us/articles/47358348786969-Custom-View
- https://www.marvelousdesigner.com/manuals/articles/47358311619225 (Show/Hide 3D Color)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358263791769 (Graphic Options)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358126161177-UI-Changes-Ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358170563481 (2024.0 to 2024.2 feature lists)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358154415897-Animation-Changes-Above-ver-11
- https://support.marvelousdesigner.com/hc/en-us/articles/47358202002073-2-View-Control (video landing only)
- https://support.marvelousdesigner.com/hc/en-us/categories/51985515993625-Manual (section index)
- https://support.marvelousdesigner.com/hc/ja/community/posts/48276654260761/comments/48276624520089 (Python capture thread)
- https://support.marvelousdesigner.com/hc/ja/community/posts/48276756027673/comments/48276726442137 (frame sequence snapshot thread)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358145573401 (Unreal workflow tips)
- https://www.marvelousdesigner.com/products/release (2026.1 features)
- https://www.marvelousdesigner.com/resources/patch-note (2026.1.237 fix)
- https://www.marvelousdesigner.com/support/news/view/ed12ee696c144726ab0e1cb5abaf2242 (2026.0, Toon Shader)
- https://www.marvelousdesigner.com/explore/guide/marvelous-designer-vs-blender-3d-clothing (no render info)
- https://s3.marvelousdesigner.com/newmdweb/case/20240626/MD+User+Guide+2024.pdf (official MD User Guide 2024, read via pdftotext)
- https://s3.marvelousdesigner.com/mdweb/magazine/0780-MD_2nd_edition-Digital-60-v2pp.pdf (MD Magazine issue 2, March 2024)

Official CLO Virtual Fashion (sibling product, used only for [CLO only] items and a Fit Maps cross-check):
- https://support.clo3d.com/hc/en-us/articles/360000436368-Garment-Fit-Maps
- https://support.clo3d.com/hc/en-us/articles/115015559027-V-Ray-Render-Properties
- https://support.clo3d.com/hc/en-us/articles/115015558727-V-Ray-Render-Light-Properties
- https://support.clo3d.com/hc/en-us/articles/115014785207-Edit-V-Ray-Image-Video-Properties
- https://support.clo3d.com/hc/en-us/articles/115002692447-Video-Capture-3D-Garment-Turntable

Fetch attempts that failed or returned nothing usable: MD community post 48277113464473 (Render vs Closet), manual.marvelousdesigner.com Confluence (robots disallowed), several index pages.
