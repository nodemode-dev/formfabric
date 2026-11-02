# HASHOUT_LOG · native

Living context for the Rust engine. What would be comments in the crates lives here, newest first. Read before editing anything under `native/`.

## v0.7.0 to v0.9.0 · 2026-11-01

### Solver (ffb-sim)

- Units inside the solver: millimetres, seconds, grams; `Material::from_physical` converts the SI fabric values. `SeamPair` and `SeamBind` stiffness at or below 0 defaults to the cloth's largest stretch stiffness.
- `triangle_grain` is radians counter-clockwise from +y of the rest shape to the warp: weft axis (cos g, sin g), warp axis (-sin g, cos g). Winding normals must face outward for pressure and fold angles.
- Membrane per triangle: F = Ds Dm^-1 in the grain frame, energy 0.5 ku (|f_u| - 1)^2 + 0.5 kv (|f_v| - 1)^2 + 0.5 ks (f_u . f_v)^2 times rest area; compression at 5 percent of tension. Bending: dihedral hinge 0.5 B 3 l^2 / (A1 + A2) (theta - theta0)^2 with B blended between weft, warp and bias by the hinge direction; buckling softens it under compression.
- Contacts are penalties inside the sweeps (stiffness 1000 m / h^2) with smoothed Coulomb friction from the lagged normal force, then a hard projection. Friction lives inside the solve because friction applied to positions afterwards locked stiff cloth.
- Shrink scales the rest shape in the grain frame; mass uses the unshrunk area. Pressure is in pascals along the outward normal.
- `reference_step` is implicit Euler with conjugate gradient on the same energies, used by `sim.compare`.
- Examples (not tests): `drape_sphere`, `tube_capsule`, `gradcheck`, run with `cargo run --release -p ffb-sim --example ...`.
- Known: VBD under-converges on stiff woven cotton at 10 iterations and is far slower than its target (see BUILD_NOTES for the numbers).

### Simulation runner (sim_cmds)

- `adapter.rs` is the only file naming solver, material, collider and reference APIs. `SimState` holds `Arc<Mutex<Core>>`; the run thread locks Core, steps once, copies positions per panel, unlocks, publishes through the sink and sleeps to the time step. It never touches the Session lock.
- Only positions are published; vertex order per panel equals the mesh order, so the mesh module's topology stays valid. Flat mirrored panels negate x in the rest shape with flipped winding so rest matches the streamed orientation.
- Presets: normal 5 substeps and 10 iterations, fast 3 and 6, fine 10 and 20 with self collision.
- Pins are document data (one `Edit::Garment` each); grab is transient, never an undo step. `sim.apply` keeps the draped shape in an in-memory cache, because `Place3D` holds only a rigid transform and a bend; saving the drape needs a pose field (planned).
- The drape cache's placement signature does not yet include `bend`; mesh publishing does.

### Sewing (ffb-pattern sew, sew_cmds)

- `seam_bindings` returns `Pair{a, b}` (to `SeamPair`) or `Edge{a, b0, b1, w}` (to `SeamBind`: a bound to b0 (1 - w) + b1 w), with rest length 0 and stiffness from the seam's tension. Tacks are read by the runner from `garment.tacks`.
- Full-segment seams with ratio 1 force equal vertex counts on both segments (the larger natural count) through `MeshState::set_segment_counts`; deleting the seam releases them.
- Seam spans follow outline edits through a thread-local queue (`sew/pending.rs`: split, reproject, reverse) drained in `sew_cmds::after_change` and folded into the step with `History::amend_last`. Dart, slash, offset and inner-line edits do not re-map spans yet.
- A turned seam locks fold angle 180 and strength 0. Side `auto` is derived from the panel normal and placement; per PATENT_NOTES there is no guideline-plus-symbol relation, and the layer rank is not built.

### Bodies and placement (ffb-body, body_cmds, place_cmds)

