# Marvelous Designer Encyclopedia, Slice 03: SEWING

Researcher slice: Sewing. Purpose: give the formfabric team enough verified detail to clone every sewing feature of Marvelous Designer (MD, by CLO Virtual Fashion).

Date of research: 2026-11-01. Product versions referenced in the manuals: 4.2.0 (3D sewing articles re-posted May 2025), 5.x, 6.x, 7.3, 2025.0, 2026.0, 2026.1.

## Summary

Sewing in MD is a pair-based constraint system. A Sewing Line is a pair of "halves" (two segments or sub-segments on pattern outlines or internal lines) with a direction (shown as Directional Notches). During simulation the two halves are pulled together. The only primary technical description of the mechanism found is a CLO patent (US10810794B2): mesh lattice points on the two lines are connected one-to-one by springs, one side is fixed and the other is moved to overlap it, and a Sublayer setting adds a "layer limitation" virtual force that prevents the layered patterns from interpenetrating.

Key verified facts:

1. Creation tools: Segment Sewing, Free Sewing, plus 1:N and M:N variants of each, available in both 2D and 3D windows. 1:N uses Shift, M:N uses Enter to commit groups. Delete undoes last action, Esc cancels all.
2. Length mismatch is shown live as a blue number next to the cursor, turning red above a threshold (1 inch in the current MD manual; one older CLO page says 1 mm, see Discrepancies). Separate Check Sewing Length tool bolds mismatched lines in red (default threshold 5 mm per CLO manual).
3. Sewing Line properties (Edit Sewing tool, shortcut B): Name, Sewing Line Type (Custom vs Turned), Fold Strength, Fold Angle (0 to 360, 180 flat), Tension (Ease or Stretch, Strength, Ratio max 200%), Sublayer (Reverse Direction, Front/Back), Topstitch, Puckering, 3D Seamline normal map. Presets saved as .ssp files.
4. Turned sewing = 180 degree folded, flat, two-ply. Fold Strength and Fold Angle are locked when Turned.
5. Tack (temporary point-to-point or point-to-avatar constraint) is a separate tool family, shown as yellow lines.
6. Pleats are an internal-line fold tool plus a Pleats Sewing tool that auto-groups 3 segments per pleat.
7. Auto Sewing exists (rule-based on avatar arrangement points, tops and one-pieces only). It is not AI/ML as far as documentation shows.
8. Gathering/shirring is done via Elastic on pattern lines (Ratio up to 200%) and via sewing length mismatch plus Tension, not through a dedicated "shirring" sewing tool.

Items I could NOT verify as discrete MD features (do not invent): a feature literally named "Sewing Layer", a "3-layer sewing" tool, "Merge sewing" or "Split sewing" commands, a separate "Stitch display" toggle for plain seams (only Topstitch objects and the 3D Seamline normal map are documented). Details in the "Unverified" section.

## 1. Mental model for implementation

| Concept | MD behaviour | Clone implication |
|---|---|---|
| Sewing Line | Pair of line halves, one "fixed" and one "moving" | Data model: SewingLine { a: EdgeRange, b: EdgeRange, reversed, type, props } |
| Directional Notch | Arrow marks on each half showing traversal direction. Parallel notches = normal sew. Crossed notches = one pattern flipped when sewn | Store direction per half. Matching param t along each half by arclength fraction |
| Segment vs Free | Segment Sewing snaps to whole segments (between points). Free Sewing sets arbitrary start/end on outlines or internal lines | Free sewing must be able to insert points at start/end (see Add Point to Pattern on Start/End) |
| 2D vs 3D window | Same sewing data. Tools exist in both windows. 3D tools sew on the draped garment surface | One data model, two picking front ends |
| Simulation treatment | Corresponding lattice points on both lines joined by springs. Fixed side holds, moving side travels to overlap. Sublayer adds anti-penetration virtual force (patent) | Implement as stitch springs / distance constraints with rest length near zero between matched particles, plus ordering for collision layers |
| Length mismatch | If halves differ, the lengths are matched by proportional distribution (arclength fraction), and tension (Ease or Stretch) governs whether the longer side is eased or stretched | Mismatched seam yields gathering/ease. Tension param controls how strongly |

## 2. Sewing creation tools

### 2.1 Segment Sewing (2D)

Source: support.marvelousdesigner.com Segment Sewing (2D).

| Item | Detail |
|---|---|
| Purpose | Establish Sewing Lines on segments of Patterns or internal shapes/lines |
| Location | Main Menu > Sewing > Segment Sewing; 2D Toolbar > Segment Sewing |
| Step 1 | Click once on a pattern segment to set the first half |
| Step 2 | Hover over the segment to be sewn to the first |
| Step 3 | Drag the mouse until the Directional Notches are parallel to each other |
| Step 4 | Click to set the second half |
| Crossed notches | Sewing Lines also cross in 3D. On simulation, one pattern is reversed before being sewn to the other |
| Live feedback | Length difference of the two halves appears in blue next to the cursor |
| Warning | Red when the difference exceeds 1 inch |
| Undo notch direction | CLO older page: Esc or Ctrl+Z resets Directional Notches on a pair |
| Related | "Dividing Sewing Lines Based on Notches" (CLO manual) describes the same flow and says sewing lines on segments split by points connect the patterns |

