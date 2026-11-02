# Trims

Trims are the details added to a garment: buttons, zippers, stitching, edge finishes, prints and quilting. Most are hardware or render data attached to a panel; some (piping, binding, elastic, quilting) generate real panels and real seams.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/trim_cmds/` | trim commands and the hardware model |
| `frontend/src/workspace/trims/` | the Trims panel, tools, 3D hook, images |
| `docs/encyclopedia/07-trims-details.md` | the behaviour specification |

## Commands

| Trim | Commands | What happens |
| --- | --- | --- |
| Styles | `style_create`, `style_update`, `style_delete`, `style_list` | named styles for buttons, buttonholes, zippers, topstitch, puckering and imported trims; a style in use cannot be deleted |
| Buttons | `button_add`, `buttonhole_add`, `button_fasten`, `hardware_move`, `hardware_delete` | a button is a small rigid disc on the panel triangle; fastening holds a button to a buttonhole with a pair constraint during the simulation (Shift+B, Shift+H, Shift+F, Shift+A) |
| Zippers | `zipper_add`, `zipper_set`, `zipper_delete` | two sides listed in the zipping direction; the closed part binds like a seam, the open part does not; `zipper_set` sets how far it is closed (Shift+Z) |
| Stitching | `stitch_add`, `stitch_set`, `stitch_delete`, `fray_add` | topstitch or puckering along spans at an offset; render data with slight stiffening; fray is a stitch whose style carries the fray length (Shift+S) |
| Edge finishes | `finish_add`, `finish_set`, `finish_delete` | piping and binding make a strip panel sewn by a real seam; elastic and shirring act through a seam ratio (Shift+E) |
| Prints | `print_add`, `print_set`, `print_delete` | an image on a panel face with position, rotation, size and optional emboss (Shift+I) |
| Quilting | `quilt_make` | clones a panel as a lining with opposite pressure and sews fold lines |
| Imported trims | `trim_import` | an OBJ becomes a rigid trim glued to a panel point, recentred and scaled to millimetres, stored inside the project |
| Inspection | `trim_debug` | read only: what the simulation would read from the trims right now |

## Notes

- Deleting a finish removes its generated strip panels and every seam touching them in one undo step.
- FBX trims are not built (open parity row); imported trims are OBJ.
- Trim styles can be kept in the Library as kind `trim`, see [[Library-Snapshots-and-Components]].

## Contract for new trims

A trim that affects the simulation must say how, in its command description (rigid disc, pair constraint, binding, stiffening). Render-only data must not change the solver. Generated panels must be recorded so deleting the trim removes them.
