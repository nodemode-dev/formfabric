# Draping and the solver

The drape runs the cloth simulation on the placed panels around the body. The Rust crate `ffb-sim` is the solver, `ffb-gpu` an optional wgpu compute path, and `ffb-commands/src/sim_cmds/` the commands.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-sim/src/` | the model, vertex block descent (VBD), the implicit solver (`implicit/`), contact, self collision, energy |
| `native/crates/ffb-gpu/src/` | the GPU solver (`vbd.wgsl`, `contact.wgsl`) |
| `native/crates/ffb-mesh/src/` | constrained Delaunay meshing at the particle distance |
| `native/crates/ffb-commands/src/sim_cmds/` | `sim_` and `pin_` commands, wind, grab, pinch |
| `frontend/src/workspace/drape/` | the Drape bar, simulation panel, pins, panel states |

## Running a drape

1. `sim_reset` rebuilds the cloth from the garment and returns every panel to its placed state (frame zero).
2. `sim_step` with `frames` advances on the calling thread (60 to 120 frames settles a sample); `sim_start` and `sim_stop` run it in the background and stream positions to the window; `sim_toggle` is the Space key.
3. `sim_state` reports the frame, vertex count, seam gap, method, device and the last step statistics.
4. `sim_apply` writes the draped state into the document as one undo step (each panel gets the placement that best fits its draped vertices), so the shape survives saving; `sim_positions` reads vertices.

## Presets

`sim_settings` with `preset` sets the quality; with no fields it reads the settings. Settings are not saved in the document.

| Preset | Block descent (substeps by iterations) | Implicit |
| --- | --- | --- |
| fast | 3 x 6 | 2 substeps |
| normal | 3 x 8 | 3 substeps |
| quality | 8 x 15, self contact on | 4 substeps, self contact on |
| final | 12 x 25, self contact on | 6 substeps, 3 Newton iterations, self contact on |

Other fields: `time_step` (1/240 to 1/15 s), `gravity`, `friction`, `collision_thickness`, `skin_offset`, `air_damping`, `self_collision`, and the closing phase (`close_frames`, `close_gap`, `seam_boost`) that starts a drape with stiffened seams, reduced gravity and strong damping so seams shut before the cloth falls.

## Auto method

`method` is `auto` by default: block descent (`vbd`) when the garment has seams or links, implicit (Gauss-Newton with preconditioned conjugate gradients) otherwise. Naming `vbd` or `implicit` pins the choice. Sewn garments use block descent because the implicit solver did not converge on them (the cloth crumpled and rode up the shoulders). The implicit solver is the accuracy reference: a hanging square stays within 0.007 mm of the conjugate gradient reference.

## Contact and self contact

Contact reaches 40 mm per substep and clamps the approach to the body, with penetration at 0 mm on the sample garments in the recorded checks. Self contact is on in Quality and Final, off in Normal. Layer order comes from each panel's layer. `sim_compare` runs the interactive and reference solvers side by side and reports the vertex distance.

## Pins, grab, wind

`pin_add`, `pin_box` (Shift+X), `pin_lasso` (Shift+L), `pin_remove`, `pin_clear`; `sim_grab` holds a vertex as the mouse drag does; `sim_pinch` pulls a vertex with a falloff (Shift+C); `wind_set` sets directional or radial wind with strength, decay and turbulence; `panel_state` freezes, disables, stiffens or shrinks panels; `fold_place` folds a panel before the drape (Shift+D).

## GPU (opt-in)

`sim_settings` with `device` gpu needs an engine built with the `gpu` feature and an adapter; otherwise the state says unavailable and the simulation stays on the CPU. The GPU path supports block descent only. It is not the default.

## Rayon

The `ffb-sim` feature `rayon` is on by default for non-wasm builds; `RAYON_NUM_THREADS` limits threads. Block descent is deterministic for a given thread count and bit-identical across thread counts.

## Known limits

- CPU speed misses its targets: about 15 to 22 ms per frame at 676 vertices (target 8 ms) and 150 to 260 ms at 5,000 (target 50 ms) on a shared two-core machine; the 30k vertex shirt at 30 fps is unmeasured.
- The Quality preset's self contact was unstable on sewn cloth and a recorded walk showed cloth crossing itself in most frames at v0.15.0; work on both was in progress at the audit, so check `docs/encyclopedia/15-OPEN-GAPS.md` for the current state.
- GPU: wind direction, layered contact, self contact and real-hardware speed were open; no GPU was available for the 30 fps and 3 mm checks.
- Untangle is an opt-in pass: `sim_settings` with `untangle` `normal` or `flipped` (default `off`) separates intersecting pairs that began intersecting this frame along the triangle normal, by at most 2 mm per vertex per frame, never moving seam-near or pinned vertices; pairs of different layers are always handled, with the higher layer outside for `normal` and inside for `flipped`. It did not reliably cut the pairs on a T-shirt walk (84 to 37 on a hard walk, 23 to 65 on a gentler one), so it stays off by default and zero pairs is not reached.
- Refit ended 0.0 mm inside the target body on the drafted T-shirt at v0.22.0; see [[3D-Editing]].

## Contract for new solver work

Measure before and after with the examples in `ffb-sim/examples/`; report penetration, seam gap and self intersections, not only speed. Keep wasm builds free of rayon and threads.