### 2.2 Free Sewing (2D)

Source: support.marvelousdesigner.com Free Sewing (2D); cross-check support.clo3d.com [2D Tool] Free Sewing and [3D Tool] Free Sewing.

| Item | Detail |
|---|---|
| Purpose | Freely establish Sewing Lines on Pattern outlines and internal shapes/lines, not restricted to whole segments |
| Location | Main Menu > Sewing > Free Sewing; 2D Toolbar > Free Sewing |
| Step 1 | Click, drag, click on an outline or internal line to set start and end of the first half |
| Step 2 | Click another outline or internal line to set the start of the second half |
| Blue guide point | While dragging the second half, a blue guide point shows where the second half would end to equal the first half length |
| Snap | The cursor snaps to the blue guide point |
| Step 3 | Align the Directional Notches to be parallel, click to complete |
| Direction rule | The direction of a notch depends on the order in which you set its start and end points |
| Exact length | Right-click while creating a line to open the Create Sewing Line window and type an exact length |
| Mismatch display | Blue number by cursor, red above 1 inch |
| Tools hidden under | Press and hold the Free Sewing toolbar button to reveal the M:N Free Sewing tool |

### 2.3 1:N Segment Sewing (2D)

Source: support.marvelousdesigner.com 1:N Segment Sewing (2D) (posted May 29, 2025).

| Item | Detail |
|---|---|
| Purpose | Sew multiple segments (N) to a single segment (1) |
| Location | Main Menu > Sewing > Segment Sewing; 2D Toolbar > Segment Sewing (same tool, Shift modifier) |
| Length rule | If the combined N length differs from the single segment, the single segment is divided equally among the N segments (as stated by the article) |
| Steps | 1) Select the single segment. 2) Hold Shift: sewing lines turn neon. 3) While holding Shift drag over N segments, ensure notches are parallel, click each. 4) Release Shift to finish |
| Keys | Delete = undo last action; Esc = cancel all |
| Invalid segment | Turns red, cannot be sewn |
| Mismatch display | Blue by cursor, red above 1 inch |

Note on wording: the 1:N page says the single segment is "divided equally", whereas the M:N page says N segments are "divided proportionally to the M segments". Treat equal vs proportional as an open question to verify by experiment [single source each].

### 2.4 1:N Free Sewing (2D and 3D)

Sources: support.marvelousdesigner.com 1:N Free Sewing (2D), 3D 1:N Free Sewing (VER4.2.0).

| Item | Detail |
|---|---|
| Purpose | Sew multiple patterns to one pattern edge using free sewing lines |
| Location | Main Menu > Sewing > Free Sewing; 2D or 3D Toolbar > Free Sewing |
| Steps | 1) Open Free Sewing. Sewable segments are bolded; all sewing lines show in 2D and 3D. 2) Set start and end points on the outline of the "1" pattern. 3) Hold Shift: line turns neon. 4) While holding Shift, add sewing lines to N patterns, watching notches. 5) Release Shift |
| Keys | Delete undoes last; Esc cancels all |
| 3D variant | Identical gesture performed in the 3D window |

### 2.5 M:N Segment Sewing

Sources: support.marvelousdesigner.com M:N Segment Sewing (2D) and 3D M:N Segment Sewing (VER4.2.0); cross-check support.clo3d.com M:N Segment Sewing index entries.

| Item | Detail |
|---|---|
| Purpose | Sew M segments to N segments at once, editable as a group. Typical use: a seam whose one side is broken into several segments by points (e.g. sleeve cap joined to armhole made of front and back pieces) |
| Length rule | If total lengths of M and N differ, the N segments are divided proportionally to the M segments |
| Location | Main Menu > Sewing > M:N Segment Sewing; 2D or 3D Toolbar > M:N Segment Sewing. Tip: press and hold the Segment Sewing tool to reveal it |
| Step 1 | Click the M segments, press Enter. M lines turn neon |
| Step 2 | Using Directional Notches as guide, create sewing lines on N segments, press Enter. Neon disappears |
| Keys | Enter = commit M group, then commit N group. Delete = undo last. Esc = cancel all |
| 3D behaviour | In the 3D tool, all sewable segments are bolded and lines show in 2D and 3D |
| Invalid | Clicking an inappropriate segment turns it red; cannot create a line on it |
| Mismatch | Blue next to cursor, red above 1 inch |

### 2.6 M:N Free Sewing

Sources: support.marvelousdesigner.com M:N Free Sewing (2D); 3D equivalent listed in CLO manual index.

| Item | Detail |
|---|---|
| Purpose | Freely draw sewing lines on M patterns and sew to N patterns at once |
| Location | Main Menu > Sewing > M:N Free Sewing; 2D Toolbar > M:N Free Sewing. Press and hold Free Sewing tool to reach it |
| Step 1 | Draw Free Sewing Lines on M patterns, press Enter. Lines turn neon |
| Step 2 | Draw lines on the patterns to be sewn to M, with notches in mind, press Enter |
| Keys | Enter commit stage; Delete undo last; Esc cancel all |
| Invalid | Red segment, cannot apply |
| Mismatch | Blue, red above 1 inch |

### 2.7 Tool matrix (all creation tools)

