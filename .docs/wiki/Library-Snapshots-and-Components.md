# Library, snapshots and components

Three workspace features built in Phase 16, all in one drawer: Window, Library (tabs Library, Snapshots, Components).

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/library_cmds.rs` (+ `library_cmds/store.rs`) | the user library on disk |
| `native/crates/ffb-commands/src/snapshot_cmds.rs` | whole-garment snapshots inside the project |
| `native/crates/ffb-commands/src/component_cmds.rs` (+ `component_cmds/`) | Components and the sample |
| `native/crates/ffb-pattern/src/component.rs` | the Component, Port and Join model |
| `frontend/src/workspace/{library,snapshots,components}/` | the drawer tabs |

## Library

Items are files in the per-kind formats: fabrics `.ffab`, trims and styles `.ffstyle`, components `.ffcomp`, garments `.ffbrc`, poses `.ffpose`, motions `.ffmotion`, with a small `index.json` and thumbnails under `<root>/thumbs/`. The default folder is `[data dir]/library`; `library_roots` and `library_set_roots` (up to 8 absolute folders, the first receives new items) change it, and the list persists in `[data dir]/library_roots.json`.

- `library_list` takes `kind` (fabric, trim, style, component, garment, pose, motion) and `query` (matches name or tags). Ids look like `fabric/denim`.
- `library_add` saves a selected item (fabric id, style id, component id, a body for a pose, a motion id, or the open project for a garment) with `tags` and a thumbnail.
- `library_insert` brings an item into the project as one undo step. A garment is not inserted; open it with `file_open`.
- `library_remove` deletes the file, thumbnail and index entry from disk and cannot be undone. Reads resolve inside the configured roots only.

In the window: kind select, search, tag filter, a virtualized thumbnail grid, double-click or Enter to insert, drag a tile onto the Pattern or 3D pane, Add selected, Remove, Roots.

## Snapshots

`snapshot_save` stores the whole garment state inside the project (entries `snapshot/<id>.json` and an optional PNG thumbnail, at most 1 MiB); at most 64 per project. `snapshot_list`, `snapshot_restore` (one undo step labelled Restore, bodies and history are kept) and `snapshot_remove`. The window takes a 240 by 180 thumbnail of the 3D View and offers Compare, which reuses the compare maps.

## Components

A Component groups panels you choose, with named Ports on their edges. It is a grouping of real panels and real seams, not a template: there is no registry of shapes, no inference from panel names, and no depth-level editing mode.

| Command | Does |
| --- | --- |
| `component_create` | `panels` (ids) and `ports` (name, spans, tolerance in mm, default 5) |
| `component_list` | panels, ports with live length against stored length, validity, joins |
| `component_save` | writes a `.ffcomp` into the library: panels, internal seams, ports, nothing else |
| `component_insert` | inserts a library component beside the garment, with optional joins |
| `component_swap` | replaces a component and rebuilds joins by port name; refused with the numbers when a port is missing or a length is outside tolerance, unless `ease` sews with a seam ratio |
| `component_check` | compares by port names and lengths only |
| `component_join` | joins a port of one component to a port of another as ordinary seams |
| `component_remove` | removes the grouping and joins; panels stay unless `delete_panels` |
| `component_sample` | `tshirt` groups the drafted T-shirt (Torso, Sleeve L, Sleeve R); `sleeveless` adds a binding component to swap in |

In the window, create from the selected panels, use Pick ports to click outline segments in the Pattern View, then Add port, Check, Swap, Join and Save to library.

## Known limits

The sleeveless binding is two straight strips with no 3D placement, so place them before a drape. An edit that splits a port's segment flags the port `valid: false` rather than remapping it. Journal replay (see [[Recovery]]) needs the files that `file_open` and `library_insert` named to still exist.

## Contract for new library kinds

Add the file extension, the add and insert paths, and the thumbnail rule together; reads must stay inside the roots; every insert is one undo step.
