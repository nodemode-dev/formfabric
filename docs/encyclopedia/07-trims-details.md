# Marvelous Designer (MD): Trims, Garment Details and Finishing Tools

Research slice 07 for the formfabric clone effort. Scope: buttons, buttonholes, zippers, topstitch, puckering, piping, binding, elastic, shirring, fold tools, pressing, seam tape, bond/skive, trims, graphics, fur, pleats, padding/pressure, tacks, lacing, and related finishing tools.

## Summary

- MD splits "details" into three families that a clone should model separately:
  1. Style-based hardware objects (button, buttonhole, zipper, topstitch, puckering, piping, binding). Each has a named **style** held in the Object Browser, assigned to placed instances, and saved to a preset file.
  2. Per-line or per-pattern attributes set in the Property Editor (elastic, fold angle/strength, tension, pressure, bond, skive, seam tape, freeze, solidify, deactivate). These alter the physics solver directly.
  3. Imported rigid geometry (trims .TRM from OBJ, buckles and rings as avatar-type objects) glued to a cloth surface point.
- Most of the official manual is terse. Many numeric ranges are NOT published. Where a range exists it is listed below with its source. Everything else is marked "not specified".
- Version labels used: the manual is versioned (MD 10, 11, 2024.0 to 2026.0). Source article ids are in the Sources list.
- Items asked for but NOT found in the official manual during this research (do not treat as MD features without further verification): Smocking tool, Hem tool, Rib knit tool, Bead/sequin feature, Fold Back tool, Line weld tool (only an export "Weld Turned Sewing Lines" option was found). See "Gaps" section.

## 1. Buttons and Buttonholes

### 1.1 Tools (3D Toolbar and Main Menu > Materials > Button)

| Tool | What it does | How operated |
|---|---|---|
| Button | Places a button on a garment or pattern | Click a point on the 3D garment or 2D pattern. A red dot (3D) or blue cross (2D) previews placement. |
| Buttonhole | Places a buttonhole | Click a point. A dialog asks for the number of garment layers, then OK. |
| Buttonhole by measurement | Exact positioning | Right-click on the 2D pattern; a Moving Distance window opens; a preview follows the mouse and updates with the typed value. OK to place. |
| Select/Move Button | Select, move, edit buttons in 2D or 3D | Property Editor shows instance properties. |
| Fasten Button | Links a button to a buttonhole | Click button then buttonhole; both highlight yellow; a purple dotted line marks the relationship and a lock icon appears at the button top right. Then Simulate. |
| Unfasten Button | Reverses fastening | Listed as related article (not fetched). |
| Create Buttonholes along Pattern Outline | Batch placement | Listed as related article, ver 5.1 (not fetched). |
| Register Custom Button / Buttonhole | Custom OBJ button or image buttonhole | Main Menu > Materials > Button > Register Buttonhole; or Object Browser > Buttonhole tab > style > Property Editor > Shape > plus icon. Name + image required. Saved under Assets > Preferences > Buttons_Preset. |

### 1.2 Button style properties (Object Browser > Button tab > Property Editor), ver 2025.0

| Property | Description | Range |
|---|---|---|
| Name | Style name | text |
| Shape | Button type from dropdown; plus icon registers OBJ button or thread; CONNECT icon opens the online store | list |
| Thread Type | Cross, Parallel, Square | enum of 3 |
| Width (mm) | Button width | not specified |
| Thickness (mm) | Button thickness | not specified |
| Weight (g) | Mass used in simulation | not specified |
| Link to Fastened Buttonhole | ON/OFF | bool |
| Material | Base Color, Base Color Map (with Recolor, Desaturate, Switch Color), Normal Map (Intensity), Displacement Map (Amount, Shift, Clipping, Particle Distance, Keep Continuity for welded OBJ), Opacity (+ Opacity Map RGB or Alpha mode), Reflection (Roughness Intensity, Roughness Map, Map Intensity, Invert, Reflection Intensity, Metalness, Metalness Map) | Normal Map Intensity -100 to 100; Opacity 0 to 100; others not specified |
| Use Same Material as Button (thread) | Toggle; off lets thread have its own Texture, Color, Glossiness, Opacity | bool |
| Polygon Optimization (2024.0) | Count of polygons, or Plane (renders as a texture; Count disabled when Plane on) | not specified |

