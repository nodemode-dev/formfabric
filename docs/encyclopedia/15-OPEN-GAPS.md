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

## Open build gaps at v0.18.0

What formfabric itself does not yet do, as recorded in CHANGELOG.md, DEV_PLAN.md and BUILD_NOTES.md. Each row names where its parity-matrix status is kept.

| Area | Gap | State | Where recorded |
| --- | --- | --- | --- |
| Solver speed | Multi-threaded since v0.18.0, but on the 2-core sandbox 676 vertices take about 15 to 18 ms CPU per frame (target 8 ms) and 5,041 take 150 to 180 ms (target 50 ms); at the current per-vertex cost the targets need about 4 and 8 cores; the 30k vertex shirt at 30 fps is unmeasured | open; next ideas: a cheaper per-vertex solve, structure-of-arrays data, early exit | DEV_PLAN Phase 10, BUILD_NOTES |
| Self contact in motion | Stable at every preset since v0.18.0 (no explosions, seams within 3.3 mm, no cloth inside the body), but the T-shirt walk still shows up to 61 intersecting pairs at Normal and 70 at Quality; the skirt reaches 0 at Quality | open | CHANGELOG v0.18.0 |
| GPU | Opt-in, VBD only; wind and layered contact match the CPU since v0.18.0 (the v0.15.0 reports were test artefacts); the sample T-shirt on the GPU is unmeasured | owner run on real hardware | BUILD_NOTES v0.18.0 |
| GPU measurements | The 30 fps check on a 30k vertex shirt and the 3 mm CPU match need a real GPU (the sandbox has none) | owner run | DEV_PLAN Phase 10 exit |
| Alembic | Export (an Ogawa writer) and body import are deferred; USD, glTF, MDD, PC2, FBX and OBJ sequence cover the same pipelines | deferred | CHANGELOG v0.15.0, export.formats note |
| QuadriFlow regularity | The min-cost-flow regularity step of the quad remesh is not implemented; the field-aligned remesh ships without it | open | DEV_PLAN Phase 14 |
| Refit | Refit ended 0.0 mm inside the male body in v0.15.0 (from 22.3 mm), but on a garment that did not yet hang well; not re-measured on the drafted samples | open | BUILD_NOTES v0.12.0 and v0.15.0 |
| Seam Side layer rank | The layer rank between sublayers is not built, by the PATENT_NOTES constraint | open by design | DEV_PLAN Phase 8 |
| Parity gaps recorded in the matrix | Body soft body and divide mesh, Skive, FBX trims, Cache Layers cut, merge, move, lock and override, auto key, keyed pins and tacks, bake of diffuse, metal, roughness and opacity, export weld, ASTM and AI pattern files, a single Asset Browser, settings file export and import, Animate and Render workspace tabs | open | 14-PARITY-MATRIX |
| Deferred audit items | Key scoping per layout (needs a design change); id reuse after delete (needs a persisted counter); the `sim.positions` id cast; id counter not persisted for animation; the cache header parsed on every `anim.get`; a second in-memory copy of cache bytes; folder fsync after atomic writes; a draft built in place; two colour fallbacks in `viz/stage.ts`; the `ffb-gpu/examples` and `ffb-sim/examples` harnesses kept as measurement tools | open | BUILD_NOTES v0.12.0 and v0.15.0, auditfix notes |
| Untested in the app | Unreal Live Link (USD Stage reload) has not been run against Unreal; FBX motion import (BVH and glTF motions are exercised, FBX motion is not) has not been tried in the app; USD and FBX import were checked in Blender 5.2 only; the 60 fps orbit at 200k triangles needs real GPU hardware; the `desktop dev` splash exit needs a display | owner run | BUILD_NOTES, DEV_PLAN Phases 1, 5 and 17 |
| Unmeasured fabric values | 8 of 15 presets are provisional until measured data is found; Fabric Lab formulas not yet cited from verified sources | open | DEV_PLAN Phase 6, row 13b above |
| Phase 15 capture | Re-captured on the drafted T-shirt in v0.18.0, but early in its drape and slowly under software rendering; a settled capture on real GPU hardware is an owner check | owner run | DEV_PLAN Phase 15 |
| External builds | formcompiler and formportal builds need the pushed repo and only the owner can run them | owner run | BUILD_CONTRACTS_v0.16.0 |

## Patents to respect

Granted CLO Virtual Fashion patents found by the research (chapter 13b): US 10,810,794; US 11,280,036; US 12,371,830; US 10,789,774. They claim workflows (layered sewing, automatic sewing from a pattern database, transfer of patterns between avatars). formfabric's design for seams, auto seam and refit is to be reviewed against their claims before those phases build. This is not legal advice; the owner decides whether counsel reviews it.