- Body space: millimetres, y up, +z front, left is +x. `Joint.rest` is the local offset in the parent frame; positions and normals are the bind mesh; poses use linear blend skinning.
- The Basic Form is formfabric's own parametric body (ellipse sections along a 20-joint skeleton, smooth union, surface nets, about 21k triangles). The library scans `assets/bodies/*/body.glb` (`FF_BODIES_DIR` overrides).
- Bodies stream as object 1_000_000 + id, resent only on a signature change; a hidden or removed body sends empty topology. Undo uses `Edit::Bodies` (light snapshots for pose, show and set, full ones for add, remove and resize).
- Bodies persist in `.ffbrc` as `bodies/[id].json` plus `bodies/[id].bin`.
- Placement follows PATENT_NOTES: volumes come from joint transforms and a fixed radius profile, never from the mesh surface; points are stored as volume, around, along, offset and are never recomputed when a volume changes. `place.panel` anchors at the point the caller chose; there is no arrange-all.
- `place3d.bend` is a signed radius applied in `ffb-mesh/src/layout.rs`: the panel wraps a cylinder about the volume axis.
- Auto Seam proposes seams from notches with one global max-weight matching, optional per-call tags and a small 3D tie-break; it stores nothing until `apply`, and never resizes panels.

### Hardening

- Release builds unwind (`panic = "unwind"`) and `ffb-py` wraps dispatch, `stream_info` and `stream_demo` in `catch_unwind`, returning `internal` errors; `FFB_CRASH_TEST` enables a forced panic for checking this.
- Writes need absolute paths and the right extension (`.ffbrc`, `.dxf`, `.ffab`) and go through a temp file and rename. Zip reads cap entries (4096), per-entry and total sizes; DXF reads and writes cap coordinates to 100,000 mm and vertex counts.
- Names refuse control characters. Particle distance accepts 0.8 to 700 mm; a panel whose mesh would exceed 2 million triangles is refused. Panels carry `pressure`, passed to the cloth material.
- The stream caps 8 unauthenticated and 32 total connections.

## v0.4.0 to v0.6.0 · 2026-11-01

### Shared plumbing

- `hooks::after_change(session, command_id)` runs once after any command that moved the epoch, undo and redo included. It calls the pattern-tools hook (linked panels) then the mesh hook (re-mesh and stream). Derived state never needs its own undo step.
- `History::amend_last` folds the link regeneration into the step just recorded, so a source edit and its mirrored or instanced copies undo together; it respects an open drag group.
- `Session.sink` is an optional `FrameSink`; `ffb_engine` installs one backed by the ffb-stream publisher when the frame server starts.
- serde_json parses with `float_roundtrip`; without it a reopened project could differ in the last bit of a coordinate.

### Pattern tools

