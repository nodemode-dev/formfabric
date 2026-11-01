---
name: formfabric-modeling
description: Draft garment sewing patterns in formfabric, the pattern and cloth simulation app, by asking in plain language through its MCP server. Use whenever the person wants a pattern, panel, bodice, sleeve, skirt or other flat piece drawn or changed, points moved, curves adjusted, inner lines or shapes added, panels measured, mirrored, duplicated or renamed, units changed, a project saved or opened, or a step undone, and the formfabric tools (pattern_get, pattern_draw_panel and the rest) are available or the person mentions formfabric. Covers reading the document, working in millimetres, building outlines as point lists, curve handles, checking measurements and recovering from tool errors. Not for editing formfabric's own source code (use formfabric-harness for that).
license: Apache-2.0
---

# Drafting patterns in formfabric

formfabric's engine holds one garment document on this computer: a list of panels, each an outline of points with optional Bezier handles, plus inner lines and shapes. The tools change that document directly; the formfabric window, when open, shows each change live. Every change is one undoable step (`history_undo`, `history_redo`, `history_list`).

The tool reference below is generated from the engine's command registry by `./formfabric agents sync`. Read it when a parameter is unclear.

## Before anything

1. Call `pattern_get` to see the panels, their ids, outlines and the selection. An empty document has no panels.
2. All geometry is in millimetres, x to the right and y up, in the pattern plane. Panel outline points are panel-local; `position`, `rotation` and `mirror` place the panel.
3. Call `view_units` if the person speaks in centimetres or inches; convert to millimetres for arguments unless a tool says otherwise.

## Drafting a panel

1. For a rectangle use `pattern_rectangle` (x, y, width, height); for an ellipse `pattern_ellipse`; for any other shape `pattern_draw_panel` with `points` in order around the outline.
2. A point has a `kind` (corner, curve or smooth) and optional handles `h_in` and `h_out` as offsets from the point. A segment between two points is a cubic Bezier when either end has a handle, otherwise a straight line.
3. Shape curves with `pattern_curvature` (a segment and an amount) or `pattern_set_handle`; add detail with `pattern_add_point` or `pattern_split_line`; relax noise with `pattern_smooth`.
4. Add darts, notches and guide lines as inner lines with `pattern_inner_line` and shapes with `pattern_inner_shape`.
5. Use `pattern_set_props` for particle distance, layer and grain angle.

## Checking work

1. `pattern_measure` returns segment lengths, perimeter and area; compare with the figures the person gave and correct with `pattern_move_point`.
2. Check mirrored pairs (left and right sleeve) have equal perimeters. Use `pattern_duplicate` then `pattern_transform` with `mirror` rather than redrawing.
3. `pattern_snap` is read-only: it reports where a point would snap and to what.
4. Save with `file_save` (a `path` ending in `.ffbrc`); reopen with `file_open`.

## When a tool fails

- `bad_args`: the schema failed; read the parameter list below and resend with the right shape.
- `unknown_command`: the name is not in the catalogue; check the reference.
- A 503 or "engine module is not built": tell the person to run `./formfabric engine build`.
- Anything that goes wrong after several edits: `history_list`, then `history_undo` back to the last good step.

## Tool reference

[ffb-tools:begin]
Generated from the engine's command registry (formfabric 0.6.0) by `./formfabric agents sync`; do not edit between the markers by hand.

- Engine: `system_info`, `system_capabilities`
- Files: `file_new`, `file_save`, `file_open`, `exchange_import_dxf`, `exchange_export_dxf`
- History: `history_undo`, `history_redo`, `history_list`
- Viewing: `view_units`, `mesh_get`, `mesh_stats`
- Pattern: `pattern_get`, `pattern_select`, `pattern_measure`, `pattern_snap`, `pattern_draw_panel`, `pattern_rectangle`, `pattern_ellipse`, `pattern_inner_shape`, `pattern_inner_line`, `pattern_move_point`, `pattern_set_handle`, `pattern_add_point`, `pattern_split_line`, `pattern_delete_point`, `pattern_set_point_kind`, `pattern_curvature`, `pattern_smooth`, `pattern_transform`, `pattern_duplicate`, `pattern_delete`, `pattern_rename`, `pattern_set_props`, `pattern_begin_drag`, `pattern_end_drag`, `pattern_seam_allowance`, `pattern_offset_outline`, `pattern_notch_add`, `pattern_notch_set`, `pattern_notch_delete`, `pattern_grain_line`, `pattern_set_inner`, `pattern_segment_lengths`, `pattern_match_length`, `pattern_dart`, `pattern_slash_spread`, `pattern_unfold`, `pattern_spiral`, `pattern_trace`, `pattern_mirror_link`, `pattern_instance`, `pattern_unlink`
- Simulation: `mesh_build`, `arrange_place`, `arrange_reset`
- Fabrics: `fabric_presets`, `fabric_list`, `fabric_create`, `fabric_update`, `fabric_duplicate`, `fabric_delete`, `fabric_assign`, `fabric_set_default`, `fabric_lab`, `fabric_export`, `fabric_import`, `colorway_create`, `colorway_update`, `colorway_delete`, `colorway_activate`

### system_info

The engine's name and version. Use it to confirm the engine is reachable before other calls.

- no parameters

Note: read-only.

### system_capabilities

What this engine build offers: command count and groups, the length units, the project file format, the history limits and whether it runs as WebAssembly.

- no parameters

Note: read-only.

### file_new

Discard the current garment and history and start an empty project in millimetres. Unsaved work is lost.

- no parameters

Note: removes or replaces (undoable); in the window: Mod+N.

### file_save

Write the garment to a .ffbrc project file (a zip with manifest.json and garment.json). Entries of an opened project this engine does not know are kept.

- `path`: string; required; File path of the .ffbrc project.

Note: in the window: Mod+S.

### file_open

