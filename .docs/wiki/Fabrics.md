# Fabrics

A fabric is a set of physical properties in SI-based units plus a look. Each panel is assigned one fabric; the solver reads the physical values and the 3D View reads the look.

## Where it lives

| Path | Holds |
| --- | --- |
| `native/crates/ffb-fabric/src/` | the fabric model, presets, validation, Fabric Lab formulas, the `.ffab` container |
| `native/crates/ffb-commands/src/fabric_cmds/` | fabric and colorway commands |
| `frontend/src/workspace/fabric/` | the Fabric drawer: list, physical values, look, Lab, colorways, files |
| `docs/encyclopedia/16-FABRIC-DATA.md` | the published measurements and literature the values come from |

## Commands

| Command | Does |
| --- | --- |
| `fabric_presets` | lists formfabric's own presets with SI values, look, source keys, a provisional flag and the unit and range of every field with the 0 to 99 slider mapping |
| `fabric_list`, `fabric_create`, `fabric_update`, `fabric_duplicate`, `fabric_delete` | manage the fabrics in the project |
| `fabric_assign`, `fabric_set_default` | give panels a fabric; choose the fabric new panels get |
| `fabric_lab` | turns bench measurements (areal density, thickness, cantilever bending lengths, tensile strips, shear) into physical properties, returning the formula and source key |
| `fabric_export`, `fabric_import` | write and read `.ffab` files |
| `colorway_create`, `colorway_update`, `colorway_delete`, `colorway_activate` | one colour per fabric; the active colorway replaces front colours in views and renders |

## Presets and honesty about the numbers

There are 15 presets. Eight are provisional (silk, chiffon, taffeta, rib, fleece, spandex jersey, poplin bending, leather) because no measured data was found for them; the provisional flag is shown in the drawer and returned by `fabric_presets`. The values are formfabric's own, derived from published measurements and textile-testing literature, and no dataset is redistributed. Fabric Lab formulas are cited once verified; the FAST and KES bending formulas are partly unverified (see `docs/encyclopedia/15-OPEN-GAPS.md`).

## Using fabrics

1. Pick a preset in the Fabric drawer, or `fabric_create` from one and edit values; physical edits are range checked and out-of-range values are refused.
2. Assign it to panels with `fabric_assign`. Deleting a fabric that panels use reassigns them in the window.
3. Changing a physical value rebuilds the cloth at the next `sim_reset`; look edits are instant.
4. Keep a fabric for reuse in the Library: `library_add` with kind `fabric`, see [[Library-Snapshots-and-Components]].

## Contract for new fabric work

- Physical fields carry a unit and a range in the preset reference; a new field must add both, and a slider mapping if it is shown in the drawer.
- Presets cite a source key. A value with no source stays provisional.
- Never copy another product's presets or file formats; fabric files are `.ffab`, formfabric's own.
