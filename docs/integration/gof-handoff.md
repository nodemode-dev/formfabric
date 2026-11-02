# GoF hand-off: a proposal for the formsculpt owner

Status: PROPOSAL. No format change was made on either side. Nothing in formsculpt or formsurface was edited. The contract change below needs the formsculpt owner's agreement first (Phase 17 rule: never a unilateral change to `gof.json`). Written from reading formsculpt's `gof_cmds.rs`, `routers/gof.py`, its CLAUDE.md note on GoF and its three receivers (Blender add-on, Unreal Python, Unity editor script). No code is copied from formsculpt into formfabric.

## What GoF is today (as read)

One folder (`FORMSCULPT_GOF`, default `Formsculpt/GoF` in the home folder) and one manifest `gof.json` in it:

- top level: `gof` (1), `app` (the writer), `version`, `up_axis` ("y"), `mm_per_unit` (the sender's scene size, may be null), `objects`;
- each object entry: `id` (integer), `name`, `file`, `format` (glb, fbx, obj or package), `target` (blender, unreal, unity, formsurface, substance, any), `revision`, `vertices`, `triangles`, `sent` (seconds), `returns` (true when the file is a bridge that comes back), `units` (m, cm or scene) and `scale` (scene units to file units).

Receivers read the manifest, keep entries whose `target` is their own name or `any` and whose `format` they can import, and import entries whose `sent` is newer than what they last saw. They ignore keys they do not know. The FormSurface hop (`routers/gof.py`) copies a GLB named by the caller from the folder into FormSurface's meshes folder and loads it when its Agent link is on. A file marked `returns` is a bridge: when another app saves over it, formsculpt's `bridge_sync` reads the edit back into the object with the same `id`.

## What formfabric can do with no format change

formfabric can already act as a plain sender, because everything a receiver needs is in the existing fields. `formfabric.gof.send(name, target, folder)` (in `python/formfabric/gof.py`) does this and nothing more:

1. exports the draped garment with `export.run` as GLB (Blender, Unreal, FormSurface, any) or FBX (Unity) into the GoF folder, in metres for GLB and centimetres for FBX, y up, with UVs and materials;
2. reads `gof.json` if it exists, replaces only the entry whose `file` is the same, keeps every other entry as it was, and writes the file back by atomic replace;
3. writes the entry with the existing fields, `returns` false (formfabric has no bridge back), `units` and `scale` set for millimetres to the file unit, and an `id` of 2,000,000 or more so it cannot collide with a formsculpt object id.

It adds one key to its own entries, `app` ("formfabric"), which receivers ignore. The top level keeps the existing `app`, `version` and `mm_per_unit` when the file exists and is created with `app` "formfabric" otherwise.

Known risks of this no-change path, to be checked by the formsculpt owner:

- formsculpt's `gof_send` rewrites the top level with `app` "formsculpt" on its next send. That is harmless to receivers but loses the sender name at the top level; the per-entry `app` survives because formsculpt keeps entries it did not write.
- A `returns` false entry should never be touched by `bridge_sync`, but that was read, not run. If `bridge_sync` walks every entry by `id`, an edit saved over a formfabric file could be applied to the wrong formsculpt object; the high `id` range makes a collision unlikely, not impossible.
- Unity needs FBX; formfabric writes ASCII or binary FBX 7.4, which was not run in Unity here.
- None of the receivers was run against a formfabric bundle.

## Proposed contract additions (need agreement)

These are additive and optional so every current receiver keeps working.

1. Per-entry `app`: the name of the app that wrote the entry ("formsculpt", "formfabric"). Receivers keep ignoring it; `bridge_sync` and the window's list act only on entries whose `app` is "formsculpt" or absent. This closes the id collision risk above.
2. `kind`: what the object is, `mesh` (default) or `garment`. A garment entry says the mesh is a draped garment on a body, so formsculpt can offer "Use as sculpt reference" instead of "Edit".
3. Garment extras, only when `kind` is `garment`: `body` (the avatar's file name in the same folder, when formfabric also sent it), `panels` (panel names in file order, one object per panel) and `fabric` (the fabric name per panel). All informational.
4. A return path for avatars: formsculpt sends a body, formfabric imports it with `body.import` (GLB), drapes on it and sends the garment back. Today formfabric can read GLB, FBX, OBJ bodies by `body.import`; the manifest would list the body entry with `target` "formfabric", so the only agreed change is the new target name `formfabric` in the enum. The matching formfabric receiver (`gof.receive`) would be written after the target name is agreed.
5. `target` gains `formfabric`; `format` stays glb, fbx, obj.
6. Units: formfabric keeps writing `units` and `scale` as formsculpt defines them; no change.

## Order of work if accepted

1. Formsculpt owner confirms additions 1, 2 and 5 (smallest set that removes the collision risk and names the new target).
2. Formsculpt adds `app` to the entries it writes and filters `bridge_sync` on it; its three receivers need no change.
3. formfabric adds `app` and `kind` to its entries (one line in `gof.py`), then the garment extras if wanted.
4. Formsurface: no change; it reads GLBs that formsculpt's router copies. A garment sent for FormSurface needs UVs, which formfabric writes by default.
5. Both sides run the other's file once against the real apps (Blender, Unreal, Unity) and record the result.

## What was not done

- No change to `gof.json` handling in formsculpt or formsurface.
- No `gof.receive` in formfabric (it depends on the new target name).
- No test against the receivers; only the manifest shape was checked, by `tools/family/check_family.py`, which reads back what `gof.send` wrote.
