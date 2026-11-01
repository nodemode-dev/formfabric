# Marvelous Designer: The 2D Pattern Window and Pattern Tools (Research Slice 02)

Scope: encyclopedia slice for the formfabric open-source clone. Primary source throughout is the official manual at support.marvelousdesigner.com (Zendesk help center, "Manual" category). Every claim below comes from a fetched official page unless marked otherwise. Claims backed by only one page are the norm here (the official manual is the only authoritative source for UI behaviour), so the tag [single source] is used only where it is notable; the default confidence for a manual-derived item is "official manual, one page". Items I could not verify are listed under "Unverified or not found" and are not described as facts.

Conventions: Main Menu paths are written with the interpunct. "ver X" is the version in which the manual says a behaviour was added. Mac equivalents are given when the manual states them.

## Summary

- The 2D window is a vector pattern editor with a small set of Create tools (Polygon, Rectangle, Ellipse, Internal Polygon/Line, Internal Rectangle, Internal Ellipse, Dart, Internal Dart) and a large set of Edit tools. Nearly all edit tools share one interaction vocabulary: click to select, Shift+click to add, marquee drag, arrow keys to nudge, Shift or Ctrl (Mac Command) while dragging to constrain to horizontal, vertical, diagonal or slope, and right-click or F1 (Mac Ctrl) during a drag to type an exact value.
- Geometry model: a Pattern has an outline made of points (segment points, curve points, Bezier "AI" curve points) joined by segments. A pattern can contain Internal Shapes, Internal Lines and Internal Points. Internal lines can carry fold properties (Fold Angle, Fold Strength) and are used by Slash and Spread, Offset, Trace and sewing.
- Almost every numeric entry path is the same pattern: perform the gesture, then right-click to open a small dialog where exact lengths, ratios, counts, or distances are typed. A clone should model every gesture as "interactive preview with optional numeric dialog".
- Symmetry is first-class: Mirror Creation at draw time (ver 5.1.0), half patterns with Unfold, and Linked Editing (Symmetric Pattern with or without sewing, Instance Pattern) that propagates point edits to clones.
- Per-pattern simulation properties (Shrinkage Weft/Warp, particle distance, layer) live in the Property Editor, not in the 2D tools. Shrinkage affects only the 3D garment; the 2D window is unchanged.
- Gaps in public manual coverage: Rectangle, Circle, Notch, Grain line, Grading, DXF/AI import details, Snap and Grid settings, Baseline/Reference lines, and Pattern Library were not retrievable from the fetched pages. See the "Unverified" section.

## 1. Window environment and view controls

Source: 2D Pattern View Controls article.

| Function | How |
|---|---|
| Zoom to Selected | Display, Viewpoint, Focus Zoom; or right-click a pattern, Zoom to Selected; or left-click pattern then press F |
| Zoom Extents All | Right-click empty area of 2D window, Zoom Extents All |
| Smart Panning | 2D window auto-pans to follow the cursor while using pattern/internal shape creation tools, sewing line create/edit, and topstitch create/edit |
| Right-click pattern | Opens the pattern pop-up menu (see section 8) |
| Right-click empty area | Opens the 2D window pop-up menu |

Hotkeys confirmed from the manual: F (zoom to selected), A (Transform Pattern), Z (Edit Pattern) per the Clone as Pattern article ("Transform Pattern (A) / Edit Pattern (Z)"). Arrow keys nudge selections. Delete removes the last placed point while drafting. Esc or Ctrl+Z cancels a draft.

Snap and grid: the View Controls page states it contains no snap or grid information. Not verified.

## 2. Creation tools

Menu path for all: Main Menu, 2D Pattern, Create, tool. Also on the 2D Toolbar.

### 2.1 Polygon

Source: Polygon article.

Operation:
1. Choose Polygon.
2. Click in the 2D window to set the start point.
3. Drag and click to place subsequent points.
4. Right-click mid-draft to open the Create Polygon window.
5. Click the start point again to close.

Modifier keys and hotkeys:

| Input | Effect |
|---|---|
| Delete | Removes the most recently placed point, repeatable |
| Esc or Ctrl+Z | Cancels the whole polygon |
| Ctrl + place point | Makes the point a Free Curve Point (curved segment) |
| Shift + place point | Shows a guideline snapping horizontal, vertical or 45 degrees |

Create Polygon window:

| Field | Meaning |
|---|---|
| Length | Length of the next line. With Mirror Creation on, also applies to the mirrored line |
| Mirror Creation (ver 5.1.0) | Draws symmetrically; the starting point becomes a closed point |
| Axis: Local | Symmetry lines based on the start point |
| Axis: X-Axis | Symmetry based on X axis |
| Axis: Y-Axis | Symmetry based on Y axis |

Edge behaviours: Delete undoes one point, Esc/Ctrl+Z discards all. Mirror Creation closes the start point.

### 2.2 Rectangle, Circle

Rectangle: a Rectangle article exists in the manual (linked from Polygon), but its content was not fetched. Not verified in detail. What is verified indirectly: Rectangle supports the Replicate dialog (below), and the Ellipse behaviour (Shift to constrain, Ctrl for centre) is documented for Ellipse. Do not assume Rectangle has identical modifiers without confirmation.

Circle: there is no separate Circle tool in the manual pages fetched. A circle is an Ellipse constrained by Shift or defined by Diameter, Radius or Circumference in the Create Ellipse window. The manual's Replicate article explicitly does not list a Circle tool.

### 2.3 Ellipse

Source: Ellipse article.

Operation: click and drag for a live preview, release to create. Or click once to open the Create Ellipse window and type values.

| Modifier | Effect |
|---|---|
| Shift | Constrain to a circle |
| Ctrl (ver 5.1.0) | Draw from the centre point |
| Shift + Ctrl (ver 5.1.0) | Draw a full circle from the centre point |

Create Ellipse window fields: Diameter, Radius (ver 3.2.0), Circumference (ver 3.2.0), Link (keeps width to height ratio). Ranges not documented.

### 2.4 Replicate on creation (Rectangle, Internal Rectangle, Ellipse, Internal Ellipse)

Source: Create Multiple Patterns or Internal Shapes and Input Intervals.

Click once in the 2D window to open the creation pop-up, then in the Replicate category set:

| Field | Meaning |
|---|---|
| Number of Shapes | Count of copies |
| Interval | Spacing between each pattern, internal shape or internal line |
| Angle | Direction along which replicas are placed |

### 2.5 Internal Polygon/Line

Source: Internal Polygon/Line article.

Operation: click inside a pattern to set the start point; drag and click for further points; right-click mid-draft to open Create Internal Polygon/Line. Close a polygon by clicking the start point; end a line by double-clicking the last point.

| Modifier | Effect |
|---|---|
| Ctrl (hold) | Creates curves while drawing |
| Shift (hold) | Constrains to straight lines or 45 degree diagonals |

Window fields: Length (next segment, also mirrored when Mirror Creating is on), Mirror Creating (ver 5.1.0, start point becomes a closed point), Axis (Local, X-Axis, Y-Axis), and Position (distance from the pattern outline: Left, Right, Top, Bottom).

Value entry notes: right-clicking inside a pattern with the tool opens the Create Internal Polygon window and creates a polygon from typed values (ver 2.5.0). For a line, click to start, then drag and right-click to open the window with values (ver 2.5.0).

### 2.6 Internal Rectangle, Internal Circle/Ellipse

Individual articles were not fetched. Confirmed: Internal Rectangle and Internal Ellipse exist and support Replicate (above) and the 2D Measurements hover guides (section 11). Internal Circle as a distinct tool is not confirmed; treat as Internal Ellipse with Shift.

### 2.7 Dart and Internal Dart

Source: Dart article.

Quick create: select Dart, drag inside a pattern, release. Numeric create: click once inside the pattern to open the Create Dart window.

| Field | Meaning |
|---|---|
| Dimension: Width (Left), Width (Right), Height (Up), Height (Down) | All measured from the dart centre |
| Position Outline (ver 2.5.0) | Margin between the dart outline and the pattern outline |
| Center (ver 2.5.0) | Distance between dart centre and pattern outline |