| Tool | Window | Snap unit | Group modifier | Commit |
|---|---|---|---|---|
| Segment Sewing | 2D and 3D | whole segment | none | click |
| 1:N Segment Sewing | 2D and 3D | whole segment | hold Shift | release Shift |
| M:N Segment Sewing | 2D and 3D | whole segments | Enter after each group | Enter |
| Free Sewing | 2D and 3D | arbitrary range on outline or internal line | none | click |
| 1:N Free Sewing | 2D and 3D | arbitrary range | hold Shift | release Shift |
| M:N Free Sewing | 2D and 3D | arbitrary range | Enter after each group | Enter |
| Pleats Sewing | 2D | groups of 3 segments | drag across range | click end point |
| Auto Sewing | 3D | rule-based | none | single click |

The CLO manual section index lists 3D versions of all six tools separately ([3D Tool] Segment, 1:N Segment, M:N Segment, Free, 1:N Free, M:N Free), confirming 2D and 3D parity.

## 3. Sewing in 2D vs 3D window

| Aspect | 2D window | 3D window |
|---|---|---|
| Picking | On flat pattern outlines and internal lines | On draped garment pattern edges |
| Visibility | Edit Sewing shows all sewing lines in 2D automatically (Show 2D Sewing turns on) | Sewable segments bolded; sewing lines show in both windows |
| Utility | Good for measured seams, exact length entry | Good when patterns are already positioned around avatar |
| Data | Same sewing line object | Same |
| Hide | Show Sewing option in 2D window (v5.0+) | Hide Sewing (Selected/All) hides in both 2D and 3D |

## 4. Edit Sewing and sewing operations

### 4.1 Edit Sewing tool

| Item | Detail |
|---|---|
| Location | Main Menu > Sewing > Edit Sewing; 2D Toolbar or 3D Toolbar > Edit Sewing. Shortcut: B |
| Select | Click a point or segment on a Sewing Line to open the Property Editor |
| Edit length | Click and drag either endpoint. For exact values, right-click while dragging an endpoint in 2D to open the Moving Distance window: enter distance to move or total length |
| Right-click menu | Reverse Sewing, Delete, Hide Sewing (Selected/All), Deactivate/Activate Sewing (Selected), Add Point to Pattern on Start/End, Set Sublayer controls |

### 4.2 Operations table

| Operation | How | Shortcut | Notes |
|---|---|---|---|
| Reverse Sewing | Edit Sewing, select lines, right-click > Reverse Sewing | Ctrl+B | Flips Directional Notches on selected lines. Flips which orientation the pair is sewn (sewing direction/flipping) |
| Delete Sewing | Edit Sewing, select, right-click > delete, or press Delete | Delete | Removes sewing lines |
| Show/Hide Sewing | Edit Sewing right-click, or Object Browser right-click: Hide Sewing (Selected) or (All) | none | Display only, does not affect simulation. Disappears from 2D and 3D |
| Activate/Deactivate | Edit Sewing right-click, or Object Browser Scene tab: Deactivate/Activate Sewing (Selected) | none | Deactivated lines display white in 2D and 3D and their names are dark grey in Scene tab. Use after draping, e.g. back slits on a men's jacket. Simulation ignores deactivated lines (implied; page says "turn sewing lines on or off") |
| Add Point to Pattern on Start/End | Edit Sewing, right-click line, choose start or end | none | Adds a point on the outline where the selected half sits. Repeat for the other half |
| Edit sewing line length | Drag endpoint, right-click for numeric | none | See 4.1 |
| Seam presets | Open/Save .ssp (Sewing Style Preset) in the property editor | none | Presets recall sewing properties on other lines |

### 4.3 Sewing direction and flipping

* Notches on both halves must be parallel for a normal (non-twisted) seam.
* If notches cross, 3D sewing lines cross and one pattern is reversed (flipped) at simulation start. This is normally an error condition, but is how a user intentionally sews a piece "inside out".
* Direction of a half is determined by the order of clicking start then end in Free Sewing. In Segment Sewing it is selected by dragging the mouse while hovering.
* Reverse Sewing (Ctrl+B) fixes direction after creation.

## 5. Sewing Line Properties

Source: support.marvelousdesigner.com Seamline Property (ver 2025.0); cross-check support.clo3d.com Edit Sewing property table, Sewing Tension, Fold Seam Line, Set Sublayer.

### 5.1 Information and presets

| Property | Meaning | Range / default |
|---|---|---|
| Seam Presets Open / Save | Load or save a Sewing Style Preset (.ssp) | file |
| Name | Rename the sewing line | text |
| Sewing Line Type | Custom (Fold Strength and Angle active) vs Turned (flat two-ply). The MD 2025 page words it as Off or On for "Turned" | enum |

### 5.2 Fold properties (Custom type)

| Property | Meaning | Range / default |
|---|---|---|
| Fold Angle | 180 degrees = flat. Below 180 folds toward the front of the pattern. Above 180 folds toward the back | 0 to 360 degrees. Default not published |
| Fold Strength | How strongly the seam follows the Fold Angle. Higher = closer to set angle. Set 0 to flatten sewn boundaries (e.g. jacket to lining where defaults make it puff) | numeric, range and default not published |
| Variable Fold Angle (2026.1) | On internal lines the fold can vary from a set angle at one end to flat 180 at the other. Gradient direction reversible. Linked symmetric lines mirror the change unless link is removed | 0 to 360 |

