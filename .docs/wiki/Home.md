# formfabric wiki

formfabric is the form family's garment app: draft sewing patterns in 2D, sew them, drape them on bodies with a cloth simulation, edit and animate the result, and export it. Every action is one command in the Rust engine registry (`native/crates/ffb-commands`), so the window, the MCP server, the Python API and the plugins all drive the same document.

Version in the root `VERSION` file · Rust engine, vanilla TypeScript and Vite window, Python and FastAPI service, Tauri 2 desktop shell · 255 engine commands · team: nodemode-ai.

## Status at a glance

| Area | State | Note |
| --- | --- | --- |
| Pattern drafting, DXF-AAMA exchange | stable | exercised by scripted scenarios and round trips |
| Sewing, bodies, placement | stable | seam spans do not follow dart, slash or offset edits |
| Fabrics | stable | 8 of 15 presets are provisional until measured data is found |
| Draping, CPU | in testing | drapes sewn samples without penetration; speed misses its targets |
| Draping, GPU | experimental | opt-in, block descent only, no self contact, never run on a real GPU here |
| 3D editing, trims | stable | Refit ends 0.0 mm inside on the drafted samples; Skive and FBX trims landed in v0.22.0 and v0.21.0 |
| Animation | in testing | cache layers, auto key and pin keys landed; walks can show cloth crossing itself; FBX motion untried in the app |
| Export, UV, remesh | stable | checked in Blender 5.2; Alembic is written by our own Ogawa writer and awaits an owner check in Blender; weld, topology drawing and the extra bake maps landed |
| Visualization and capture | stable | high triangle counts need a real GPU to judge |
| Library, snapshots, Components, recovery | in testing | built in Phase 16, checked by scripts and a kill test |
| Python API, MCP, plugins | stable | 255 commands checked through both backends |
| Live Link to Unreal | untested | written and checked as files only; never run against Unreal |
| Desktop builds | not run here | `./build_app --dry-run` passes; the three-OS build is an owner run |

The full list of what is open is in `docs/encyclopedia/15-OPEN-GAPS.md`, and every feature's status is in the Status column of `docs/encyclopedia/14-PARITY-MATRIX.md`.

## Where to go

- Start: [[Getting-Started]]
- Making the garment: [[Pattern-Drafting]], [[Sewing]], [[Bodies-and-Placement]], [[Fabrics]], [[Trims]]
- Simulating: [[Draping-and-the-Solver]], [[3D-Editing]], [[Animation]]
- Output: [[Export-and-UV]], [[Visualization-and-Capture]]
- Workspace: [[Library-Snapshots-and-Components]], [[Recovery]]
- Automation: [[Agents-and-MCP]], [[Python-API-and-Plugins]]
- Shipping: [[Building-Desktop-Apps]]
- Problems: [[Troubleshooting]]

## Editing and publishing this wiki

The source of truth is `.docs/wiki/` in the code repository, changed through a pull request like code. The GitHub wiki is only a publishing target. To publish, clone `https://github.com/nodemode-dev/formfabric.wiki.git`, copy the contents of `.docs/wiki/` into it, commit and push. Do not edit pages in the GitHub web UI; the next publish overwrites them. When a command, setting or screen changes, update its page in the same pull request.

Writing rules for these pages: full words, real paths and real command names (the names in the generated skill reference `.claude/skills/formfabric-modeling/SKILL.md`), no emoji, and no U+2014 em dash (use the interpunct).
