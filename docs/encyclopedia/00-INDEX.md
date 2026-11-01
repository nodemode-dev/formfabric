# The formfabric encyclopedia of Marvelous Designer

A sourced reference of what Marvelous Designer (CLO Virtual Fashion) does and how each feature behaves, written so formfabric can reach feature parity without guessing. It describes behaviour and outcomes. It copies no code, artwork, presets, avatars or manual text from the product; every chapter paraphrases and links its sources.

Marvelous Designer and CLO are trademarks of CLO Virtual Fashion Inc. They are named here only to describe compatibility and the reference being studied. formfabric is not affiliated with or endorsed by CLO Virtual Fashion.

Research date: 2026-10-31 (UTC 2026-11-01). Reference versions: MD 12.0 through 2026.1 (August 2026).

## How the chapters were made

Sixteen research passes, each confined to one domain, each restricted to primary sources: the official manual at support.marvelousdesigner.com, marvelousdesigner.com release notes, the shared CLO manual at support.clo3d.com (marked [CLO only] where a feature is CLO-only), official PDF guides, patents, peer-reviewed papers and the license files of open-source projects. Wikipedia and every other wiki, forums, Reddit, Quora, AI summaries and sponsored listicles were excluded. Claims that rest on one source carry [single source]; anything that could not be confirmed is listed as unverified and is not treated as fact.

Source count across chapters: 228 citations of the official MD manual, 55 of the CLO manual, 50 GitHub repositories and license files, plus SIGGRAPH, Eurographics, arXiv, patent and textile-testing sources.

## Chapters

| No. | Chapter | Covers | Lines |
| --- | --- | --- | --- |
| 01 | [Overview, UI and files](01-overview-ui-files.md) | maker, editions, release history MD12 to 2026.1, requirements, workspace modes, window anatomy, every native and interchange file format | 431 |
| 02 | [2D pattern window](02-pattern-2d.md) | creation tools, selection and transform, curve editing, offsets, darts, slash and spread, fold, symmetry, pattern properties, superimpose, 2D measure | 435 |
| 02b | [2D pattern gaps](02b-pattern-2d-gaps.md) | trace, grain line, notch, rectangle, grading hints, seam allowance status, follow-up queue | 224 |
| 03 | [Sewing](03-sewing.md) | segment, free and M:N sewing, sewing line properties, turned and custom fold, fold angle, tension, check length, tack, pleats, gathering, auto sewing, how seams are simulated, data model | 588 |
| 04 | [Avatars and arrangement](04-avatar-arrangement.md) | avatar files, import, avatar editor and sizing, collider properties, joints and skin, poses and motions, morph, bounding volumes, arrangement points, fitting and re-target draping | 701 |
| 05 | [Simulation](05-simulation.md) | simulate modes, presets and quality, global solver properties, CPU and GPU, per-pattern properties, freeze, deactivate, strengthen, solidify, pins, collisions, pinching, wind, gravity, intersection resolution | 461 |
| 06 | [Fabric and materials](06-fabric-material.md) | physical property model and every parameter, unit conversion, fabric digitization, .zfab, material types, PBR channels, texture placement, prints, colorways, fur | 504 |
| 07 | [Trims and details](07-trims-details.md) | buttons, buttonholes, zippers, topstitch, puckering, piping, binding, elastic, shirring, fold tools, pressing, tape, bond, trims, graphics, fur, padding | 376 |
| 08 | [Animation](08-animation.md) | animation mode, editor, layers and tracks, keyframes, recording, motion import and retargeting, wind and pin animation, soft body, cache export, blend shapes | 610 |
| 09 | [Import, export, UV, retopology, pipelines](09-io-uv-retopo-pipeline.md) | format matrix, every export dialog, thickness export, quad and remesh, UV editor, texture baking, Python API, Unreal LiveSync and USD to Chaos Cloth, presets, CONNECT | 543 |
| 10 | [Rendering and visualization](10-render-visualization.md) | viewport display toggles and styles, graphic options, lights and environment, cameras, snapshot, video capture, turntable, fit maps, 3D measure | 429 |
| 11 | [3D tools and newest features](11-3d-tools-new-features.md) | per-version feature table, 3D editing tools, pinching, fold arrangement, 3D pencil, flatten, modular blocks, AI Studio, game-ready pipeline | 501 |
| 12 | [Menus, hotkeys, preferences](12-menus-hotkeys-prefs.md) | window anatomy, menu bar, default hotkeys, hotkey editor, user settings, object browser, property editor, history and undo, library, layouts, autosave | 579 |
| 12b | [Menu and simulation gaps](12b-menus-sim-gaps.md) | the pin family, intersection resolution, CPU and GPU limits, what does not exist under common names | 362 |
| 13 | [Cloth science and open source](13-cloth-science-oss.md) | implicit, FEM, PBD, XPBD, projective dynamics, Vertex Block Descent, IPC, measured fabric models, triangulation, GPU options, library license table, recommended solver | 443 |
| 13b | [Science and license gaps](13b-science-license-gaps.md) | Wang 2011 data-driven cloth, untangling, VBD license, quad remesher licenses, xatlas, ufbx, OpenUSD, Alembic, CLO patents, KES-F, WebGPU in webviews | 242 |
| 14 | [Parity matrix](14-PARITY-MATRIX.md) | every feature mapped to a formfabric module, name and build phase | |
| 15 | [Open gaps](15-OPEN-GAPS.md) | everything the research could not verify, and how formfabric decides each one | |

## Reading order for builders

1. Chapter 01, then 14, to see the whole surface and where each part lands.
2. The chapter for the phase being built (DEV_PLAN names it per phase).
3. Chapter 15 before inventing a default: if a value is unpublished, the gap entry says how formfabric picks it.
4. Chapters 13 and 13b before touching the solver or adding any dependency.
