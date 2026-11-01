# HASHOUT_LOG · native

Living context for the Rust engine. What would be comments in the crates lives here, newest first. Read before editing anything under `native/`.

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
