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
Generated from the engine's command registry (formfabric 0.3.0) by `./formfabric agents sync`; do not edit between the markers by hand.

- Engine: `system_info`, `system_capabilities`
- Files: `file_new`, `file_save`, `file_open`
- History: `history_undo`, `history_redo`, `history_list`
- Viewing: `view_units`
- Pattern: `pattern_get`, `pattern_select`, `pattern_measure`, `pattern_snap`, `pattern_draw_panel`, `pattern_rectangle`, `pattern_ellipse`, `pattern_inner_shape`, `pattern_inner_line`, `pattern_move_point`, `pattern_set_handle`, `pattern_add_point`, `pattern_split_line`, `pattern_delete_point`, `pattern_set_point_kind`, `pattern_curvature`, `pattern_smooth`, `pattern_transform`, `pattern_duplicate`, `pattern_delete`, `pattern_rename`, `pattern_set_props`, `pattern_begin_drag`, `pattern_end_drag`

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
[ffb-tools:end]

## Not yet available

Sewing, simulation, bodies, fabrics and animation commands arrive in later phases; only the tools listed in the reference exist. Say so plainly if asked for one, rather than approximating with pattern edits.
