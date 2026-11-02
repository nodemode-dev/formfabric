# Animation

An animation is a timeline of body motion and key tracks, recorded into a cache layer of cloth positions that you scrub, play and export.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-anim/src/` | BVH, glTF and FBX readers, retargeting, interpolation, clips, the cache format |
| `native/crates/ffb-commands/src/anim_cmds/` | timeline, motions, keys, recording, playback |
| `frontend/src/workspace/anim/` | the timeline dock, motion panel, key widgets |

## Motions

`motion_import` reads a `.bvh`, `.glb` or `.gltf` animation (linear, step and cubic spline channels) or, on desktop builds, an `.fbx`, and retargets it onto a body by joint-name mapping with rest-pose alignment (A pose or T pose source). Options: `root` (`keep`, `inplace`, `off`), `scale`, `loop`, `align`, `lead_in` (frames blending from the current pose to the first motion pose), `start`, `fps`, `units`, `up`. It returns which joints mapped. `motion_set` changes them later; `motion_remove` deletes. BVH and glTF motions are exercised; FBX motion has not been tried in the app.

## Keys, morphs and the timeline

- `anim_get` reads frame rate, play range, current frame, motions, tracks and cache layers; `anim_set` sets frame rate, range or current frame.
- `key_set` and `key_remove` set keys on tracks: `body.ID.pose`, `body.ID.MEASUREMENT`, `wind.FIELD`, `panel.ID.state.FIELD`, `fabric.ID.FIELD`, `pin:ID`, with `step`, `linear` or `smooth` interpolation. A pin track takes an `[x, y, z]` position in millimetres through `target: "pin:ID"`; recording applies the body, measurement, wind and pin tracks every frame, pins before each solver step. Pin keys do not move the cloth in live `sim_step` runs.
- `anim_auto_key` with `on` true keys every change you make to wind, panel state, fabric physical values or a pin target at the current frame, amending the same undo step. It skips body pose and measurement values, which live outside the garment document. The timeline has an Auto Key toggle.
- `morph_set` morphs one body into another made from the same mesh over a number of frames; the collider moves through the blend like a motion.

## Recording and playback

1. Drape the garment first (see [[Draping-and-the-Solver]]); recording continues from the current draped state unless `reset` is true.
2. `anim_record` runs on a background thread: for each frame it poses the bodies, moves the colliders, applies wind tracks and steps the cloth (`steps` solver frames per timeline frame, default 2). With `validate` it measures body penetration, self intersections and vertex displacement each frame. `storage` is `f32` or `quantized`.
3. `job_state` reports phase, frame and warnings; when `ready` is true `job_harvest` stores the result as a cache layer (one undo step). `anim_stop` keeps the frames recorded so far.
4. `cache_set` renames, shows, hides or activates a cache; `cache_remove` deletes one; `cache_frame` streams one frame; `anim_play` and `anim_pause` play and pause. Caches are saved in the project.

## Cache layers

A cache layer has a `kind` (`pattern` or `trim`), a start frame, an `offset` and a `locked` flag. A recording that has hardware bindings (buttons, snaps, trim items) also stores a `trim` layer with one point per hardware item; trim layers are data, so playback does not stream them and `cache_frame` returns their points as `trims`.

| Command | Does |
| --- | --- |
| `cache_cut` | `id`, `frame`: splits a layer into two layers at that frame |
| `cache_merge` | `ids`: merges layers into the top selected one; gaps between them are filled by linear interpolation of panel positions and body offsets |
| `cache_move` | `id`, `frames`: moves a layer in time |
| `cache_lock` | `id`, `locked`: a locked layer refuses move, cut, remove, rename, merge and re-record |
| `anim_record` with `layer` | records again into an existing layer |

`cache_frame` and `anim_play` without an `id` resolve the visible pattern layers top down (the last layer in the list is the top, and it wins at a frame it covers); with an `id` they use that layer alone. Merge needs the same panels and bodies in every layer. The timeline offers cut, merge (shift or ctrl click to select several), lock and move by a number of frames in a fixed-height row.

## Known limits

The walk (`tshirt`, 120 frames) keeps the T-shirt out of the body, but cloth crossing itself was measured in most frames at v0.15.0, and the skirt moved up to 72 mm between frames; the recorded validation numbers tell you this per cache, so read them before exporting. Not built: layer reorder by drag, per-layer override blending, tack keys, body pose and measurement auto key, and strip, zipper and stitch trim caches. Cut and merge of a quantized cache re-quantize it and were not measured.

## Contract for new animation work

Recording must stay cancellable and report per-frame validation. A new track target must be added to the key target list in the command description and to the replay path used by recording.
