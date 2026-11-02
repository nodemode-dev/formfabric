# Live Link to Unreal Engine through USD

Status: the formfabric side is built and checked here. The Unreal side is NOT tested: no Unreal Engine was available in the build sandbox, so every Unreal menu name and behaviour below comes from Unreal's public documentation and may differ in your engine version. Treat the Unreal steps as a recipe to try, and report what differs.

## What formfabric does

`link.start` writes the current draped garment as a USD text layer (`usda`) into a folder you choose, then replaces that file atomically every N simulation frames while the simulation runs. Atomic means the layer is written to a hidden temporary file in the same folder, flushed, and renamed over the old one, so a reader never sees a half-written layer.

| Command | What it does |
| --- | --- |
| `link.start` | `folder` (absolute path, created if missing), `every_frames` (default 1), `format` (`usd`), optional `file` (name without extension, default `garment`), `thickness` (default false), `axis` (`y_up` default, or `z_up`). Writes the first layer at once and starts following the simulation. Starting again replaces the running link. |
| `link.stop` | Stops the link. The last layer stays in the folder. |
| `link.state` | Reports `running`, `folder`, `path`, `every_frames`, `writes`, `last_frame`, `last_bytes`, `last_error` and `stopped` (the reason, when it stopped by itself). |

The layer is rewritten when:

- the simulation frame has advanced by `every_frames` since the last write;
- the garment was edited (a new draft, a moved point, a changed fabric), at once;
- the simulation went back to an earlier frame (reset);
- the simulation stopped moving for a quarter of a second after a frame that was not written (so the last frame of a run is never missing).

The link does not touch the document and is not an undo step. Any file command (`file.new`, `file.open`) stops it, because the garment it follows is gone; `link.state` then says why. The layer is exported with the same exporter as `export.run` with format `usd`: one mesh per panel, UVs from the island layout, fabric materials as UsdPreviewSurface, `metersPerUnit` 0.001 (the geometry is in millimetres) and the chosen up axis.

Start it from anywhere the commands are reachable:

- an MCP client: call `link_start` with `folder` and `every_frames`;
- the Python API: `formfabric.commands.link_start(folder="/path/to/folder", every_frames=2)` or `Garment.link(folder, every_frames)`;
- the HTTP service: `POST /api/engine/dispatch` with `{"name": "link.start", "args": {...}}` and the window token header (see `python/README.md`).

Run the simulation with `sim.start` (it keeps stepping in the background) or step it by hand with `sim.step`; the link follows either.

## Pointing Unreal at it (untested)

1. In Unreal, enable the USD plugins (Edit, Plugins: USD Importer, and the USD Stage editor module that comes with it) and restart the editor.
2. Open the USD Stage window (Window, Virtual Production, USD Stage in the Unreal 5 versions this was written from).
3. Open the file `[folder]/garment.usda` from the USD Stage window (File, Open). Unreal creates a USD Stage Actor in the level that loads the layer and builds the meshes and materials.
4. Set the garment's axis. The layer declares its own `upAxis` and `metersPerUnit`, which Unreal reads when it opens the stage. If the garment appears rotated or tiny, re-run `link.start` with `axis` set to `z_up` and reopen the stage.
5. Reload on change. Unreal's USD Stage reloads layers when asked and, in versions that watch layers on disk, when the file changes. If your version does not reload by itself, use the stage's Reload action (the layer's context menu in the USD Stage window, or the Stage Actor's reload function) after formfabric writes; an editor Python loop that reloads on a timer is the fallback. To check, run `link.state` and watch `writes` count up while the simulation runs.
6. Because every write is a whole new layer, Unreal re-reads the mesh each time. Use `every_frames` of 2 to 4 for heavy garments, and stop the link (`link.stop`) before opening other projects in formfabric.

## What to report back

When this is tried in Unreal, note: the engine version, whether the stage reloaded on its own, how long a reload took for a draped tshirt, and whether materials and UVs came through. Update this page then and drop the untested notice.

## Limits

- The layer is the current mesh, not a time-sampled animation; for a recorded animation use `export.run` with `format` `usd` and `animation` true.
- Text USD grows with vertex count (about 80 bytes per vertex); a 5,000 vertex garment writes about half a megabyte per frame, so keep `every_frames` above 1 for large meshes on slow disks.
- Thickness (`thickness` true) writes the back face and side strips set by `thickness.set`; it doubles the vertices.
- The link needs the native engine (not the in-page browser engine).
