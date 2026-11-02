# Export and UV

Writing the draped garment to files other tools read: 3D formats with UVs and materials, baked maps, quad meshes, game-ready meshes and technical drawings.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/output_cmds/` | export, UV, thickness, bake, game ready, the Live Link scene |
| `native/crates/ffb-remesh/src/` | field-aligned quad remeshing and the quadric decimator |
| `native/crates/ffb-commands/src/remesh_cmds/` | `quad_` and `retopo_` commands |
| `native/crates/ffb-io/src/export/` | the format writers (OBJ, glTF, FBX, USD, MDD, PC2); the project file, DXF-AAMA and body import are in the same crate |
| `native/crates/ffb-commands/src/output_cmds/bake.rs` | the map baker |
| `frontend/src/workspace/output/` | the Output panel, UV Editor, guides, bake and export dialogs |

## Formats

`export_formats` lists them: OBJ (with MTL), glTF, GLB, FBX 7.4 (ASCII, or binary with `binary` true), USD (usda text), MDD and PC2; Alembic (`abc`) is reported unavailable. `export_run` writes the draped garment to an absolute `path` whose extension matches `format`.

| Option | Meaning |
| --- | --- |
| `single_object`, `panels` | merge panels into one object, or export chosen panels |
| `uv` | `packed` (the island layout, UDIM tiles along U), `tiled` (grain-aligned millimetres over the fabric map scale) or `none` |
| `materials` | base colour, roughness, metalness, opacity, texture paths |
| `thickness` | extrude back face and border strips using `thickness_set` |
| `units`, `scale`, `axis` | `mm`, `cm`, `m`, `in`; `y_up` or `z_up` (Blender, 3ds Max); glTF is always metres and Y up |
| `animation`, `cache`, `start`, `end`, `step` | write a recorded cache layer as animation |
| `quads` | keep the quad mesh from `quad_remesh` (OBJ, FBX, USD keep quads; glTF triangulates); not with thickness or animation |

Exports were checked in Blender 5.2 with UVs, materials and animation. USD and FBX import were checked there only.

## UV

`uv_get` lists each panel's island (offset, rotation, scale, UDIM tile, size). `uv_layout` packs all islands with a shelf packer at one common scale with padding and up to 16 UDIM tiles. `uv_island_set` moves, rotates, rescales or retiles one island. The window's UV Editor edits the same data. Layout is stored in the project.

## Maps, remesh and game ready

- `bake_run` writes 8 bit PNG normal, displacement and ambient occlusion maps in the UV layout, one file per map and UDIM tile (`size` 64 to 4096, `padding`, `samples`, `smooth`).
- `quad_remesh` builds a quad-dominant mesh in pattern space aligned to the outline, grain and `retopo_set` guide curves; sewn seams get equal vertex counts on both sides. `quad_clear` drops it; `retopo_get` reports state. The min-cost-flow regularity step of QuadriFlow is not built.
- `gameready_run` writes one merged, packed-UV mesh reduced to `target_tris` with panel borders locked.
- `schematic_export` writes SVG or PDF drawings, see [[Visualization-and-Capture]].
- `exchange_export_dxf` writes the flat pattern, see [[Pattern-Drafting]].

## Hand-off and streaming

`link_start`, `link_stop` and `link_state` run the Live Link to Unreal through USD, see [[Agents-and-MCP]]. The Python helper `formfabric.gof.send` writes a GoF bundle for formsculpt and formsurface, see [[Python-API-and-Plugins]].

## Contract for new exporters

A writer must honour units, axis, UVs and materials the same way as the others, write atomically (`args::write_atomic`), refuse a path whose extension does not match, and be checked by opening the result in another tool.
