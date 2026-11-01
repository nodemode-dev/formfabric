# 02b: Pattern 2D Gap Fill (Marvelous Designer, with CLO cross-check)

Scope: gap-fill for features that 02-pattern-2d.md could not verify. Method: WebSearch and WebFetch against primary sources only (support.marvelousdesigner.com, the official MD Creator's Field Guide 2024 PDF, support.clo3d.com).

Important limits of this pass (read first):
- The shared web search budget ran out after about 6 searches. Several planned queries (DXF/AAMA/ASTM, Seam Allowance, Grading in the MD or CLO manual, Pattern Library, Snap and Grid, Annotation) were never run. Those items are marked "not found in sources checked", which is NOT proof the feature is absent. A follow-up session should run them.
- Direct HTTP to the Zendesk search pages returned 403, and manual.marvelousdesigner.com (legacy Confluence manual) blocks fetching via robots rules. Only search-result snippets of titles were seen for that site.
- Status vocabulary: [MD] verified in Marvelous Designer sources. [CLO only] exists in CLO sources and not in MD sources checked. [not found] nothing located in sources checked. [single source] one page only.
- The page https://support.marvelousdesigner.com/hc/en-us/articles/47358246789529-2D-Window is only a high-level intro (2D area to design patterns, set seamlines, edit fabric textures; opened via Main Menu, Window, 2D Pattern Window, or the Viewport Title Bar right-click menu, ver 2025.0). It lists no sub-tools.

## Status table

| Feature | Status | Confidence |
|---|---|---|
| Notch tool | [MD] mentioned (Add Notch on a segment point, sewing notches); full tool page seen only for CLO | MD: field guide only [single source]; CLO: full page |
| Grain line | [MD] display toggle and editable arrow | MD: two pages |
| Grading (graded sizes) | [MD] only a visual hint (green outline on segment points); [CLO] full Grade tools not fetched | weak |
| DXF / AI / AAMA / ASTM pattern import and export | [not found] for DXF/AAMA/ASTM; AI found only as a graphic (image) import format | weak |
| Pattern Library and block patterns | [not found] as a pattern block library; "Library" panel for assets found | weak |
| Snap and Grid in 2D window | [not found]; smart guides found; ruler and ruler guideline toggles found | weak |
| Particle Distance (per pattern) | [MD] defined in field guide; per-pattern location not confirmed from a property-editor page | partial |
| Pressure (per pattern) | [MD] field guide | single source |
| Rectangle tool | [MD] field guide | single source |
| Seam Allowance | Not located as tool documentation in MD; Notch page (CLO) documents seam allowance behaviour from v5.0.0 | unresolved |
| Trace (2D baselines to patterns) | [MD] full page | good |
| Trace from 3D | [not found] by that name; "3D Pattern Line" draws lines on 3D garment | weak |
| Superimpose | [MD] full page | good |
| 2D measurement tools | [MD] "Show 2D Measurement" toggle and Show Line Length; 3D measure tools in field guide | partial |
| Pattern annotations | [not found] | none |

## 1. Notch tool

### 1.1 MD status
- The MD Creator's Field Guide 2024 contains these notch statements [single source]:
  - "A sewing notch will appear while sewing, to show the direction of the line" (in the Free Sewing section).
  - Another way to add a notch is right-clicking a segment point and choosing Add Notch (in the Smooth Curve section).
  - Notches show sewing direction and must line up when using Segment Sewing.
- A dedicated MD manual page for the Notch tool was NOT located in search results; the search for it returned only sewing pages (Free Sewing 2D, Segment Sewing 2D, M:N Segment Sewing 2D). The MD 2D Window Display Icons page (ver 5.1.4 and above) has no notch display toggle.
- Conclusion: notches exist in MD at least as segment-point notches and as sewing-direction indicators. Whether MD has the full Notch tool with Type, Angle and Length properties is unverified.

### 1.2 CLO status (full page fetched) [CLO documented]
Source: support.clo3d.com articles "Notch" (115015735467) and "Edit Notch Properties" (115015735687).

Access:
- 2D toolbar, Notch tool; or Main Menu, 2D Pattern, Create, Notch.

Creation:
- Manual: hover over a pattern outline or internal line and click the segment. A red point previews the position; the new notch is selected and highlighted yellow.
- Specific position: right-click a segment to open the Notch window and enter the exact location. Its modes:
  - Split into Two Lines: divides the line at the reference point created by the right-click. The shorter side becomes Line 1 and the longer side Line 2. Default ratio 100%; typing a ratio in either field sets the split.
  - Split by Length: divides the selected line into equal-length segments. Fields: Segment Length, Number of Segments, Direction (reverses the direction in which points are added).
  - Uniform Split: enter a number of segments; line divided equally.

Selection, move, delete:
- Click a notch to select (yellow).
- Move: click and drag, release at target; or right-click while dragging and type a Moving Distance.
- Delete: select and press Delete, or right-click, Delete. Delete is the only shortcut listed.

Properties (Edit Notch Properties):
- Name: label of the notch.
- Type: notch style (the enumerated styles were not listed on the fetched page).
- Angle: Perpendicular (right angle to the segment) or Custom (angle relative to the segment).
- Direction: shown only for corner notches. Point (Center), Segment 1, Segment 2.
- Flip: moves the notch to the opposite side of the segment.
- Notch Length: added in CLO 5.0.0; adjustable length.

Display settings (v5.0.0+): Main Menu, Setting, User Setting, 2D, Notch.
- Length Fixed to Screen: constant on-screen size when zooming; Notch Length hidden in Property Editor.
- Actual Length: scales with zoom; Notch Length shown in Property Editor.

Seam allowance interaction (v5.0.0+):
- Notches appear on both the pattern outline and its seam allowance.
- Adding seam allowance to a pattern with notches also shows the notches on the allowance.
- A notch added at a corner with seam allowance appears on one side of the corner, chosen at random.

Clone implication: model a notch as (owner segment id, parametric position t, type, angle mode, flip, length, direction for corner notches). Do not assume an MD Notch tool UI without further verification; label this "CLO-documented, MD partial".

## 2. Grain line

[MD] verified in two MD sources.
- Display: 2D Window Display Icons page, section 2D Information, "Show Grainline: Show or hide grain lines from the 2D window." (verbatim text).
- Editing (field guide, Edit Texture section) [single source for the quote]: "To edit the Grainline of a pattern, click on it and select the Grainline Arrow that appears."
- Rotating the texture arc changes only the print rotation, not the grainline.
- The grainline can also be changed in the Property Editor, Fabric section, with the Transform Pattern tool.
- Model: each pattern has a grain direction angle (drives warp/weft for simulation and texture orientation) distinct from the texture image rotation. A numeric field name for the Fabric section was not captured.
- CLO: not fetched in this pass. Marked unknown whether CLO differs.

## 3. Grading and graded sizes

- MD field guide (Segment Points section) [single source]: "Segment points with a thin green outline have grading applied." This proves MD stores grading data per segment point and visualizes it.
- No MD page describing the size list, grade rules, Edit Grade, or size switching was located. No parameters can be stated.
- CLO: the search surfaced only community (forum) threads on graded nests, which are banned sources. No CLO manual page for grading was fetched. Status: [not found] for the tool workflow. Do not invent grade rule semantics.
- Clone guidance: keep a per-point optional "grade" field (delta per size) and a size list; defer UI design until a primary source is found.

## 4. DXF, AI, AAMA, ASTM import and export

- Field guide Import/Export section [single source]: graphics (images for textures/prints) can be imported as JPEG, PNG, AI, PDF or PSD; transparency is preserved by saving as AI or PNG. Transform Graphic section lists PNG, JPEG, PDF, AI, PS.
- This is graphic import, not pattern import. The guide says nothing on DXF.
- The guide's 3D import list includes OBJ, FBX and other 3D formats, not DXF.
- DXF, AAMA, ASTM pattern import/export: [not found] in sources checked. Searches for these were blocked by the exhausted search budget. Unverified; do not claim either way.
- Pattern AI (Illustrator) as a 2D pattern source: not confirmed.

## 5. Pattern Library and block patterns

- Field guide, Library section: "The Add button will link a folder to the Library." This is the asset Library panel (linking folders), not a block/sloper library.
- No page for a block pattern collection was found. [not found].

## 6. Snap and Grid in the 2D window

- Field guide: "Hold shift while clicking to activate smart guides." [single source]
- 2D Window Display Icons page: Show Ruler (2024.1) and Ruler guideline toggles exist; Lock All Guidelines exists, implying guidelines objects.
- A 2D grid/snap settings dialog: [not found]. The 3D "World Coordinate Gizmo" statement ("Selected object moves along the grid axes") concerns 3D only.
- Shared interaction vocabulary from 02-pattern-2d.md (Shift/Ctrl constrain while dragging, right-click to type values) remains the documented way to constrain placement.

## 7. Per-pattern Particle Distance and Pressure

Particle Distance (field guide) [single source]:
- Definition: "the average distance between the points that make up the mesh of the garment."
- Guidance: set 20mm when making a garment and dressing an avatar for faster work.
- Defaults: Hi-Res 5mm, Low-Res 20mm.
- The quotes describe the garment-wide simulation setting. A property-editor page proving a per-pattern override was not fetched here (legacy Property Editor page is robots-blocked). Per-pattern location: unverified.

Pressure (field guide) [single source]:
- "Use pressure to inflate pattern pieces to create quilted/wadded/padded effects."
- Garment must be sewn and constructed before applying Pressure, or pieces float away.
- Apply only to pattern pieces that have a Layer Clone.
- Related: Layer Clone "Duplicate pattern pieces with sewing attached" for linings or quilting.
- Numeric parameter name and range: not captured.

## 8. Rectangle tool

[MD] field guide [single source]: "Create a rectangular pattern shape." Hold Shift while drawing to constrain to square proportions.
- Polygon tool: draw free-form shapes; hold Ctrl and click to create curve points.
- The Summary of 02-pattern-2d.md already lists Rectangle among Create tools (with Internal Rectangle). The numeric size dialog (right-click on draw) is in line with the shared vocabulary but was not directly verified for Rectangle here.

## 9. Seam Allowance

- Field guide table of contents lists "Walk Patterns & Seam Allowance"; the body gives no instructions (fetched text: not covered).
- MD Seamline Property (ver. 2025.0) mentions seam allowance only as an example of cloth that "folds on itself such as seam allowances and pleats", for the Sublayer setting [single source].
- CLO Notch page documents seam allowance behaviour in v5.0.0+ (see 1.2), proving CLO has a Seam Allowance feature at least from CLO 5.0.0.
- Whether MD has an Add Seam Allowance command: unresolved. The field guide TOC title suggests MD has some seam allowance function (Offset or Walk Patterns). Offset was reported "not covered" by the field guide. Do not mark CLO only; mark unresolved.

## 10. Trace

[MD] full page, support.marvelousdesigner.com Trace article.
Purpose: convert selected lines, shapes or areas into patterns or internal lines/shapes.

Access: Main Menu, 2D Pattern, Trace; or 2D Toolbar, Trace.

Operation:
1. Open the tool.
2. Select baselines to trace.
3. Right-click the selection.
4. Choose one:
   - Trace as a Pattern: new pattern from selected baselines.
   - Trace as an Internal Shape: joins intersecting selected lines into one shape.
   - Trace as Internal Lines/Shapes: keeps selected lines as they are.

Options:
- Half-symmetry: select internal lines or outlines on both sides; symmetry carries to the new pattern; internal lines and baselines inside the selection are included.
- Double-click an internal line to select a closed shape.
- Marquee-drag to select all internal lines inside the box.
- Field guide shortcut: select lines and press Enter to trace into internal lines and shapes.
- No separate numeric parameters.

## 11. Trace from 3D

- The Trace page covers only 2D baselines and internal lines; it does not mention 3D.
- Field guide: "Draw lines on the 3D garment." under the 3D Pattern Line tool. This is the closest feature: it draws lines in 3D that relate to 2D patterns.
- Status: [not found] as a named "Trace from 3D". Not claimed as CLO only either (CLO was not checked).

## 12. Superimpose

[MD] full page, Superimpose article; field guide corroborates ("arrange patterns directly over, under, or next to another pattern").

Purpose: arrange a newly sewn pattern so it lies directly on its corresponding pattern; for facings, linings, cuffs, collars.

Access: 3D Toolbar, Select/Move, right-click a 3D pattern. Pop-up: Superimpose (Over), Superimpose (Under), Superimpose (Side).

Parameters: none beyond Over, Under, Side.

Operation:
1. Sew the new pattern to an existing draped pattern with sewing tools.
2. Right-click the 3D pattern, Select/Move, choose the option.
3. The pattern is rearranged to that placement.

Limitation: only works on patterns already sewn together.
Note: this is a 3D-window operation, not a 2D tool; the original request grouped it with 2D features.

## 13. 2D measurement tools

- Display toggles (2D Window Display Icons page): Show Line Length (shows line length; label says Angle but only length is described), Show 2D Measurement (description duplicates the grain line text, an apparent page error), Show Ruler (2024.1), Ruler guideline.
- Field guide Measure section [single source]: Basic and Surface Circumference Measure, Basic and Surface Tape Measure, Linear Tape Measure, Height Measure, Garment Measure tools; editing existing Measures and Tapes; Circumference Garment Measure wraps a 3D garment parallel to the floor; Attach to Measure links a garment edge to an Avatar Tape. These are 3D tools.
- A dedicated 2D measure/dimension tool page: [not found]. 02-pattern-2d.md already covers Edit Length/Angle style editing.

## 14. Pattern annotations

[not found]. No text annotation or comment object on 2D patterns appeared. Show Pattern Names, Show Pattern Layer (2024.1) are display toggles, not annotations.

## 15. Other verified 2D display facts (from Display Icons page)

- Lock groups: Lock All Pattern Outlines, Lock All Internal Lines, Lock All Guidelines, Lock All Remesh Patterns.
- Rendering styles: Texture Surface (Shift+T, 2025.0), Monochrome, Opaque (Ver 11) / Translucent (Ver 12), Transparent, Mesh/Wireframe, Back Texture Surface, Random Color Surface (Ver 10).
- Sewing display: Show Sewing Lines, Show Topstitches, Puckering toggle; Show 3D Pen; Show Symmetric/Instance Lines.

## 16. Follow-up queue (not done due to search cap)

1. DXF-AAMA/ASTM import and export in MD and CLO (look for "Import/Export, Open DXF, Export Pattern as DXF").
2. CLO Grading: "Edit Grade", "Size Editor", "Add Size".
3. CLO and MD Seam Allowance / Offset page.
4. Pattern Library or Block library in CLO.
5. Property Editor pattern section for Particle Distance and Pressure fields (non-legacy MD article).
6. Snap and Grid in User Setting, 2D (CLO has a User Setting 2D section per the Notch page).

## Sources

- https://support.marvelousdesigner.com/hc/en-us/articles/47358246789529-2D-Window
- https://support.marvelousdesigner.com/hc/en-us/articles/47358289149465-Trace
- https://support.marvelousdesigner.com/hc/en-us/articles/47358389619481-Superimpose
- https://support.marvelousdesigner.com/hc/en-us/articles/47358205499033-2D-Window-Display-Icons-Ver-5-1-4-and-above
- https://support.marvelousdesigner.com/hc/en-us/articles/47358171818521-Seamline-Property-ver-2025-0
- https://s3.marvelousdesigner.com/newmdweb/case/20240626/MD+User+Guide+2024.pdf (Creator's Field Guide 2024)
- https://support.clo3d.com/hc/en-us/articles/115015735467-Notch
- https://support.clo3d.com/hc/en-us/articles/115015735687-Edit-Notch-Properties