Replace the current garment with the one in a .ffbrc project file and clear the history. Unsaved work is lost.

- `path`: string; required; File path of the .ffbrc project.

Note: removes or replaces (undoable); in the window: Mod+O.

### history_undo

Undo the last step of the pattern history (a grouped drag counts as one step). Fails with nothing_to_undo when the history is at its start.

- no parameters

Note: in the window: Mod+Z.

### history_redo

Redo the step last undone. Fails with nothing_to_redo when there is none; a new edit after an undo drops the redo steps.

- no parameters

Note: in the window: Mod+Shift+Z.

### history_list

Every history step with its label and whether it is applied, the current position, the step limit and whether undo and redo are available.

- no parameters

Note: read-only.

### view_units

Get or set the display length unit (mm, cm or in). Geometry is always stored in millimetres; only the unit shown and typed changes. Setting it is an undo step.

- `units`: string (mm | cm | in); Length unit to switch to. Leave out to read the current unit.

Note: safe to repeat.

### pattern_get

Return the garment document: units, every panel with outline points, handles, inner shapes and placement, and the selection. Pass panel to return only that panel.

- `panel`: integer; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: read-only.

### pattern_select

Hit-test without changing the document. With at, return the panels under the position (top layer first) and the points and segments within tolerance, nearest first. With box, return panels fully inside (or touched when touch is true) and the points and segments inside the rectangle.

- `at`: array of number; World position to hit-test.
- `box`: array of number; World rectangle [x0, y0, x1, y1] to select inside.
- `tolerance`: number; default 5.0; range 0.1 to 1000.0; Pick distance for points and segments under at.
- `touch`: boolean; default false; With box, also select panels the box only touches.

Note: read-only.

### pattern_measure

Measure a panel: segment lengths and chords, perimeter, area, orientation, centroid, bounds, self-intersections and the same for each inner shape. Without panel it reports every panel. All values are in mm and mm squared.

- `panel`: integer; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: read-only.

### pattern_snap

Find where a world position snaps to: an outline point, a segment midpoint, an inner shape, the nearest spot on a segment, or the grid, within a tolerance in mm. Does not change the document. Returns the snapped position and the kind of target.

- `exclude`: array of integer; Panel ids to ignore, such as the panel being dragged.
- `grid`: number; default 10.0; range 0.0 to 1000.0; Grid spacing. 0 turns the grid target off.
- `point`: array of number; required; World position to snap.
- `targets`: array of string; Snap targets to try, in priority order point, midpoint, inner, segment, grid. Defaults to all. inner covers inner shape points, midpoints and lines.
- `tolerance`: number; default 5.0; range 0.01 to 1000.0; Largest distance a target may be from the point.

Note: read-only.

### pattern_draw_panel

Create a pattern panel from outline points given in world mm. Points may carry curve kinds and Bezier handles. The outline is stored counter-clockwise, placed at its lower left corner, and rejected when it has no area or crosses itself. Returns the new panel id.

- `name`: string; Panel name. Defaults to Panel and the new id.
- `particle_distance`: number; default 20.0; range 1.0 to 200.0; Simulation particle spacing for this panel.
- `points`: array of value; required; Outline vertices in world mm, at least 3, in drawing order. A point is [x, y] or an object with p, kind, h_in and h_out. The outline is stored counter-clockwise.

### pattern_rectangle

Create a rectangular panel from its lower left corner, width and height in mm, with an optional corner radius. Returns the new panel id.

- `corner_radius`: number; default 0.0; range 0.0 to 20000.0; Round all four corners by this tangent length; 0 keeps them sharp. Limited to half of the shorter side.
- `height`: number; required; range 0.1 to 20000.0; Height along y.
- `name`: string; Panel name. Defaults to Panel and the new id.
- `particle_distance`: number; default 20.0; range 1.0 to 200.0; Simulation particle spacing for this panel.
- `width`: number; required; range 0.1 to 20000.0; Width along x.
- `x`: number; default 0.0; range -100000.0 to 100000.0; Left edge.
- `y`: number; default 0.0; range -100000.0 to 100000.0; Bottom edge.

### pattern_ellipse

Create an elliptical panel from its centre and radii in mm as 4 or more smooth curve points with exact Bezier handles. Equal radii make a circle. Returns the new panel id.

- `cx`: number; default 0.0; range -100000.0 to 100000.0; Centre x.
- `cy`: number; default 0.0; range -100000.0 to 100000.0; Centre y.
- `name`: string; Panel name. Defaults to Panel and the new id.
- `particle_distance`: number; default 20.0; range 1.0 to 200.0; Simulation particle spacing for this panel.
- `rx`: number; required; range 0.1 to 20000.0; Radius along x.
- `ry`: number; required; range 0.1 to 20000.0; Radius along y. Equal to rx gives a circle.
- `segments`: integer; default 4.0; range 4.0 to 64.0; Number of curve points around the ellipse. 4 already gives a close fit.

### pattern_inner_shape

Add an inner line, polyline, polygon, rectangle or ellipse to a panel. The shape must lie inside the outline. Rectangles take rect, ellipses take ellipse, the others take points. Optional fold angle and strength apply to open inner lines.

- `closed`: boolean; default false; Close a polyline into a loop. Polygons, rectangles and ellipses are always closed.
- `ellipse`: object; fields cx, cy, rx, ry, segments
- `fold_angle`: number; default 180.0; range 0.0 to 360.0; Fold angle of the inner line: 180 is flat, lower folds to the front, higher to the back.
- `fold_strength`: number; default 0.0; range 0.0 to 20.0; How firmly the fold holds its angle, 0 for none.
- `kind`: string (line | polyline | rectangle | ellipse | polygon); required; Shape to add. rectangle uses rect, ellipse uses ellipse, the others use points.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `points`: array of value; Vertices: exactly 2 for line, at least 2 for polyline, at least 3 for polygon.
- `rect`: object; fields height, width, x, y
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.

