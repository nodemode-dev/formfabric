# 17. Default body sources and licences

Status: researched 2026-11-01 by the body-research agent. Rule from the owner: default bodies come from a CC0 or CC-BY set, adopted only after the licence is verified from the source's own licence file, with attribution shown.

## 1. Decision

Adopted: the MPFB2 / MakeHuman community base mesh, macro targets and default rig, all stated CC0 1.0 Universal by the project's own licence files (section 3). Two bodies were baked from it (young adult, average muscle, average weight, equal blend of the three MakeHuman ethnic target sets): one female, one male. Both are in `assets/bodies/`.

CC0 means no attribution is legally required and no share-alike applies, so formfabric may resize, pose, re-skin and redistribute the bodies freely. Attribution is shown anyway, by the owner's choice (section 5).

## 2. Candidates

Licence column quotes what the source's own licence file or licence page states. "Verified" means the licence file was read in a clone or on the source page during this research; otherwise the entry says so.

| Candidate | Source URL | Licence as stated (licence file URL) | Verdict and reason |
|---|---|---|---|
| MPFB2 assets (base mesh, targets, rigs, weights) | https://github.com/makehumancommunity/mpfb2 | Assets "released under CC0 1.0 Universal" (LICENSE.md section C, covering base mesh and proxies, targets, rigs, weights); full text in LICENSE.ASSETS.md. https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.md and https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.ASSETS.md . Code is GPLv3 (LICENSE.CODE.md) and is not used. weights.default.json header also says "CC0". | ADOPTED. Verified from a clone (commit 6ff1eb35, 2026-10-24). Only data files are used, no MPFB code. |
| Anny (NAVER Labs) | https://github.com/naver/anny | Code Apache-2.0 (LICENSE). README: data/mpfb2 is MPFB2-derived and "licensed under the CC0 1.0 Universal"; data/faceunits01 CC0; data/soma adapted from SOMA-X under Apache-2.0; the separate "smplx" topology download is "non-commercial use only". https://github.com/naver/anny/blob/main/LICENSE and https://github.com/naver/anny/blob/main/src/anny/data/mpfb2/LICENSE.md | NOT ADOPTED, kept as a later option. Its CC0 mpfb2 data is the same lineage as what is adopted, and its phenotype parameters could drive parametric sizing later. Never take the smplx topology download (non-commercial). Needs PyTorch to evaluate, so heavier than a baked mesh. |
| MakeHuman 1.x system assets | https://github.com/makehumancommunity/makehuman | Same team and same CC0 assets policy as MPFB2 per MPFB2 LICENSE.md ("same license setup as MakeHuman"), code AGPL. | Not fetched (repository access was refused in this session). MPFB2 carries the same base mesh and targets, so nothing is lost. Not needed. |
| MakeHuman community asset repositories (clothes, hair, skins, poses) | https://static.makehumancommunity.org/assets.html | Per-asset licences vary (CC0, CC-BY, CC-BY-SA, some AGPL and CC-NC listings). | Not adopted. Bodies only are needed now. If any asset is added later, read the licence of that individual asset file, and refuse CC-NC and CC-ND. |
| Quaternius Universal Base Characters | https://quaternius.itch.io/universal-base-characters | Page states "Creative Commons Zero v1.0 Universal" and "(CC0 License)". No licence file is mentioned in the zips; the licence is only on the sales page. | Not adopted. Stylised proportions (Superhero, Regular, Teen), about 13k triangles, clothed or hair-bearing variants, source .blend files, 122 MB and 600 MB downloads from itch.io (not reachable from GitHub). No licence file means the owner's verification rule is not met. Possible later source for stylised avatars if a licence file ships in the zip. |
| Blender Studio Human Base Meshes bundle | https://www.blender.org/download/demo-files/ | Listed as CC0 (v1.4.1, 49 MB), made by Blender Studio and community contributors. | Not adopted. Sculpt base meshes of varied quality and topology, packaged as a Blender asset library (needs Blender 4.2 or newer to extract), contributors vary per mesh, no licence file seen in the bundle, and not on GitHub. |
| Kenney character packs | https://kenney.nl/assets | Kenney states CC0 on each asset page. | Not adopted. Low-poly stylised figures, not anatomical adults. Licence file not read. |
| MB-Lab (ManuelBastioniLAB fork) | https://github.com/animate1978/MB-Lab | README fetched shows no licence text. Code is understood to be AGPL but the data licence could not be verified from a file here. | Rejected: no verified licence file for the data. |
| CharMorph | https://github.com/Upliner/CharMorph | Not read. Its base data comes from MakeHuman or MB-Lab lineage with mixed terms. | Rejected: not verified. |
| Khronos glTF sample assets (CesiumMan etc.) | https://github.com/KhronosGroup/glTF-Sample-Assets | Per-model licences, mostly CC-BY 4.0 for the humanoid samples. | Rejected: stylised or clothed single figures, no adult female and male pair, not body-like. |
| SMPL, SMPL-X, STAR, SUPR, GHUM and similar | https://smpl.is.tue.mpg.de/modellicense.html | Non-commercial research licences. | Rejected by owner rule. |
| Mixamo | https://helpx.adobe.com/creative-cloud/faq/mixamo-faq.html | Adobe terms allow use in projects but forbid redistributing the characters as a standalone library. | Rejected by owner rule. |
| Ready Player Me | https://readyplayer.me/terms | Terms forbid redistribution of avatar assets outside the service. | Rejected by owner rule. |
| CAESAR, SizeUSA and other scan databases | https://www.sae.org/standards/content/caesar/ | Commercial or restricted licences. | Rejected: no open licence. |
| Anything CC-NC, CC-ND, or without a licence file | n/a | n/a | Rejected by owner rule. |
| CC-BY-SA candidates | n/a | n/a | None adopted. Consequence if one is ever considered: formfabric resizes and poses bodies, so every derived mesh would have to be shared alike under CC-BY-SA, which conflicts with the Apache-2.0 distribution. Only usable unmodified, which defeats the purpose. |

