# 3D editing

Tools that change the garment while it hangs on the body, without going back to the flat pattern.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-commands/src/edit3d_cmds/` | Style Lines, Fit Shell, Refit, pinch, panel states, body tapes |
| `native/crates/ffb-commands/src/sim_cmds/` | pins, grab, fold, wind |
| `frontend/src/workspace/drape/` | the Drape tools, pin and panel state controls |
| `frontend/src/workspace/styleline/` | Style Line drawing and editing in the 3D View |

## The tools

| Tool | Commands | Notes |
| --- | --- | --- |
| Pinch | `sim_pinch` | pulls a picked vertex to a 3D target while the neighbours follow by a falloff (bump or linear, radius 1 to 500 mm, straight or surface distance); not saved, not an undo step; Shift+C |
| Pin | `pin_add`, `pin_box`, `pin_lasso`, `pin_remove`, `pin_clear` | pins are part of the document and an undo step; box is Shift+X, lasso Shift+L |
| Grab | `sim_grab` | holds a vertex like the mouse drag; releases on a null target |
| Freeze, stiffen, hold shape, shrink | `panel_state` | per panel; the simulation rebuilds and keeps the other panels where they are |
| Fold | `fold_place` | folds a panel about a line before the drape, saved as the panel's pose; Shift+D |
| Style Lines | `styleline_add`, `styleline_set`, `styleline_remove`, `styleline_list`, `styleline_list3d` | a polyline on a panel that can stiffen a band or crease the cloth along it (modes none, stiffen, fold) |
| Fit Shell | `fitshell_set`, `fitshell_get` | an offset surface along smoothed normals that replaces the body as the collider, so garments rest at a fitting ease |
| Body tape | `body_tape` | holds a panel point to a body point with one rest length and one stiffness; Shift+K |
| Refit | `refit` | moves a draped garment from body A to body B without opening a seam, by morphing the body in steps while the solver runs |

## Refit

`refit` takes `from` and `to` bodies and a `method`: `blend` (identical topology, per-vertex blend), `sweep` (body A is stepped through a measurement sweep to B's measurements, then the garment settles), `redrape` (no morph, the garment returns to its placed layout and drapes on B), or `auto`. There is no reference garment, correspondence map, stored fit data or pattern change; the garment follows by simulation and contact only. `fix_frames` pushes cloth out of the target body, and a deep push before the first step projects cloth lying up to 400 mm inside the new body to its surface plus the skin offset; the result reports `inside_start_mm` (before the push) and `inside_mm` (after). It ends with one undo step. On the drafted T-shirt all three methods ended 0.0 mm inside, but check `inside_mm` on your own garment. The `maintain` object (`curvature_pct`, `graphic_size`, `texture_size`, `topology`) is accepted at its maintaining values only, because Refit never edits the pattern; lower values are refused with code `unsupported`.

## Contract for new 3D tools

- A 3D edit that changes the document is one undo step; a live interaction (pinch, grab) is not saved and ends cleanly on a null target.
- The tool must not infer what a panel is from its name or shape; targets come from the user's picks or ids.
- Interface strings go to every language table; shortcuts are registered in `frontend/src/ui/commands.ts` and show in the Shortcut Editor.