### 1.3 Button instance properties (Select/Move Button)

| Property | Description |
|---|---|
| Angle | Rotation of the button |
| Thread Length (mm) | Length of the thread shank/stitching |
| Style | Choose which button style applies |
| Collision | ON/OFF; set OFF to fix location and prevent collision with patterns |
| 2D Measurements | Shows position and size in the 2D pattern; Left/Right/Up/Down checkboxes |

Style assignment: select a button, pick a style in the Property Editor; or Object Browser > Button tab > "Assigned to Selected Patterns"; or "Assigned to New Fabric" to create a new style and apply at once.
Style right-click menu: Delete (only if unassigned), Delete All Unused, Rename, Reset.

### 1.4 Buttonhole properties

| Property | Description | Range |
|---|---|---|
| Style | Buttonhole style | list not given |
| Angle | Rotation | not specified |
| Z-Offset (mm) | Lift above surface | default 0.13 mm (2024.2 added the option) |
| Bind Position | Where fastened button and hole are located; individual property since v2.5.0 (buttons and holes editable separately) | n/a |
| 2D Measurement | Left/Right/Up/Down position readouts | bool each |

### 1.5 Simulation interaction

- A fastened button is pulled to its buttonhole during simulation (lock icon), so it behaves as a constraint between cloth point(s) and the button rigid body.
- Official advice: sew the garment closed before adding and fastening buttons for a more stable simulation, then remove the sewing once the buttons are fastened. [single source: User Guide 2024 PDF]
- Collision toggle on the button decouples it from pattern collision.
- Not found: shank property, sew-through hole count, button size presets (button size is Width/Thickness).

## 2. Zippers

### 2.1 Create and operate

| Step | Detail |
|---|---|
| Open | 3D Toolbar > Zipper, or Main Menu > Materials > Zipper > Zipper |
| First side | Click the 3D garment or 2D pattern edge, move the cursor; segment highlights light blue by direction; double-click the end point |
| Second side | Click the other edge; a blue point appears at a distance equal to the first half length; double-click it |
| Result | Zipper with tape, puller, slider and stopper |
| Direction | The direction the tool is drawn is the direction the zipper zips |
| Anchors | A single click while hovering drops anchor points |
| Simulate | The zipper simulates closed |
| Unzip | Select/Move tool, Gizmo inactive, drag the dark grey reference along the tape. Simulation must be off to move the puller (PDF guide). |
| Cancel | Esc or Ctrl+Z cancels all; Delete or Backspace cancels the last action |

### 2.2 Zipper tape properties (Select/Move > click tape)

| Property | Description | Range |
|---|---|---|
| Name | Rename | text |
| Line Length | Length of the tooth line | not specified |
| Preset (2025.0) | Custom, Plastic_01, Plastic_02, Metal, Nylon, Reverse_Nylon, Invisible | enum |
| Texture (2025.0) | Teeth type: Texture or OBJ | enum |
| Size (v11), Teeth Width, Total Width, Width, Thickness | Dimensions; the texture image changes with entered Width | not specified |
| Particle Distance | Mesh resolution of the tape | not specified |
| Fasten Zipper | Zip or unzip | n/a |
| Fold | Strength and angle of the fold used for the sewing between zipper and pattern | not specified |
| Material | Type, texture, color, opacity of teeth and tape | n/a |
| Physical Property | Physical property of the selected teeth | n/a |

### 2.3 Slider, puller, stopper properties

| Property | Description |
|---|---|
| Type (2025.0) | One Way (one slider+puller) or Two Way (two sliders+pullers moving in opposite directions) |
| Direction (Two Way only) | Bottom to Bottom (sliders near the end stopper) or Head to Head (stay at original position); both can lock per 2025.1 |
| Shape | Slider shape, Puller shape, Stopper (Open End or Closed End, plus Top and Bottom shapes) |
| Specification | Gauge (size by gauge or percent), Weight (g) (v4.2); 2025.1 allows sizes and weights per component |
| Fasten Zipper | Reverse Zip Direction; Move to Opposite Side (which side of tape slider sits when open) |
| Material | Per component; uncheck "Use Same Material as Slider" for puller or stopper |
| Polygon Optimization (2024.0) | Count, or Plane (texture) |

