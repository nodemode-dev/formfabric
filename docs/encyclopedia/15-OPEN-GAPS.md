# 15 · Open gaps

What the research could not verify from primary sources, and how formfabric decides each item. The rule: an unpublished value is never copied from memory or a forum. formfabric picks its own value, states it in its own documentation, and makes it adjustable.

## Decision policy

| Gap type | formfabric decision |
| --- | --- |
| A parameter exists but its default or range is unpublished | Choose a physically motivated default in SI units, expose the full range, record the choice in BUILD_NOTES.md |
| A preset maps to unpublished solver numbers | Define formfabric's own presets from measured behaviour (Phase 10 calibration), never reverse-engineered |
| A feature name was asked about but does not exist in the reference | Not built unless it serves the user; listed here so nobody hunts for it again |
| A feature exists only in CLO, not MD | Built when cheap and useful (DXF-AAMA pattern exchange, seam allowance); marked as an extension in the parity matrix |
| A format is proprietary and undocumented | No reader or writer; interop through open formats |

## Unverified or unpublished, by chapter

| Ch | Item | Status | Decision |
| --- | --- | --- | --- |
| 01 | Full release history before MD12; meaning of .zmdr, .zpos, VRM, ZSE | unverified | not needed for parity |
| 02, 02b | Full Notch tool properties | CLO manual only | build Notch with CLO-level properties |
| 02, 02b | Grading | only a hint on segment points in MD | Sizes deferred past v1.0.0; data model keeps room for it |
| 02, 02b | Seam allowance as a named MD tool | unresolved (Offset Outline is the MD tool) | build Seam Allowance on top of Offset Outline |
| 02, 02b | DXF, AAMA, ASTM, AI pattern import and export | not in MD | build DXF-AAMA exchange in Phase 4 as an extension |
| 02b | Pattern library, 2D snap and grid, 2D annotations | not found | build snap and grid (Phase 3); library is the general Library (Phase 16) |
| 03 | Live length-mismatch threshold (1 inch in the MD manual, 1 mm on an old CLO page) | conflicting | user setting, default 1 mm |
| 03 | Sewing layer, 3-layer sewing, merge and split sewing, stitch display toggle | not found | layering through Seam Side and Layer |
| 04 | OBJ morphing, IK joint mapping, adjust avatar pose pages | failed to load | rebuild from formfabric's own retargeting design |
| 04 | Skin weight editing, mannequin or hanger, shoe height | not found | skin weights computed on import; hanger as a prop |
| 05 | Numeric values behind presets (time step, substeps, CG iterations) | unpublished | own presets, calibrated |
| 05 | Hard and soft pin, Fix, Settle, Untangle as named commands | do not exist | Pins plus Untangle (formfabric's own intersection repair) |
| 05 | GPU hardware limits | unpublished; GPU path is NVIDIA CUDA only in MD | formfabric GPU path is wgpu (Metal, Vulkan, DX12) on all three OSes |
| 06 | Slider 0 to 99 mapping to physical constants, preset values, .zfab layout, Fabric Kit procedure | unpublished | SI-unit model with KES-F style measurement in Fabric Lab |
| 07 | Smocking, hem, rib knit, bead and sequin, fold back, line weld tools | not found | not built in v1.0.0 |
| 08 | BVH import, auto cache, pre-roll parameters, AI motion | not found | glTF and FBX motion import; lead-in transition is formfabric's design |
| 09 | Collada and PLY export, GoZ, DCC bridges other than Unreal, MD Cloud, AO baking, LOD | unverified | open formats cover DCCs; family GoF link to formsculpt instead of GoZ |
| 10 | Offline render window in MD, light and HDRI parameters, video codecs | not found for MD (CLO has V-Ray) | real-time stage plus capture; no offline renderer in v1.0.0 |
| 12, 12b | Items inside each top menu, macOS key mapping, Maya and Max navigation bindings, view-preset keys, undo depth | unpublished | formfabric's menu map follows the family menu bar; navigation presets designed in Phase 5 |
| 13, 13b | CLO solver papers | none found; CLO patents are workflow patents (layered sewing, auto sewing from a pattern database, pattern transfer between avatars) | solver built from public literature; avoid the patented workflows as claimed, see DEV_PLAN open questions |
| 13b | WebGPU inside WKWebView and WebKitGTK | not confirmed | simulation and heavy compute run natively, never relying on webview WebGPU |
| 13b | FAST formulas and KES bending formula | partly unverified | Fabric Lab ships with its formulas cited once verified |
| 13, 13b | QuadriFlow license: chapter 13 lists MIT, chapter 13b read the license file and found BSD-3 with the same grant-back clause as Instant Meshes | conflicting, 13b is the primary read | treat both as BSD-3 plus grant-back until the owner decides |

## Patents to respect

Granted CLO Virtual Fashion patents found by the research (chapter 13b): US 10,810,794; US 11,280,036; US 12,371,830; US 10,789,774. They claim workflows (layered sewing, automatic sewing from a pattern database, transfer of patterns between avatars). formfabric's design for seams, auto seam and refit is to be reviewed against their claims before those phases build. This is not legal advice; the owner decides whether counsel reviews it.