Source for Fold Angle 0 to 360: CLO Fold Seam Line page and DigitalProduction 2026.1 article, two independent sources.

### 5.3 Tension

| Property | Meaning | Range / default |
|---|---|---|
| Tension Type: Ease | Adds a small allowance when seam lengths differ: softer tension | enum |
| Tension Type: Stretch | No allowance when lengths differ: more precise, higher tension | enum |
| Strength | Increases or decreases tension. Same Ratio can give different results at different Strength | numeric, not published |
| Ratio | Adjusts tension ratio to change drape, mimicking different sewing methods | Maximum 200 percent (CLO Sewing Tension page) [single source for the 200 percent cap on sewing tension; the same 200 percent cap is separately documented for Elastic Ratio] |

Purpose per manual: reduce excess stretch when the 3D garment appears more stretched than the real one.

### 5.4 Sublayer

| Property | Meaning |
|---|---|
| Set Sublayer | Assigns layer order to sewing lines on a pattern that folds on itself (seam allowances, pleats) to improve simulation stability. A red arrow shows the order of the selected pair |
| Fixed half | The half next to the arrowhead stays fixed; the half next to the arrow tail moves |
| Reverse Direction | Swaps the fixed Sewing Line and the moving (Sewed) line |
| Front / Back | Chooses whether the fixed Sewing Line attaches to the Back or Front side of the fabric |
| Apply | Activate Simulation to see the effect |

The patent describes the same UI (checkbox, guideline with plus or minus symbol, Reverse Direction) and says the layer limitation is a virtual force between layered patterns that stops intersection and collision.

### 5.5 Topstitch (sewing line attached)

| Property | Meaning |
|---|---|
| Name | Rename topstitch |
| Topstitch Property | Change topstitch style |
| Extend | Extend a stitch that stops short or overshoots the outline |
| Corner Type | Corner Curved (smoother around corners), Right Angled, Smooth (continues across breaks in edge) |
| Offset | Z Offset of the stitch |
| Placement | Seam A, Seam B, or Seam Both (visible on both sides) |

Related: "Create Additional Topstitch Style" listed as a related article.

### 5.6 Puckering on seams

| Property | Meaning |
|---|---|
| Name, Puckering Property | Rename and change style |
| Extend | Extends start and end to the 2D outline |
| Offset (mm) | Distance from outline; user can choose whether puckering width is included in the offset |
| Z Offset (mm) | Distance from the pattern along Z |
| Placement | Seam A, Seam B, Seam Both |

Puckering is a visual texture/normal-map effect, not a physical simulation of fabric pucker. Details (Create puckering page):

| Item | Detail |
|---|---|
| Creation modes | Segment Puckering (click segment), Free Puckering (click first and last points, works on outlines and internal lines), Seamline Puckering (click a Sewing Line; result on one or both lines depending on Placement) |
| Multi-select | Drag an area in 2D to apply to many segments |
| Restriction | Cannot apply puckering twice to the same line |
| Edit | Drag line to move; drag endpoint to change length; right-click for Moving Distance dialog; Delete removes |
| Style presets | Cotton, Denim, Leather, Nylon, Polyester |
| Style parameters | Texture, Desaturation, Shadow Intensity, Shadow Brightness, Normal Map (Intensity percent, Color, Opacity), Density (percent), Width |
| Menu paths | Main Menu > Materials > Puckering > Segment Puckering; show/hide via Display > Materials > Show Puckering |
| Internal-line extras | Both Sides placement, Flip |

### 5.7 3D Seamline (stitch appearance)

| Property | Meaning |
|---|---|
| Plain Seam: Normal Map | Image used as normal map along the seamline |
| Intensity (percent) | Strength of normal map effect |
| Thickness (mm) | Thickness of the effect |
| Directional: Placement | Seam A or Seam B (which side carries the seam allowance direction) |
| Directional: Normal Map, Intensity, Thickness | Same as Plain Seam |
| Version | Introduced ver 5.2 (CLO page) |
| Export | A user feature request asks to export seamline normal maps with FBX (not confirmed implemented) |

## 6. Sewing types (Turned, Custom Angle, others)

| Type | Behaviour | Parameters | Use |
|---|---|---|---|
| Custom (Custom Angle, regular) | Default for seams between two flat pieces; pieces lie side by side at the Fold Angle. Fold Strength and Angle active | Fold Angle 0 to 360, Fold Strength | Side seams, front/back joins |
| Turned | Folded 180 degrees, "Double layered Patterns are sewn together crisp and flat". Fold Strength and Angle disabled. Default when a pattern is sewn onto an internal line (per Versluis blog, secondary source) | None besides type | Hems, cuffs, collars, facings, lining |
| Turned with seam allowance | Not a documented distinct sewing line type in the pages found. Seam allowance is a pattern-edge feature. Unverified as a type | n/a | n/a |
| Flat | Not a named MD sewing type in the pages found. "Flat" appears only as a description of Turned result and Fold Angle 180 | n/a | n/a |

Behavioural note (Versluis): using Custom Angle on a hem tries to place the pieces side by side and curves the hem; Turned kinks sharply at 180 degrees like an ironed hem. The official "Sewing Line Type" page requires running Simulation to see the Turned result.