## 3. Licence verification trail for the adopted source

Cloned `https://github.com/makehumancommunity/mpfb2` (shallow, commit 6ff1eb355d, committed 2026-10-24) and read:

- `LICENSE.md`, section A: "The MPFB addon consists of two separate parts: source code ... Assets". Section C: the assets are "the base mesh and proxies; targets and modifiers; textures; clothes; rigs, poses and expressions; JSON data with mesh information" and "have been released under CC0 1.0 Universal". Section D: no claim over output produced with MPFB.
- `LICENSE.ASSETS.md`: the CC0 1.0 Universal legal text.
- `src/mpfb/data/rigs/standard/weights.default.json`: field "license": "CC0".

Copied verbatim into each body folder: `LICENSE.txt` is `LICENSE.ASSETS.md` (byte identical, checked with cmp) and `LICENSE-SCOPE.txt` is `LICENSE.md` (the statement that the assets are CC0). Not copied and not used: any MPFB Python code (GPLv3).

Source files used: `src/mpfb/data/3dobjs/base.obj`, the macro detail targets `src/mpfb/data/targets/macrodetails/{african,asian,caucasian}-{female,male}-young.target.gz` and `universal-{female,male}-young-averagemuscle-averageweight.target.gz` (empty, the base already is that point), `src/mpfb/data/rigs/standard/rig.default.json`, `weights.default.json`.

## 4. Chosen bodies

Both are glTF 2.0 binary files built by `tools/body-research/build_bodies.py` (reads the MPFB2 data folder, writes `body.glb`; run with `python3 -I build_bodies.py MPFB_DATA OUTDIR`, needs numpy only).

| Item | Female | Male |
|---|---|---|
| Folder | `assets/bodies/mpfb2-female/` | `assets/bodies/mpfb2-male/` |
| Mesh file | `body.glb` (993,580 bytes) | `body.glb` (993,728 bytes) |
| Licence files | `LICENSE.txt`, `LICENSE-SCOPE.txt` | `LICENSE.txt`, `LICENSE-SCOPE.txt` |
| Vertices | 13,380 | 13,380 |
| Triangles | 26,756 (13,378 quads split along the shorter diagonal) | 26,756 |
| Height (bounding box) | about 1.59 m | about 1.73 m |
| Bounding box min / max (m) | -0.485,-0.818,-0.091 / 0.485,0.771,0.292 | -0.566,-0.819,-0.100 / 0.566,0.912,0.377 |
| Closed mesh | no boundary edges and no non-manifold edges after welding coincident positions; enclosed volume 0.048 m3 | no boundary edges, no non-manifold edges; 0.063 m3 |
| Units | metres (MakeHuman source unit is the decimetre, scaled by 0.1 when baked) | metres |
| Axes | +Y up, +Z toward the front of the figure, +X toward the figure's left side (bones named .L sit at +X) | same |
| Origin | near the pelvis, not at the feet (feet at Y about -0.82) | same |
| Pose | neutral A-pose, arms about 45 degrees below horizontal, fingers relaxed | same |
| Skeleton | yes: 163 bones, MakeHuman "default" rig, bone parent hierarchy as glTF nodes, rest pose identity rotations, inverse bind matrices included | same |
| Skin weights | yes: from `weights.default.json`, reduced to the 4 strongest influences per vertex and renormalised, every vertex weighted | same |
| UVs, textures | none baked in (not needed for collision bodies) | none |
| Not included | eyes, teeth, tongue, eyebrows, hair, clothes, helper geometry (tights, joint cubes) | same |

