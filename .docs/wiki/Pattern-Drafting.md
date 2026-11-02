# Pattern drafting

A garment is a list of panels. Each panel is an outline of points with optional Bezier handles, plus inner lines and inner shapes, notches and a grain line. The Pattern View draws and edits them; every edit is one undoable engine command.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-pattern/src/` | the document model: panels, points, seams, components |
| `native/crates/ffb-commands/src/pattern_cmds/` | draw, edit and measure commands |
| `native/crates/ffb-commands/src/pattern_tools_cmds/` | seam allowance, offsets, notches, darts, slash and spread, mirror links |
| `native/crates/ffb-commands/src/exchange_cmds/` | DXF-AAMA import and export |
| `frontend/src/workspace/pattern-view/` | the 2D canvas, tools, rulers, snapping, measure |
| `frontend/src/workspace/inspector/` | the Inspector sections for the selected panel |

## Drawing

- `pattern_rectangle` (x, y, width, height), `pattern_ellipse`, and `pattern_draw_panel` with `points` in order around the outline. A point has a kind (corner, curve or smooth) and optional `h_in` and `h_out` handles as offsets. A segment is a cubic Bezier when either end has a handle, otherwise a straight line.
- `pattern_inner_line` and `pattern_inner_shape` add darts, fold lines and guides. `pattern_set_inner` changes an inner line's role (fold angle, strength).
- Shape curves with `pattern_curvature` or `pattern_set_handle`; add detail with `pattern_add_point` or `pattern_split_line`; remove with `pattern_delete_point`; change a point's kind with `pattern_set_point_kind`; relax noise with `pattern_smooth`.
- `pattern_move_point` moves one point (window key Z). `pattern_begin_drag` and `pattern_end_drag` group a mouse drag into one undo step.
- `pattern_transform` moves, rotates, scales or mirrors panels (key A); `pattern_duplicate` (Mod+D), `pattern_delete` (Delete), `pattern_rename` (F2).
- `pattern_set_props` sets particle distance (the mesh edge length), layer and grain angle. `pattern_grain_line` places the grain line.

## Allowance, notches and pattern tools

| Command | Does |
| --- | --- |
| `pattern_seam_allowance` | adds an allowance offset to chosen segments |
| `pattern_offset_outline` | offsets an outline inward or outward |
| `pattern_notch_add`, `pattern_notch_set`, `pattern_notch_delete` | notches on outline segments; they drive automatic sewing |
| `pattern_segment_lengths`, `pattern_match_length` | read segment lengths; adjust one segment to match another |
| `pattern_dart`, `pattern_slash_spread`, `pattern_unfold`, `pattern_spiral`, `pattern_trace` | dart, slash and spread, unfolding a symmetric half, spiral and tracing tools |
| `pattern_mirror_link`, `pattern_instance`, `pattern_unlink` | linked mirrored or instanced copies that follow their source |

## Measuring and snapping

`pattern_measure` returns segment lengths, perimeter and area in millimetres. `pattern_snap` is read only and reports where a point would snap and to what. The View menu toggles grid, point and edge snapping and the display unit (mm, cm, in); rulers and the measure tool are in the Pattern View. `panel_state` is a simulation property (freeze, disable, stiffen), not a drafting one.

## Exchange

`exchange_import_dxf` and `exchange_export_dxf` read and write DXF-AAMA (and plain DXF) following the public ASTM D6673 conventions. A round trip stayed within 0.046 mm on outlines and 0.027 mm on notches in the recorded checks. `schematic_export` writes a flat technical drawing, see [[Visualization-and-Capture]].

## Contract for new pattern work

- A new drafting command is declared only in `ffb-commands`; the menus, palette, MCP and skill reference pick it up after `./formfabric engine build` and `./formfabric agents sync`.
- Seam spans follow point edits, but not dart, slash, offset or inner-line edits (open item): check `sew_check` after such edits.
- Add the interface strings to every language table in `frontend/src/ui/i18n/`.