### 2.4 Zipper styles (2025.1, ZPS files)

- Styles are managed in the Object Browser; new styles start from the default style and can be created from the Object Browser or in the 3D window; duplicate with settings; save and open as `.ZPS`; editing a style updates every zipper using it; each component (chain, slider, puller, stopper) can be named; right-click Delete, Rename, Reset.
- Library: Library > Hardware and Trims > Zippers holds extra tapes, sliders, pullers and stoppers.

### 2.5 Simulation interaction

- Zipper tape is a physical strip sewn to the pattern edges; the "Fold" strength/angle controls the stiffness of that sewing joint. Slider position is a constraint parameter along the tape. Weight of the slider (g) is an input to dynamics.
- Clone implications: model as a parametric strip with a scalar `zip_position` in [0,1], two sides, with gauge and weights.

## 3. Topstitch

### 3.1 Tools

| Tool (shortcut) | Behavior |
|---|---|
| Segment Topstitch (K) | Point to point on a segment; affected by segment points; like Segment Sewing |
| Free Topstitch (L) | Ignores segment points and corners; click start, click end; like Free Sewing |
| Seamline Topstitch (;) | Along both sides of a seam or straddling it; perforated lines show the target seam; cannot traverse multiple seamlines; placement can be limited to one side via Edit Topstitch |
| Edit Topstitch (J) | Assign, lengthen, shorten, move, delete. Selected lines highlight bright pink. Drag the circular end point to resize, drag the middle to move. Delete key deletes. |

- Styles live in Object Browser > Topstitch tab; add by drag and drop from Library > Hardware and Trims > Topstitch, or the "+ Add" button then Open. Assign by selecting lines in 2D and clicking the Assign arrow next to the style name. To delete a style, reassign its segments elsewhere; a trash icon appears.
- Symmetry: topstitch applies symmetrically if pattern symmetry is on.

### 3.2 Topstitch style properties (ver 2025.0)

| Property | Description | Range |
|---|---|---|
| Name | Style name | text |
| Type | OBJ (3D mesh) or Texture (material effect) | enum |
| Offset | Distance from the garment | inches (dropdown) or mm (input) |
| Stitch Count | Number of parallel stitches; each can have its own material; "Use Same Specification" / "Use Same Material" | 1 to 5 |
| Shape | Bartack, Buttonhole, Single, Overlock, Pickstitch, Zigzag; plus icon registers a custom OBJ | enum |
| Length (SPI), Space, Thread Thickness | Stitch length expressed as stitches per inch (example SPI-4; SPI-11 noted as common for standard garments) | SPI based |
| Stitch Width | Only for types with width: Overlock, Zigzag, Buttonhole, Bar Tack. Types without: Default (Single Needle), Default Offset 0, Pick Stitch. Texture-based stitches use Material > Preset (e.g. 5 mm) | mm |
| Material | Base Color, Base Color Map, Desaturate, Color Switch, Normal Map (Intensity -100 to 100), Displacement Map (Amount, Shift, Clipping, Particle Distance, Keep Continuity), Opacity 0 to 100 with Opacity Map | as listed |
| Reflection | Roughness Intensity, Roughness Map, Map Intensity, Invert, Reflection Intensity, Metalness, Metalness Map | not specified |
| Configuration | Face (side), Number of Lines, Distance between lines (in, mm, cm) | not specified |
| 3D (Exaggerate Topstitch) | Shows a caved (recessed) line like a real stitched garment; Normal Map, Intensity (%), Thickness | not specified |

### 3.3 Topstitch instance properties (2D, per line)

| Property | Description |
|---|---|
| Name / Topstitch Property | Rename; choose style |
| Extend (Start/End) | Extend to pattern outline |
| Corner | Curved (only with Free Topstitch), Right Angled, Smooth (seamline context) |
| Z Offset (mm) | Distance from pattern surface |
| Placement (seamline topstitch) | Seam A, Seam B, Seam Both |
| Length | Edit Topstitch drag handles |

### 3.4 Custom OBJ stitch requirements