### pattern_inner_line

Add an open inner line through two or more points inside a panel, for folds, darts guides or sewing references. Returns the inner shape id.

- `fold_angle`: number; default 180.0; range 0.0 to 360.0; Fold angle of the inner line: 180 is flat, lower folds to the front, higher to the back.
- `fold_strength`: number; default 0.0; range 0.0 to 20.0; How firmly the fold holds its angle, 0 for none.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `points`: array of value; required; Vertices of the inner line, at least 2. Two points make a line, more make a polyline.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.

### pattern_move_point

Move one outline or inner-shape point to a position (to) or move one or several points by an offset (delta). Handles travel with the point and curve-kind neighbours re-flow. Use pattern.begin_drag and pattern.end_drag around many calls to make one undo step.

- `delta`: array of number; Offset to add to every selected point.
- `index`: integer; range 0.0 to 100000.0; Point to move.
- `indices`: array of integer; Several points to move together by delta; counts from 0.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `to`: array of number; New position of the single point.

Note: safe to repeat; in the window: Z.

### pattern_set_handle

Set or remove the in or out Bezier handle of a point as an offset vector. On smooth points the opposite handle keeps its length and turns to stay collinear; with independent it does not.

- `independent`: boolean; default false; Break tangent continuity so the opposite handle stays put and the point becomes a corner.
- `index`: integer; required; range 0.0 to 100000.0; Point whose handle to edit.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `vector`: array of number; Handle as an offset from the point. Leave out to remove the handle.
- `which`: string (in | out); required; in: handle toward the previous point. out: handle toward the next point.

Note: safe to repeat.

### pattern_add_point

Insert a point on a segment at curve parameter t or at the spot nearest to a position. The shape does not change; a curved segment is split into two exact curves. Returns the new point index.

- `at`: array of number; Place the point at the spot on the segment nearest to this position.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; range 0.0 to 100000.0; Segment to add the point on; segment i runs from point i to point i+1. Optional with at.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `t`: number; range 0.001 to 0.999; Curve parameter between 0 and 1 on the segment.

### pattern_split_line

Split a segment by curve parameter t, by distance from its start in mm, or into equal pieces by count. Returns the indices of the new points.

- `count`: integer; range 2.0 to 64.0; Split into this many pieces of equal length.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `length`: number; range 0.001 to 20000.0; Split at this distance along the segment from its start.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; required; range 0.0 to 100000.0; Segment to split; segment i runs from point i to point i+1.
- `t`: number; default 0.5; range 0.001 to 0.999; Split at this curve parameter between 0 and 1. The default is 0.5.

### pattern_delete_point

Remove a point; its two segments merge into one. An outline keeps at least 3 points, an open inner line at least 2.

- `index`: integer; required; range 0.0 to 100000.0; Point to remove.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: removes or replaces (undoable).

### pattern_set_point_kind

Convert a point to corner (sharp, no handles), curve (automatic handles that follow the neighbours) or smooth (editable collinear handles).

- `index`: integer; required; range 0.0 to 100000.0; Point to convert.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `kind`: string (corner | curve | smooth); required; corner: sharp, handles removed. curve: handles follow the neighbours automatically. smooth: keeps its handles collinear.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: safe to repeat.

### pattern_curvature

Bend a segment into a symmetric curve by the bulge of its middle in mm; 0 straightens it. Positive bulges to the right of the segment direction.

- `amount`: number; required; range -20000.0 to 20000.0; How far the middle of the segment bulges from the straight chord. Positive bulges to the right of the segment direction, which is outward on a counter-clockwise outline. 0 straightens the segment.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; required; range 0.0 to 100000.0; Segment to bend; segment i runs from point i to point i+1.

Note: safe to repeat.

### pattern_smooth

Round corner points with a Bezier arc that starts the given length from each corner along both edges. Returns how many corners were rounded and how many skipped.

- `curvature`: number; default 1.0; range 0.0 to 1.5; 1 is a circular arc, 0 a straight bevel, above 1 a fuller corner.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `length`: number; default 20.0; range 0.1 to 20000.0; Distance from the corner where the rounding starts along each edge. Limited to half of the shorter neighbouring edge.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `points`: array of integer; required; Corner points to round, counted from 0. Corners next to curved segments are skipped.

### pattern_transform

Mirror, scale, rotate and move whole panels in this order about a pivot (default: the centre of the selection). Rotation and translation change the panel placement; uneven scale and mirror rewrite the outline and keep it counter-clockwise. One undo step for all panels.

- `mirror`: string (x | y); x flips left to right across the vertical line through the pivot, y flips top to bottom across the horizontal line.
- `panels`: array of integer; required; Panel ids to transform together.
- `pivot`: array of number; Pivot for rotate, scale and mirror. Defaults to the centre of the selected panels.
- `rotate`: number; default 0.0; range -360.0 to 360.0; Rotate counter-clockwise about the pivot.
- `scale`: ['number', 'array']; range 0.01 to 100; Scale factor about the pivot: one number for uniform scaling or [sx, sy] along the pattern axes. Uneven scaling bakes the panel rotation into its outline.
- `translate`: array of number; Move by this offset.

Note: in the window: A.

### pattern_duplicate

Copy panels with their inner shapes under new ids, shifted by offset per copy. Returns the created panel ids.

- `count`: integer; default 1.0; range 1.0 to 50.0; Number of copies of each panel.
- `offset`: array of number; Shift between one copy and the next, starting from the original. Defaults to [20, 20].
- `panels`: array of integer; required; Panel ids to copy.

Note: in the window: Mod+D.

### pattern_delete

Remove panels and their inner shapes. Undo restores them at the same positions in the list.

- `panels`: array of integer; required; Panel ids to remove, with their inner shapes.

Note: removes or replaces (undoable); in the window: Delete.

