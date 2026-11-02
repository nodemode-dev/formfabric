---
name: formfabric-modeling
description: Draft garment sewing patterns in formfabric, the pattern and cloth simulation app, by asking in plain language through its MCP server. Use whenever the person wants a pattern, panel, bodice, sleeve, skirt or other flat piece drawn or changed, points moved, curves adjusted, inner lines or shapes added, panels measured, mirrored, duplicated or renamed, units changed, a project saved or opened, or a step undone, and the formfabric tools (pattern_get, pattern_draw_panel and the rest) are available or the person mentions formfabric. Also covers sewing seams, draping a sample or a pattern on a body, fabrics, exporting the draped garment, the Live Link that streams it to Unreal as USD, and scripting formfabric from Python. Covers reading the document, working in millimetres, building outlines as point lists, curve handles, checking measurements and recovering from tool errors. Not for editing formfabric's own source code (use formfabric-harness for that).
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

## Sewing and draping

1. Fastest start: `sample_list`, then `sample_load` with `name` tshirt, skirt or dress and `force` true (it replaces an unsaved project; ask first). It drafts the panels, sews them, loads the default body and places the panels.
2. Own patterns: draft the panels, then `sew_segment` (one outline segment to another), `sew_multi` or `sew_auto`; `sew_check` reports length mismatches before the drape. Assign a fabric with `fabric_presets`, `fabric_create` and `fabric_assign`; `body_load_default` brings a body and `place_panel` or `arrange_place` puts panels around it.
3. Drape: `sim_reset` (frame zero, placed state), then `sim_step` with `frames` (about 60 to 120 settles a sample) or `sim_start` and `sim_stop` for a background run; `sim_state` reports the frame, vertex count and seam gap. Pins (`pin_add`) hold points; `sim_settings` with `preset` fast, normal, quality or final trades speed for accuracy.
4. Look at it: `map_show` streams strain, stress, pressure, fit or thickness maps; `map_stats` returns numbers without changing the display.
5. Every change from the window or an agent shares the one document, so tell the person before `sample_load` with `force` replaces it.

## Exporting and the Live Link

1. `export_formats` lists obj, gltf, glb, fbx, usd, mdd and pc2. `export_run` writes the draped garment to an absolute `path` whose extension matches `format`; `animation` true writes a recorded cache layer (`anim_record` first).
2. `link_start` with an absolute `folder` and `every_frames` writes `[folder]/garment.usda` at once and replaces it atomically every N simulation frames, after edits and when the simulation settles; point Unreal's USD Stage at that file (`docs/integration/unreal-live-link.md`, untested against Unreal). `link_state` shows writes and any error; `link_stop` ends it, and so does any `file_new` or `file_open`.
3. To hand a garment to another Form app, the Python helper `formfabric.gof.send` writes a GoF-compatible bundle (`docs/integration/gof-handoff.md`).

## Scripting from Python

- `./formfabric py script.py` runs Python with the `formfabric` package: `formfabric.commands` has one checked function per command, `Garment`, `Sim` and `Link` wrap the common steps, and an in-process or HTTP backend is chosen for you (`python/README.md`).
- A plugin is a `.py` file in `backend/plugins` with a `PARAMS` dict and `run(ffb, args)`; `drape_and_export` is the sample (drape a sample, export a GLB). Run it with `POST /api/plugins/run`.
- Outside agents reach the same commands through `.mcp.json` (stdio, `./formfabric mcp`) or `POST /api/mcp`, which asks the person in the window once before the first change.

## Tool reference

[ffb-tools:begin]
Generated from the engine's command registry (formfabric 0.15.0) by `./formfabric agents sync`; do not edit between the markers by hand.

- Engine: `system_info`, `system_capabilities`
- Files: `file_new`, `file_save`, `file_open`, `sample_list`, `sample_load`, `exchange_import_dxf`, `exchange_export_dxf`, `snapshot_save`, `snapshot_list`, `snapshot_restore`, `snapshot_remove`, `library_list`, `library_add`, `library_insert`, `library_remove`, `library_roots`, `library_set_roots`, `component_save`
- History: `history_undo`, `history_redo`, `history_list`
- Viewing: `view_units`, `mesh_get`, `mesh_stats`, `map_show`, `map_stats`, `compare_snapshot`, `compare_set`, `compare_clear`, `compare_get`, `measure3d_add`, `measure3d_remove`, `measure3d_list`, `viz_get`, `stage_set`, `light_add`, `light_set`, `light_remove`, `view_save`, `view_remove`, `fur_add`, `fur_set`, `fur_remove`
- Pattern: `pattern_get`, `pattern_select`, `pattern_measure`, `pattern_snap`, `pattern_draw_panel`, `pattern_rectangle`, `pattern_ellipse`, `pattern_inner_shape`, `pattern_inner_line`, `pattern_move_point`, `pattern_set_handle`, `pattern_add_point`, `pattern_split_line`, `pattern_delete_point`, `pattern_set_point_kind`, `pattern_curvature`, `pattern_smooth`, `pattern_transform`, `pattern_duplicate`, `pattern_delete`, `pattern_rename`, `pattern_set_props`, `pattern_begin_drag`, `pattern_end_drag`, `pattern_seam_allowance`, `pattern_offset_outline`, `pattern_notch_add`, `pattern_notch_set`, `pattern_notch_delete`, `pattern_grain_line`, `pattern_set_inner`, `pattern_segment_lengths`, `pattern_match_length`, `pattern_dart`, `pattern_slash_spread`, `pattern_unfold`, `pattern_spiral`, `pattern_trace`, `pattern_mirror_link`, `pattern_instance`, `pattern_unlink`, `panel_state`, `styleline_add`, `styleline_set`, `styleline_remove`, `styleline_list`, `styleline_list3d`, `style_create`, `style_update`, `style_delete`, `style_list`, `print_add`, `print_set`, `print_delete`, `component_create`, `component_list`, `component_insert`, `component_swap`, `component_join`, `component_check`, `component_remove`, `component_sample`
- Simulation: `mesh_build`, `arrange_place`, `arrange_reset`, `sim_start`, `sim_stop`, `sim_toggle`, `sim_step`, `sim_reset`, `sim_settings`, `sim_state`, `sim_positions`, `sim_compare`, `pin_add`, `pin_remove`, `pin_clear`, `sim_grab`, `sim_apply`, `sim_pinch`, `refit`, `place_panel`, `place_reset`, `wind_set`, `pin_box`, `pin_lasso`, `fold_place`, `body_tape`, `fitshell_set`, `fitshell_get`
- Fabrics: `fabric_presets`, `fabric_list`, `fabric_create`, `fabric_update`, `fabric_duplicate`, `fabric_delete`, `fabric_assign`, `fabric_set_default`, `fabric_lab`, `fabric_export`, `fabric_import`, `colorway_create`, `colorway_update`, `colorway_delete`, `colorway_activate`
- Sewing: `sew_segment`, `sew_free`, `sew_multi`, `sew_edit`, `sew_flip`, `sew_delete`, `sew_list`, `sew_check`, `sew_bindings`, `tack_add`, `tack_delete`, `sew_pleat`, `sew_gather`, `sew_auto`, `button_add`, `buttonhole_add`, `button_fasten`, `hardware_move`, `hardware_delete`, `zipper_add`, `zipper_set`, `zipper_delete`, `stitch_add`, `stitch_set`, `stitch_delete`, `fray_add`, `finish_add`, `finish_set`, `finish_delete`, `sew_rip`, `sew_mend`, `quilt_make`, `trim_import`, `trim_debug`
- Bodies: `body_load_default`, `body_import`, `body_remove`, `body_show`, `body_set`, `body_resize`, `body_pose`, `body_list`, `body_measure`, `body_joints`, `place_volume_set`, `place_point_set`, `place_points`, `place_defaults`
- Animation: `anim_get`, `anim_set`, `motion_import`, `motion_set`, `motion_remove`, `key_set`, `key_remove`, `cache_set`, `cache_remove`, `morph_set`, `anim_record`, `anim_stop`, `job_state`, `job_harvest`, `cache_frame`, `anim_play`, `anim_pause`
- Export: `export_formats`, `export_run`, `uv_get`, `uv_layout`, `uv_island_set`, `thickness_set`, `bake_run`, `gameready_run`, `quad_remesh`, `quad_clear`, `retopo_get`, `retopo_set`, `schematic_export`, `capture_write`, `link_start`, `link_stop`, `link_state`

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

### sample_list

List the sample garments: a basic T-shirt, an A-line skirt with waistband and a sleeveless dress. Each is drafted from the body measurements.

- no parameters

Note: read-only.

### sample_load

Replace the garment with a sample drafted on the given or current body: the pattern panels are drawn from the body measurements, sewn, and placed around the body, ready for sim.reset and sim.run. Keeps the body and discards the previous garment and the whole undo history (the result says history_cleared). Fails with the code unsaved_changes when the open project has edits, unless force is true.

- `body`: ['integer', 'string']; Body id in the project, or the name of a library body to load (MPFB2 Female or MPFB2 Male). Defaults to the first body, or MPFB2 Female when the project has none.
- `force`: boolean; default false; Replace a project that has unsaved edits. Without it the call fails with the code unsaved_changes so the caller can ask the user to confirm.
- `name`: string (tshirt | skirt | dress); required; Sample garment, as listed by sample.list.

Note: removes or replaces (undoable).

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
- `particle_distance`: number; default 20.0; range 0.8 to 700.0; Simulation particle spacing for this panel.
- `points`: array of value; required; Outline vertices in world mm, at least 3, in drawing order. A point is [x, y] or an object with p, kind, h_in and h_out. The outline is stored counter-clockwise.

### pattern_rectangle

Create a rectangular panel from its lower left corner, width and height in mm, with an optional corner radius. Returns the new panel id.

- `corner_radius`: number; default 0.0; range 0.0 to 20000.0; Round all four corners by this tangent length; 0 keeps them sharp. Limited to half of the shorter side.
- `height`: number; required; range 0.1 to 20000.0; Height along y.
- `name`: string; Panel name. Defaults to Panel and the new id.
- `particle_distance`: number; default 20.0; range 0.8 to 700.0; Simulation particle spacing for this panel.
- `width`: number; required; range 0.1 to 20000.0; Width along x.
- `x`: number; default 0.0; range -100000.0 to 100000.0; Left edge.
- `y`: number; default 0.0; range -100000.0 to 100000.0; Bottom edge.

### pattern_ellipse

Create an elliptical panel from its centre and radii in mm as 4 or more smooth curve points with exact Bezier handles. Equal radii make a circle. Returns the new panel id.

- `cx`: number; default 0.0; range -100000.0 to 100000.0; Centre x.
- `cy`: number; default 0.0; range -100000.0 to 100000.0; Centre y.
- `name`: string; Panel name. Defaults to Panel and the new id.
- `particle_distance`: number; default 20.0; range 0.8 to 700.0; Simulation particle spacing for this panel.
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
- `particle_distance`: number; range 0.8 to 700.0; Simulation particle spacing.
- `position`: array of number; Absolute panel position on the pattern plane.
- `pressure`: number; range -1000000.0 to 1000000.0; Inflation pressure of the panel in the solver's units, along the panel normal; 0 is none.
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
- `particle_distance`: number; default 20.0; range 0.8 to 700.0; Simulation particle spacing for this panel.
- `start_angle`: number; default 0.0; range -360.0 to 360.0; Angle of the start of the strip.
- `turns`: number; default 3.0; range 0.25 to 20.0; Number of turns from start to end.
- `width`: number; required; range 0.1 to 20000.0; Width of the strip.

### pattern_trace

Create a new panel from a closed inner shape (by inner id, or the smallest one containing the point at) or from a closed region given as vertices. The new panel keeps the traced geometry at the same place on the pattern plane. remove true also deletes the inner shape from the source panel.

