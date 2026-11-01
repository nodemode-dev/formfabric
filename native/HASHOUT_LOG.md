# HASHOUT_LOG · native

Living context for the Rust engine. What would be comments in the crates lives here, newest first. Read before editing anything under `native/`.

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