- Face toward the Y axis; attachment point at origin (0,0,0); units mm; under 1,000 vertices recommended; ground distance 0 recommended (adjustable in MD).

### 3.5 Simulation interaction

- Topstitch is primarily a visual and render feature (mesh or normal-mapped texture). It does not by itself change the physical properties of the cloth. [inference from manual: no physical parameters listed]. The 3D shadow option fakes the recess.
- Binding has its own Topstitch style slot (see 5).

## 4. Puckering and Seam Puckering

| Aspect | Detail |
|---|---|
| Tools | Segment Puckering, Free Puckering, Seamline Puckering, Edit Puckering. Same interaction as topstitch tools. Edit lines highlight bright purple. |
| Style properties | Material, Density (%), Width (Object Browser > Puckering tab). Puckering textures in Library > Hardware and Trims > Puckering; drag into the style Texture field. |
| Seamline properties | Name/Property, Extend, Offset (mm) (optionally includes Puckering Width), Z Offset (mm), Placement Seam A / Seam B / Both |
| Render | Seam Puckering Intensity in Render Properties makes puckering more dramatic. |
| Simulation | Puckering is a texture/appearance effect; the manual describes it only as a material and render property. [inference] |
| Deletion | Reassign lines to another style; trash icon appears |

## 5. Piping

| Aspect | Detail |
|---|---|
| Tool | Piping (3D Toolbar). Works like Zipper tool: click start, hover, double click to end; single click drops anchors. 2D and 3D windows. Edit Piping selects and deletes. |
| Properties | Name; Line Length (read only); Width (mm) 1 to 15; Particle Distance (mm) 1 to 5; Finishing Start / End closed (Closed End option added 2024.2); Fabric (dropdown, edit with Edit Texture); Hide |
| Access | Select/Move or Edit Piping; right-click for more options |
| Simulation | Piping is a cloth strip attached to the seam or edge and simulated (Particle Distance is solver mesh resolution). |

## 6. Binding

| Property | Range / Options |
|---|---|
| Create | Main Menu > Materials > Binding > Binding or 3D Toolbar; eligible outlines and internal lines show as dashed; click start, double-click end |
| Type | Under (inside), Over, Both |
| Width | 1 to 50 mm |
| Length | 50% to 200%, or a specific value |
| Particle Distance | 4 to 20 mm |
| Add'l Thickness - Collision | adjustable, no range |
| Fabric | any fabric in the Object Browser |
| Grain Direction | Bias, Warp, Custom (with Angle) |
| Topstitch | any style from the Topstitch tab |
| Sewing Line Type | editable |
| Extend | Extends binding after partial-segment placement (applies when both segment angles exceed 180 degrees) |
| Right-click | Delete; Reset Arrangement (fixes unstable simulation); Deactivate/Activate; Freeze/Unfreeze; Flip Normal |

Binding is a real simulated strip (has collision thickness, particle distance, freeze).

## 7. Elastic

| Aspect | Detail |
|---|---|
| Where | Edit Pattern or Transform Pattern select outer or internal line(s) > Property Editor > Elastic checkbox |
| Highlight | Elastic segments show green in 2D |
| Strength | Intensity of elastic; same ratio behaves differently at different strengths. No numeric range. |
| Ratio | Percentage the segment is reduced or increased; default 80 percent (a 20 percent reduction, 1:0.8); max "200% of the maximum segment" per article; minimum not stated |
| Length / Entire Length | Enter a length; applies per segment or to the combined total of selected segments |
| Special case | Ratio 100 percent acts like mobilon (stay) tape: neither stretches nor contracts [PDF] |
| Related | Seam Taping "Elastic" physical property; Tension Type/Strength/Ratio on seamlines (Ease vs Stretch) |
| Simulation | Rest length of the edge is scaled by Ratio and Strength sets stiffness of the spring-like constraint. Causes gathering and shirring wrinkles. |

## 8. Shirring and Crease

- Official article "Shirring and Crease Principle" is a video-only course page: wrinkle shape depends on fabric "Detail items in the Fabric Property". No steps or parameters in text.
- Practical workflow (combining sources): put internal lines with Elastic ratio less than 100 percent, optionally with Fold angle, then simulate. Wrinkle width depends on fabric bending and buckling properties. There is no separate "shirring tool" documented.