Notes for body-engine:
- Joint positions in glTF come from the MPFB2 rig definition evaluated on the baked, gendered mesh (cube centres, vertex positions and means), so they fit each body.
- The gender and ethnic blend is the macro default: each of the three ethnic target sets at weight 1/3, gender at 0 (female) or 1 (male), age young (25), average muscle and weight. Other builds can be produced by changing the weights in the script; all macro targets are in the same CC0 data folder.
- Body groups are the first 13,380 vertices of `base.obj`, so the weights file vertex indices map directly.
- No child body was built (not needed now). The baby, child and old macro targets exist in the same CC0 data.
- Not run here: a viewer check of the mouth and eye socket regions. The mesh passes the closed-surface edge test, but the inside of the mouth is a shallow cavity and cloth should not need to reach it.

## 5. Attribution formfabric must show

CC0 does not require it; the owner wants it. Put the following in `NOTICE.md` (lead-owned file, request in notes) and in the About box under "Third-party content":

```
Default bodies: the female and male base bodies are derived from the MakeHuman
community assets distributed with MPFB2 (base mesh, macro targets, default rig
and skin weights), dedicated to the public domain under CC0 1.0 Universal.
Source: https://github.com/makehumancommunity/mpfb2 (assets, not the GPLv3
code; no MPFB code is included). Licence text: assets/bodies/mpfb2-female/LICENSE.txt
and assets/bodies/mpfb2-male/LICENSE.txt. Converted, scaled and re-skinned by
formfabric.
```

Short About-box line: `Default bodies from MakeHuman community assets via MPFB2 (CC0 1.0).`

The `Body` struct fields: `source` = `MPFB2 / MakeHuman community assets (makehumancommunity/mpfb2)`, `license` = `CC0-1.0`.

## 6. Rejected and why (summary)

- SMPL family, STAR, SUPR: non-commercial licences.
- Mixamo, Ready Player Me: redistribution forbidden by terms.
- MB-Lab, CharMorph: no verifiable licence file for the data in this research.
- Quaternius, Blender Human Base Meshes, Kenney: stated CC0 but only on a web page or listing with no licence file in the download, and not fetchable from GitHub here; stylised or sculpt-quality meshes rather than adult female and male A-pose bodies with weights. Revisit if their archives are obtained and contain a licence file.
- Anny: acceptable (Apache-2.0 and CC0) but redundant; avoid its non-commercial smplx topology.
- Any CC-BY-SA, CC-NC, CC-ND or unlicensed body: excluded; SA would force shared-alike on every resized or posed derivative.

## 7. Sources

- https://github.com/makehumancommunity/mpfb2 (LICENSE.md, LICENSE.ASSETS.md, LICENSE.CODE.md, src/mpfb/data)
- https://github.com/naver/anny (LICENSE, README License section, src/anny/data/mpfb2/LICENSE.md)
- https://quaternius.itch.io/universal-base-characters
- https://www.blender.org/download/demo-files/
- https://github.com/animate1978/MB-Lab
- https://github.com/KhronosGroup/glTF-Sample-Assets
- https://static.makehumancommunity.org/assets.html
- https://smpl.is.tue.mpg.de/modellicense.html
- https://helpx.adobe.com/creative-cloud/faq/mixamo-faq.html
- https://readyplayer.me/terms
- Build script: `/home/claude/ff/tools/body-research/build_bodies.py`; clones in `/home/claude/ff/tools/body-research/{mpfb2,anny}`
