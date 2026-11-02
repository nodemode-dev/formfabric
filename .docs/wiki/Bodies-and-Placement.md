# Bodies and placement

A body is a collider for the cloth: a mesh with a skeleton and skin weights, a collision skin offset and friction. Placement puts flat panels around it before the drape.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-body/src/` | bodies, import, measurements, resize, poses, skinning, placement volumes and points |
| `native/crates/ffb-commands/src/body_cmds/` | body commands |
| `native/crates/ffb-commands/src/place_cmds/` | placement and Place Panel |
| `assets/bodies/` | the two CC0 bodies (licence text beside each) |
| `frontend/src/workspace/body/` | the Body drawer, pose, placement and collision editors |

## Bodies

- `body_load_default` adds formfabric's own Basic Form (about 20k triangles, 20 joints) or a library body: MPFB2 Female or MPFB2 Male, derived from the CC0 MakeHuman community assets.
- `body_import` reads OBJ, glTF, GLB (skins and skeletons included) or FBX (native builds only) and stands the body on y = 0. Give `units` and `up` when the file is not metres and y up. Alembic body import is deferred.
- `body_list`, `body_show`, `body_remove`, and `body_set` (collision skin offset 0 to 100 mm, default 3, friction, name).
- `body_resize` deforms to height, inseam, chest or bust, waist, hip, shoulder width and arm length in millimetres; targets are met within 1 mm. `body_measure` reads the same measurements back.
- `body_pose` poses by linear blend skinning: a named pose (rest, A, T) or joint rotations; `body_joints` lists the skeleton.
- `fitshell_set` replaces the body as the collider with an offset shell for a fitting ease, see [[3D-Editing]].

## Placement

- `place_defaults` builds the default placement volumes (torso, neck, arms, thighs, shins) and points (front, back, left, right, skirt, neck, sleeves, cuffs, trouser legs) from the joints.
- `place_volume_set` and `place_point_set` create, edit or remove your own; `place_points` lists them with world position and normal for the current pose.
- `place_panel` puts a panel on a point (offset, `flip` to face inward); `place_reset` and `arrange_reset` return panels to the flat layout. `arrange_place` sets an exact origin and rotation in 3D, which the gizmo in the 3D View also does as one undo step.
- The samples (`sample_load`) place their panels for you; hand-drafted garments need `place_panel` or `arrange_place` before `sim_reset`.

## Contract for new body work

- Volumes are authored data on the body, sized from joint distances, never fitted to a mesh and never learned from garments.
- A body import must convert to millimetres, y up, standing on the ground, and keep skin and skeleton when the file has them.
- Body data is part of the project file; keep it undoable.

## Known limits

FBX body import is compiled and exercised only lightly; a dedicated X-ray joints toggle was not found; Pose transitions come from pose keys and lead-in, see [[Animation]].