- `at`: array of number; World point: traces the smallest closed inner shape of the panel that contains it.
- `inner`: integer; range 1.0 to 4294967295.0; Closed inner shape id to trace into a new panel.
- `name`: string; Panel name. Defaults to Traced and the new id.
- `panel`: integer; range 1.0 to 4294967295.0; Panel id as listed by pattern.get.
- `particle_distance`: number; range 0.8 to 700.0; Simulation particle spacing for the new panel. Defaults to the source panel's.
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

### sew_segment

Sew one whole segment to another. reversed (default true) pairs the start of side a with the end of side b, which is right for counter-clockwise outlines meeting along a shared edge. With ratio 1 and full segments both segments get the same number of mesh vertices. Returns the seam with both side lengths and the mismatch in mm.

- `fold`: string (turned | flat | custom); default "turned"; turned: folded back 180 degrees, two plies, angle and strength locked. flat: butt joint, panels stay in their plane. custom: use fold_angle and fold_strength.
- `fold_angle`: number; range 0.0 to 360.0; Fold angle for custom seams: 180 flat, lower folds to the front, higher to the back.
- `fold_strength`: number; range 0.0 to 20.0; How firmly a custom seam holds its fold angle, 0 for none.
- `inner_a`: integer; range 1.0 to 4294967295.0; Inner line of side a. Leave out for the outline.
- `inner_b`: integer; range 1.0 to 4294967295.0; Inner line of side b. Leave out for the outline.
- `panel_a`: integer; required; range 1.0 to 4294967295.0; Panel of side a.
- `panel_b`: integer; required; range 1.0 to 4294967295.0; Panel of side b.
- `ratio`: number; default 1.0; range 0.05 to 20.0; Intended arc length of side b over side a. Above 1 gathers b onto a.
- `reversed`: boolean; default true; Pair the start of side a with the end of side b. Outlines drawn counter-clockwise that meet along a shared edge run in opposite directions, so the usual seam is reversed.
- `segment_a`: integer; required; range 0.0 to 100000.0; Segment of side a; segment i runs from point i to point i+1.
- `segment_b`: integer; required; range 0.0 to 100000.0; Segment of side b.
- `side`: string (auto | front | back); Which face of the fabric of side a the other side attaches to. auto picks it from the 3D placement and the panel normal; sew.list reports the result as side_resolved.
- `tension`: number; default 1.0; range 0.0 to 20.0; Stiffness of the seam pull relative to the cloth; 1 is normal, above 1 pulls harder, 0 leaves the seam slack.

### sew_free

Sew a span of one segment (t0 to t1 as curve parameters) to a span of another, so a seam can start and end anywhere on an outline or inner line. Sides are matched by arc length fraction.

- `fold`: string (turned | flat | custom); default "turned"; turned: folded back 180 degrees, two plies, angle and strength locked. flat: butt joint, panels stay in their plane. custom: use fold_angle and fold_strength.
- `fold_angle`: number; range 0.0 to 360.0; Fold angle for custom seams: 180 flat, lower folds to the front, higher to the back.
- `fold_strength`: number; range 0.0 to 20.0; How firmly a custom seam holds its fold angle, 0 for none.
- `inner_a`: integer; range 1.0 to 4294967295.0; Inner line of side a. Leave out for the outline.
- `inner_b`: integer; range 1.0 to 4294967295.0; Inner line of side b. Leave out for the outline.
- `panel_a`: integer; required; range 1.0 to 4294967295.0; Panel of side a.
- `panel_b`: integer; required; range 1.0 to 4294967295.0; Panel of side b.
- `ratio`: number; default 1.0; range 0.05 to 20.0; Intended arc length of side b over side a. Above 1 gathers b onto a.
- `reversed`: boolean; default true; Pair the start of side a with the end of side b. Outlines drawn counter-clockwise that meet along a shared edge run in opposite directions, so the usual seam is reversed.
- `segment_a`: integer; required; range 0.0 to 100000.0; Segment of side a; segment i runs from point i to point i+1.
- `segment_b`: integer; required; range 0.0 to 100000.0; Segment of side b.
- `side`: string (auto | front | back); Which face of the fabric of side a the other side attaches to. auto picks it from the 3D placement and the panel normal; sew.list reports the result as side_resolved.
- `t0_a`: number; required; range 0.0 to 1.0; Start of the span on side a as a curve parameter on its segment.
- `t0_b`: number; required; range 0.0 to 1.0; Start of the span on side b.
- `t1_a`: number; required; range 0.0 to 1.0; End of the span on side a.
- `t1_b`: number; required; range 0.0 to 1.0; End of the span on side b.
- `tension`: number; default 1.0; range 0.0 to 20.0; Stiffness of the seam pull relative to the cloth; 1 is normal, above 1 pulls harder, 0 leaves the seam slack.

### sew_multi

Sew a list of spans (M) to another list of spans (N) as one seam, for example a sleeve cap to an armhole made of front and back pieces. Spans are listed in walking order and matched by total arc length fraction.

- `a`: array of object; required; Spans that make side a, in walking order.
- `b`: array of object; required; Spans that make side b, in walking order.
- `fold`: string (turned | flat | custom); default "turned"; turned: folded back 180 degrees, two plies, angle and strength locked. flat: butt joint, panels stay in their plane. custom: use fold_angle and fold_strength.
- `fold_angle`: number; range 0.0 to 360.0; Fold angle for custom seams: 180 flat, lower folds to the front, higher to the back.
- `fold_strength`: number; range 0.0 to 20.0; How firmly a custom seam holds its fold angle, 0 for none.
- `ratio`: number; default 1.0; range 0.05 to 20.0; Intended arc length of side b over side a. Above 1 gathers b onto a.
- `reversed`: boolean; default true; Pair the start of side a with the end of side b. Outlines drawn counter-clockwise that meet along a shared edge run in opposite directions, so the usual seam is reversed.
- `side`: string (auto | front | back); Which face of the fabric of side a the other side attaches to. auto picks it from the 3D placement and the panel normal; sew.list reports the result as side_resolved.
- `tension`: number; default 1.0; range 0.0 to 20.0; Stiffness of the seam pull relative to the cloth; 1 is normal, above 1 pulls harder, 0 leaves the seam slack.

### sew_edit

Change any field of a seam: its sides (spans), reversed, fold (turned, flat, custom), fold angle and strength, ratio, tension and side. Only the given fields change. A turned seam locks the fold angle to 180 and the strength to 0.

- `a`: array of object; Replace side a with these spans.
- `b`: array of object; Replace side b with these spans.
- `fold`: string (turned | flat | custom); turned: folded back 180 degrees, two plies, angle and strength locked. flat: butt joint, panels stay in their plane. custom: use fold_angle and fold_strength.
- `fold_angle`: number; range 0.0 to 360.0; Fold angle for custom seams: 180 flat, lower folds to the front, higher to the back.
- `fold_strength`: number; range 0.0 to 20.0; How firmly a custom seam holds its fold angle, 0 for none.
- `ratio`: number; range 0.05 to 20.0; Intended arc length of side b over side a. Above 1 gathers b onto a.
- `reversed`: boolean; Pair the start of side a with the end of side b. Outlines drawn counter-clockwise that meet along a shared edge run in opposite directions, so the usual seam is reversed.
- `seam`: integer; required; range 1.0 to 4294967295.0; Seam id as listed by sew.list.
- `side`: string (auto | front | back); Which face of the fabric of side a the other side attaches to. auto picks it from the 3D placement and the panel normal; sew.list reports the result as side_resolved.
- `tension`: number; range 0.0 to 20.0; Stiffness of the seam pull relative to the cloth; 1 is normal, above 1 pulls harder, 0 leaves the seam slack.

Note: safe to repeat; in the window: B.

### sew_flip

Reverse the direction of side b of one or several seams, so the start of side a is paired with the other end of side b.

- `seam`: integer; range 1.0 to 4294967295.0; Seam id as listed by sew.list.
- `seams`: array of integer; Several seam ids.

### sew_delete

Remove one seam, several, or all of them. Forced mesh vertex counts that existed for the seam are released.

- `all`: boolean; default false; Remove every seam.
- `seam`: integer; range 1.0 to 4294967295.0; Seam id as listed by sew.list.
- `seams`: array of integer; Several seam ids.

Note: removes or replaces (undoable).

### sew_list

Every seam with the length of each side in mm and the mismatch (side b minus ratio times side a), plus the tacks.

- `seam`: integer; range 1.0 to 4294967295.0; Only this seam.

Note: read-only.

### sew_check

List the seams whose side lengths differ by more than a tolerance (default 1 mm), measured as side b minus ratio times side a.

- `tolerance`: number; range 0.0 to 100000.0; Largest allowed difference between side b and ratio times side a.

Note: read-only.

### sew_bindings

Count the solver bindings the seams produce on the current meshes: vertex pairs and vertex-to-edge bindings, per seam.

- `seam`: integer; range 1.0 to 4294967295.0; Only this seam.

Note: read-only.

### tack_add

Add a temporary point-to-point constraint between two panel points, or between a panel point and a body point. Points are in world or local space and are stored panel-local.

- `body_point_a`: array of number; End a on a body instead of a panel, in body space.
- `body_point_b`: array of number; End b on a body instead of a panel, in body space.
- `length`: number; range 0.0 to 100000.0; Rest distance the two ends are held at; 0 holds them together.
- `panel_a`: integer; range 1.0 to 4294967295.0; Panel of end a.
- `panel_b`: integer; range 1.0 to 4294967295.0; Panel of end b.
- `point_a`: array of number; Point of end a on its panel.
- `point_b`: array of number; Point of end b on its panel.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.

### tack_delete

Remove one tack by id, several by tacks, or every tack with all.

- `all`: boolean; default false; Remove every tack.
- `tack`: integer; range 1.0 to 4294967295.0; Tack id as listed by sew.list.
- `tacks`: array of integer; Several tack ids.

Note: removes or replaces (undoable).

### sew_pleat

Create knife or box pleats on a panel: parallel fold lines from a start line (from and to, or an existing inner line) added to one side, with fold angles 0 (front) or 360 (back, reverse) on the creases, and seams that close each pleat. A knife pleat has three lines with the middle one creased and the outer two sewn together; a box pleat has five lines, creases at the second and fourth, both outer lines sewn to the middle one. Returns the new inner line ids per pleat and the seam ids.

- `count`: integer; range 1.0 to 64.0; Number of pleats.
- `depth`: number; range 1.0 to 1000.0; Depth of one fold, the distance between neighbouring lines of a pleat.
- `from`: array of number; Start of the first pleat line. Give from and to, or inner.
- `inner`: integer; range 1.0 to 4294967295.0; Existing inner line to take the direction and length of the first pleat line from.
- `interval`: number; range 1.0 to 100000.0; Distance from the first line of one pleat to the first line of the next. Default is the pleat width plus twice the depth.
- `kind`: string (knife | box); default "knife"; knife: one fold, three lines per pleat, the outer two sewn together. box: two knife folds facing apart, five lines per pleat.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel that receives the pleat fold lines.
- `reverse`: boolean; default false; Fold to the back (360 degrees) instead of the front (0 degrees).
- `sew`: boolean; default true; Create the seams that close the pleats.
- `side`: string (left | right); default "left"; Side of the first pleat line, walking from to, where the next lines are added.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.
- `strength`: number; range 0.0 to 20.0; Fold strength of the crease lines.
- `to`: array of number; End of the first pleat line.

### sew_gather

Gather side b onto side a at a ratio of 1 or more: side a is shortened from its end so that side b is the given multiple of it, and the seam ratio is stored. The cloth of side b bunches up to fit when simulated.

