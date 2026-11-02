# Visualization and capture

Analysis maps drawn on the draped garment, comparison between states, 3D measures, a lit stage, saved views, fur, technical drawings, and snapshots, videos and turntables.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/viz_cmds/` | maps, measures, stage, lights, schematic, capture |
| `frontend/src/workspace/viz/` | the Visualization panel: maps, ramps, measure, stage, views, capture |
| `frontend/src/workspace/view3d/` | the renderer: look, stage, fur, annotations, picking |
| `docs/encyclopedia/10-render-visualization.md` | the behaviour specification |

## Maps

`map_show` streams a per-vertex map over the draped garment: `strain` (principal stretch in percent against the rest shape), `stress` (kPa from the fabric model), `pressure` (kPa on the body, a Laplace estimate), `fit` (signed distance to the body in mm), `contact` or `thickness`. `component` picks max, min, weft, warp or shear for strain and stress; `range` fixes the colour range, and `kind` none clears it. `map_stats` returns min, max, mean, percentiles and per-panel numbers without changing the display. A strain map PNG captured from a draped sample is the Phase 15 re-capture exit.

## Compare and measure

- `compare_snapshot` keeps the current draped positions in session memory under a name; `compare_set` streams the per-vertex distance in mm between two states (live, placed or a snapshot name); `compare_clear` and `compare_get`. These are session snapshots, different from project snapshots in [[Library-Snapshots-and-Components]].
- `measure3d_add`, `measure3d_remove`, `measure3d_list` read lengths along the surface, circumferences at a height and distances from the current draped positions.

## Stage, lights, views, fur

`stage_set` (environment image, rotation, exposure in stops, ground, background; `background` may be `{image, fit}` with a local png, jpg, jpeg or webp path and `fit` `cover` or `contain`, and `image: null` clears it), `light_add`, `light_set`, `light_remove` (directional, point, spot or area, at most 16), `view_save` and `view_remove` (camera and display state by name), `fur_add`, `fur_set`, `fur_remove` (length, density, thickness, curl, colour). `viz_get` reads all of it. These are project data and undoable.

## Drawings and capture

- `schematic_export` writes a flat technical drawing as SVG or PDF at 1 unit per millimetre: outlines, fold lines, notches, grain lines, seams with ids and lengths, trims, panel dimensions and the 3D measures.
- Snapshots (PNG), videos (WebM) and turntables are captured in the window; `capture_write` writes the bytes to an absolute path, checking the signature and capping the size at 512 MB, replacing the file atomically.
- The Capture dialog offers a multi-view snapshot: 2 to 10 of eight camera views (Front, Back, Left, Right, 3/4 Left, 3/4 Right, Top, Bottom), combined horizontally or vertically into one PNG, or saved as separate images. A resolution field writes ppi into the PNG pHYs chunk. The turntable can be saved as a GIF with a transparent background; PNG frames are not offered with alpha. The Stage drawer takes the background image path, with a file chooser on desktop builds.

## Known limits

The Capture dialog changes height when the kind select changes; the space is not reserved yet. The 60 fps orbit at 200k triangles needs a real GPU; software rendering in the sandbox managed about one frame per second. Headless Chromium prints a missing WebGPU adapter warning, which is expected.

## Contract for new visualization

A new map must say its unit and range in the command description, stream through the existing frame kind rather than a new socket, and stay off the solver thread's critical path.