Internal Dart: the article mentions it under the Internal Shape tool group with no further detail. A dart is an internal shape that can later be converted (see Convert to Hole/Internal Shape in section 6) and used as a fold or cut target; the manual page for Dart does not describe stitching of the dart legs. Modifier keys, hotkeys and edge behaviours not specified.

### 2.8 Trace

Source: Trace article. Path: Main Menu, 2D Pattern, Trace, or 2D Toolbar.

Operation: open Trace, select baselines, right-click the selection, choose one of three:

| Option | Result |
|---|---|
| Trace as a Pattern | Selected baselines become a new pattern |
| Trace as an Internal Shape | Intersecting selected lines are joined and traced as a single shape |
| Trace as Internal Lines/Shapes | Lines traced as they are |

Edge behaviours: for half-symmetry patterns, select internal lines or outlines on both sides and the symmetry relationship is kept. Internal lines and baselines inside the selection range are included. Double-click an internal line to select a closed shape. Marquee drag selects all lines inside.

Note: "baselines" in this article refers to the lines derived from the avatar or 3D garment projection that Trace consumes. The manual page does not describe how baselines are generated from the avatar (Trace from avatar or 3D is not detailed in the fetched page). Related: "Trace as / Convert to Internal Shape" article exists (not fetched).

### 2.9 Pattern Drafter (Beta)

Listed in Polygon's related articles; a Pattern Drafter (Beta) article exists (id 48920347302297). Content not fetched. Not verified.

## 3. Selection and transform

### 3.1 Transform Pattern (hotkey A)

Source: Select/Move/Transform Pattern article. Path: Main Menu, 2D Pattern, Edit, Transform Pattern; or 2D Toolbar.

| Action | Input |
|---|---|
| Select pattern | Click (highlight yellow) |
| Add to selection | Shift + click |
| Marquee select | Drag on empty area |
| Select regardless of box coverage (ver 2024.0) | Ctrl + drag |
| Move | Drag, or arrow keys |
| Constrain move | Hold Shift or Ctrl (Mac Command): vertical, horizontal, diagonal, or slope |
| Typed move distance | Right-click or F1 (Mac Ctrl) while moving |
| Scale | Drag a point on the marquee box |
| Rotate | Drag the single Rotation Point above the marquee box |
| Pivot | Pivot at the pattern centre; white means inactive and rotation uses the pattern centre. Double-click to activate; then transform and rotation use the pivot |

The "gizmo" in 2D is therefore: marquee box with scale handles, one rotation handle, and a movable pivot.

### 3.2 Edit Pattern (hotkey Z)

Source: Select/Move Point/Segment article.

Select a point or segment by click. When several are close, a pop-up lets you choose which (ver 3.0.0). Shift+click multi-select; marquee drag selects all points and segments inside. Move by drag or arrow keys; Shift or Ctrl (Mac Command) constrains to vertical, horizontal, diagonal, or slope; right-click or F1 (Mac Ctrl) opens the Distance Moved dialog.

Distance Moved dialog: Distance Moved; X-Axis and Y-Axis (ver 4.2.0), points only, hidden for segments.

The manual recommends Transform Pattern when selecting whole patterns with Edit Pattern is awkward.

### 3.3 Transform Point/Segment

Article exists (id 47358256808857), not fetched. Per the Transform Pattern page, individual points/segments are selected with Edit Pattern or Transform Point/Segment.

### 3.4 Polygon Lasso Selection, Invert Selection

Articles exist ("Polygon Lasso Selection", "Invert Selection"), not fetched. Behaviour not verified.

## 4. Curve editing tools

### 4.1 Edit Curvature

Source: Edit Curvature article. Click and drag a straight line to turn it into a curve with Bezier handles at both ends. Click and drag an existing curved line to change its curvature. No modifiers, hotkeys or parameters documented.

### 4.2 Edit Curve Point

Source: Edit Curve Point article. Path: Main Menu, 2D Pattern, Edit, Edit Curve Point.