Export interplay (DigitalProduction 2026.0): FBX/OBJ export offers "Weld Turned Sewing Lines" to merge vertices on turned seams.

Known issue (from CLO page comment): turned sewing can show loose stitches and a hollow gap between the fabrics; no official answer was given.

## 7. Fold Angle and Fold Seam Line

| Item | Detail |
|---|---|
| Location | Edit Sewing, select seamline, adjust Fold Angle (Pattern Property Editor > Fold > Fold Angle in the CLO page) |
| Purpose | Press effect on sewn parts |
| Range | 0 to 360, 180 flat |
| Direction | Near 0 folds internal line toward front; near 360 toward back |
| Strength | Higher follows the set angle more. Set 0 to flatten |
| Restriction | Turned type locks both (noted since version 4.0.0) |
| Internal lines | Fold angle can also be set on internal lines in the pattern (fold effect on internal lines used by Pleats) |

## 8. Check Sewing Length

| Item | Detail |
|---|---|
| Purpose | Find sewing line pairs with significant length differences |
| Location | Main Menu > Sewing > Check Sewing Length; 2D Toolbar icon |
| Behaviour | Pairs whose difference exceeds the threshold appear bold and red |
| Default threshold | 5 mm (CLO manual) [single source] |
| Live mismatch during creation | Blue number next to cursor; red above 1 inch (MD manual) |

Discrepancy: the CLO page "Dividing Sewing Lines Based on Notches" says the live warning turns red above "1 mm", the MD manual pages say "1 inch". The MD manual is more recent and consistent across five articles, so use 1 inch for the live indicator, but flag it for verification in the app.

Clone recommendation: compute per-pair absolute length difference; expose two thresholds (live warn, batch check).

## 9. Tack

Sources: support.clo3d.com Tack on Garment, Tack on Avatar, Edit Tack.

### 9.1 Tack (on garment)

| Item | Detail |
|---|---|
| Purpose | Temporarily join two points on a garment to preview fit adjustments |
| Location | Main Menu > Sewing > Tack > Tack; 3D Toolbar > Tack |
| Steps | 1) Click start point on the draped garment; dotted line follows cursor. 2) Click end point; a yellow line connects them, the tack also appears in 2D. 3) Activate Simulate: points stay fixed to each other |
| 2D | Since version 3.1.0 tacks can be created and modified in 2D |

### 9.2 Tack on Avatar

| Item | Detail |
|---|---|
| Purpose | Temporarily fix a garment point to an avatar point |
| Location | Main Menu > Sewing > Tack > Tack on Avatar; 3D Toolbar icon |
| Steps | 1) Click a point on 3D garment; point created, dotted line follows cursor, garment goes semi-transparent. 2) Click a point on the avatar; garment opaque again; tack appears on the 2D pattern. 3) Simulate: garment point stays fixed to the avatar point |

### 9.3 Edit Tack and Copy/Paste Tack

| Item | Detail |
|---|---|
| Edit Tack | Main Menu > Sewing > Tack > Edit Tack, or 3D Toolbar icon. Garment turns semi-transparent. Drag an endpoint to change position and spacing |
| Delete | Click a tack (highlights yellow), press Delete or right-click > Delete |
| Copy/Paste Tack | Listed in the CLO Sewing manual index as [Popup] Copy/Paste Tack. Page not fetched. Behaviour unverified |

Simulation treatment (inferred, not documented by MD): tack is a distance constraint (spring) between two surface points, or between a garment point and a static avatar point. Mark as inference.

## 10. Pleats

Sources: support.marvelousdesigner.com Pleats and Fold/Sew Pleats; cross-check Versluis tutorial (secondary, personal blog by a long-time MD instructor; not on the banned list but not a primary source).

### 10.1 Pleats tool (generate pleats)

| Item | Detail |
|---|---|
| Location | Main Menu > 2D Pattern > Pleats > Pleats; 2D Toolbar > Pleats |
| Steps | 1) Click start and end point of a reference line along the pattern outlines; pattern divides in two. 2) Click the side where pleats will be added. 3) Pleats window opens, set options, OK |
| Types | Knife, Box, Accordion |
| Number of Pleats | Count |
| Depth | Depth of the folded pleat |
| Interval | Spacing between pleats when two or more |
| Reverse Direction | Reverses the side where pleats are added |
| Option (fold angles) | Sets angles of Outside Fold and Inside Fold lines; larger difference gives sharper folds |
| Result | Internal lines move and rotate with the pattern |
| Auto-sewing | If the pleat segment is sewn to another pattern, pleats are sewn automatically per type. Else use Pleats Sewing |
| Error | A pleat cannot extend past the next segment point; red error shown |
| Ranges | Not published |

### 10.2 Pleats Fold (fold existing internal lines)

| Item | Detail |
|---|---|
| Location | Main Menu > 2D Pattern > Pleats > Pleats Fold; 2D Toolbar |
| Steps | Click once for start of pleat direction, double-click for end. Pleats Fold window opens. Choose type, optional Reverse Direction, OK |
| Fold angle colour key | Yellow: outside fold line, 0 degrees. Light blue: inside fold line, 360 degrees. Not highlighted: no fold line, 180 degrees |

### 10.3 Pleats Sewing