### pattern_rename

Give a panel a new name of 1 to 64 characters.

- `name`: string; required; New panel name.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: safe to repeat; in the window: F2.

### pattern_set_props

Set particle distance, layer, grain direction, position or rotation on one or more panels, and the fold angle (0 to 360, 180 flat) and strength (0 to 20) of one inner line. Only the given values change.

- `fold_angle`: number; range 0.0 to 360.0; Fold angle of the inner line: 180 is flat, lower folds to the front, higher to the back.
- `fold_strength`: number; range 0.0 to 20.0; How firmly the fold holds its angle, 0 for none.
- `grain`: number; range -360.0 to 360.0; Grain line direction inside the panel.
- `inner`: integer; range 1.0 to 4294967295.0; Inner line id whose fold is set by fold_angle and fold_strength; needs exactly one panel.
- `layer`: integer; range -100.0 to 100.0; Layer order for overlapping panels; higher lies on top.
- `panels`: array of integer; required; Panel ids to change.
- `particle_distance`: number; range 1.0 to 200.0; Simulation particle spacing.
- `position`: array of number; Absolute panel position on the pattern plane.
- `rotation`: number; range -360.0 to 360.0; Absolute panel rotation.

Note: safe to repeat.

### pattern_begin_drag

Open an undo group so many edits, such as repeated pattern.move_point calls during a mouse drag, become one history step. Close it with pattern.end_drag; undo and redo also close it.

- `label`: string; Name of the undo step the drag becomes. Defaults to Drag.

Note: safe to repeat.

### pattern_end_drag

Close the open undo group and commit its edits as a single history step. Safe to call when no group is open.

- no parameters

Note: safe to repeat.

### pattern_seam_allowance

Give panels a seam allowance stored on the panel: a general width in mm, a corner style (miter, square, round, mirror) and optional widths for single outline segments. clear removes it. The allowance outline is derived from the panel outline and follows later edits; the reply carries the derived outline, its length and area. Notches stay on the sewing outline.

- `clear`: boolean; default false; Remove the seam allowance from the panels.
- `corners`: string (miter | square | round | mirror); How a corner of the offset line is finished where the outline turns away from the offset. miter: the offset edges meet in a sharp point, cut back to a square corner beyond the miter limit. square: each edge is extended by the offset distance and cut square. round: a circular arc around the corner. mirror: a chamfer cut across the corner at the offset distance from it, as for a folded mitred corner.
- `panels`: array of integer; required; Panels to give or change the seam allowance of.
- `per_segment`: array of ['number', 'null']; Allowance width per outline segment, counted from segment 0; null uses the general width, and a shorter list leaves the rest on the general width. Needs exactly one panel.
- `width`: number; range 0.0 to 500.0; Allowance width on every segment without its own override. A new allowance starts at 10 mm.

Note: safe to repeat.

### pattern_offset_outline

Offset a panel outline by a distance in mm (positive outward, negative inward), with Bezier segments refitted as cubic curves, per-segment distances, corner styles miter, square, round or mirror, and loops from inward offsets removed. Creates closed inner lines (inward only) or new panels laid over the source. Count makes several offsets in one step.

- `as`: string (inner | panel); inner: closed inner lines inside the panel, which needs a negative distance. panel: new panels with the offset outline, placed over the source.
- `corners`: string (miter | square | round | mirror); How a corner of the offset line is finished where the outline turns away from the offset. miter: the offset edges meet in a sharp point, cut back to a square corner beyond the miter limit. square: each edge is extended by the offset distance and cut square. round: a circular arc around the corner. mirror: a chamfer cut across the corner at the offset distance from it, as for a folded mitred corner.
- `count`: integer; default 1.0; range 1.0 to 20.0; Number of offsets, each one distance further than the last.
- `distance`: number; required; range -1000.0 to 1000.0; Offset distance. Positive grows the outline outward, negative retracts it inward.
- `miter_limit`: number; default 4.0; range 1.0 to 20.0; Longest miter, as a multiple of the distance from the corner; beyond it a miter corner is cut square.
- `name`: string; Name of the new panel. Defaults to the source name and offset.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `per_segment`: array of ['number', 'null']; Offset distance per outline segment, counted from segment 0, with the same sign as distance; null uses distance. Segments the list does not reach use distance.

### pattern_notch_add

Add a notch to the panel outline at curve parameter t of a segment, at a distance along a segment (length), or at the spot nearest a position (at). Kinds slit, v, t and u with depth, width and angle. Notches are stored by segment and t, so they keep their place when points are added, segments split or the panel moves. Returns the notch id, its local and world position and its outward normal.

- `angle`: number; default 90.0; range 10.0 to 170.0; Angle between the notch and the outline tangent; 90 is square to the edge.
- `at`: array of number; Place the notch at the spot on the outline nearest to this position.
- `depth`: number; default 6.0; range 0.5 to 100.0; How far the notch reaches from the outline.
- `kind`: string (slit | v | t | u); slit: a short cut into the allowance. v: a V-shaped wedge. t: a T-shaped cut. u: a U-shaped cut.
- `length`: number; range 0.0 to 20000.0; Distance from the start of the segment along the curve.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; range 0.0 to 100000.0; Outline segment the notch sits on; segment i runs from point i to point i+1. Optional with at.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `t`: number; range 0.0 to 1.0; Curve parameter between 0 and 1 along the segment.
- `width`: number; default 3.0; range 0.1 to 100.0; Width of the notch across its opening; ignored by slits.

### pattern_notch_set

Change a notch: move it with segment and t, length or at, or change its kind, depth, width and angle. Only the given values change.