| Action | Input |
|---|---|
| Add curve point | Click a segment |
| Split at curve point (ver 7.1+) | Right-click a segment |
| Move | Drag or arrow keys |
| Typed move | Right-click while dragging; Distance Moved, plus X/Y axis for points |
| Move multiple points | Supported (ver 4.2.0+) |

Split options from right-click: Split into Two Lines (length of each or ratio), Split by Length (enter number of segments), Uniform Split (equal segments), Direction (relative to start, opposite end, or centre).

Point types: Free Curve Point (freely editable) and Bezier (AI Bezier) points with handles. Adding a Free Curve Point to an AI Bezier curve removes its handles and reshapes around the new point.

Related articles (not fetched): Edit AI Bezier Curve, Delete Curve Point, Convert Curve Points and Segment Points, Convert to Curve Point (AI Bezier Curve), Optimize Curve Points.

### 4.3 Smooth Curve (ver 2.3.0)

Source: Smooth Curve article. Rounds a corner of a pattern or internal shape. Activate the tool, click and drag a corner or curved segment. Right-click during the drag (ver 2.4.0) to open the Round Corner by Length window with Line length and Curvature fields. A separate "Create Smooth Curve" article exists (id 47358352313881), not fetched, so the difference is unverified.

### 4.4 Add Point / Split Line

Source: Add Point/Split Line article.

Click a segment to add a point. Hover and right-click to open Split Line.

| Field | Meaning |
|---|---|
| Split Into Two Lines (Line 1 / Line 2) | Type length of either side from the reference point |
| Ratio | Percent split, default 100 percent; for example even halves or 33.3 percent |
| Split by Length | Divide the line into intervals of the entered length |
| Reverse Direction / Direction (ver 3.2.0) | Which end additions start from |
| Number of Segments | Default 2 (ver 3.2.0) |
| Uniform Split | Equal segments for the entered count |

Edge behaviours: Line 1 is always the shorter side, Line 2 the longer. Multiple selected segments are treated as one line; to add points across them, select the segments, right-click and choose Split (ver 3.2.0).

Related articles (names only): Add Point to Intersection, Divide Internal Lines, Change Length, Merge Points on Internal Lines, Add Perpendicular Internal Line, Align Points, Delete All Overlapping Points.

### 4.5 Extend/Trim and Add Point

Source: article of that name (MD10 for internal-line target). Using Edit Pattern, right-click an internal line or point, choose Extend/Trim and Add Point, then Pattern Outline or Internal Line. The point is extended or trimmed to the target and a point is added in one step. No parameters; behaviour with no valid target is not documented.

## 5. Offsets and seam allowance

### 5.1 Offset Pattern Outline (this is the seam-allowance-style operation)

Source: Offset Pattern Outline article. Select all or part of the outline with Transform or Edit Pattern, right-click, Offset Pattern Outline. Curve points are optimised automatically (MD10+).

| Field | Meaning |
|---|---|
| Direction | Extend or Retract (Retract needs MD10+; only Distance can be edited when Retract) |
| Distance | Offset amount |
| Number of Offsets | Count |
| Total Distance | Distance times Number of Offsets (derived) |
| Create Internal Line | Leaves internal lines where the original outline was |
| Side Segment Type | Perpendicular; Extend Side Segments (match neighbour angles); Mirror (use corner angle with adjacent segment) |
| Corner | Default Corner; Right Angled Corner; Unfilled Corner |

The manual does not use the phrase "seam allowance" on this page and gives no numeric ranges or defaults. In practice it is the tool for adding allowance with Create Internal Line leaving the sewing line visible, but that workflow statement is an inference, not stated on the page.

### 5.2 Offset as Internal Line

Source: Offset as Internal Line article. Right-click a pattern outline segment, Offset As Internal Line.

| Field | Meaning |
|---|---|
| Number of Offsets | Count of internal lines |
| Distance | Spacing |
| All / Individual (2025.0) | One distance for all segments, or per segment |
| Offset From (2025.0) | For Individual: previous line or selected line |
| Reverse Direction | Flip side |
| Both Sides (2025.0) | Lines on both sides of the segment |
| Extend to Pattern Outline / to Internal Line | Line end targets |