| Item | Detail |
|---|---|
| Location | Main Menu > 2D Pattern > Pleats > Pleats Sewing; 2D Toolbar |
| Steps | Click initial point and drag to the end of the seam line on the pleats pattern. Click first sewing line, move to end, click. Sewing applied for every three segments (the number needed per pleat). Fluorescent lines change to seam colour |
| Per-pleat segments | 3 |

### 10.4 Pleat geometry (Versluis, secondary)

| Type | Internal lines | Fold angles |
|---|---|---|
| Knife | 3 per pleat | Two 180-degree folds in opposite directions |
| Box | 3 per pleat | Each fold bent 90 degrees in different directions, forming a small rectangle |
| Accordion | 2 folds per pleat | 90 degrees in opposite directions |

Tutorial advice: internal line count rule "multiple of 3 plus 2" for knife/box. Do not use Pleats Sewing for accordion; use M:N Free Sewing. Extend internal lines to the outline with "Extend/Trim & Add Point to Pattern Outline" first; points are not created automatically. Single secondary source, mark [single source] for the accordion tool caveat.

Clone note: a pleat is just internal lines with fold angles (0, 180, 360 or intermediate) plus sewing lines linking line N to N+1 groups. Fold angle on internal lines is realised as rest dihedral angle in the bending model.

## 11. Gathering and shirring

| Mechanism | Detail | Source |
|---|---|---|
| Sewing length mismatch | Sew a longer edge to a shorter edge: the longer is eased/gathered. Tension Ease vs Stretch determines allowance | Seamline Property page |
| Elastic on a pattern line | Pattern Property Editor > Select a Line > Elastic checkbox. Creates shirring. Strength = intensity, Ratio = elastic amount relative to the segment at 100 percent, max 200 percent, Length per selected segment, Entire Length across all selected segments | support.marvelousdesigner.com Elastic |
| Elastic dependency | If the checkbox is off, strength and ratio have no effect | same |
| Tension Ratio | Max 200 percent on sewing tension | CLO Sewing Tension |

There is no documented tool called "shirring" in sewing; shirring is Elastic on lines or tension on sewing lines. Equivalent behaviour for the clone: Elastic is a per-edge rest-length scale (ratio 100 to 200 percent, interpreted as shortening or stretching of the edge) with a strength coefficient.

## 12. Auto Sewing

Source: support.clo3d.com Auto Sewing (ver 6.0).

| Item | Detail |
|---|---|
| Purpose | Automatically sews patterns based on placement on avatars |
| Location | Main Menu > Sewing > Auto Sewing; 3D Toolbar |
| Supported garments | Tops (T-shirts, jackets, jumpers) and one-piece. Bottoms (skirts, pants) not supported in ver 6.0; ver 6.1 adds leg arrangement points |
| Supported parts | Bodies, sleeves, collars, darts, pleats, cuffs, sleeves with vent and placket |
| Layers | One-ply patterns (pockets, lining). Two-layer patterns not supported |
| Method | Rule-based: uses arrangement points (collar/band: neck; bodies: body; sleeves: arm, cuffs arm or wrist; bottoms: legs) and CLO-provided bounding volumes. No ML is described |
| Existing sewing | Kept; remaining lines added |
| Darts and pleats | Sewn without arranging; pleats only if internal lines are defined by fold direction |

Not an AI feature per documentation. For the clone, treat as a heuristic matcher using named edge roles and proximity.

## 13. Sewing Layer, 3-layer sewing, merge/split

| Requested feature | Finding |
|---|---|
| Sewing Layer | No feature named "Sewing Layer" found. Closest are Sublayer (section 5.4), Set Sublayer (Pattern) and the Pattern Layer/Arrangement layer order. Pattern sublayer: related article titled Set Sublayer (Pattern) exists, not fetched |
| 3-layer sewing | Not found as a distinct feature. A search hit shows a community post "Superimpose and Sublayer"; forum content is excluded by sourcing rules. Auto Sewing explicitly does not support two-layer patterns. Multiple layers are handled by Sublayer ordering plus collision layers |
| Merge sewing | Not found. Related: Seamless Blocking and Linked Editing (CLO related articles) concern linked patterns, not sewing merge |
| Split sewing | Closest: Add Point to Pattern on Start/End (splits segment so sewing aligns), "Dividing Sewing Lines Based on Notches", and splitting by M:N proportional division |
| Internal line sewing | Verified: sewing lines can be placed on internal shapes/lines in all tools; turned sewing is the default when a pattern is sewn to an internal line (secondary) |

## 14. Seam colour coding and display

| State | Appearance | Source |
|---|---|---|
| Sewable segments during tool use | Bolded | 3D Free Sewing / M:N pages |
| Group in progress (1:N, M:N) | Neon colour | 1:N and M:N pages |
| Invalid target segment | Red, cannot sew | all creation pages |
| Length difference | Blue number by cursor, red when over threshold | Segment/Free pages |
| Length mismatch (Check Sewing Length) | Bold red | CLO Check Sewing Length |
| Deactivated sewing line | White in 2D and 3D, dark grey name in Scene tab | Activate/Deactivate |
| Tack | Yellow | Tack, Edit Tack |
| Pleat fold lines | Yellow: outside fold 0 degrees; light blue: inside fold 360 degrees | Pleats Fold |
| Pleats Sewing | Fluorescent lines then turn to colour of respective seam | Pleats Fold/Sew |
| Sewing line after completion | Each seam pair gets its own colour (stated for pleats; general per-seam colouring is implied) | Pleats page |