## 9. Fold tools

### 9.1 Fold Angle on internal lines and seams

| Property | Range |
|---|---|
| Fold Angle | 0 to 360 degrees; 180 is flat; near 0 folds toward the front (away from viewer per the PDF), near 360 folds toward the back |
| Fold Strength | 0 to 20; higher pulls closer to the set angle; 0 flattens sewn boundaries (e.g. jacket and lining) |
| Fold Rendering | Toggle soft versus sharp edge appearance |
| Sewing Line Type | Turned Off: fold controls active; Turned On: layers sewn flat, fold controls disabled |

### 9.2 Fold Arrangement

- Main Menu > 3D Garment > Fold Arrangement, or 3D Toolbar. Click an internal line or shape; a gizmo appears with red and green arrows rotating about the line as hinge axis. Works on straight and curved lines. Simulation should be off. Folds symmetric patterns symmetrically (v11). Available also after draping. Use for seam allowances, collars, cuffs.
- Tip: counteract gravity by using Strengthen before simulating.

### 9.3 Pleats

| Tool | Behavior |
|---|---|
| Pleats Fold | Create internal lines between points on opposite segments, click across them, double click; window offers knife, box, accordion; number of internal lines per pleat; fold angle presets or manual; Reverse Direction checkbox; lines recolor by fold angle |
| Pleats Sewing | Sews one pleat or continuous pleating, auto distributing sewing; sew flat attached pieces with regular sewing first |

## 10. Pressing, Seam Tape, Bond, Skive

| Feature | Operation and parameters |
|---|---|
| Press | 3D Toolbar > Press. Click first layer (turns transparent), click second layer; Sewing Lines on the first switch to Turned type; simulate to flatten bulging seam edges. |
| Seam Taping | Seam Taping tool: click a seam or marquee several; orange highlight; right-click > Delete Seam Tape; show/hide from the vertical toggle. Properties: Physical Property = Fabric or Elastic; Preset (folder to load, save icon); Detail sliders; Extend (2024.2): None, To Outline, To Seam Taping; direction arrow indicator. Purpose: keep edges from drooping. |
| Bond | Property Editor > Bond/Skive > Bond for patterns or internal shapes. Beige display (toggle in 3D Trims Display). Preset, Detail sliders. Simulates interfacing: stiffer region. |
| Skive | Same section; Percent dropdown; shaved down, softer look. |
| Strengthen, Freeze, Solidify, Deactivate | Temporary pattern settings. Freeze: ice-blue, fixed. Solidify: stiff and keeps shape in simulation, useful for bags. Deactivate (pattern only, Ctrl+J) or (pattern and sewing). Hide 3D Pattern (Shift+Q). |

## 11. Trims, buckles, rings

| Aspect | Detail |
|---|---|
| Format | `.TRM` trim files in Library > Hardware and Trims; import `.OBJ` or ZIP via right-click > Add to Workspace |
| Import dialog | Object Type = Add as Trim; Scale = original unit (wrong unit gives wrong size); Translation = load position; loads at floor |
| Attach | Glue bottle icon on the gizmo; hover on garment and click to place; gizmo icon toggles Scale tool; click glue again to re-place |
| Properties | Edit in Property Editor; Polygon Optimization (2024.0); Collide toggle and trim simulation with collision (2024.1, 2024.2); a Trim Window (2024.1) manages trims like fabrics |
| Buckles and rings | Added with Object Type = Add as Avatar. Avatar-type objects do not follow pose change, so finish avatar pose first. To thread fabric: place with gizmo, use Select Mesh to pass fabric through, set buckle Skin Offset to 0.5 to 1 mm and fabric Collision Thickness 0.5 to 1 mm, simulate. Needs testing per mesh. |
| Ribbon | Example trim from Library glued to a pattern |

## 12. Graphics, logos

