# Sewing

A seam joins a side of one panel to a side of another. Each side is a list of spans of outline or inner-line segments. During the drape the solver binds the two sides together, so the flat pieces close into a garment.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/sew_cmds/` | seams, tacks, pleats, gathers, automatic sewing |
| `native/crates/ffb-sim/src/` | seam binding and the closing phase in the solver |
| `frontend/src/workspace/drape/seams.ts` | the Sew tab and seam picking |

## Making seams

| Command | Does |
| --- | --- |
| `sew_segment` | sews one whole segment to another; `reversed` defaults to true, which suits counter-clockwise outlines meeting along a shared edge; a `ratio` above 1 gathers |
| `sew_free` | sews a span of one segment (curve parameters `t0` to `t1`) to a span of another, so a seam can start and end anywhere |
| `sew_multi` | sews a list of spans to another list as one seam, for a sleeve cap into an armhole made of front and back pieces |
| `sew_auto` | proposes seams from the notches; `apply` true creates them as one undo step; pairs need equal notch counts and are chosen together as one minimum-cost assignment |
| `sew_pleat` | knife or box pleats on a panel with fold lines and closing seams |
| `sew_gather` | gathers side b onto side a at a ratio of 1 or more |
| `sew_edit`, `sew_flip`, `sew_delete`, `sew_list` | change fields (sides, reversed, fold, ratio, tension), reverse side b, remove, list with side lengths |
| `tack_add`, `tack_delete` | temporary point-to-point constraints between panel points or to a body point |
| `sew_rip`, `sew_mend` | leave seams out of the simulation without deleting them, and close them again |

## Checking before a drape

`sew_check` lists seams whose side lengths differ by more than a tolerance (default 1 mm), measured as side b minus ratio times side a. `sew_bindings` counts the solver bindings the seams produce on the current meshes. A full 1:1 seam forces both segments to the same number of mesh vertices, which is why the mesh follows the seams (`mesh_build`).

## Auto sewing and its limit

`sew_auto` only considers open outline segments that carry notches. Segments given the same join tag in one call pair at zero cost, and when both panels are placed in 3D a small distance term settles ties. The layer rank between sublayers of one seam is not built, by design (a patent constraint recorded in the planning notes); seams carry a `side` field instead.

## Sewing in the window

Switch to the Sew tab, click a segment, click its partner, and press B to edit the seam in the Inspector. The Sew menu lists every `sew_` command and the Drape menu the simulation commands. A seam that is wrong in length shows in `sew_check` before you drape; fix it with `pattern_match_length` or a ratio.

## Contract for new sewing work

- Seams are ordinary records in the garment; anything that joins panels later (finishes, Components, quilting) must create real seams through the same model, not a parallel structure.
- A new seam command must keep span ends valid when points are edited, and must release forced vertex counts on delete.