Curve sources get automatic curve point optimisation. Shift while selecting picks all sub-segments of a split outline or internal line.

Variant "Offset as Internal Line along Curved" (ver 3.2.0): select two outlines that bound the new lines, right-click one. Fields: Number of Offsets, Offset, Distance to Segment Edge, Reverse Direction (flip horizontally), Opposite Segment (flip vertically), Straight Line, Perpendicular Direction.

Units, ranges and defaults are not given.

### 5.3 Add Perpendicular Internal Line, Add Pattern Thickness

Articles exist (ids 47358458722457, 47358238232985), not fetched.

## 6. Dart, slash and spread, fold, symmetry

### 6.1 Slash and Spread

Source: Slash & Spread article. Path: Main Menu, 2D Pattern, Edit, Slash & Spread.

Interactive: click the pattern outline, drag (line follows cursor), click the outline again to define the slash line. Alternatively click an internal line to use it as the slash line (ver 2.5.0). Select the side to rotate (highlighted yellow), drag in the rotation direction for a preview, click to finish.

Numeric: after defining the slash line, right-click the 2D window to open the Spread window: Distance (length spread) and Angle. Click OK.

Edge behaviours: with an internal line as slash line, the side nearer the cursor spreads. Clicking the slash line itself spreads both sides (MD10). The pattern previews when closed and the outline is smoothed into a natural curve (MD10). Units and ranges not stated. A second video article ("5. 2D Pattern Editing: Slash and Spread") exists.

### 6.2 Fold (pattern property based)

Source: Fold Pattern article. Select an internal line with Transform or Edit Pattern, then Pattern Property Editor, Fold.

| Property | Range and meaning |
|---|---|
| Fold Angle | 0 to 360 degrees; 180 is flat; nearer 0 folds toward front, nearer 360 toward back |
| Fold Strength | 0 to 20; higher approaches the set angle |
| Fold Rendering | Soft or sharp pleat edges |

Additional articles exist: Fold Arrangement, Fold Seam Line, Internal Line Variable Fold Angle (not fetched). The Seamline Property article repeats Fold Strength and Fold Angle for sewing lines, and states that with Sewing Line Type set to Turned On, Fold Strength and Fold Angle cannot be adjusted. It also suggests Fold Strength 0 to flatten sewn boundaries of a jacket and lining.

### 6.3 Unfold (half pattern to full)

Source: Unfold article. Use Edit Pattern, click the fiducial segment, right-click, Unfold. The half pattern (or internal shape) is mirrored across that segment and the segment disappears. Related article "Unfold Symmetric Editing (with Sewing) (ver 5.1.0)" exists, not fetched.

### 6.4 Linked Editing and symmetric clones

Source: Linked Editing article. Right-click menu on a pattern (Transform Pattern tool):

| Command | Variants |
|---|---|
| Clone Pattern with Linked Editing | Symmetric Pattern (With Sewing), Symmetric Pattern, Instance Pattern |
| Apply Linked Editing | Same three types; can be applied to an existing symmetric pair (ver 3.0.0) |
| Remove Linked Editing | Unlinks |

Clone follows the cursor; symmetric clone outline highlights light blue (ver 2.5.0); click to place. For Instance Pattern, right-click the 2D window to open Paste with Interval and Number of Shapes. Linked patterns are edited simultaneously. No modifiers documented.

Related: Merge Symmetric Patterns article exists, not fetched. "Clone as Symmetric" in the task list corresponds to "Clone Pattern with Linked Editing, Symmetric Pattern (With Sewing)"; there is no command literally named "Clone as Symmetric" in the fetched pages.

### 6.5 Clone as Pattern / Internal Shape

Source: Clone as Pattern/Internal Shape article.

- Clone as Pattern (ver 2.4.0): right-click an internal shape, copy follows cursor, click to place.
- Clone as Pattern (With Sewing) (ver 12.2): same, plus sewing lines auto-generated between original shape and copy. No sewing lines when a segment's first and last points are collinear (no angle) or selected segments are not connected.
- Clone as Internal Shape: right-click a pattern, click target pattern to place.
- Shift while placing constrains to straight or diagonal.