| Aspect | Detail |
|---|---|
| Formats | PNG, JPEG, PDF, AI, PS (export in RGB) |
| Apply | Right-click graphic in Library > Add as Graphic, or Graphic icon in 2D/3D window and pick from File Explorer; Transform Graphic tool autoselects; click a pattern to place; in 2D an Add Graphic window sets scale and position |
| Transform Graphic | Only tool that can select and edit graphics; scale, rotate; right-click in 2D: Duplicate to Symmetric Pattern, Flip, Tile, precision scale box |
| Properties | Information; Material (maps and color like fabric); Configuration: face, Over Seamline (Tolerance must exceed the sewing line length difference), Tile (Tile and Over Seamline are mutually exclusive); Transformation: angle, dimensions, Z-Offset (float above the fabric, matters with displacement maps); 2D Measurements (from center or outline) |
| 2024.0 | Parts outside the pattern show translucent while editing |
| 2024.1 | AI Graphic Generator (keywords to graphic) |
| Emboss | 3D emboss is not a separate documented tool; relief comes from Normal/Displacement maps and Z-Offset. [inference] |

## 13. Fur

| Aspect | Detail |
|---|---|
| Enable | Object Browser > Fabric > Property Editor > Material > Type = Fur_Strand (beta, ver 2025.0). Windows only (not macOS). Not on Avatar and Stitch objects. USD export can include strand data. |
| Presets | Default, Angora, Fox, Mink Skin, Mohair, Muskrat Belly, Plush, Rabbit, Raw Edge Denim, Terry Cloth, Custom |
| Color | Melanin, Pheomelanin (scaled by Melanin), Glossiness, Softness, gradation colors (mid, tip), Position, Interpolation |
| Shape | Base Length (mm), Thickness (mm), Bend, Taper, Density (per inch squared), Segments |
| Maps | Per-pixel multipliers (black 0, white 1) for Length, Thickness, Density, Curl, Gravity, Bend, Bend Direction, Initial Direction |
| Variation | Gravity Force + Vector, Curl Radius (mm), Number of curls, Variance (Length, Thickness, Direction, Gravity, Curl Radius), Randomization Seed |
| Rules | Gravity affects only when Bend above 0; long strongly bent fur needs more Segments. Flip Normal matters for fur direction. |
| Simulation | Fur is a render-time strand layer on the cloth surface; it is not simulated as cloth (the manual describes preview in Default, Quality, Interactive render). Ranges: none published. |

## 14. Padding, quilting, puffer

| Aspect | Detail |
|---|---|
| Pressure | Property Editor > Simulation Properties > Pressure on selected patterns. Positive pushes into the back face; negative into the front face. For filled effect apply equal and opposite values to outer and inner layers; mismatch makes patterns fly away. No numeric range published. |
| Layer Clone (Over/Under) | Transform Pattern > right-click > Layer Clone; copies pattern with internal lines and auto sews; used for lining or quilting |
| Rules | Sew and construct garment first; pressure only on layer-cloned pieces |
| Quilting recipe | Layer clone, internal lines as quilting seams (sewn together), opposite pressure on both layers, simulate |
| Solidify | Not padding; makes a pattern stiff (see 10) |
| Tacks | Tack attaches two places of a pattern or two patterns; Tack on Avatar; Edit Tack moves and deletes |
| Pins | Single Pin (W+click), Pin Box/Lasso, segment pin; hold area in 3D during sim |
| Steam | Shrink or stretch areas; Shrinkage %, Size mm, Hardness; positive stretches (orange), negative shrinks (blue) |

## 15. Lacing (MD 2026.0) [secondary source]

- Lacing tool: click holes in sequence, double click or Enter to finish; creates an editable lace pattern piece in 2D with thickness, material and physical properties. Curvature slider reduces clipping. Click a hole to flip cord direction. Backspace undoes last step. [single source: DigitalProduction news article, not official manual]

## 16. Clone design implications

| Feature | Suggested data model |
|---|---|
| Style registry | Per-category dictionaries (button, buttonhole, zipper, topstitch, puckering) with preset import/export, assign, reset, delete-unused |
| Fold | Per-edge hinge with angle 0 to 360 and strength 0 to 20 (bending constraint rest angle and stiffness) |
| Elastic | Per-edge rest-length scale (default 0.8) plus stiffness |
| Pressure | Per-pattern signed scalar pressure force along the normal, per face side |
| Bond, Seam tape | Per-pattern or per-edge override of bending/stretch material |
| Button | Rigid body + fasten constraint |
| Zipper | Strip with slider scalar |
| Trims | Rigid mesh with glue anchor (barycentric on cloth triangle) and collide flag |
| Topstitch, puckering, graphics, fur | Render-layer features (mesh or normal map), no solver impact |
| Binding, piping | Auxiliary cloth strips with width, particle distance, collision thickness |