- `angle`: number; range 10.0 to 170.0; Angle between the notch and the outline tangent; 90 is square to the edge.
- `at`: array of number; Place the notch at the spot on the outline nearest to this position.
- `depth`: number; range 0.5 to 100.0; How far the notch reaches from the outline.
- `kind`: string (slit | v | t | u); slit: a short cut into the allowance. v: a V-shaped wedge. t: a T-shaped cut. u: a U-shaped cut.
- `length`: number; range 0.0 to 20000.0; Distance from the start of the segment along the curve.
- `notch`: integer; required; range 1.0 to 4294967295.0; Notch id as listed in the panel's notches.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; range 0.0 to 100000.0; Outline segment the notch sits on; segment i runs from point i to point i+1. Optional with at.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `t`: number; range 0.0 to 1.0; Curve parameter between 0 and 1 along the segment.
- `width`: number; range 0.1 to 100.0; Width of the notch across its opening; ignored by slits.

Note: safe to repeat.

### pattern_notch_delete

Remove one notch by id, several by notches, or every notch of the panel with all.

- `all`: boolean; default false; Remove every notch of the panel.
- `notch`: integer; range 1.0 to 4294967295.0; Notch id to remove.
- `notches`: array of integer; Several notch ids to remove.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: removes or replaces (undoable).

### pattern_grain_line

Set the grain line arrow of a panel from two points (from, to) or from an angle (centred on the panel, with an optional length), or clear it. Without a stored line the panel shows a default: vertical through the panel centre, half the panel height long; pattern.get reports it as grain_line_effective with computed true. Does not change the panel grain angle unless sync_grain is set.

- `angle`: number; range -360.0 to 360.0; Direction of the line, counter-clockwise from the pattern x axis; 90 is vertical. The line is centred on the panel unless center is given.
- `center`: array of number; Centre of a line made from angle. Defaults to the panel centre.
- `clear`: boolean; default false; Remove the stored line so the default, vertical through the panel centre, applies again.
- `from`: array of number; Start of the grain line.
- `length`: number; range 1.0 to 20000.0; Length of a line made from angle. Defaults to half the panel height, at least 20 mm.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `sync_grain`: boolean; default false; Also set the panel's grain angle to the line direction.
- `to`: array of number; End of the grain line; the arrow points here.

Note: safe to repeat.

### pattern_set_inner

Set the role (shape, fold, dart, reference), fold angle (0 to 360, 180 flat, lower folds to the front, higher to the back), fold strength (0 to 20) and closed state of an inner shape. Only the given values change; geometry is untouched.

- `closed`: boolean; Whether the shape closes back on its first point.
- `fold_angle`: number; range 0.0 to 360.0; Fold angle of the inner line: 180 is flat, lower folds to the front, higher to the back.
- `fold_strength`: number; range 0.0 to 20.0; How firmly the fold holds its angle, 0 for none.
- `inner`: integer; required; range 1.0 to 4294967295.0; Inner shape id inside the panel.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `role`: string (shape | fold | dart | reference); shape: ordinary inner line. fold: the line folds at fold_angle. dart: a dart. reference: a construction line that does not shape the cloth.

Note: safe to repeat.

### pattern_segment_lengths

List the length of every segment of a panel outline or inner shape in mm, with chord, end points, running total and the notches on each segment at their distance along it. With segments, also return the sum of those segments. Without panel it lists every panel outline. Does not change the document.

- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `panel`: integer; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segments`: array of integer; Only these segments count toward selected_total; the list still shows every segment.

Note: read-only.

### pattern_match_length

Make a run of consecutive segments as long as the sum of reference segments (of this or another panel) or a given length, plus an optional ease. move_end slides the end point of the run along its chord; scale scales the run about its first point. Returns the lengths before and after.

- `ease`: number; default 0.0; range -20000.0 to 20000.0; Added to the wanted length; negative shortens, as for easing a sleeve cap into an armhole.
- `how`: string (move_end | scale); move_end: slide the end point of the run along its chord until the run has the wanted length, the shape of the points before it unchanged. scale: scale the run about its first point, keeping its shape.
- `inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `length`: number; range 0.1 to 20000.0; Wanted length instead of a reference.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `reference_inner`: integer; range 1.0 to 4294967295.0; Inner shape id inside the panel. Leave out to edit the panel outline.
- `reference_panel`: integer; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `reference_segments`: array of integer; Segments of the reference whose lengths add up to the wanted length. The reference panel defaults to the target panel.
- `segments`: array of integer; required; Consecutive segments of the target to resize together. For an outline the run may wrap past the last segment.

Note: safe to repeat.

### pattern_dart

Cut a dart into a panel outline: the opening is width mm wide, centred at t on a segment, and its two equal legs meet at an apex length mm along the inward normal (or at a given apex point). The outline stays closed and valid or the command fails with invalid_geometry. With internal true it adds a closed triangular inner shape with role dart instead. Linked copies of the panel follow. Fails with linked_panel on a mirror copy or instance.

- `angle`: number; range -360.0 to 360.0; Internal dart only: direction of the apex from the base centre, used with center and length.
- `apex`: array of number; Apex point instead of a length.
- `center`: array of number; Internal dart only: base centre, used instead of segment and t.
- `inset`: number; default 5.0; range 0.0 to 1000.0; Internal dart only: distance the base is moved inside the outline from the segment point.
- `internal`: boolean; default false; Create a closed inner dart shape with role dart instead of cutting the outline.
- `length`: number; range 0.1 to 20000.0; Dart length from the middle of the opening to the apex, along the inward normal. Give length or apex.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; range 0.0 to 100000.0; Outline segment that carries the dart opening. Segment i runs from point i to point i+1. For an internal dart it only locates the base.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `t`: number; default 0.5; range 0.0 to 1.0; Position of the dart centre along the segment, 0 at its start and 1 at its end.
- `width`: number; required; range 0.1 to 20000.0; Dart opening (intake) measured along the segment, centred on t. For an internal dart the width of its base.

### pattern_slash_spread