## 7. Pattern organisation tools (names confirmed, content not fetched)

From the Edit 2D Pattern section index: Delete, Cut / Cut and Sew, Merge, Order, Rotate, Flip Horizontally/Vertically, Align, Distribute, Match 2D Pattern Measurements, Layer Clone, Convert to Hole/Internal Shape, Replace as Pattern Outline, Divide Internal Lines, Group Internal Lines and Shapes, Roll Up (ver 12.2), Steam, Lacing, Optimize Curve Points, Change Length. The only verified fact is existence. A clone should treat these as a checklist, to be specified in a later pass.

## 8. Pattern pop-up (right-click) menu

The article "4. 2D Pattern Editing: Pattern Pop-up Menu" is a video summary with no item list. Verified items from other articles: Zoom to Selected, Offset Pattern Outline, Offset As Internal Line, Clone as Internal Shape, Clone Pattern with Linked Editing, Apply/Remove Linked Editing, Unfold, Extend/Trim and Add Point, Split (multi-segment), Archive/Unarchive, Select All with Same Property (Particle Distance, Layer, Fabric).

## 9. Pattern properties (Property Editor)

| Property | Source and meaning |
|---|---|
| Shrinkage Weft/Warp | Pattern, Property Editor, Simulation Properties. Default 100 percent (natural state). Shrinks or stretches the 3D pattern horizontally and vertically; the 2D window is not changed. Range not stated. Use cases: simulate washing shrinkage, or shrink an inner layer to fit closer on the avatar. |
| Particle Distance | Existence confirmed ("Select All with Same Property: Particle Distance/Layer/Fabric", and an article "Particle Distance Setting"), but the article fetch failed. Meaning, range and default not verified here. |
| Layer | Confirmed as a selectable same-property group; semantics not fetched. |
| Pressure | Not verified. |
| Fold Angle, Fold Strength, Fold Rendering | See 6.2. |
| Archive | 3D pattern hidden; 2D pattern shown as transparent dotted outline. |

Seamline properties (Seamline Property ver 2025.0): Sewing Line Type (Turned Off or On), Fold Strength, Fold Angle (180 flat), Tension (Ease softer, Stretch more precise, with Strength and Ratio), Sublayer (order of seam allowances and pleats, Reverse Direction, Back or Front). Topstitch properties: Extend, Z Offset, Placement (Seam A, B, Both), corner types Curved, Right Angled, Smooth. Puckering: Extend, Offset (mm), Z Offset (mm), Placement. 3D Seamline: Normal Map, Intensity, Thickness (mm), Directional Placement. The mention of "seam allowances" appears only inside the Sublayer description; no allowance width control was found on that page.

## 10. Superimpose

Source: Superimpose article. This is a 3D window feature (3D Toolbar, Select/Move, right-click a 3D pattern): Superimpose (Over), (Under), (Side). It arranges a newly sewn pattern directly on its corresponding pattern (facings, linings, cuffs, collars). Only works on patterns already sewn together. No numeric parameters. Note it is not a 2D overlay feature.

## 11. Measurement in 2D

Sources: Show 2D Measurements (ver 2025.0), Create 2D Measurements (ver 2.5.0).

- Toggle: 2D Display toolbar, Information Display, 2D measurements; or Line Property Editor; or Main Menu, Display, 2D information, Show 2D Measurements. Off by default.
- With an Internal Shape tool active (Internal Polygon/Line, Internal Rectangle, Internal Ellipse, Dart), hovering a piece shows purple dashed guides with numbers giving the cursor distance to the four sides.
- Per item (Property Editor, 2D Measurements): To Center and To Outline modes for shapes, each with independent Left, Right, Up, Down toggles. Internal lines and internal points use To Center.
- Edge behaviours: editing an internal segment line (add, cut, modify) clears its measurement settings. Multiple selections apply individually. Only internal shapes and lines are measurable this way.
- Match 2D Pattern Measurements: article exists, not fetched.

## 12. Import and file topics