## 17. Gaps and unverified items

- No official page found for: Smocking, Hem, Rib knit, Bead/sequin, Fold Back, Line Weld tool. Related partial finds: "Weld Turned Sewing Lines" on export (secondary article); Hem and Rib knit appear only as ordinary pattern construction (internal lines, fabric physical properties) not as dedicated tools in the sources read.
- Numeric ranges not published for most properties (zipper, button, fur, pressure, topstitch).
- Seam puckering physical simulation not documented; treated as render effect.
- Pages read were the 2024 PDF user guide and English support articles; Japanese-locale article for Button Property Editor was used because the English slug was unavailable in search.
- Single-source markers: fold strength 0 to 20, piping and binding ranges, fur parameters, zipper styles ZPS, each come from one official article.

## Sources

- https://s3.marvelousdesigner.com/newmdweb/case/20240626/MD+User+Guide+2024.pdf
- https://support.marvelousdesigner.com/hc/en-us/articles/47358161151129-Marvelous-Designer-User-Guide-2024
- https://support.marvelousdesigner.com/hc/en-us/articles/47358245443993-Edit-Topstitch
- https://support.marvelousdesigner.com/hc/en-us/articles/47358128286617
- https://support.marvelousdesigner.com/hc/en-us/articles/47358155220249
- https://support.marvelousdesigner.com/hc/en-us/articles/47358394757273-TOPSTITCH-STYLE-PROPERTIES-Topstitch-Length
- https://support.marvelousdesigner.com/hc/en-us/articles/47358420787993-TOPSTITCH-STYLE-PROPERTIES-Exaggerate-Topstitch
- https://support.marvelousdesigner.com/hc/en-us/articles/47358168494745-ZIPPER-Create-Zippers
- https://support.marvelousdesigner.com/hc/en-us/articles/47358259456665
- https://support.marvelousdesigner.com/hc/en-us/articles/47358231940889
- https://support.marvelousdesigner.com/hc/en-us/articles/47358241901593-Create-Buttonhole
- https://support.marvelousdesigner.com/hc/en-us/articles/47358387376025
- https://support.marvelousdesigner.com/hc/en-us/articles/47358446869401
- https://support.marvelousdesigner.com/hc/ja/articles/47358121260441-Button-Property-Editor-ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358460972057-BUTTON-STYLE-PROPERTIES-Button-Thread-Material-Above-ver4-2-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358413628185-Apply-Button-Style
- https://support.marvelousdesigner.com/hc/en-us/articles/47358239485593-Elastic
- https://support.marvelousdesigner.com/hc/en-us/articles/47358244724761-Shirring-and-Crease-Principle
- https://support.marvelousdesigner.com/hc/en-us/articles/47358260153881
- https://support.marvelousdesigner.com/hc/en-us/articles/47358354961049-Fold-Pattern
- https://support.marvelousdesigner.com/hc/en-us/articles/47358171818521-Seamline-Property-ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358232127129
- https://support.marvelousdesigner.com/hc/en-us/articles/47358290200217
- https://support.marvelousdesigner.com/hc/en-us/articles/47358127687193--Beta-Fur-Rendering-in-3D-Window-ver-2025-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358218656921-Seam-Taping
- https://support.marvelousdesigner.com/hc/en-us/articles/47358358998809-Bond
- https://support.marvelousdesigner.com/hc/en-us/articles/47358312998809-Express-Air-Pressure
- https://support.marvelousdesigner.com/hc/en-us/articles/47358170563481-Marvelous-Designer-2024-0-2024-1-2024-2-New-Feature-Lists
- https://support.marvelousdesigner.com/hc/ko/articles/48920706512665
- https://digitalproduction.com/2026/04/15/marvelous-designer-2026-0-adds-3d-pencil-and-lacing/
- https://80.lv/articles/marvelous-designer-topstitch-guide (fetched; contained no usable parameters)
