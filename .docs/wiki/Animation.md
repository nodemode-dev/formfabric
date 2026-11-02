# Animation

An animation is a timeline of body motion and key tracks, recorded into a cache layer of cloth positions that you scrub, play and export.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-anim/src/` | BVH, glTF and FBX readers, retargeting, interpolation, clips, the cache format |
| `native/crates/ffb-commands/src/anim_cmds/` | timeline, motions, keys, recording, playback |
| `frontend/src/workspace/anim/` | the timeline dock, motion panel, key widgets |
| `docs/encyclopedia/08-animation.md` | the behaviour specification |

## Motions

`motion_import` reads a `.bvh`, `.glb` or `.gltf` animation (linear, step and cubic spline channels) or, on desktop builds, an `.fbx`, and retargets it onto a body by joint-name mapping with rest-pose alignment (A pose or T pose source). Options: `root` (`keep`, `inplace`, `off`), `scale`, `loop`, `align`, `lead_in` (frames blending from the current pose to the first motion pose), `start`, `fps`, `units`, `up`. It returns which joints mapped. `motion_set` changes them later; `motion_remove` deletes. BVH and glTF motions are exercised; FBX motion has not been tried in the app.

## Keys, morphs and the timeline

- `anim_get` reads frame rate, play range, current frame, motions, tracks and cache layers; `anim_set` sets frame rate, range or current frame.
- `key_set` and `key_remove` set keys on tracks: `body.ID.pose`, `body.ID.MEASUREMENT`, `wind.FIELD`, `panel.ID.state.FIELD`, `fabric.ID.FIELD`, with `step`, `linear` or `smooth` interpolation. Recording applies the body, measurement and wind tracks every frame.
- `morph_set` morphs one body into another made from the same mesh over a number of frames; the collider moves through the blend like a motion.

## Recording and playback

1. Drape the garment first (see [[Draping-and-the-Solver]]); recording continues from the current draped state unless `reset` is true.
2. `anim_record` runs on a background thread: for each frame it poses the bodies, moves the colliders, applies wind tracks and steps the cloth (`steps` solver frames per timeline frame, default 2). With `validate` it measures body penetration, self intersections and vertex displacement each frame. `storage` is `f32` or `quantized`.
3. `job_state` reports phase, frame and warnings; when `ready` is true `job_harvest` stores the result as a cache layer (one undo step). `anim_stop` keeps the frames recorded so far.
4. `cache_set` renames, shows, hides or activates a cache; `cache_remove` deletes one; `cache_frame` streams one frame; `anim_play` and `anim_pause` play and pause. Caches are saved in the project.

## Known limits

The walk (`tshirt`, 120 frames) keeps the T-shirt out of the body, but cloth crossing itself was measured in most frames at v0.15.0, and the skirt moved up to 72 mm between frames; the recorded validation numbers tell you this per cache, so read them before exporting. Cache Layer cut, merge, move, lock and override, auto key and keyed pins are not built.

## Contract for new animation work

Recording must stay cancellable and report per-frame validation. A new track target must be added to the key target list in the command description and to the replay path used by recording.