- Pattern Archive: see section 9.
- File formats: the "Which file formats are compatible" page covers only 3D formats (obj, fbx, alembic). A related title about Adobe Illustrator curves being importable and editable exists, content not fetched. DXF/AI pattern import, AF, and size-grading file import were not verified.
- 3D File (OBJ) and Alembic import/export articles exist; out of this slice.
- Modular Library (Ver. 2025.0) and Modular Mode Sew Blocks articles exist; likely relevant to Pattern Library and block patterns but not fetched. Not verified.

## 13. Unverified or not found (do not treat as facts)

- Rectangle tool specifics, Internal Circle, Circle as separate tool
- Notch tool, Grain line tool, Grading, Baseline or Reference line management, Pattern annotations
- Snap and Grid settings (View Controls page says none)
- DXF/AI import options (units, scale, layer mapping)
- Pattern Library, block pattern library contents
- Particle Distance semantics, Pressure property, Layer semantics
- Create Smooth Curve vs Smooth Curve
- Full right-click menu item list
- How Trace obtains baselines from avatar or 3D garment

## 14. Implications for formfabric (clone design notes)

1. Data model: Pattern { outline: ordered points with type (segment, free curve, bezier with handles), segments; internalShapes; internalLines (with foldAngle 0..360 default 180, foldStrength 0..20); internalPoints; properties (shrinkage weft/warp default 100 percent, particle distance, layer, fabric, archived); link group (symmetric, symmetric with sewing, instance) }.
2. Interaction model: one shared selection and constrain vocabulary (click, Shift add, marquee, Ctrl marquee regardless of area, arrows, Shift or Ctrl constrain, F1 or right-click typed value, Delete pops last draft point, Esc cancels).
3. Each tool should have both a drag-preview path and a typed-value dialog path with the field names above.
4. Offset Pattern Outline needs corner modes (default, right angled, unfilled), side segment types (perpendicular, extend, mirror), extend or retract, count, and optional internal line output.
5. Linked editing requires an edit-propagation graph between patterns, with mirror transforms, and an instance mode with count and interval.
6. Shrinkage must be applied to the 3D rest state only, never to the 2D geometry.

## Sources

Fetched (official manual, support.marvelousdesigner.com/hc/en-us/articles/):
- 47358178952345-Polygon
- 47358185711257-Select-Move-Transform-Pattern
- 47358216311193-Select-Move-Point-Segment
- 47358201768857-2-2D-Pattern-Editing-Edit-Pattern-Tools (video summary only)
- 47358248416153-4-2D-Pattern-Editing-Pattern-Pop-up-Menu (video summary only)
- 47358313649177-2D-Pattern-View-Controls
- 47358276422297-Internal-Polygon-Line
- 47358225947545-Ellipse
- 47358375200153-Create-Multiple-Patterns-or-Internal-Shapes-and-Input-Intervals
- 47358289149465-Trace
- 47358304858265-Dart
- 47358405908633-Slash-Spread
- 47358354961049-Fold-Pattern
- 47358434063001-Unfold
- 47358366174233-Linked-Editing
- 47358332785689-Clone-as-Pattern-Internal-Shape
- 47358435895065-Edit-Curve-Point
- 47358445809305-Edit-Curvature
- 47358322811161-Smooth-Curve
- 47358427280153-Add-Point-Split-Line
- 47358445056025-Extend-Trim-Add-Point-to-Pattern-Outline-or-Internal-Line
- 47358359572121-Offset-Pattern-Outline
- 47358442871705-Offset-as-Internal-Line
- 47358171818521-Seamline-Property-ver-2025-0
- 47358324821785-Shrink-Pattern
- 47358318005913-Property-Editor
- 47358389619481-Superimpose
- 47358466150041-Create-2D-Measurements
- 47358141416985-Show-2D-Measurements-ver-2025-0
- 47358186497817-Which-file-formats-are-compatible-with-Marvelous-Designer
- 55690089563417-Pattern-Archive
- Section indexes: support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern (pages 1 and 2) and categories/51985515993625-Manual

Attempted and failed: manual.marvelousdesigner.com (robots disallowed), Particle-Distance-Setting article (fetch error).