Cut a panel along a line from edge to edge and rotate the part on one side about the hinge end, by a spread distance at the free end or by an angle. The outline stays closed; the opened end is joined by a curve that continues the neighbouring tangents. Inner shapes and the grain line move with their part. Fails with invalid_geometry when the result would cross itself or an inner shape crosses the cut.

- `angle`: number; range -180.0 to 180.0; Rotation of the moving part in degrees instead of a distance. Positive opens the cut.
- `from`: array of number; required; First point of the cut line. The line runs through from and to and is used between the outline crossings nearest to them.
- `hinge`: string (from | to); default "from"; Which end of the cut stays joined. The part rotates about it.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `side`: string (left | right | both); default "left"; Which part moves, seen from the hinge looking along the cut: left, right, or both by half the spread each.
- `smooth`: boolean; default true; Join the opened end with a curve that continues the neighbouring outline tangents. false uses a straight edge.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply.
- `spread`: number; range -20000.0 to 20000.0; Opening of the cut at its free end, measured as a straight distance. Give spread or angle. Negative closes the cut.
- `to`: array of number; required; Second point of the cut line.

### pattern_unfold

Mirror a half panel across one of its straight outline segments into one symmetric outline; the segment disappears. Inner shapes, notches and seam allowance values are mirrored too (inner shapes lying on the axis are not doubled).

- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `segment`: integer; required; range 0.0 to 100000.0; Straight outline segment to unfold about, usually the centre front or centre back. It disappears; the half is mirrored across it.

### pattern_spiral

Create a panel shaped as a flat spiral strip of a given width that winds from the inner radius to the outer radius over a number of turns. Fails with invalid_geometry when neighbouring turns would overlap.

- `center`: array of number; Centre of the spiral on the pattern plane.
- `direction`: string (ccw | cw); default "ccw"; Winding direction outwards.
- `inner_radius`: number; required; range 0.1 to 20000.0; Radius of the strip centre line where the spiral starts.
- `name`: string; Panel name. Defaults to Spiral and the new id.
- `outer_radius`: number; required; range 0.1 to 20000.0; Radius of the strip centre line where the spiral ends.
- `particle_distance`: number; default 20.0; range 1.0 to 200.0; Simulation particle spacing for this panel.
- `start_angle`: number; default 0.0; range -360.0 to 360.0; Angle of the start of the strip.
- `turns`: number; default 3.0; range 0.25 to 20.0; Number of turns from start to end.
- `width`: number; required; range 0.1 to 20000.0; Width of the strip.

### pattern_trace

Create a new panel from a closed inner shape (by inner id, or the smallest one containing the point at) or from a closed region given as vertices. The new panel keeps the traced geometry at the same place on the pattern plane. remove true also deletes the inner shape from the source panel.

- `at`: array of number; World point: traces the smallest closed inner shape of the panel that contains it.
- `inner`: integer; range 1.0 to 4294967295.0; Closed inner shape id to trace into a new panel.
- `name`: string; Panel name. Defaults to Traced and the new id.
- `panel`: integer; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `particle_distance`: number; range 1.0 to 200.0; Simulation particle spacing for the new panel. Defaults to the source panel's.
- `points`: array of array; A closed region given by its vertices in world mm, in any winding. The panel is then optional and only supplies fabric, layer and grain.
- `remove`: boolean; default false; Remove the traced inner shape from the source panel.

### pattern_mirror_link

Create a linked mirror copy of a panel across an axis (two points, or a straight outline segment such as the centre front), optionally set apart by a gap. The copy is derived: after every change to the source it is regenerated, inside the same undo step. Geometry edits made directly on the copy are discarded; pattern.dart, pattern.slash_spread, pattern.unfold and pattern.mirror_link fail with linked_panel on it. Name, fabric, layer, particle distance and 3D placement of the copy stay editable. Use pattern.unlink to make it independent.

- `axis`: array of array; Mirror axis as two points [[x, y], [x, y]] in world mm. Give axis or axis_segment.
- `axis_segment`: integer; range 0.0 to 100000.0; Use the straight line through this outline segment of the panel as the axis (for example the centre front).
- `gap`: number; default 0.0; range 0.0 to 20000.0; Extra distance between the panel and its mirror image across the axis.
- `name`: string; Name of the mirror copy. Defaults to the source name and mirror.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

### pattern_instance

Create count linked copies of a panel, each shifted by the interval from the previous one (the first by one interval from the source). Copies are derived exactly like a mirror copy. Running it again on the same source changes the count and interval of its existing instances.

- `count`: integer; default 1.0; range 1.0 to 50.0; Number of linked copies after the source. Running it again on the same panel changes the number and interval of its existing copies.
- `interval`: array of number; Shift between one copy and the next, in world mm. Defaults to the panel width plus 20 mm along x, or to the interval already in use.
- `name`: string; Base name of new copies. Defaults to the source name and the copy number.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

### pattern_unlink

Make a mirror copy or instance an ordinary independent panel that keeps its current geometry. Given a source panel it unlinks every copy derived from it.

- `panel`: integer; required; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.

Note: safe to repeat.

### exchange_import_dxf

Read a DXF pattern file and create panels from it as one undo step. AAMA files (ASTM D6673 layers) give panels with name, outline (curve points on layer 3 become smooth Bezier handles), notches (layers 4, 80 to 83), grain line (7), internal lines (8, role reference) and, when a sew line (14) lies inside the cut line (1), the sew line as outline with the gap as seam allowance. Plain DXF gives a panel per closed outline from lines, polylines, arcs, circles, splines and inserted blocks, with open lines inside it as internal lines. Reports what was skipped.