## 15. Stitch display

* Seam appearance in 3D is not a physical stitch but a rendering: Topstitch objects (style, offset, placement), 3D Seamline normal map, Puckering textures.
* Show/Hide Sewing toggles the sewing line overlay (not stitches).
* No dedicated "stitch display" toggle for plain seams was found. Mark as not verified.

## 16. How the simulation treats sewn seams

Primary source: CLO patent US10810794B2 (assignee Clo Virtual Fashion Inc., inventor Seung Woo Oh), which the patent describes as "3D clothing draping simulation".

| Aspect | Documented |
|---|---|
| Cloth model | Triangular or rectangular meshes of lattice points with mass, connected by springs (mass-spring) |
| Seam model | Select first and second sewing line; extract meshes containing each line as a segment; connect lattice points on the first line's meshes to corresponding points on the second line's meshes by springs, one-to-one |
| Overlay step | Region containing the second line is fixed; region containing the first line is moved toward it until they overlap. The "first line" is the first designated |
| Sublayer | User sets whether first line is above/front (plus) or below/behind (minus). Reverse Direction swaps fixed and moving |
| Layer limitation | The coupling springs create a virtual force between layered patterns to prevent intersection and collision |
| Collision | Resolved during draping, patent cites Volino and Magnenat-Thalmann (2006) |
| Not specified | Spring stiffness, rest length, damping, integration, solver, any stitch merge or topology change |

Corroboration: SIGGRAPH "True Seams" (Rodriguez and Cirio, 2022) states that most 3D fashion CAD and VFX tools simply treat seams as joint edges (no physical seam geometry). The page does not describe the exact method for conventional tools.

Implication for clone: a baseline implementation is:
1. Resample both halves by arclength, map matched vertices (proportional distribution when lengths differ).
2. Add stitch springs / zero rest length distance constraints between matched vertex pairs (or vertex weld at assembly time, then keep constraint).
3. At sew-start, run a "closing" phase (fixed half stays, moving half interpolated toward it) before enabling full gravity. This matches the documented "fixed/moving" semantics.
4. Fold Angle and Strength act as rest bend angle and bend stiffness on the dihedral across the seam.
5. Tension Ease vs Stretch changes whether differing lengths produce slack (ease) or enforce stretch.

Edge behaviours worth replicating:

| Case | Behaviour |
|---|---|
| Unequal lengths | Proportional division and tension; shown red beyond threshold |
| Crossed notches | Piece reversed (twisted seam) at simulation |
| Turned | Locked 180 degree fold, flat two-ply |
| Sublayer | Anti-penetration ordering on self-folding areas |
| Deactivated line | Ignored in simulation |
| Pleat beyond next segment point | Error |
| Puckering twice on one line | Disallowed |
| Seamline Rip (2026.1 beta) | Seam opens when forces meet Rip Resistance. Rip Resistance depends on Yield Strength (damage onset) and Damage Rate (damage growth). Activate toggle. Paired rip objects on both sides, separately adjustable. Marked beta |

## 17. Newer sewing-adjacent features (2026)

| Feature | Detail | Source |
|---|---|---|
| Seamline Rip (beta, 2026.1) | See above | DigitalProduction |
| Variable Fold Angle (2026.1) | See 5.2 | DigitalProduction |
| 3D Pencil sew (2026.0) | Drawing on both sides and flattening automatically sews front and back | DigitalProduction |
| Weld Turned Sewing Lines (export) | Vertex merge on export | DigitalProduction |
| Lacing tool (2026.0) | Creates a lace pattern through a sequence of holes. It is a pattern, not a sewing line | DigitalProduction |
| Trim glue | Attach trim to clothing instead of sewing | DigitalProduction |
| Zipper Extend | New Extend property in 2026.1 | DigitalProduction |

## 18. Unverified and open questions

1. Numeric defaults for Fold Strength, Fold Angle, Tension Strength, and ranges for Offset/Intensity/Thickness are not published in the manual.
2. 1 inch vs 1 mm threshold for the live length warning.
3. Equal vs proportional distribution of length mismatch for 1:N vs M:N.
4. Whether Tension Ratio's 200 percent cap is "percent of seam length".
5. Exact spring stiffness and solver; the patent does not give them. Needs black-box experimentation.
6. Merge/split sewing, Sewing Layer, 3-layer sewing, stitch display toggle: not found.
7. Copy/Paste Tack and Set Sublayer (Pattern) pages not fetched.
8. Seam allowance related sewing type ("Turned with seam allowance"): not found as a sewing line type.
9. The MD 2025 property page labels Sewing Line Type as "Off or On" where CLO labels it "Custom or Turned"; interpret as Turned off/on.

## 19. Suggested data model for formfabric