- `ratio`: number; required; range 1.0 to 20.0; Arc length of side b over side a, 1 or more. Side a is shortened from its end until it fits; side b keeps its spans.
- `seam`: integer; required; range 1.0 to 4294967295.0; Seam id as listed by sew.list.

Note: safe to repeat.

### sim_start

Start the background simulation at the frame rate. Positions stream to the viewer. Native builds only; use sim.step elsewhere.

- no parameters

Note: safe to repeat.

### sim_stop

Stop the background simulation. The draped state stays where it is.

- no parameters

Note: safe to repeat.

### sim_toggle

Start the simulation if it is stopped, stop it if it runs.

- no parameters

Note: in the window: Space.

### sim_step

Advance the simulation by a number of frames on the calling thread and publish the result. Works without a background thread.

- `frames`: integer; default 1; range 1 to 2000; How many frames to step. Each frame runs the configured substeps.

### sim_reset

Rebuild the cloth from the garment and return every panel to its placed state, frame zero.

- no parameters

### sim_settings

Get or set the solver parameters and the quality preset (fast, normal, quality, final). Settings are not saved in the document.

- `air_damping`: number; range 0 to 1
- `cg_iterations`: integer; range 1 to 2000; Conjugate gradient iterations per Newton iteration (implicit method).
- `cg_tolerance`: number; range 1e-06 to 0.1; Conjugate gradient relative tolerance (implicit method).
- `close_frames`: integer; range 0 to 600; Seam closing phase at the start of a drape: up to this many frames with stiffened seams, reduced gravity, strong damping and collisions on, then the normal simulation. 0 turns it off. Only applies when the garment has seams or links.
- `close_gap`: number; range 0.1 to 50; The closing phase may end early once the largest seam gap is under this.
- `collision_thickness`: number; range 0 to 100
- `device`: string (cpu | gpu); Where the solver runs. gpu needs an engine built with the gpu feature and an adapter; otherwise it is reported unavailable and the simulation stays on the cpu. The gpu path supports the vbd method only.
- `friction`: number; range 0 to 5
- `gravity`: array of number
- `iterations`: integer; range 1 to 200; Solver iterations per substep.
- `method`: string (auto | vbd | implicit); Solver method: vbd is vertex block descent, implicit is implicit Euler with conjugate gradient, auto (the default) uses vbd when the garment has seams or links and implicit otherwise. Naming vbd or implicit pins the choice.
- `newton`: integer; range 1 to 10; Newton iterations per substep (implicit method).
- `preset`: string (fast | normal | quality | final); Implicit method: fast 2 substeps, normal 3, quality 4 with self contact, final 6 with 3 Newton iterations and self contact. Block descent: fast 3x6, normal 3x8, quality 8x15 and final 12x25 (substeps x iterations), the last two with self contact. Leave out every field to read the settings.
- `seam_boost`: number; range 1 to 100; Peak seam stiffness multiplier during the closing phase (1 keeps the seams at their normal stiffness).
- `self_collision`: boolean
- `skin_offset`: number; range 0 to 100
- `substeps`: integer; range 1 to 40
- `time_step`: number; range 0.004 to 0.0667; Seconds per frame, from 1/240 to 1/15. The run thread steps at 1 / time_step frames per second.

Note: safe to repeat.

### sim_state

Read whether the simulation runs, the frame, the vertex count, the last step statistics and the preset.

- no parameters

Note: read-only.

### sim_positions

Read the current simulated 3D positions of one panel's vertices. decimate keeps every n-th vertex; the answer never holds more than 5000 vertices, so large panels are decimated further. Read only.

- `decimate`: integer; default 1; range 1 to 1000; Keep every n-th vertex of the panel mesh.
- `panel`: integer; required; range 1 to inf; Panel id as listed by pattern.get.

Note: read-only.

### sim_compare

Run the same start for n frames with the interactive solver and with the reference solver and report the mean and maximum vertex distance in mm. Does not touch the running simulation.

- `frames`: integer; default 30; range 1 to 300; Frames to run with the block descent solver and with the reference solver from the same start.

Note: read-only.

### pin_add

Pin a mesh vertex of a panel in 3D. Pins are part of the document and an undo step; they hold the vertex where it is (or at target) during simulation.

- `panel`: integer; required; range 1 to inf; Panel id as listed by pattern.get.
- `point`: array of number; Panel-local point; the nearest mesh vertex is pinned. Give point or vertex.
- `target`: ['array', 'null']; Where the pinned vertex is held in 3D. Leave out to hold it where it is now (the simulated position when a simulation exists, else the placed position).
- `vertex`: integer; range 0 to inf; Vertex index in the panel mesh as streamed for that panel.

### pin_remove

Remove one pin by id.

- `id`: integer; required; range 1 to inf; Pin id as returned by pin.add.

### pin_clear

Remove every pin, or those of one panel.

- `panel`: integer; range 1 to inf; Only clear the pins of this panel. Leave out to clear every pin.

### sim_grab

Hold a vertex at a 3D target while the simulation runs, as the mouse drag does. A null or missing target releases the grab. Not saved and not an undo step.

- `panel`: integer; range 1 to inf; Panel id. With vertex, the vertex index is in that panel's mesh; with point, the nearest vertex is taken.
- `point`: array of number; Panel-local point (needs panel).
- `target`: ['array', 'null']; Where to hold the grabbed vertex in 3D. Null or missing releases the grab. Send again to drag.
- `vertex`: integer; range 0 to inf; Vertex index in the panel mesh when panel is given, else an index into the whole cloth.

Note: safe to repeat.

### sim_apply

Write the draped state into the document as one undo step: each panel gets the rigid placement (place3d) that best fits its draped vertices, and the draped vertex positions are saved in place3d.pose (kept by save and open). The viewer and the next simulation or reset start from the pose while the mesh vertex count matches. Undo restores the earlier placement.

- no parameters

### sim_pinch

Pull a picked vertex to a 3D target while the simulation runs, with the vertices around it following by a falloff weight (bump or linear, radius 1 to 500 mm, power -2 to 2, straight or surface distance). Send again with a new target to drag; a null target releases. Not saved and not an undo step.

- `distance`: string (straight | surface); default "straight"; straight: every panel within the radius. surface: along the picked panel's mesh only.
- `falloff`: string (bump | linear); default "bump"
- `frames`: integer; default 0; range 0 to 120; Frames to step after setting the pinch (use with a stopped simulation).
- `panel`: integer; range 1 to inf; Panel id. With vertex, the index is in that panel's mesh; with point, the nearest vertex is taken.
- `point`: array of number; Panel-local point (needs panel).
- `power`: number; default 0; range -2 to 2; Falloff power: above 0 narrows the weights, below 0 widens them.
- `radius`: number; default 50; range 1 to 500; Falloff distance.
- `target`: ['array', 'null']; Where the picked vertex is pulled to in 3D. Neighbours follow by the falloff weight. Null or missing releases the pinch.
- `vertex`: integer; range 0 to inf; Vertex index in the panel mesh when panel is given, else an index into the whole cloth.

Note: in the window: Shift+C.

### refit

Move the draped garment from body A to body B without opening a seam. The body is morphed from A to B in steps while the solver runs, so the garment follows only by simulation and contact: no reference garment, no correspondence map between the bodies, no stored fit data, no pattern change. Bodies of identical topology blend per vertex; other bodies are stepped through a measurement sweep of body A and then the garment settles on B, or use method redrape to drape from the placed layout on B. Ends by hiding A, showing B and saving the draped shape (one undo step).

- `fix_frames`: integer; default 150; range 0 to 1000; Frame budget after the settle: each frame pushes cloth out of the target body and steps with collisions on, until the deepest vertex is under 1 mm inside or the budget is spent. The result reports inside_mm and the frames used.
- `frames`: integer; default 4; range 1 to 30; Solver frames run at each increment.
- `from`: integer; range 1 to inf; Body id the garment is draped on. Defaults to the first visible body.
- `method`: string (auto | blend | sweep | redrape); default "auto"; blend: bodies of identical topology, per-vertex linear blend from A to B. sweep: other bodies, body A is stepped through a measurement sweep to B's measurements, then the garment settles on B. redrape: no morph, the garment returns to its placed layout and drapes on B. auto picks blend when the topology matches, else sweep.
- `settle`: integer; default 40; range 0 to 600; Frames run on the target body at the end.
- `steps`: integer; default 20; range 1 to 400; Increments along the body morph path.
- `to`: integer; required; range 1 to inf; Body id to move the garment to.

### body_load_default