- `fit_tolerance`: number; default 0.05; range 0.005 to 1.0; Largest distance in mm between a point of the file and the Bezier curve fitted through the curve points.
- `flavour`: string (auto | aama | plain); default "auto"; aama reads one pattern piece per block with the ASTM D6673 layer numbers (1 cut line, 2 turn points, 3 curve points, 4 notches, 7 grain line, 8 internal lines, 14 sew line). plain reads closed outlines from lines, polylines, arcs, circles, splines and inserted blocks. auto picks aama when the file carries Piece Name or Style Name text.
- `mode`: string (add | as_new); default "add"; add appends the pieces to the pattern to the right of what is there. as_new replaces all panels of the pattern with the imported ones. Either is one undo step.
- `path`: string; required; Path of the DXF file to read. ASCII DXF, release 12 or later.
- `smooth_facets`: boolean; default true; Plain files only. Treat runs of short polyline edges that turn by less than 15 degrees at each vertex as curves, so a curve that another program flattened comes back as a few Bezier segments instead of many corners.
- `units`: string (mm | cm | in); Unit of the numbers in the file. Omit to detect it: AAMA files declare Units METRIC or ENGLISH, other files may carry the header variable INSUNITS, and millimetres is the fallback.
- `use_sew_line`: boolean; default true; When a piece has a closed sew line (layer 14) inside its cut line (layer 1), make the sew line the panel outline and turn the gap into the seam allowance.

### exchange_export_dxf

Write panels to an ASCII DXF file (release 12 entities). The aama flavour follows ASTM D6673: one block per piece with Piece Name, Quantity and Category text, the cut line on layer 1 (outline plus seam allowance when there is one) and the outline as sew line on layer 14, curves flattened to polylines within the tolerance with turn points on layer 2 and curve points on layer 3, notches on layers 4, 80 and 83, grain line on layer 7, internal lines on layer 8 and an annotation on layer 15. The plain flavour writes world-space polylines on named layers. Importing the aama file again restores outline, notches, grain line, internal lines and seam allowance width.

- `flavour`: string (aama | plain); default "aama"; aama writes one block per piece with the ASTM D6673 layer numbers and system text. plain writes one drawing in the layout positions on named layers CUT, SEW, NOTCH, GRAIN, INTERNAL and NAME.
- `grain_line`: boolean; default true; Write grain lines (layer 7).
- `inner_lines`: boolean; default true; Write internal lines and shapes (layer 8). A text label at the start of each line keeps its role, fold angle and fold strength.
- `internal_labels`: boolean; default true; Write the annotation text with the piece name (layer 15) and the role labels of internal lines.
- `notches`: boolean; default true; Write notches.
- `panels`: array of integer; Panel ids to export. Omit to export every panel.
- `path`: string; required; Path of the DXF file to write.
- `seam_allowance`: boolean; default true; Write the panel outline plus its seam allowance as the cut line (layer 1) and the outline itself as the sew line (layer 14). When off, or when a panel has no allowance, the outline is the cut line.
- `tolerance`: number; default 0.1; range 0.005 to 5.0; Largest distance in mm between a curve and the polyline written for it.
- `units`: string (mm | cm | in); default "mm"; Unit of the numbers in the file. AAMA files accept mm and in.

### mesh_build

Triangulate panels for simulation: constrained Delaunay at each panel's particle distance with the outline resampled to that edge length, fold and dart inner lines as constraint edges and interior refinement to a 20 degree minimum angle. Meshes are cached per panel by geometry and particle distance, so only changed panels rebuild unless force is set. Returns per-panel vertex and triangle counts and the time taken. Optional segment_counts forces the vertex count of outline segments of one panel so sewn seams match.

- `force`: boolean; default false; Rebuild even when the cached mesh still matches the panel geometry and particle distance.
- `panels`: array of integer; Panel ids as listed by pattern.get. Leave out for every panel.
- `segment_counts`: object; Forced vertex count per outline segment for a single panel, keyed by segment index. This is how sewn seams get matching vertex counts. An empty object clears the forced counts. Needs exactly one panel.

Note: safe to repeat.

### mesh_get

One panel's mesh: panel-local 2D positions in mm, triangles, the boundary vertex loop with the outline segment and t of each boundary vertex, inner line vertex chains, and by default the 3D positions and indices the stream carries. Builds the mesh when it is not cached.

- `include_3d`: boolean; default true; Also return the 3D positions (flat x y z in mm, from the arrangement or the default flat layout) and the triangle indices as streamed.
- `panel`: integer; required; range 1 to inf; Panel id as listed by pattern.get.

Note: read-only.

### mesh_stats

Mesh statistics per panel: vertex, triangle and boundary counts, minimum angle, a histogram of per-triangle minimum angles in 10 degree bins, edge length range, area, build time and cache counters.

- `panels`: array of integer; Panel ids as listed by pattern.get. Leave out for every panel.

Note: read-only.

### arrange_place

Set where a panel sits in 3D: an origin in mm and a quaternion rotation applied to the panel-local points (x, y, 0). Panels without a placement use the default flat layout, x and y from the pattern plane and z 0. One undo step; the stream positions for that panel update.

- `bend`: number; default 0; range -1000 to 1000; Pre-bend curvature stored with the placement for the solver.
- `origin`: array of number; required; Where the panel-local origin goes in 3D, x y z in mm, y up.
- `panel`: integer; required; range 1 to inf; Panel id as listed by pattern.get.
- `rotation`: array of number; default [0, 0, 0, 1]; Quaternion x y z w applied to the panel-local points (x, y, 0). It is normalised; identity keeps the panel facing +z.

Note: safe to repeat.

### arrange_reset

Clear the 3D placement of the given panels, or of all panels, so they return to the default flat layout. One undo step.

- `panels`: array of integer; Panel ids to return to the default flat layout. Leave out for every panel.

### fabric_presets

List formfabric's own fabric presets with physical values in SI based units, look, source keys, whether each is provisional, and the unit and range of every physical field with the 0 to 99 slider mapping.

- `id`: string; Preset id to return only that preset.

Note: read-only.

### fabric_list