- Offsets: each Bezier segment is flattened to 0.02 mm, vertices moved along mitered normals (exact end tangents keep continuity), joins added only at gaps (miter with a limit of 4 times the distance, square, round, mirror), self-crossings cut by dropping wrongly oriented loops, then refitted with Schneider least-squares cubics at 0.03 mm. A collapsing inward offset is `invalid_geometry`.
- Seam allowance is stored as width, corners and per-segment overrides; the outline is derived (`pattern.get` returns `seam_outline`). Per-segment widths follow splits, deletes and outline reversal.
- Notches stay `{segment, t}`. Splitting re-maps by the de Casteljau interval rule (exact); deleting or smoothing re-projects the old world position. Mirror and uneven scale reverse the outline, so notch segment and t and seam widths are re-mapped there too. Notch angle is measured counter-clockwise from the outline tangent, 90 square, pointing into the panel.
- The drawn `grain_line` (panel-local endpoints) is separate from `panel.grain` (the solver's angle); `pattern.get` adds `grain_line_effective`, defaulting to a vertical line through the centre.
- Derived panels (mirror, instance): regenerated from the source on every change; a direct edit of a derived panel is snapped back in the same step. Mirror `axis` stores the gap-shifted mirror line; instance `axis` stores `[interval, [ordinal, 0]]`.

### Exchange

- formfabric's own DXF R12 reader and writer. AAMA (ASTM D6673) layer use: 1 cut, 2 turn points, 3 curve points (fitted back to cubics), 4 notches, 7 grain, 8 internal lines, 14 sew line (its distance from the cut line gives the seam allowance width), 15 annotation, 80 and 83 for T and U notches. AAMA export writes panel-local coordinates; plain export writes world coordinates.
- Layer 8 lines carry a `FF:role=..;kind=..;angle=..;strength=..` label so fold role and angles survive a round trip; foreign files import those lines as role reference.
- Plain DXF facets: a vertex between two edges under 40 mm turning under 15 degrees becomes a curve point (`smooth_facets`), so curves flattened elsewhere do not import as dozens of corners.
- Graded nests import one size (the sample size); grade rules are not read or written.

### Mesh

- Outline segments are flattened adaptively and resampled to `round(length / d)` points, or a forced count per segment (`MeshState::set_segment_counts`, `mesh.build segment_counts`): the seam API for Phase 7. Fold and dart inner lines are constraint chains; shape and reference lines are not meshed.
- Interior: a triangular lattice clipped 0.65 d from edges, constrained Delaunay with spade (`try_bulk_load_cdt`) plus four frame points, refinement to a 20 degree minimum angle with outline vertex counts preserved. Inside faces come from formfabric's own flood fill, because spade treats closed inner loops as holes and internal darts must stay filled.
- Cache per panel: FNV-1a of outline, fold and dart lines, particle distance, options and forced counts. Without a sink, meshing happens on demand only.
- Streaming: object id is the panel id. Topology is re-sent only when its signature changes; positions follow topology. 3D uses `place3d` (origin plus quaternion on (x, y, 0)) or the flat layout; `bend` and mirror are not applied in 3D yet.

### Fabric

- Stored in SI: stretch and shear N/m, bending micro newton metres per metre width, density g/m2, thickness mm, ratios 0 to 1. Validation lives in `validate.rs` so nested values report paths such as `physical.density`.
- Presets come from encyclopedia chapter 16 with source keys; provisional ones are flagged. The default preset is cotton poplin shirting; `file.new` creates the default fabric.
- Every fabric and colorway change records one `Edit::Garment` snapshot. Colorways override `look.front.color` per fabric at render time.
- `.ffab` is a zip: `fabric.json` (`format formfabric-fabric`, schema 1) plus `maps/` images.

## v0.2.0 to v0.3.0 · 2026-10-31

### Workspace

- `members = ["crates/*"]`: a new crate joins without editing `Cargo.toml`. `rust-version` 1.88; the sandbox builds on 1.97, CI pins 1.99.1 through `rust-toolchain.toml`.
- Release profile: one codegen unit, thin LTO, `panic = "abort"`.

### ffb-core

- `bvh.rs` is formsculpt's `fsc-core` BVH with hidden-face handling removed. `mesh.rs` keeps positions, normals, UVs and indices only. `units.rs` (mm, cm, in) and `ids.rs` are new. Geometry is always millimetres; units are a display choice.

### ffb-commands

- One registry in the shape of formsculpt's `fsc-commands`, assembled per module: each `*_cmds.rs` returns its commands and `lib.rs` concatenates them in a `LazyLock`. Dispatch accepts the dotted id or the MCP name (id with dots replaced by underscores).
- `args.rs` validates every call against the command's own `inputSchema` (required, type, enum, `additionalProperties: false`) after coercing quoted scalars, so every command gets `bad_args` for free. Error codes: `unknown_command`, `bad_args`, `not_found`, `io`, `open_failed`, `save_failed`, `nothing_to_undo`, `nothing_to_redo`, `history_conflict`, `invalid_geometry`.
- History: typed `Edit` (Units, Garment, Panel, Many). `begin_group` and `end_group` make a drag one step; while a group is open, consecutive `Panel` edits on the same panel merge (first before, last after), so a 200-move drag is one edit. Undo and redo close an open group. Camera-per-step from formsculpt is not carried; the view is not in the document.
- `Session` holds the garment, history, path, unknown zip entries and an `epoch` that bumps on every change for the service's `engine.changed` event.
- JSON keys in the catalogue print alphabetically (serde_json without `preserve_order`).

### ffb-pattern and the pattern commands

- Segment i runs from point i to point i+1, closed outlines wrap. A segment is cubic when either end has a handle, else a line. Handles are offsets from their point.
- Point kinds: `corner` keeps handles as given; `curve` recomputes handles from the neighbours (chord-scaled tangent) after moves, deletes and kind changes; `smooth` keeps user handles collinear. `add_point` and `split_line` never refresh handles, so the shape is unchanged exactly (de Casteljau).
- `curvature(amount)` builds a symmetric cubic whose midpoint sits `amount` mm off the chord (handle offset `amount / 0.75`); positive is to the right of the segment direction, outward on a counter-clockwise outline.
- `smooth` is an exact circular fillet (handle `4/3 tan(sweep/4) R`).
- Placement: world = position + rotate(rotation, mirrored local point). Outlines are stored local and counter-clockwise; `draw_panel` reverses clockwise input and puts the position at the lower left of the bounds. Every point command takes `space: world | local`, world by default.
- `transform` applies mirror, scale, rotate, translate about a pivot (default the centre of the selection's world bounds). Uniform scale, rotation and translation change placement; mirror and uneven scale are baked into the outline.
- Panels and inner shapes share one id space. History addresses panels by index, commands by id. Multi-panel commands build one `Edit::Many` themselves so they work inside an open drag group. A command that changes nothing records no step and returns `changed: false`.
- `select` is a pure hit test; selection is a UI concern. Area uses a 0.02 mm flatten tolerance.
- Keys: A transform, Z edit, Delete, Mod+D duplicate, F2 rename. F belongs to the view.
- Later phases: offset outline, mirror and linked edits, unfold, dart, slash and spread, notch, grain line editing.

### ffb-io

- `.ffbrc` is a zip: `manifest.json` (`format`, `schema` 1, `app_version`) and `garment.json`. Entries it does not know are read into `Session.unknown` and written back on save. A schema above 1 is refused.

### ffb-py

- Module `ffb_engine` (abi3, Python 3.12 and later): `version()`, `catalogue()`, `Engine().dispatch(name, args_json)`, `stream_info()` (starts the frame server lazily, returns port and token), `stream_demo(n)` (a 32 by 32 test grid waving at about 30 fps; Phase 7 replaces it with the solver).
- Built by `./formfabric engine build` with maturin into `backend/venv`.

### ffb-wasm

- Plain C ABI, no wasm-bindgen, no threads, no imports from the page: `ffb_alloc`, `ffb_free`, `ffb_dispatch` and `ffb_load` return a pointer to a little-endian u32 length followed by JSON; free it with `ffb_free(ptr, 4 + length)`. `ffb_load` accepts a bare `pattern.get` result or the reply envelope.

### ffb-stream

- Frame v1, little endian, 44-byte header: `FFBS`, version u16, kind u16 (1 topology, 2 positions), frame u32, object u32, vertex count u32, origin 3 f32, scale 3 f32. Positions are 3 u16 per vertex, quantised per frame to the bounding box (worst error extent / 131070 per axis). Topology is a u32 triangle count, the indices, then one u16 panel id per triangle. Decoders reject anything malformed.
- Server: 127.0.0.1 on an OS port, a 48-hex-character token from `getrandom` checked in constant time (query `?token=` or the first text message within 5 s), non-loopback peers dropped. Each client has a slot with pending topology in order and only the newest positions per object, so a slow client never queues stale frames. Late joiners get the retained topology and newest positions first.
- One targeted clippy allow (`result_large_err`) on the handshake callback, where the error type is tungstenite's.

### v0.10.0 to v0.12.0

- Body import: files capped at 1 GB, non-regular files refused, vertices 5 million, triangles 10 million, joints 1024. glTF external buffers must be plain relative paths that canonicalize inside the model folder; data URIs capped at 256 MB, buffers at 512 MB. `Body.source` is a file name only.
- ffb-gpu: `GpuSolver` runs the VBD substep in WGSL over a 7-colour graph colouring with sphere, plane and mesh colliders. Kept out of ffb-wasm; the runner reaches it through a `gpu` cargo feature. float32 and lagged hinges make it drift from the CPU path after first contact (24 mm mean at 1 s on the sphere drape); no self collision yet.
- ffb-sim `external.rs`: `Solver::set_external_forces` feeds per-vertex forces (wind) into `ext` in `begin_substep`. Both the VBD and implicit paths read it.
- Panel pose: `Place3D.pose` holds the simulated vertex positions written by `sim.apply` (one undo step). Assembly starts from `pose` when its vertex count still matches the mesh, else from the placement; `bend` is part of the drape cache signature.
- Runner: frozen panels become pins, disabled panels and their seams are dropped, stiffen scales stretch and bending, hold shape adds a weak spring to the current shape, shrink feeds the material. Ripped seams are not bound. Bodies hidden with `body.show` do not collide.
- Runner limits: `sim.step` and `sim.compare` refuse above 60k vertices and stop at a wall-time budget, reporting frames done; the core lock is released between frames; state reads never wait on a step. `time_step` is clamped to 1/240 to 1/15 s. Pins are deduplicated per vertex and capped at 10,000; pin edits record `Edit::Pins` (pins only), and a box or lasso gesture is one undo step.
- Refit (PATENT_NOTES): blend for the same topology, a measurement sweep for different topology, or redrape. The body morphs while solving; no correspondence map and no reference garment.
- Trims: `ffb_pattern::trim` holds styles, hardware, zippers, stitches, finishes and prints. Sew edits re-map stitch, zipper and finish spans in the same undo step (`sew_cmds/sync.rs` clones the drained ops to `trim_cmds::follow`). A zipper's `Binding.seam` is the zipper id. Elastic shortens the strip through `shrink_weft`, not a seam ratio. Piping and elastic strips use a split edge so the host mesh is not refined. `trim.import` stores the OBJ mesh in the style params. Crease is a fold line and shirring is an alias of elastic. `stitch.set` and `finish.set` edit after creation; a strip finish regenerates its panel and seam, so those ids change.
- Implicit solver: `Method::Vbd | Implicit`, default Implicit. Gauss-Newton blocks with exact gradients in a block-CSR matrix, block-Jacobi PCG with pin and contact filtering. A diagonal-modified incomplete Cholesky preconditioner was tried and rejected (fewer iterations, slower overall, diverged at first contact).
- Presets fast, normal, quality, final (the engine still accepts `fine`). Implicit substeps 2, 3, 4, 6; block descent 3x6, 5x10, 8x15, 12x25; self contact on for quality (1.0 mm) and final (0.8 mm). `Params::default()` is implicit normal.
- Self contact: `contact.rs` (vertex-triangle and edge-edge over `selfcol::SpatialHash`), `self_intersections()`, `untangle()`. Layer rank comes from `Panel.layer` through `Cloth.vertex_layer`; pairs of different rank get the contact thickness times min(rank difference, 3) on the higher layer's side, in contact only, with no seam link or pair data (PATENT_NOTES 2.1). It runs only with self contact on.
- Registry keys: Shift+B, H, F, Z, S, E, I, A trim tools; Shift+C pinch; X box pins; L lasso pins; D fold place; K body tape.

### v0.13.0 to v0.15.0

- Data model: `Garment.anim`, `output`, `viz` and `style_lines` (`ffb_pattern::{anim, output, viz}`). Hook order: pattern tools, sew, body, place, trim, edit3d, mesh, sim, anim, output, remesh, viz. Large data never enters `Garment` or undo snapshots.
- Collision: body contact used to search only within the skin offset, so vertices that went deeper were never resolved (12 to 30 mm inside). `collide.rs` now reaches `DEEP_REACH` (40 mm), and `end_substep` clamps each vertex's approach toward a collider (half the skin offset within max(8 skin offsets, 20 mm)) for both solvers. The closing phase ends on one continuous ramp.
- Placement: the `Wrapper` in `ffb-body/src/place.rs` gains `PanelSpan` and `fit_span`, a shoulder-ridge pass (arc-length blend of the top 200 mm band), per-row ease of the radial offset, and sleeve starts along the arm axis. `mesh.get` returns the analytic bend, not the pose the solver starts from.
- Implicit solver: `coarse.rs` adds a two-level preconditioner, used when under 1/12 of vertices are constrained; the first Newton step solves with a 5 times looser tolerance.
- ffb-gpu: `AnyStepper` over the CPU and GPU solvers; GPU spatial hash and vertex-triangle self contact. Sphere drape parity needs the CPU's frozen-gradient scheme ported.
- Animation: `Session.anim`. Caches are `cache/<id>.bin` entries (magic FFAC, JSON header with per-frame stats, f32 or 16-bit quantized positions, body joint quaternions) and converted motions `motion/<id>.bin` (FFMC); orphans move to a six-entry trash so undo keeps their data. `anim.record` pins `time_step` to 1/(fps times steps), skips the closing phase, replaces mesh colliders every sub-frame and marks the cloth stale afterwards. `job.state` stores a finished recording as a cache layer (one undo step). Retarget is a world-space delta transfer with per-bone alignment from rest directions; root motion defaults to in place.
- Export: `ffb-io/src/export` writes OBJ and MTL, glTF and GLB (metres, Y up always), FBX 7.4 ASCII and binary, usda, MDD and PC2; Alembic is deferred. UV islands use shelf packing with UDIM tiles; thickness is an extrusion at export; bake writes normal, AO and displacement maps; the `quads` option writes the quad remesh to OBJ, FBX and USD. New dependency `png` 0.17.
- Remesh: `ffb-remesh` prescribes boundary samples so both sides of a seam match, then fills from 4-RoSy orientation and position fields, pairs a border band into quads and cleans up; `decimate` (quadric) and `lift` are public; `Session.remesh` and `quad_meshes()` feed exporters. The QuadriFlow min-cost-flow step is not implemented.
- Visualization: ffb-stream frame kind 3 carries per-vertex scalars (u16 over the header min and max, 65535 for no data, map id and component in the scale fields); a follower thread re-streams maps while the sim runs. Measures live under `Garment.viz.compare["measures"]`; Style Line and Fit Shell settings under `Garment.selection["edit3d2"]`, both until they get real fields. Schematic export has its own SVG and PDF writers.

### v0.16.0 to v1.0.0

- Solver threads: ffb-sim's `rayon` feature is on by default for non-wasm targets (a target-gated optional dependency, absent from wasm32). Block descent's state evaluation, colour sweeps and end-of-substep collision, and the implicit assembly, preconditioner build and SpMV run in parallel; `RAYON_NUM_THREADS` limits them. Vertices farther than offset plus edge plus 2 min(edge, 10) mm from every collider skip collider queries. `Solver::vbd_profile()` gives phase timers.
- Self contact (`contact.rs`):
  - The push is limited to 1,200 mm/s per substep and never enters the velocity.
  - Three rings around each vertex are excluded.
  - `guard_self_contact`, called from `end_substep`, rolls back moves that create a new intersection.
- Components (PATENT_NOTES 2.6, never "block"): `ffb-pattern/src/component.rs` has Component, Port, PortRef and Join; `Garment.components` and `Garment.joins`. Joins become real seams, and checks compare port names and lengths only. A port whose segment is split is marked invalid, not remapped.
- New command modules:
  - `snapshot_cmds`: snapshots are `snapshot/<id>.json` entries with optional PNG thumbnails.
  - `library_cmds`: a user library of new `.ffstyle`, `.ffpose`, `.ffmotion` and `.ffcomp` JSON formats with an index, read only inside configured roots.
  - `component_cmds`, plus `b64.rs`.
  - `link_cmds`: Live Link writes a usda by atomic replace when the sim frame advances by `every_frames` or the document changes.