```
SewingLine {
  id, name, active, visible,
  halfA: { patternId, edgeRange(startT, endT, ordered), isInternalLine },
  halfB: { ... },
  reversed: bool,            // Reverse Sewing
  type: 'custom' | 'turned',
  foldAngleDeg: 0..360 (180 flat), foldStrength: number,
  tension: { mode: 'ease'|'stretch', strength: number, ratioPct: 0..200 },
  sublayer: { enabled, fixedSide: 'A'|'B', face: 'front'|'back' },
  topstitch?: {...}, puckering?: {...}, seamlineNormal?: {...},
  rip?: { activate, yieldStrength, damageRate }   // 2026.1 beta
}
Tack { a: SurfacePoint, b: SurfacePoint | AvatarPoint }
PleatSet { type: 'knife'|'box'|'accordion', count, depth, interval, outsideFoldDeg, insideFoldDeg, reverse }
```

UI hotkeys to copy: B (Edit Sewing), Ctrl+B (Reverse Sewing), Delete (delete or undo during creation), Esc (cancel), Enter (commit group), Shift (1:N).

## Sources

Fetched (primary, official):

* https://support.marvelousdesigner.com/hc/en-us/articles/47358199517209-Segment-Sewing-2D
* https://support.marvelousdesigner.com/hc/en-us/articles/47358239854361-Free-Sewing-2D
* https://support.marvelousdesigner.com/hc/en-us/articles/47358225900825-1-N-Segment-Sewing-2D
* https://support.marvelousdesigner.com/hc/en-us/articles/47358236137113-1-N-Free-Sewing-2D
* https://support.marvelousdesigner.com/hc/en-us/articles/47358238742553-M-N-Segment-Sewing-2D
* https://support.marvelousdesigner.com/hc/en-us/articles/47358268690585-M-N-Free-Sewing-2D
* https://support.marvelousdesigner.com/hc/en-us/articles/47358330855193-3D-M-N-Segment-Sewing-VER4-2-0
* https://support.marvelousdesigner.com/hc/en-us/articles/47358348450329-3D-1-N-Free-Sewing-VER4-2-0
* https://support.marvelousdesigner.com/hc/en-us/articles/47358171818521-Seamline-Property-ver-2025-0
* https://support.marvelousdesigner.com/hc/en-us/articles/47358411103001-Sewing-Line-Type
* https://support.marvelousdesigner.com/hc/en-us/articles/47358364299033-Fold-Sew-Pleats
* https://support.marvelousdesigner.com/hc/en-us/articles/51752931944857-Pleats
* https://support.marvelousdesigner.com/hc/en-us/articles/47358239485593-Elastic
* https://support.marvelousdesigner.com/hc/en-us/articles/47358162160409 (Create puckering)
* https://support.marvelousdesigner.com/hc/en-us/articles/47358392328857-Edit-Sewing-Line-Length
* https://support.marvelousdesigner.com/hc/en-us/articles/47358393261593-Reverse-Sewing

Fetched (CLO Virtual Fashion official manual, cross-check):

* https://support.clo3d.com/hc/en-us/sections/115000641848-Sewing
* https://support.clo3d.com/hc/en-us/articles/115002324608-Edit-Sewing
* https://support.clo3d.com/hc/en-us/articles/360001754628--3D-Tool-Free-Sewing
* https://support.clo3d.com/hc/en-us/articles/360053353574-Auto-Sewing-ver-6-0
* https://support.clo3d.com/hc/en-us/articles/115000495467--2D-Tool-Check-Sewing-Length
* https://support.clo3d.com/hc/en-us/articles/115000511548--3D-Tool-Tack-on-Garment
* https://support.clo3d.com/hc/en-us/articles/115000511648--3D-Tool-Tack-on-Avatar
* https://support.clo3d.com/hc/en-us/articles/115000511728--3D-Tool-Edit-Tack
* https://support.clo3d.com/hc/en-us/articles/115000496367--Property-Set-Sublayer
* https://support.clo3d.com/hc/en-us/articles/115000496727--Popup-Reverse-Sewing
* https://support.clo3d.com/hc/en-us/articles/115000528308--Popup-Activate-Deactivate-Sewing-Lines
* https://support.clo3d.com/hc/en-us/articles/115002324968--Popup-Add-Point-to-Pattern-on-Start-End
* https://support.clo3d.com/hc/en-us/articles/115000528428--Popup-Show-Hide-Sewing
* https://support.clo3d.com/hc/en-us/articles/115013404848--Property-3D-Seamline
* https://support.clo3d.com/hc/en-us/articles/115000528508--Popup-Delete-Sewing
* https://support.clo3d.com/hc/en-us/articles/360001749347 (Sewing Tension)
* https://support.clo3d.com/hc/en-us/articles/115012381248 (Divide Sewing Lines Based on Notches)
* https://support.clo3d.com/hc/en-us/articles/115000414567-Fold-Seam-Line

Other:

* https://patents.google.com/patent/US10810794 (CLO Virtual Fashion patent, primary technical source for seam simulation)
* https://history.siggraph.org/?p=117836 (SIGGRAPH "True Seams", peer-reviewed context)
* https://digitalproduction.com/2026/08/25/marvelous-designer-2026-1-rips-seams/ (trade press, release coverage)
* https://digitalproduction.com/2026/04/15/marvelous-designer-2026-0-adds-3d-pencil-and-lacing/ (trade press)
* https://www.versluis.com/2020/12/md-pleats/ and https://www.versluis.com/2021/01/md-turned-sewing/ (secondary: personal tutorial blog, used only for pleat geometry and turned-vs-custom behaviour, marked as secondary)
