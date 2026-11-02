# Export and UV

Writing the draped garment to files other tools read: 3D formats with UVs and materials, baked maps, quad meshes, game-ready meshes and technical drawings.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/output_cmds/` | export, UV, thickness, bake, game ready, the Live Link scene |
| `native/crates/ffb-remesh/src/` | field-aligned quad remeshing and the quadric decimator |
| `native/crates/ffb-commands/src/remesh_cmds/` | `quad_` and `retopo_` commands |
| `native/crates/ffb-io/src/export/` | the format writers (OBJ, glTF, FBX, USD, Alembic, MDD, PC2) and `weld.rs`; the Ogawa container and archive writer are in `ffb-io/src/abc/`; the project file and DXF-AAMA are in the same crate |
| `native/crates/ffb-body/src/import/abc.rs` | the Alembic reader for body import |
| `native/crates/ffb-commands/src/output_cmds/bake.rs` | the map baker |
| `frontend/src/workspace/output/` | the Output panel, UV Editor, guides, bake and export dialogs |

## Formats

`export_formats` lists them: OBJ (with MTL), glTF, GLB, FBX 7.4 (ASCII, or binary with `binary` true), USD (usda text), Alembic (`abc`, Ogawa container), MDD and PC2. `export_run` writes the draped garment to an absolute `path` whose extension matches `format`.

| Option | Meaning |
| --- | --- |
| `single_object`, `panels` | merge panels into one object, or export chosen panels |
| `uv` | `packed` (the island layout, UDIM tiles along U), `tiled` (grain-aligned millimetres over the fabric map scale) or `none` |
| `materials` | base colour, roughness, metalness, opacity, texture paths |
| `weld` | `unweld` (default, every panel keeps its own vertices), `weld` (vertices of sewn seam edges become shared points in one object) or `weld_selected` (only seams whose two panels are both in `panels`, at least two panels). UVs stay per corner, so UV seams are kept; cache formats use one index map for all frames |
| `thickness` | extrude back face and border strips using `thickness_set`; quad meshes extrude as quads with quad side walls, and with `weld` the walls follow the welded boundary |
| `units`, `scale`, `axis` | `mm`, `cm`, `m`, `in`; `y_up` or `z_up` (Blender, 3ds Max); glTF is always metres and Y up |
| `animation`, `cache`, `start`, `end`, `step` | write a recorded cache as animation; without `cache` every visible pattern layer is used, resolved top down per frame from the layer start (trim layers are skipped, a frame no layer covers is an error), with `cache` only that layer |
| `quads` | keep the quad mesh from `quad_remesh` or `topology_remesh` (OBJ, FBX, USD keep quads; glTF triangulates); not with animation |

Exports were checked in Blender 5.2 with UVs, materials and animation. USD and FBX import were checked there only.

Alembic is written by formfabric's own Ogawa writer: Y up, metres by default (`axis` is ignored), faces reversed to clockwise order, UVs per face corner, vertex normals, time starting at 0 with samples 1/fps apart, positions per frame with constant topology. It was verified against our own strict reader and the written format description; opening it in Blender is an owner check. The HDF5 layout is not written. Welded output and thickness pass through because the writer takes the finished scene. `body_import` reads `.abc` files (polygon meshes of the first sample, or the one named by `sample`).

## UV

`uv_get` lists each panel's island (offset, rotation, scale, UDIM tile, size). `uv_layout` packs all islands with a shelf packer at one common scale with padding and up to 16 UDIM tiles. `uv_island_set` moves, rotates, rescales or retiles one island. The window's UV Editor edits the same data. Layout is stored in the project.

## Maps, remesh and game ready

- `bake_run` writes 8 bit PNG maps in the UV layout, one file per map and UDIM tile (`size` 64 to 4096, `padding`, `samples`, `smooth`): normal, displacement, ambient occlusion, and the look maps diffuse, metalness, roughness and opacity. `fill_px` (0 to 256) extends islands past their border to hide seams; 0 leaves the PNG transparent outside the islands. Only PNG textures are sampled (bilinear, repeat); texture rotation and offset are ignored, like the exporters.
- `quad_remesh` builds a quad-dominant mesh in pattern space aligned to the outline, grain and `retopo_set` guide curves; sewn seams get equal vertex counts on both sides. `quad_clear` drops it; `retopo_get` reports state. With `regularize` true it first runs an integer edge offset pass and a min-cost flow over lattice offsets, and falls back to the plain extraction when the flow is not full; it is off by default because the gain is mixed (263 against 300 irregular interior vertices over four T-shirt panels, worse at the two lowest densities). Flip repair, SAT and the multi-resolution hierarchy are not built.
- `gameready_run` writes one merged, packed-UV mesh reduced to `target_tris` with panel borders locked.
- `schematic_export` writes SVG or PDF drawings, see [[Visualization-and-Capture]].
- `exchange_export_dxf` writes the flat pattern, see [[Pattern-Drafting]].

## Drawing topology

The Retopo section of the Output drawer, opened by Window, Retopo or by the `ffb:output-section` event, turns on Draw topology in the 3D View. You click points on a panel; a click on the first point closes a face of 3 or 4 points, and Enter or a double click finishes an open line. Points snap to an existing point, a ghost point, or a sewn edge. Everything is stored in the project, with one undo step per command.

| Command | Does |
| --- | --- |
| `topology_add_point` | `panel` with `pos` (mm) or `uv` (fractions of the panel box), and `snap` (mm) |
| `topology_close`, `topology_finish`, `topology_delete_last`, `topology_cancel` | close the open shape as a face, finish it as a line, drop its last point, or drop it |
| `topology_merge`, `topology_merge_all` | merge two points, or all points closer than `distance` |
| `topology_divide` | splits the quad loop across an `edge` into `count` parts |
| `topology_delete`, `topology_clear` | remove a face or point, or all topology |
| `topology_remesh` | `mode` `replace` swaps the panel's automatic quads for the drawn faces; `duplicate` keeps both |
| `topology_list` | points, faces, lines, the open shape and the ghost points |

Ghost points are computed, not stored: a point within 1 mm of a sewn edge is mirrored at the same arc length fraction onto the partner side of the seam (flipped for reversed seams), and `topology_list` reports it as occupied once a real point sits there. `quad_clear` does not clear a drawn topology; use `topology_clear`. Not built: Edit Topology drag and box select, Patch Topology, face loop add by Shift+Ctrl, hiding points on the far side of the garment, and a second OBJ object for duplicate mode (the OBJ holds the automatic mesh only).

## Hand-off and streaming

`link_start`, `link_stop` and `link_state` run the Live Link to Unreal through USD, see [[Agents-and-MCP]]. The Python helper `formfabric.gof.send` writes a GoF bundle for formsculpt and formsurface, see [[Python-API-and-Plugins]].

## Contract for new exporters

A writer must honour units, axis, UVs and materials the same way as the others, write atomically (`args::write_atomic`), refuse a path whose extension does not match, and be checked by opening the result in another tool.