List the garment's fabrics with the panels using each, the default fabric, the colorways and the active colorway. The color of each fabric already accounts for the active colorway.

- no parameters

Note: read-only.

### fabric_create

Add a fabric to the garment from a preset, with optional partial physical and look values. Out-of-range values are refused with bad_args. The first fabric becomes the default.

- `look`: object; Look fields to change; fields left out keep their value.; fields back, front, kind, side
- `name`: string; Fabric name. Defaults to the preset name.
- `physical`: object; Physical properties to change; fields left out keep their value. SI based units.; fields bending_bias, bending_warp, bending_weft, buckling_ratio, buckling_stiffness, damping, density, friction, shear, stretch_warp, stretch_weft, thickness
- `preset`: string; Preset id from fabric.presets. Defaults to the default preset.

### fabric_update

Change a fabric's name, physical properties (SI based units, ranges in the schema) and look. Only the fields given change; every value is range checked and the whole call is refused with bad_args if one is out of range.

- `id`: integer; required; range 1 to 4294967295; Fabric id from fabric.list.
- `look`: object; Look fields to change; fields left out keep their value.; fields back, front, kind, side
- `name`: string; New fabric name.
- `physical`: object; Physical properties to change; fields left out keep their value. SI based units.; fields bending_bias, bending_warp, bending_weft, buckling_ratio, buckling_stiffness, damping, density, friction, shear, stretch_warp, stretch_weft, thickness

Note: safe to repeat.

### fabric_duplicate

Copy a fabric, including its colorway colours, under a new id.

- `id`: integer; required; range 1 to 4294967295; Fabric to copy.
- `name`: string; Name of the copy. Defaults to the original name with copy added.

### fabric_delete

Delete a fabric. The last fabric cannot be deleted, and a fabric used by panels needs reassign naming the fabric that takes them over.

- `id`: integer; required; range 1 to 4294967295; Fabric to delete.
- `reassign`: integer; range 1 to 4294967295; Fabric that takes over the panels using the deleted one. Required when panels use it.

Note: removes or replaces (undoable).

### fabric_assign

Assign a fabric to panels, or clear the assignment so they follow the default fabric. A garment with no fabric gets the default fabric first.

- `clear`: boolean; default false; Remove the explicit assignment so the panels follow the default fabric.
- `fabric`: integer; range 1 to 4294967295; Fabric to assign. Leave out for the default fabric.
- `panels`: array of integer; required; Panel ids that take the fabric.

Note: safe to repeat.

### fabric_set_default

Choose the fabric that panels without an explicit assignment use.

- `id`: integer; required; range 1 to 4294967295; Fabric that panels without an explicit assignment use.

Note: safe to repeat.

### fabric_lab

Turn bench measurements (areal density, thickness, cantilever bending lengths, tensile strip results, shear test) into physical properties, with the formula and source keys behind every value. Fields not measured keep the fabric's values. With apply the result is written to the fabric.

- `apply`: boolean; default false; Write the computed values into the fabric as one undo step.
- `bending_length`: object; fields bias, warp, weft
- `bias_extension`: number; range 0.01 to 100.0; Bias extension EB5 at a load of 5 gf/cm (4.903 N/m) on a 45 degree strip; gives shear stiffness 123 / EB5 when no shear test is given.
- `buckling_ratio`: number; range 0.0 to 1.0; Buckling ratio to use.
- `buckling_stiffness`: number; range 0.0 to 1.0; Buckling stiffness to use.
- `damping`: number; range 0.0 to 1.0; Internal damping to use.
- `density`: number; range 1.0 to 3000.0; Areal density of the cloth.
- `fabric`: integer; range 1 to 4294967295; Fabric whose current values fill the fields not measured, and the target of apply.
- `friction`: number; range 0.0 to 1.0; Measured friction coefficient.
- `shear`: object; fields angle, force, length
- `tensile`: object; fields warp, weft
- `thickness`: number; range 0.01 to 10.0; Thickness of the cloth.

### fabric_export

Write a fabric to a .ffab file: a zip with fabric.json and the map images it uses under maps/.

- `fabric`: integer; required; range 1 to 4294967295; Fabric to export.
- `path`: string; required; File path of the .ffab file to write.

### fabric_import

Add the fabric in a .ffab file to the garment as a new fabric. Its map images are stored inside the project and saved with it.

- `name`: string; Name for the imported fabric. Defaults to the name in the file.
- `path`: string; required; File path of the .ffab file to read.

### colorway_create

Add a colorway: one colour per fabric, copied from another colorway or from each fabric's front colour, with optional overrides.

- `fabric_colors`: object; Colour per fabric id (keys are fabric ids). Fabrics left out use their own front colour.
- `from`: integer; range 1 to 4294967295; Colorway id to copy the colours from. Leave out to start from each fabric's front colour.
- `name`: string; required; Colorway name.

### colorway_update

Rename a colorway or change the colour it gives some fabrics. A null colour removes that fabric's entry.

- `fabric_colors`: object; Colour per fabric id (keys are fabric ids); null removes the entry so the fabric's own colour shows.
- `id`: integer; required; range 1 to 4294967295; Colorway id.
- `name`: string; New name.

Note: safe to repeat.

### colorway_delete

Delete a colorway. If it was active, the fabrics show their own colours again.

- `id`: integer; required; range 1 to 4294967295; Colorway id.

Note: removes or replaces (undoable).

### colorway_activate

Show a colorway: its colour per fabric replaces the fabric's front colour in views and renders. Without id the colorway is switched off.

- `id`: integer; range 1 to 4294967295; Colorway to show. Leave out to show each fabric's own colour.

Note: safe to repeat.
[ffb-tools:end]

## Not yet available

Sewing, simulation, bodies, fabrics and animation commands arrive in later phases; only the tools listed in the reference exist. Say so plainly if asked for one, rather than approximating with pattern edits.