Add a default body to the project: the built-in Basic Form (formfabric's own parametric body, about 20k triangles, 20 joints, skin weights) or a library body from assets/bodies by name. One undo step.

- `name`: string; Body name; default Basic Form. body.list shows the library.

### body_import

Import a body from an OBJ, glTF, GLB or FBX file (FBX on native builds only), convert to millimetres with y up and stand it on the ground. Skins and skeletons are read from glTF and FBX; Alembic is deferred. One undo step.

- `ground`: boolean; Stand the body on y = 0; default true.
- `name`: string; Name for the body; default the file name.
- `path`: string; required; File path of the body.
- `units`: string (auto | mm | cm | m | in | ft); Units of the file; auto reads glTF as metres and guesses OBJ from its height.
- `up`: string (y | -y | z | -z | x | -x); Up axis of the file; default y.

### body_remove

Remove a body from the project. One undo step.

- `id`: integer; Body id. Defaults to the first body.

Note: removes or replaces (undoable).

### body_show

Show or hide a body (all bodies when no id is given). A hidden body is removed from the 3D view but stays in the project.

- `id`: integer
- `visible`: boolean; Default true.

Note: safe to repeat.

### body_set

Set a body's collision skin offset in millimetres (0 to 100, default 3), friction (0 to 10) or name.

- `friction`: number
- `id`: integer
- `name`: string
- `skin_offset`: number

Note: safe to repeat.

### body_resize

Deform a body to target measurements in millimetres: height and inseam (vertical scale), chest (or bust), waist and hip (cross-sections scaled around the vertical axis with smooth falloff), shoulder width and arm length. The skeleton moves with its region. Targets are met within 1 mm. One undo step.

- `arm_length`: number
- `bust`: number; Same as chest.
- `chest`: number
- `height`: number
- `hip`: number
- `id`: integer
- `inseam`: number
- `shoulder_width`: number
- `waist`: number

### body_pose

Pose a body with linear blend skinning: a named pose (rest, A, T) or joint rotations as quaternions [x, y, z, w], {euler: degrees} or {axis, angle}, applied on top of the rest pose. Joints given alone merge into the current pose; replace clears it first. One undo step.

- `id`: integer
- `joints`: object; Joint name to rotation.
- `name`: string; rest, A or T.
- `replace`: boolean

Note: safe to repeat.

### body_list

List the bodies in the project and the body library (formfabric's Basic Form plus the files under assets/bodies), and the named poses.

- no parameters

Note: read-only.

### body_measure

Standard measurements in millimetres computed from the posed mesh: height, chest (also bust), waist, hip (widest girth below the waist), inseam, shoulder width and arm length. Girths are convex hull perimeters of horizontal slices.

- `id`: integer; Body id. Defaults to the first body.

Note: read-only.

### body_joints

The skeleton of a body: each joint with parent, local rest offset, rest and posed world positions and its pose rotation as a quaternion [x, y, z, w].

- `id`: integer; Body id. Defaults to the first body.

Note: read-only.

### place_volume_set

Create, edit or remove a Placement Volume of a body: a cylinder stretched between two joints with a sideways and a front to back radius. Volumes are authored data: a body without any gets the default set from its joints the first time you edit one. Radius, height, scale and kind follow to the mirrored side (_L and _R names) unless mirror is false. One undo step.

- `axis`: array of number; Rest axis direction in body space; it is normalised.
- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first body.
- `center`: array of number; Rest centre in body space.
- `height`: number; range 1 to 5000; Length along the axis.
- `joints`: array of ['integer', 'string']; The two joints the volume follows, by name or index. It moves with them when the body is posed.
- `kind`: string (cylinder | box | plane); Shape of the cross section. Placement wraps cylinders.
- `mirror`: boolean; default true; Apply radius, height, scale and kind to the opposite side volume as well.
- `name`: string; Volume name. A name ending _L or _R pairs it with the other side.
- `radius`: array of number; Radius sideways (left right) and radius front to back.
- `remove`: boolean; default false; Delete the volume and the points on it.
- `scale`: number; range 0.1 to 10; Multiply the radius by this.
- `volume`: ['integer', 'string']; Volume id or name. Leave out to create a new volume (name and radius required).

### place_point_set

Create, edit or remove a Placement Point: a named spot on a volume given by an angle around the axis, a fraction along it, an offset from the surface and whether panels wrap. Points are stored in volume coordinates and follow the volume when the body is posed. Mirrored partners change together unless mirror is false. One undo step.

- `along`: number; range -1 to 2; Position along the axis from the first joint end (0) to the second (1).
- `around`: number; range -720 to 720; Angle around the volume axis: 0 front, 90 left, 180 back, 270 right.
- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first body.
- `mirror`: boolean; default true; Apply the change to the mirrored partner point as well.
- `name`: string; Point name.
- `offset`: number; range -200 to 1000; Gap between the volume surface and a panel placed here.
- `point`: ['integer', 'string']; Point id or name. Leave out to create a new point (name and volume required).
- `remove`: boolean; default false; Delete the point.
- `volume`: ['integer', 'string']; Volume the point sits on, by id or name.
- `wrap`: boolean; Whether a panel placed here bends around the volume.

### place_points

List the Placement Volumes and Points of a body with their world position, outward normal and up direction for the current pose. A body without stored volumes shows the default set that would be created, with stored false.

- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first body.

Note: read-only.

### place_defaults

Replace the volumes and points of a body with the default set built from its joints: torso, neck, upper and lower arms, thighs and shins, with points front, back, left, right, skirt, neck, sleeves, cuffs and trouser legs in mirrored pairs. The volumes are sized by a fixed profile from the joint distances, not from the mesh. One undo step.

- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first body.

### place_panel

Put a panel on a Placement Point: the panel centre goes to the point plus the offset, the panel front faces away from the volume (flip faces it inward), its top points up the body, and the panel is bent about the volume axis with a radius that clears the volume so it wraps instead of cutting through. Sets place3d (origin, rotation, bend); one undo step. Run sim.reset afterwards so the drape starts from this shape.

- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first body.
- `flip`: boolean; default false; Face the panel front toward the body instead of away from it.
- `offset`: number; range -200 to 1000; Gap between the volume surface and the panel. Defaults to the point's own offset.
- `orientation`: number; default 0; range -360 to 360; Turn the panel about its normal. 0 puts the panel top toward the head or shoulder; 180 turns it upside down and still wraps. Other angles place the panel flat on the tangent plane.
- `panel`: integer; required; range 1 to inf; Panel id as listed by pattern.get.
- `point`: ['integer', 'string']; required; Placement point id or name, as listed by place.points.
- `wrap`: boolean; Bend the panel around the volume. Defaults to the point's wrap setting.

Note: safe to repeat.

### place_reset

Clear the 3D placement of the given panels, or of all panels, so they return to the flat layout, and with volumes true rebuild the body's default volumes and points from its joints. One undo step.

- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first body.
- `panels`: array of integer; Panel ids to return to the flat layout. Leave out for every panel.
- `volumes`: boolean; default false; Also rebuild the default volumes and points of the body from its joints.

### sew_auto

Propose seams for the garment from the notches on the panels' own outline segments. Every open outline segment with notches is a candidate; segments pair only when their notch counts are equal, and all pairs are chosen together as one minimum-cost assignment where the cost is the relative length difference plus the mean difference of the notch positions (segments given the same tag in this call pair at zero cost). When both panels are placed in 3D, a small term for the distance between the segment midpoints settles ties between otherwise equal candidates; panels placed by hand or not placed work the same. Panels are never resized: a length mismatch is reported and left to the seam ratio. Works on panels placed by hand or not placed at all. Returns the proposals and the open segments; with apply the seams are created as one undo step and can be edited like any other seam.

- `apply`: boolean; default false; Create the proposed seams as one undo step. Without it the command only returns proposals.
- `max_cost`: number; default 0.35; range 0.01 to 5; Largest cost of a proposed pair: relative length difference plus mean notch position difference.
- `min_length`: number; default 10; range 0 to 10000; Ignore outline segments shorter than this.
- `panels`: array of integer; Only consider the outlines of these panels. Leave out for all panels.
- `tags`: array of object; Join tags for this call: segments given the same tag are proposed as a seam, whatever their notches. Nothing is stored.

### panel_state

Set the simulation state of panels: freeze, disable, stiffen (strengthen), hold shape (solidify) and shrinkage. One undo step. The simulation rebuilds with the new state and keeps the other panels where they are.

- `disabled`: boolean; Disable: the panel is left out of the simulation together with its seams.
- `frozen`: boolean; Freeze: the panel is held as pins at its current positions and still collides.
- `hold_shape`: boolean; Hold Shape: a weak spring pulls each vertex back to the shape it has when the simulation is built.
- `panels`: array of integer; required; Panel ids as listed by pattern.get.
- `shrink_warp`: number; range 0.3 to 2; Warp shrinkage factor. 1 is none.
- `shrink_weft`: number; range 0.3 to 2; Weft shrinkage factor. 1 is none.
- `stiffen`: number; range 0.01 to 1000; Multiplier on stretch, shear and bending. 1 is none.

Note: safe to repeat.

### wind_set

Set the wind: on or off, kind (directional or radial), origin, direction, strength in m/s, decay with distance from the origin, turbulence. The simulation applies it as a force on every triangle (pressure on the face, so it also needs the face to be turned toward the wind). One undo step.

- `decay`: number; range 0 to 100; Weakening with distance from the origin. 0 is none; 100 drops the speed to 37 percent per metre.
- `direction`: array of number; Wind direction (directional).
- `enabled`: boolean
- `kind`: string (directional | radial); directional blows along direction; radial blows away from origin.
- `origin`: array of number; Where the wind starts. Decay is measured from here.
- `strength`: number; range 0 to 100; Wind speed at the origin.
- `turbulence`: number; range 0 to 100; Percent of irregularity in speed and direction (deterministic noise in space and time).

Note: safe to repeat.

### pin_box

Pin every vertex inside a 3D box, or a list of mesh vertices picked in the viewer, at its current position. One pin per vertex, all alike; one undo step.

- `max`: array of number; The opposite corner.
- `min`: array of number; One corner of the 3D box.
- `panel`: integer; range 1 to inf; Limit to this panel. Needed with vertices.
- `panels`: array of object; Mesh vertices picked in the viewer for several panels at once, recorded as one undo step. Replaces panel, vertices and the 3D region.
- `vertices`: array of integer; Mesh vertex indices picked by the viewer (of the given panel). Replaces the 3D region.

Note: in the window: Shift+X.

### pin_lasso

Pin every vertex whose projection along the view direction falls inside a lasso outline, or a list of mesh vertices picked in the viewer, at its current position. One pin per vertex, all alike; one undo step.

- `direction`: array of number; View direction the outline is extruded along.
- `panel`: integer; range 1 to inf; Limit to this panel. Needed with vertices.
- `panels`: array of object; Mesh vertices picked in the viewer for several panels at once, recorded as one undo step. Replaces panel, vertices and the 3D region.
- `polygon`: array of array; Lasso outline in 3D, drawn on the view plane.
- `vertices`: array of integer; Mesh vertex indices picked by the viewer (of the given panel). Replaces the 3D region.

Note: in the window: Shift+L.

### fold_place

Fold a panel before draping: the part of the panel on one side of a fold line is rotated about that line in the placed shape, by an angle. The folded shape is saved as the panel's pose and the next simulation starts from it. One undo step.

- `angle`: number; required; range -180 to 180; Fold angle. Positive turns the folded part toward the front of the panel; 180 lays it flat over the rest.
- `line`: array of array; required; Fold line as two panel-local points; it extends across the whole panel.
- `panel`: integer; required; range 1 to inf
- `side`: string (left | right); default "left"; Which part moves, looking along the line from its first to its second point in the panel plane.

Note: in the window: Shift+D.

### body_tape

Hold a panel point to a point of the body during simulation: a tack between the cloth vertex and a body point, one rest length, one stiffness. Without body_point the nearest body surface point is used. One undo step; remove it with tack.delete.

- `body`: integer; range 1 to inf; Body id for the nearest-point search. Defaults to the first visible body.
- `body_point`: array of number; World point to hold the cloth point to. Leave out to take the nearest point of the body surface.
- `length`: number; default 0; range 0 to 1000; Rest length of the tack.
- `panel`: integer; required; range 1 to inf
- `point`: array of number; Panel-local point; the nearest mesh vertex is held. Give point or vertex.
- `vertex`: integer; range 0 to inf; Vertex index in the panel mesh.

Note: in the window: Shift+K.

### styleline_add

Add a style line to a panel: a polyline in panel space (mm) that shows on the 3D garment. With mode stiffen or fold it also stiffens a band or creases the cloth along the line in the next simulation build. One undo step.

- `closed`: boolean; default false; Join the last point to the first.
- `fold_angle`: number; range 0 to 360; Rest dihedral angle of the crease when mode is fold. 180 is flat. Default 150.
- `fold_strength`: number; range 0 to 1; Crease stiffness 0 to 1 when mode is fold. Default 0.5.
- `mode`: string (none | stiffen | fold); What the line does in the simulation. none draws only; stiffen multiplies stretch, shear and bending in a band along the line; fold adds a crease along the mesh edges that follow the line.
- `panel`: integer; required; range 1 to inf; Panel id.
- `points`: array of array; required; Polyline in panel space as [x, y] pairs.
- `stiffen`: number; range 1 to 1000; Stiffness multiplier of the band when mode is stiffen. Default 8.
- `width`: number; range 0.5 to 500; Full width of the band or of the edge search for a fold. Default 12.

### styleline_set

Change a style line: its points, closed flag, panel, or what it does in the simulation (mode none, stiffen or fold with width, stiffness, crease angle and strength). One undo step.

- `closed`: boolean
- `fold_angle`: number; range 0 to 360; Rest dihedral angle of the crease when mode is fold. 180 is flat. Default 150.
- `fold_strength`: number; range 0 to 1; Crease stiffness 0 to 1 when mode is fold. Default 0.5.
- `id`: integer; required; range 1 to inf; Style line id as listed by styleline.list.
- `mode`: string (none | stiffen | fold); What the line does in the simulation. none draws only; stiffen multiplies stretch, shear and bending in a band along the line; fold adds a crease along the mesh edges that follow the line.
- `panel`: integer; range 1 to inf; Move the line to this panel.
- `points`: array of array; Replace the polyline (panel space).
- `stiffen`: number; range 1 to 1000; Stiffness multiplier of the band when mode is stiffen. Default 8.
- `width`: number; range 0.5 to 500; Full width of the band or of the edge search for a fold. Default 12.

Note: safe to repeat.

### styleline_remove

Delete a style line. One undo step.

- `id`: integer; required; range 1 to inf

### styleline_list

List the style lines with their panel, points in panel space, closed flag, length and simulation mode.

- `panel`: integer; range 1 to inf; Only the lines of this panel.

Note: read-only.

### styleline_list3d

Return each style line as a 3D polyline in world mm, mapped through the current draped panel mesh (simulation positions, applied pose or flat placement) by a barycentric lookup of every sample point in its panel mesh.

- `ids`: array of integer; Only these lines. Default all.
- `lift`: number; default 0.5; range 0 to 20; Raise the points along the surface normal so the line does not z-fight with the cloth.
- `step`: number; default 10; range 1 to 200; Longest panel-space distance between returned points; segments are subdivided so the line follows the surface.

Note: read-only.

### fitshell_set

Turn on a fit shell for a body: a smooth surface offset outward from the body along smoothed normals (offset in mm, reduced where the surface is concave so it cannot fold into itself) that replaces the body as the collider, so garments drape at a fitting ease. Takes effect when the simulation is next built. One undo step.

- `body`: integer; range 1 to inf; Body id as listed by body.list. Defaults to the first visible body.
- `enabled`: boolean; Use the shell as the collider in place of the body. Default true the first time.
- `offset`: number; range 0 to 100.0; Distance of the shell from the body surface, along smoothed normals. Default 10. This is the fitting ease: garments rest on the shell instead of on the skin.

Note: safe to repeat.

### fitshell_get

Report the fit shell of a body: offset, enabled, and the realised offset distances (minimum, mean, maximum and how many vertices were reduced to keep the shell free of self intersection). With mesh true, also the shell positions and triangles.

- `body`: integer; range 1 to inf; Body id. Defaults to the first visible body.
- `mesh`: boolean; default false; Also return the shell vertex positions and triangles (world mm).

Note: read-only.

### style_create

Create a named style for buttons, buttonholes, zippers, topstitch, puckering or imported trims. Parameters merge over the defaults of the kind and are range checked. Returns the style with its id.

- `kind`: string (button | buttonhole | zipper | topstitch | puckering | trim); required; button, buttonhole, zipper, topstitch, puckering or trim.
- `name`: string; Style name. Defaults to the kind and the new id.
- `params`: object; Style parameters merged over the defaults of the kind. Known keys are range checked: button width 0.5 to 100 mm, thickness 0.1 to 50 mm, weight 0 to 500 g; buttonhole length 1 to 100 mm, z_offset -5 to 5 mm; zipper width 1 to 100 mm; topstitch count 1 to 5, spi 1 to 40, thickness 0.05 to 5 mm, offset -50 to 50 mm, stiffen 1 to 5; puckering density 0 to 100 percent, width 0 to 50 mm; stiffness 0 to 20 on any kind. Unknown keys are kept as given.

### style_update

Rename a style or change parameters (merged; null removes a key). Everything that uses the style follows. The kind and an imported trim mesh cannot change.

- `name`: string; New name.
- `params`: object; Keys to set; null removes a key. Known keys are range checked: button width 0.5 to 100 mm, thickness 0.1 to 50 mm, weight 0 to 500 g; buttonhole length 1 to 100 mm, z_offset -5 to 5 mm; zipper width 1 to 100 mm; topstitch count 1 to 5, spi 1 to 40, thickness 0.05 to 5 mm, offset -50 to 50 mm, stiffen 1 to 5; puckering density 0 to 100 percent, width 0 to 50 mm; stiffness 0 to 20 on any kind. Unknown keys are kept as given.
- `style`: integer; required; range 1.0 to 4294967295.0; Style id as listed by style.list.

Note: safe to repeat.

### style_delete

Remove a style. Refused while any hardware, zipper or stitch uses it.

- `style`: integer; required; range 1.0 to 4294967295.0; Style id as listed by style.list.

Note: removes or replaces (undoable).

### style_list

List styles, optionally of one kind, with their parameters and how many hardware, zippers and stitches use each.

- `kind`: string (button | buttonhole | zipper | topstitch | puckering | trim); button, buttonhole, zipper, topstitch, puckering or trim.

Note: read-only.

### button_add

Place a button of a button style at a point inside a panel. In the simulation it is a small rigid disc attached to the panel triangle that holds the anchor.

- `panel`: integer; required; range 1.0 to 4294967295.0; Panel the hardware is attached to.
- `point`: array of number; required; Anchor point on the panel; it must lie inside the outline.
- `rotation`: number; range -360.0 to 360.0; Rotation about the surface normal, 0 to 360 degrees.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.
- `style`: integer; required; range 1.0 to 4294967295.0; Style id of the right kind, from style.list.

Note: in the window: Shift+B.

### buttonhole_add

Place a buttonhole of a buttonhole style at a point inside a panel.

- `panel`: integer; required; range 1.0 to 4294967295.0; Panel the hardware is attached to.
- `point`: array of number; required; Anchor point on the panel; it must lie inside the outline.
- `rotation`: number; range -360.0 to 360.0; Rotation about the surface normal, 0 to 360 degrees.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.
- `style`: integer; required; range 1.0 to 4294967295.0; Style id of the right kind, from style.list.

Note: in the window: Shift+H.

### button_fasten

Fasten a button to a buttonhole: during simulation the two anchor points are held together by one pair constraint. A button holds one buttonhole and the reverse. unfasten releases the button.

- `button`: integer; required; range 1.0 to 4294967295.0; Hardware id of the button.
- `buttonhole`: integer; range 1.0 to 4294967295.0; Hardware id of the buttonhole it fastens to.
- `unfasten`: boolean; default false; Release the button from its buttonhole instead.

Note: safe to repeat; in the window: Shift+F.

### hardware_move

Move a button, buttonhole or trim to another point or panel, or turn it.

- `hardware`: integer; required; range 1.0 to 4294967295.0; Hardware id.
- `panel`: integer; range 1.0 to 4294967295.0; Move to this panel; needs point.
- `point`: array of number; New anchor point; it must lie inside the outline.
- `rotation`: number; range -360.0 to 360.0; New rotation about the surface normal.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.

Note: safe to repeat; in the window: Shift+A.

### hardware_delete

Remove buttons, buttonholes or trims. A fastened partner is released. An imported trim mesh that nothing else uses is removed with it.

- `hardware`: integer; range 1.0 to 4294967295.0; Hardware id.
- `hardware_ids`: array of integer; Several hardware ids.

Note: removes or replaces (undoable).

### zipper_add

Add a zipper between two sides, each a list of spans listed in the direction the zipper zips. The closed part binds like a seam, the open part does not. Reports both side lengths and the mismatch.

- `a`: array of object; required; Spans of the first side, listed in the direction the zipper zips. A span with t0 above t1 walks against its line.
- `b`: array of object; required; Spans of the second side, listed in the same zipping direction.
- `closed`: number; range 0.0 to 1.0; How far the zipper is closed, 0 (open) to 1 (closed), measured from the start of the spans, which is the direction the zipper zips.
- `style`: integer; required; range 1.0 to 4294967295.0; Style id of a zipper style.

Note: in the window: Shift+Z.

### zipper_set

Set how far a zipper is closed (0 to 1) or change its style.

- `closed`: number; range 0.0 to 1.0; How far the zipper is closed, 0 (open) to 1 (closed), measured from the start of the spans, which is the direction the zipper zips.
- `style`: integer; range 1.0 to 4294967295.0; Move the zipper to another zipper style.
- `zipper`: integer; required; range 1.0 to 4294967295.0; Zipper id.

Note: safe to repeat.

### zipper_delete

Remove a zipper.

- `zipper`: integer; required; range 1.0 to 4294967295.0; Zipper id.

Note: removes or replaces (undoable).

### stitch_add

Add a topstitch or puckering line along spans of a panel outline or inner line, at an offset in mm. Render data, with a slight stiffening along the line (the stiffen parameter of the style).

- `offset`: number; range -50.0 to 50.0; Distance of the stitch line from the span, positive to the left of the walking direction. Default is the offset of the style.
- `spans`: array of object; required; Spans the stitch line follows, on outlines or inner lines.
- `style`: integer; required; range 1.0 to 4294967295.0; Style id of a topstitch or puckering style.

Note: in the window: Shift+S.

### stitch_set

Change a stitch after creation: its offset in mm from the span, its topstitch or puckering style, or its spans. Give at least one. One undo step.

- `offset`: number; range -50.0 to 50.0; New distance of the stitch line from the span, positive to the left of the walking direction.
- `spans`: array of object; New spans, replacing the old ones.
- `stitch`: integer; required; range 1.0 to 4294967295.0; Stitch id.
- `style`: integer; range 1.0 to 4294967295.0; New style id, a topstitch or puckering style.

### stitch_delete

Remove one stitch by id, several by stitches, or every stitch with all. Fray lines are stitches too.

- `all`: boolean; default false; Remove every stitch.
- `stitch`: integer; range 1.0 to 4294967295.0; Stitch id.
- `stitches`: array of integer; Several stitch ids.

Note: removes or replaces (undoable).

### fray_add

Fray an edge or a seam for rendering: stores a stitch whose style carries the fray length in mm. No effect on the simulation. Remove it with stitch.delete.

- `amount`: number; required; range 0.1 to 30.0; Length of loose threads for rendering.
- `seam`: integer; range 1.0 to 4294967295.0; Seam id; both sides of the seam fray. Use instead of spans.
- `spans`: array of object; Edge spans that fray.

### finish_add

Add an edge finish along spans. piping and binding create a strip panel of the given width, sewn to the spans by a real seam, and record it in generated. elastic and shirring create an elastic strip that shrinks to ratio and pulls the edge in. tape, bond and press stiffen a band of the given half width along the spans. crease adds a fold line from the start of the first span to the end of the last. One undo step.

- `angle`: number; range 0.0 to 360.0; Crease only: fold angle, 180 flat, lower folds to the front, higher to the back. Default 120.
- `fabric`: integer; range 1.0 to 4294967295.0; Fabric id of the strip; default is the fabric of the first span panel.
- `kind`: string (piping | binding | elastic | shirring | tape | bond | press | crease); required; piping and binding generate a strip panel sewn along the spans; elastic and shirring generate a narrow elastic strip (shirring when the spans lie on inner lines); tape, bond and press stiffen a band along the spans; crease adds a fold line between the ends of the spans.
- `ratio`: number; range 0.2 to 2.0; Elastic and shirring: length of the elastic at rest over its drawn length, 0.2 to 2 (default 0.8; 0.7 for shirring). Below 1 gathers the edge.
- `spans`: array of object; required; Spans of one or more panels the finish follows, in walking order.
- `strength`: number; range 0.0 to 20.0; Elastic: seam tension, default 1. Tape default 3, bond default 5, press default 1.5: stiffness multiplier of the band. Crease: fold strength 0 to 20, default 1.
- `width`: number; range 1.0 to 100.0; Piping 1 to 15 mm (default 6), binding 1 to 50 mm per side (default 12), elastic 1 to 50 mm (default 8), tape, bond and press: band half width on each side, 1 to 100 mm (default 10).

Note: in the window: Shift+E.

### finish_set

Change the width, ratio (elastic), strength or fabric (piping, binding, elastic) of an edge finish. Strip finishes regenerate their strip panel and seam, so panel and seam ids change. One undo step.

- `fabric`: integer; range 1.0 to 4294967295.0; Piping, binding and elastic only: fabric id of the strip.
- `finish`: integer; required; range 1.0 to 4294967295.0; Finish id as returned by finish.add.
- `ratio`: number; range 0.2 to 2.0; Elastic only: rest length over drawn length, 0.2 to 2.
- `strength`: number; range 0.0 to 20.0; Elastic seam tension, or stiffness multiplier of tape, bond and press, 0 to 20.
- `width`: number; range 1.0 to 100.0; New width, in the range of the finish kind (see finish.add).

### finish_delete

Remove edge finishes together with the strip panels they generated and every seam that touches those panels, in one undo step.

- `finish`: integer; range 1.0 to 4294967295.0; Finish id as returned by finish.add.
- `finishes`: array of integer; Several finish ids.

Note: removes or replaces (undoable).

### print_add

Place an image on a panel at a position (centre), rotation and size in mm, on the front, back or either face, with an optional emboss height. Render data. The image path must exist now; it is stored as given.

- `emboss`: number; range -5.0 to 5.0; Raised height of the print for rendering.
- `image`: string; required; Absolute path of a png, jpg, jpeg, webp or svg image.
- `name`: string; Print name.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel the print is placed on.
- `position`: array of number; Centre of the print on the panel.
- `rotation`: number; range -360.0 to 360.0; Rotation of the print.
- `side`: string (auto | front | back); Which face of the fabric carries the print.
- `size`: array of number; required; Width and height of the print on the panel.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.

Note: in the window: Shift+I.

### print_set

Change the image, position, rotation, size, side or emboss of a print. Only the given values change.

- `emboss`: number; range -5.0 to 5.0; Raised height of the print for rendering.
- `image`: string; Absolute path of a png, jpg, jpeg, webp or svg image.
- `name`: string; Print name.
- `position`: array of number; Centre of the print on the panel.
- `print`: integer; required; range 1.0 to 4294967295.0; Print id.
- `rotation`: number; range -360.0 to 360.0; Rotation of the print.
- `side`: string (auto | front | back); Which face of the fabric carries the print.
- `size`: array of number; Width and height of the print on the panel.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.

Note: safe to repeat.

### print_delete

Remove a print from its panel.

- `print`: integer; required; range 1.0 to 4294967295.0; Print id.

Note: removes or replaces (undoable).

### sew_rip

Open seams: the seams stay in the document but the simulation leaves them out until they are mended.

- `seams`: array of integer; required; Seam ids as listed by sew.list.

Note: safe to repeat.

### sew_mend

Close ripped seams again, the listed ones or all.

- `all`: boolean; default false; Close every ripped seam.
- `seams`: array of integer; Seam ids to close again.

Note: safe to repeat.

### quilt_make

Quilt a panel: clone it as a lining layer behind it with the opposite pressure, copy the chosen inner lines onto the lining as flat fold lines and sew each outer line to its copy; the outlines are sewn together too unless edge is false. One undo step.

- `edge`: boolean; default true; Also sew the two outlines together so the cells hold the filling.
- `gap`: number; range 0.0 to 100.0; Distance of the lining behind the outer layer in the 3D placement.
- `lines`: array of integer; required; Inner line ids of the panel that become quilting lines. Shape and reference lines turn into flat fold lines.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel to quilt; it keeps the outer layer.
- `pressure`: number; range -5000.0 to 5000.0; Pressure of the outer layer in pascals; the lining gets the opposite value so the cells puff. Default 2 Pa; the solver clamps it to 8 times the fabric weight per area (about 12 Pa for a 150 g/m2 cloth), because larger values blow the cloth apart.

### trim_import

Import an OBJ file as a rigid trim glued to a panel point. The mesh is recentred (x and y on the anchor, base at z 0), scaled to millimetres and stored inside the document as a trim style, so the project does not depend on the file afterwards. One undo step.

- `collision`: boolean; default true; Whether the trim collides with the cloth.
- `name`: string; Name of the trim style. Defaults to the file name.
- `panel`: integer; required; range 1.0 to 4294967295.0; Panel the trim is attached to.
- `path`: string; required; Absolute path of an .obj file, at most 8 MB and 20,000 triangles after fan triangulation.
- `point`: array of number; required; Anchor point on the panel; it must lie inside the outline.
- `rotation`: number; range -360.0 to 360.0; Rotation about the surface normal.
- `scale`: number; range 1e-06 to 1000000.0; Millimetres per unit of the file. Default 1.
- `space`: string (world | local); default "world"; world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply.
- `up`: string (z | y); default "z"; Axis of the file that points out of the cloth.

### trim_debug

Read only: what the simulation would read from the trims right now. Hardware attached to mesh triangles with barycentric weights, button and buttonhole fastenings, zipper binding counts and the stiffened triangle count per panel.

- no parameters

Note: read-only.

### anim_get

Read the animation timeline: frame rate, play range, current frame, motions, key tracks, cache layers with their validation numbers (worst body penetration, self intersections, vertex displacement, seam gap) and the recording job.

- no parameters

Note: read-only.

### anim_set

Set the timeline frame rate, play range or current frame. Frame rate and range are one undo step; moving only the current frame is not.

- `current`: integer; range 0 to 100000; Current frame. Changing only this is not an undo step.
- `end`: integer; range 0 to 100000; Last frame of the play range.
- `fps`: number; range 1 to 240; Timeline frame rate.
- `start`: integer; range 0 to 100000; First frame of the play range.

Note: safe to repeat.

### motion_import

Read a motion file (.bvh, .glb or .gltf animation with linear, step and cubic spline channels, and .fbx on desktop) and retarget it onto a body by joint-name mapping with rest pose alignment (A pose or T pose source), a root motion option (inplace, keep or off) and a lead-in. The converted motion is stored in the project. Returns which joints mapped.

- `animation`: string; glTF animation name or index. Default the first.
- `body`: integer; Body id the motion drives. Defaults to the first body.
- `fps`: number; range 1 to 240; Sampling rate for glTF and FBX motions. Default 30.
- `lead_in`: integer; range 0 to 1000; Frames spent blending from the current pose to the first motion pose.
- `name`: string
- `path`: string; required; Absolute path of a .bvh, .glb, .gltf or (desktop) .fbx motion file.
- `retarget`: object; {map: {motion joint: body joint}, root: keep|inplace|off, scale, loop, align}.
- `start`: integer; range 0 to 100000; Timeline frame where the motion begins. Default the play range start.
- `units`: string (auto | mm | cm | m | in | ft); Units of the motion file. BVH defaults to a guess from the skeleton height, glTF to metres. FBX is converted by the reader.
- `up`: string; BVH up axis: y, -y, z, -z, x or -x. Default y.

### motion_set

Change a motion's start frame, lead-in frames (blend from the current pose to the first motion pose), body, name or retarget options (map, root, scale, loop, align).

- `body`: integer
- `id`: integer; required; Motion id.
- `lead_in`: integer; range 0 to 1000
- `name`: string
- `retarget`: object; Options merged over the current ones: map, root, scale, loop, align.
- `start`: integer; range 0 to 100000

Note: safe to repeat.

### motion_remove

Remove a motion from the timeline.

- `id`: integer; required

Note: removes or replaces (undoable).

### key_set

Set a keyframe on a track: body.ID.pose, body.ID.MEASUREMENT, wind.FIELD, panel.ID.state.FIELD or fabric.ID.FIELD. Interpolation step, linear or smooth (cubic Hermite with Catmull-Rom tangents). Recording applies the body, measurement and wind tracks every frame.

- `frame`: integer; required; range 0 to 100000
- `interp`: string (step | linear | smooth); Interpolation from this key to the next. smooth is a cubic Hermite spline through Catmull-Rom tangents. Default linear.
- `target`: string; required; body.ID.pose (a pose name or {joint: [x, y, z, w]}), body.ID.MEASUREMENT, wind.FIELD, panel.ID.state.FIELD, fabric.ID.FIELD, where ID is a number and FIELD or MEASUREMENT a name.
- `value`: value; required; A number, a boolean, a vector, a pose name or a joint rotation object, by target.

Note: safe to repeat.

### key_remove

Remove the key at a frame from a track; an empty track is deleted.

- `frame`: integer; required; range 0 to 100000
- `target`: string; required

Note: removes or replaces (undoable).

### cache_set

Rename a cache layer, show or hide it, or make it the active cache.

- `active`: boolean; Make this the cache that anim.play and the timeline use.
- `id`: integer; required; Cache layer id.
- `name`: string
- `visible`: boolean

Note: safe to repeat.

### cache_remove

Delete a cache layer and its stored frames. Undo brings it back.

- `id`: integer; required

Note: removes or replaces (undoable).

### morph_set

Morph a body into another body made from the same mesh over a number of frames. Recording moves the collider through the blend like a motion.

- `frames`: integer; required; range 1 to 100000; Frames the morph takes.
- `from_body`: integer; required; Body id at the start. It stays the body that collides.
- `start`: integer; range 0 to 100000; Frame where the morph begins. Default the play range start.
- `to_body`: integer; required; Body id at the end. It needs the same vertex count and is left out of collisions while the morph is recorded.

Note: safe to repeat.

### anim_record

Record a cache layer on a background thread: for each frame pose the bodies (motions, pose, measurement and morph tracks, skinned through ffb-body), move the colliders, apply wind tracks, step the cloth with solver substeps, and store every panel's positions plus the body pose. Per frame it measures body penetration, self intersections and vertex displacement. It continues from the current draped state, so drape first. Poll job.state.

- `end`: integer; range 0 to inf; Last frame to record. Default the play range end.
- `name`: string; Cache layer name.
- `reset`: boolean; Rebuild the cloth from its placed state first. Default false: recording continues from the current draped state.
- `start`: integer; range 0 to inf; First frame to record. Default the play range start.
- `steps`: integer; range 1 to 8; Solver frames per timeline frame; the body moves between them. Default 2.
- `storage`: string (f32 | quantized); f32 keeps exact positions; quantized stores 16 bit positions per panel and frame (half the size, error under extent / 131070).
- `validate`: boolean; Measure body penetration, self intersections and vertex displacement every frame. Default true.

### anim_stop

Stop a running recording; the frames recorded so far are kept.

- no parameters

Note: safe to repeat.

### job_state

Read the recording job: phase, frame, total, per-frame validation statistics and warnings. It changes nothing; when ready is true, call job.harvest to store the recording as a cache layer.

- no parameters

Note: read-only.

### job_harvest

Store a finished recording as a cache layer (one undo step) and return its id in cache; null when nothing is waiting. Call it when job.state reports ready.

- no parameters

### cache_frame

Stream one cached frame's positions for every panel and the posed bodies to the viewer and return the vertex count. Frames are timeline frames inside the cache range.

- `frame`: integer; required; range 0 to inf; Timeline frame, inside the cache range.
- `id`: integer; Cache layer id. Default the active cache.

Note: safe to repeat.

### anim_play

Play a cache layer to the viewer from a frame at the cache frame rate on a background thread.

- `from`: integer; range 0 to inf; Frame to start from. Default the current frame, or the cache start when it is outside the cache.
- `id`: integer; Cache layer id. Default the active cache.
- `loop`: boolean; Start over at the end. Default false.
- `speed`: number; range 0.1 to 8; Playback speed, 1 is the cache frame rate.

Note: safe to repeat.

### anim_pause

Pause cache playback and return the frame it stopped on.

- no parameters

Note: safe to repeat.

### export_formats

List the export formats (obj with mtl, gltf, glb, fbx 7.4 ASCII or binary, usd as usda text, mdd, pc2; abc is reported unavailable) with their file sets, how animation is stored and the option schema of each.

- no parameters

Note: read-only.

### export_run

Write the draped garment to a 3D file. Per panel objects or one merged object, UVs from the stored island layout (or tiled for repeating fabric textures), normals, fabric materials (base colour, roughness, metalness, opacity, texture paths), optional thickness (back face and side strips from thickness.set), scale, units and axis. With animation true the active cache layer (or cache, start, end, step) is written as morph targets (glTF), blend shapes (FBX), time sampled points (USD), an OBJ sequence, or a point cache (MDD, PC2). The path must be absolute with the format's extension; files are written atomically and capped at 1 GiB.

- `animation`: boolean; default false; Include the active cache layer as animation.
- `axis`: string (y_up | z_up); default "y_up"; y_up keeps the engine axes. z_up rotates to Z up, Y forward (Blender, 3ds Max).
- `binary`: boolean; default false; FBX only: binary FBX 7.4 instead of ASCII.
- `cache`: integer; range 1 to inf; Cache layer id. Defaults to the active cache.
- `end`: integer; range 0 to inf; Last cache frame (inclusive).
- `format`: string (obj | gltf | glb | fbx | usd | mdd | pc2 | abc); required; Output format. abc (Alembic) is not available; the call explains what to use instead.
- `materials`: boolean; default true; Write fabric materials: base colour, roughness, metalness, opacity and texture paths.
- `panels`: array of integer; Panel ids to export. Omit for every panel.
- `path`: string; required; Absolute path of the file to write; the extension must match the format (.usda or .usd for usd). Companion files (mtl, bin, OBJ sequence frames) are written beside it.
- `quads`: boolean; Write the quad mesh made by quad.remesh (OBJ, FBX and USD keep quads; glTF is triangulated by the format). Defaults to true when the project mesh kind is quad and there is no thickness or animation. Cannot be combined with thickness or animation.
- `scale`: number; default 1.0; range 1e-06 to 1000000.0; Multiplier applied on top of units.
- `single_object`: boolean; default false; Merge every panel into one object.
- `start`: integer; range 0 to inf; First cache frame (inclusive).
- `step`: integer; default 1; range 1 to inf; Frame step.
- `thickness`: boolean; Extrude the back face and the border strips using thickness.set. Defaults to the project setting.
- `units`: string (mm | cm | m | in); default "mm"; Unit of the numbers in the file. The engine works in millimetres. glTF and GLB default to m (the glTF unit) and always use Y up.
- `uv`: string (packed | tiled | none); default "packed"; packed uses the island layout (0..1 and UDIM tiles along U). tiled uses grain aligned millimetres divided by the fabric map scale, so texture maps repeat.

### uv_get

List the UV island of every panel: offset, rotation, scale (UV units per millimetre), UDIM tile, size in millimetres, and the tile count and coverage. UVs come from each panel's 2D mesh in millimetres rotated so the grain line runs along V. When no layout was stored the automatic layout is reported with stored false.

- no parameters

Note: read-only.

### uv_layout

Pack all panel islands with a shelf packer at one common scale (so texel density is equal), with padding and up to 16 UDIM tiles along U. Stored in the project and used by export, bake and game ready output. One undo step.

- `padding`: number; range 0.0 to 0.1; Gap between islands and the tile border as a fraction of a tile (0.004 is 4 pixels on a 1024 map).
- `rotate`: boolean; default false; Allow quarter turns for a tighter fit. Off keeps every island grain aligned.
- `tiles`: integer; default 1; range 1 to 16; Number of UDIM tiles along U (1001, 1002, ...) the islands may use.

Note: safe to repeat.

### uv_island_set

Move, rotate, rescale or change the UDIM tile of one panel's UV island. Panels without a stored island get the automatic layout first. One undo step.

- `offset`: array of number; UV position of the island's lower left corner inside its tile.
- `panel`: integer; required; range 1 to inf; Panel id.
- `rotation`: number; Rotation of the island in degrees, applied after grain alignment.
- `scale`: number; range 0 to inf; UV units per millimetre.
- `tile`: integer; range 0 to 15; UDIM tile index along U (0 is 1001).

Note: safe to repeat.

### thickness_set

Set export thickness: on or off, amount in millimetres (back face offset along the normals) and whether border side strips close the solid. Applied when exporting, baking and writing game ready meshes; the simulation mesh is unchanged. One undo step.

- `amount`: number; range 0.01 to 50.0; Distance of the back face from the front face along the vertex normals.
- `enabled`: boolean; Extrude the cloth at export (and in bake and game ready output).
- `sides`: boolean; Add side strips along panel borders so the solid is closed.

### bake_run

Bake texture maps in the UV layout (uv.layout) as 8 bit PNG: a tangent space normal map and a displacement map that carry the drape detail of the simulated mesh over a smoothed base surface, and an ambient occlusion map by ray casting through the BVH, with visible bodies as occluders. Writes NAME_MAP.png beside the path, one file per map and UDIM tile. Synchronous; the ray budget is capped.

- `bodies`: boolean; default true; Let visible bodies occlude the cloth in the ambient occlusion map.
- `distance`: number; default 150; range 1 to 5000; Longest occlusion ray.
- `maps`: array of string; Maps to bake. Default all three.
- `padding`: integer; default 4; range 0 to 32; Pixels to extend islands past their borders to hide seams.
- `path`: string; required; Absolute path ending in .png. Files are written as NAME_MAP.png, with _UDIM added when the layout uses more than one tile.
- `samples`: integer; default 12; range 1 to 128; Ambient occlusion rays per pixel.
- `size`: integer; default 1024; range 64 to 4096; Pixels per side of every tile.
- `smooth`: integer; default 6; range 0 to 50; Laplacian smoothing passes that build the low frequency base surface the normal and displacement maps are measured against.

### gameready_run

Write one merged, packed-UV mesh reduced to a triangle budget with the quadric decimator of the remesh crate (panel borders locked so seams and silhouette stay). Materials are kept; the format follows the extension (.obj, .fbx, .glb, .gltf, .usda). Reports the triangles achieved against the target. Output is triangulated; quads is reserved.

- `axis`: string (y_up | z_up); default "y_up"
- `path`: string; required; Absolute output path. The extension picks the format: .obj, .fbx, .glb, .gltf or .usda.
- `quads`: boolean; default false; Reserved. Output is triangulated; use quad.remesh for quad topology.
- `scale`: number; default 1.0
- `target_tris`: integer; default 5000; range 12 to 5000000; Triangle budget for the whole merged mesh. Panel borders are locked, so a budget below the border triangles cannot be reached; the result reports what was achieved.
- `thickness`: boolean; default false; Extrude thickness (thickness.set) after decimation.
- `units`: string (mm | cm | m | in); default "mm"

### quad_remesh

Remesh panels into a quad-dominant mesh in pattern space: a 4-RoSy orientation field smoothed with alignment to the outline, the grain and any retopo guides, an Instant Meshes style position field, lattice extraction, a triangulated border band paired into quads, then cleanup (valence 2 removal, diagonal collapse to remove singularity pairs, smoothing). Quads respect the panel outline exactly and sewn seams get equal vertex counts on both sides. Density sets the edge length relative to the particle distance. Results live in session memory and the garment records mesh kind quad and the density; one undo step. Returns percent quads, valence histogram, irregular vertices, angle deviation from 90 degrees and a seam match report.

- `density`: number; range 0.1 to 8; Quad density relative to each panel's particle distance: the target quad edge is particle_distance divided by density, so 1 keeps the simulation resolution and 2 halves the edge. Leave out to reuse the stored density (1 when none).
- `panels`: array of integer; Panel ids as listed by pattern.get. Leave out for every panel.
- `pure_quads`: boolean; default false; Split every triangle and quad once so the result holds only quads (edge counts double, seams stay matched). Needed for thick export.

Note: safe to repeat.

### quad_clear

Drop the stored quad meshes of the given panels, or all panels. When no panel keeps a quad mesh the garment goes back to the triangle mesh kind. One undo step.

- `panels`: array of integer; Panel ids as listed by pattern.get. Leave out for every panel.

### retopo_get

The retopology state: mesh kind, density, whether pure quads are on, the user guide curves and the per-panel quad statistics of the stored results.

- `include_mesh`: boolean; default false; Also return the panel's quad mesh: 2D positions, 3D positions (mm), quads, triangles and the boundary vertex list with segment, index and count.
- `panel`: integer; range 1 to inf; With include_mesh, the panel whose quad mesh to return.

Note: read-only.

### retopo_set

Replace the user guide curves (panel space polylines in mm) that the orientation field aligns to. Guides apply at the next quad.remesh, or when the quad mesh is next read for output. One undo step.

- `guides`: array of object; required; User guide curves. The orientation field aligns to them and the quad lattice runs along them. An empty list removes all guides.

Note: safe to repeat.

### map_show

Show a per-vertex analysis map on the draped garment: strain (principal stretch in percent against the rest shape), stress (kPa from the fabric model), pressure (kPa the garment exerts on the body, Laplace estimate from tension and curvature), fit (signed distance to the body in mm), contact (1 in contact) or thickness (mm). The values stream as frame kind 3 per panel (object = panel id), every frame while the simulation runs and once when it is static. none clears the map. Not an undo step.

- `component`: string (max | min | weft | warp | shear); default "max"; Strain and stress component: max or min principal, along the weft, along the warp, or shear.
- `kind`: string (none | strain | stress | pressure | fit | contact | thickness); required; Map to show. none clears the overlay.
- `range`: array of number; Value range quantised into the frames, [low, high]. Default: taken from the data when the map is shown and then held.

Note: safe to repeat.

### map_stats

Compute a map for the current positions and return min, max, mean, percentiles and the same per panel. Does not stream or change the display.

- `component`: string (max | min | weft | warp | shear); default "max"
- `kind`: string (strain | stress | pressure | fit | contact | thickness); required

Note: read-only.

### compare_snapshot

Keep the current draped positions in session memory under a name for compare.set. Snapshots are not saved in the document.

- `name`: string; required; Snapshot name; saving over an existing name replaces it.

### compare_set

Stream the per-vertex distance in mm between two states (live, placed or snapshot names) as a map (id 7). Panels whose vertex counts differ between the states show no data. The names are kept in the session only (not in the document, not an undo step).

- `a`: string; required; First state: live (current positions), placed (the start of the simulation), or a snapshot name.
- `b`: string; required; Second state, same choices.
- `range`: array of number

Note: safe to repeat.

### compare_clear

Stop the comparison map and forget the compared state names. Snapshots stay.

- no parameters

Note: safe to repeat.

### compare_get

Read the compared state names, the snapshot names and whether a map is streaming.

- no parameters

Note: read-only.

### measure3d_add

Add a garment measure read from the current draped positions: length (polyline through surface points), circumference (slice of the draped garment by a horizontal height or by a plane through three points, summed over every panel it cuts; filter with panels or center and radius) or distance (straight line between two points). Definitions are kept in Garment.viz.compare under measures. An undo step.

- `center`: array of number; circumference: keep only the part of the slice within radius of this point (separates a torso from sleeves).
- `height`: number; circumference: slice with the horizontal plane y = height (mm).
- `kind`: string (length | circumference | distance); required
- `name`: string
- `panels`: array of integer; circumference: only slice these panels.
- `points`: array of object; length: two or more points of a polyline; distance: two points; circumference: three points that define the slice plane (instead of height).
- `radius`: number; range 0 to inf

### measure3d_remove

Remove one garment measure by id. An undo step.

- `id`: integer; required; range 1 to inf

### measure3d_list

List every garment measure with its live value in mm from the current positions, and the polylines to draw.

- no parameters

Note: read-only.

### viz_get

Read Garment.viz: stage, lights, saved views, fur, the active map and the compare and measure definitions.

- no parameters

Note: read-only.

### stage_set

Set the environment image, its rotation, exposure in stops, the ground plane and the background. Only the given fields change. An undo step.

- `background`: value; Background: a CSS colour string, a colour array, or an object the viewer understands.
- `exposure`: number; range -10 to 10; Exposure in stops.
- `ground`: boolean; Show the ground plane.
- `hdri`: ['string', 'null']; Environment image name or path; null clears it.
- `hdri_rotation`: number; Environment rotation in degrees.

### light_add

Add a light (directional, point, spot or area) to the stage. Returns it with its id. At most 16. An undo step.

- `color`: array of number; Linear RGB from 0 to 1.
- `direction`: array of number; Direction the light points along.
- `intensity`: number; range 0 to 1000
- `kind`: string (directional | point | spot | area)
- `position`: array of number; World position in mm.
- `shadows`: boolean

### light_set

Change the given fields of one light. An undo step.

- `color`: array of number; Linear RGB from 0 to 1.
- `direction`: array of number; Direction the light points along.
- `id`: integer; required; range 1 to inf; Light id from viz.get.
- `intensity`: number; range 0 to 1000
- `kind`: string (directional | point | spot | area)
- `position`: array of number; World position in mm.
- `shadows`: boolean

### light_remove

Remove one light by id. An undo step.

- `id`: integer; required; range 1 to inf

### view_save

Save the camera and display state under a name; saving over a name replaces that view. An undo step.

- `camera`: object; required; Camera and display state exactly as the viewer reports it.
- `name`: string; required

### view_remove

Remove one saved view by id. An undo step.

- `id`: integer; required; range 1 to inf

### fur_add

Grow fur strands on a panel (length mm, density per cm2, thickness mm, curl 0 to 1, colour). An undo step.

- `color`: array of number; Linear RGB from 0 to 1.
- `curl`: number; range 0 to 1
- `density`: number; range 0 to 100000; Strands per square centimetre.
- `length`: number; range 0 to 500; Strand length in mm.
- `panel`: integer; required; range 1 to inf; Panel id the fur grows on.
- `thickness`: number; range 0 to 10; Strand thickness in mm.

### fur_set

Change the given fields of one fur layer. An undo step.

- `color`: array of number; Linear RGB from 0 to 1.
- `curl`: number; range 0 to 1
- `density`: number; range 0 to 100000; Strands per square centimetre.
- `id`: integer; required; range 1 to inf
- `length`: number; range 0 to 500; Strand length in mm.
- `panel`: integer; range 1 to inf; Panel id the fur grows on.
- `thickness`: number; range 0 to 10; Strand thickness in mm.

### fur_remove

Remove one fur layer by id. An undo step.

- `id`: integer; required; range 1 to inf

### schematic_export

Write a flat technical drawing of the garment as SVG or PDF (vector, 1 unit = 1 mm): panel outlines, internal lines (folds dashed), notches, grain lines, seams with ids and lengths, topstitch dashes, zippers, buttons and buttonholes from the trims, panel dimensions and the 3D measures. The file is replaced atomically.

- `format`: string (svg | pdf); default "svg"
- `path`: string; required; Absolute output path ending in .svg or .pdf (matching format).

### capture_write

Write bytes captured in the window (a png snapshot, a webm video or a gif) to a file. The path must be absolute with the extension of the kind, the bytes must carry that format's signature, and the size is capped at 512 MB. The file is replaced atomically.

- `data`: string; required; The file bytes, base64 encoded (a data: URL prefix is accepted).
- `kind`: string (png | webm | gif); required
- `path`: string; required; Absolute output path whose extension matches kind.

### link_start

Start the Live Link for Unreal through USD: write the current draped garment as a usda layer into a folder at once, then replace it atomically every N simulation frames while the simulation runs (and after an edit, and when the simulation settles). Unreal's USD Stage reloads the layer when the file changes. The link follows the live positions without touching the document or undo; any file command (new, open) stops it. Starting again replaces the running link.

- `axis`: string (y_up | z_up); default "y_up"; Up axis stored in the layer. Unreal converts from the stage's upAxis.
- `every_frames`: integer; default 1; range 1 to 100000; Write a new layer every this many simulation frames. Edits to the garment write at once, and a stopped simulation writes its last frame after a quarter second.
- `file`: string; default "garment"; File name without extension; the layer is [folder]/[file].usda.
- `folder`: string; required; Absolute folder the layer is written into; created when missing. Point Unreal's USD Stage at the file inside it.
- `format`: string (usd); default "usd"; Layer format. Only usd (usda text) is written.
- `thickness`: boolean; default false; Write the back face and side strips set by thickness.set.

### link_stop

Stop the Live Link. The last layer stays in the folder.

- no parameters

Note: safe to repeat.

### link_state

Read whether the Live Link is running, the folder and layer path, the write interval, how many layers were written, the frame of the last one, its size, the last error and why it stopped.

- no parameters

Note: read-only.

### snapshot_save

Store the whole garment state as a snapshot inside the project (entry snapshot/ID.json, plus snapshot/ID.png when a thumbnail is given as base64 PNG, at most 1 MiB). Does not change the garment, so it is not an undo step. At most 64 snapshots per project.

- `name`: string; Snapshot name. Defaults to Snapshot N.
- `thumbnail`: string; Optional PNG of the 3D window as base64 (a data URL is accepted), at most 1 MiB decoded.

### snapshot_list

List the snapshots stored in the project: id, name, created (unix seconds), panels, bytes and whether a thumbnail exists. With thumbnails true each item carries thumbnail_png (base64).

- `thumbnails`: boolean; default false; Include each thumbnail as base64 PNG in thumbnail_png.

Note: read-only.

### snapshot_restore

Replace the garment with a snapshot's state. One undo step; bodies, the history and the other snapshots are kept.

- `id`: integer; required; range 1 to inf; Snapshot id from snapshot.list.

### snapshot_remove

Delete a snapshot and its thumbnail from the project. Not an undo step.

- `id`: integer; required; range 1 to inf; Snapshot id from snapshot.list.

Note: removes or replaces (undoable).

### library_list

List the user library: fabrics (.ffab), trim and style files (.ffstyle), components (.ffcomp), garments (.ffbrc), poses (.ffpose) and motions (.ffmotion) from every configured library folder, with name, tags, thumbnail path and size. Files placed in the kind folders by hand are listed too.

- `kind`: string (fabric | trim | style | component | garment | pose | motion); Only this kind. Leave out for all.
- `query`: string; Case-insensitive text that must appear in the name or a tag.

Note: read-only.

### library_add

Save a fabric, trim style, style, component, pose, motion or the open garment into the first library folder, with tags and an optional PNG thumbnail. Files are written atomically; the item can be inserted again with library.insert.

- `from`: ['integer', 'string']; What to save: a fabric id, a trim or style id (style.list), a component id (component.list), a body id for a pose (default the first body) or a motion id. Leave out for kind garment, which saves the open project.
- `kind`: string (fabric | trim | style | component | garment | pose | motion); required
- `name`: string; required; Item name.
- `tags`: array of string
- `thumbnail`: string; Optional PNG as base64, at most 1 MiB.

### library_insert

Bring a library fabric, trim, style, component, pose or motion into the open project. One undo step. A garment is not inserted: the error names the path for file.open.

- `body`: integer; range 1 to inf; Pose and motion only: the body to apply it to. Defaults to the first body.
- `id`: string; required; Library item id from library.list.
- `joins`: array of object; Component only: joins for component.insert, each {port, to: {component, port}, reversed, ease}.

### library_remove

Delete a library item, its thumbnail and index entry from disk. Only files inside the library folders can be removed. Cannot be undone.

- `id`: string; required; Library item id from library.list, for example fabric/denim.

Note: removes or replaces (undoable).

### library_roots

The folders the library reads and writes: the configured list, or the default [data dir]/library.

- no parameters

Note: read-only.

### library_set_roots

Set the library folders (absolute paths, created when missing, at most 8). The first receives new items. The list is stored in [data dir]/library_roots.json. An empty list returns to the default folder.

- `paths`: array of string; required; Absolute folders. The first one receives new items. An empty list returns to the default [data dir]/library.

Note: safe to repeat.

### component_create

Group panels into a component: the author lists the panels, and names ports as edge spans of those panels with a length tolerance in mm. A component owns its own panels and their real seams; a panel belongs to at most one component. Nothing is inferred from panel shape.

- `name`: string; required; Component name, for example Sleeve L.
- `panels`: array of integer; required; Panel ids the author groups into the component. A panel belongs to at most one component.
- `ports`: array of object; Named edge spans of the component's own panels: [{name, spans: [{panel, segment, t0, t1}], tolerance}]. tolerance is a length in mm (default 5); t0 and t1 default to the whole segment.

### component_list

List components with their panels, ports (live length against the stored length, tolerance, whether the edge still exists, the join it is in) and internal seams, and the joins with the real seams they made.

- no parameters

Note: read-only.

### component_save

Write a component to the library as a .ffcomp file: its panels, their internal seams and its ports, nothing else (no fabrics, trims or body). Path must be inside the component folder of a library folder; leave it out to use the first library folder.

- `id`: integer; required; range 1 to inf; Component id.
- `name`: string; Library name. Defaults to the component name.
- `path`: string; Absolute .ffcomp path inside the component folder of a library folder. Leave out to save into the first library folder under the component's name.
- `tags`: array of string
- `thumbnail`: string; Optional PNG as base64, at most 1 MiB.

### component_insert

Insert a library component: its panels get new ids and are placed beside the garment, its internal seams and ports are recreated, and the given joins become ordinary seams between real panel edges. One undo step.

- `from`: string; required; Library component id (component/sleeve-l) or an absolute .ffcomp path inside a library folder.
- `joins`: array of object; Joins to make now: [{port, to: {component, port}, reversed, ease}], where port is a port of the inserted component and to a port of a component in the project.

### component_swap

Replace a component with a library component: remove the old panels, insert the new ones centred where the old were, and recreate every join by port name. Refused when a joined port name is missing or its length differs by more than the tolerance (unless ease is true, which sews with a seam ratio). Panels are never graded or resized. One undo step.

- `ease`: boolean; default false; Accept ports whose length differs by more than the tolerance and sew them with a seam ratio. A missing port name always refuses the swap. Panels are never resized.
- `id`: integer; required; range 1 to inf; Project component to replace.
- `with`: string; required; Library component id or an absolute .ffcomp path inside a library folder.

### component_join

Join a port of one component to a port of another: one seam per pair of spans (or one multi-span seam when the span counts differ) between the real panel edges, recorded as a join. Refused when the port lengths differ by more than the tolerance unless ease is true.

- `a_port`: ['object', 'string']; required; {component, port} or "component:port".
- `b_port`: ['object', 'string']; required; {component, port} or "component:port".
- `ease`: boolean; default false; Allow ports whose lengths differ by more than the tolerance; the seam gets ratio = b length / a length.
- `reversed`: boolean; default true; Pair the start of side a with the end of side b (right for two counter-clockwise outlines meeting along an edge).

### component_check

Compare a library component with a project component by port names and lengths only: for every joined port of the project component, whether the other has a port of that name and its length is within the tolerance. Changes nothing.

- `id`: integer; required; range 1 to inf; Project component that would be replaced.
- `with`: string; required; Library component id or an absolute .ffcomp path inside a library folder.

Note: read-only.

### component_remove

Remove a component and its joins. The panels and seams stay unless delete_panels is true.

- `delete_panels`: boolean; default false; Also delete the component's panels and the seams on them. Without it only the grouping and its joins are removed and the panels and seams stay.
- `id`: integer; required; range 1 to inf; Component id.

Note: removes or replaces (undoable).

### component_sample

Build the sample modular T-shirt: tshirt groups the drafted sample into components and joins; sleeveless adds a no-sleeve armhole binding component for swapping.

- `like`: integer; range 1 to inf; sleeveless only: the component whose port sets the binding lengths (a sleeve).
- `name`: string (tshirt | sleeveless); required; tshirt groups the drafted sample T-shirt (sample.load tshirt) into Torso, Sleeve L and Sleeve R with an armhole port and a join each. sleeveless adds a no-sleeve variant: two armhole binding strips as one component with an armhole port the lengths of another component's port.
- `port`: string; default "armhole"; sleeveless only: the port name to match.
[ffb-tools:end]

## Not yet available

Only the tools listed in the reference exist. If asked for something not listed (for example Alembic export, which `export_run` reports as unavailable), say so plainly rather than approximating with other edits.
