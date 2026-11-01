# formfabric research 13: Cloth science and open-source building blocks for a pattern-based garment simulator

Author: researcher 13 of 13. Scope: simulation science, fabric models, pattern triangulation, GPU options, verified licenses, and a recommended solver architecture for an Apache-2.0 desktop app (Tauri 2 shell, vanilla TypeScript + Vite frontend, Python FastAPI sidecar, Rust native modules).

Evidence labels used throughout:
- [verified] means I fetched the primary source (paper page, author page, or repo page) in this session and the statement is taken from it.
- [single source] means only one fetched source supports it.
- [unverified] means it is from my background knowledge or a search-result listing only, and the team should confirm before relying on it.
- Nothing from Wikipedia, wikis, forums, Reddit, Quora, AI summary boxes, or listicles was used as evidence. Search results from listicle sites (for example WebGPU "2026 guides") were seen but discarded.

---

## 1. Summary

1. Recommended solver: a hybrid. Use an implicit, energy-based core (implicit Euler formulated as a variational problem) solved with a GPU-friendly local method (Vertex Block Descent, VBD, or Projective Dynamics style local/global) for interactive draping, with a position-based contact and collision layer (BVH, CCD or conservative-advancement thickness offsets) and an optional slower "high quality" mode that uses a barrier-based intersection-free solver (C-IPC approach, via the MIT-licensed IPC Toolkit for distance and CCD primitives). Anisotropic membrane (stretch and shear) plus discrete hinge bending, with parameters fit from KES-style measurements.
2. The core algorithms are all published and free to implement. The mature, verified-permissive code bases are: IPC Toolkit (MIT), PolyFEM (MIT), NVIDIA Warp (Apache-2.0), NVIDIA Newton (Apache-2.0), ZOZO ppf-contact-solver (Apache-2.0, but CUDA only), parry and rapier (Apache-2.0), spade (Apache-2.0 or MIT), wgpu (Apache-2.0 or MIT), nalgebra (Apache-2.0).
3. GPU: native wgpu in Rust is the only option verified in this research to run on Vulkan, Metal and DX12, which covers Windows, Linux and macOS with a single code base. CUDA is NVIDIA-only (no macOS). WebGPU in the Tauri webview is risky because Tauri uses different engines per OS (WebView2 on Windows, WKWebView on macOS, webkit2gtk on Linux) and I could not verify WebGPU in WKWebView or webkit2gtk.
4. Top licensing findings: (a) Triangle (Shewchuk) is explicitly not usable in a commercial or redistributed product without a license; use spade (Apache-2.0 or MIT) instead. (b) Blender cloth, and libigl's optional parts, are GPL-affected: Blender is GPL 2-or-later by default; libigl's repo carries both MPL-2.0 and GPL-3.0 license files, so only the MPL-2.0 portion is acceptable and care is required. (c) Autodesk FBX SDK is proprietary (from background knowledge, [unverified] here); use ufbx (MIT or public domain) for FBX import.

---

## 2. Cloth simulation methods for garment construction

### 2.1 Implicit Euler with conjugate gradient (Baraff and Witkin 1998)

Source: Baraff and Witkin, "Large Steps in Cloth Simulation", SIGGRAPH 1998, fetched from https://www.cs.cmu.edu/~baraff/papers/sig98.pdf. [verified]

Verified content:
- Cloth is stiff, so explicit integrators need very small time steps. The paper uses backward Euler, linearized with a Taylor expansion, giving one large sparse symmetric positive definite linear system per step, stable at large steps.
- Internal forces come from simple quadratic energy terms for stretch, shear and bend on a triangle mesh, rather than the quartic energies used previously. Stretch can be controlled anisotropically (important for woven fabric warp and weft).
- Damping is tied to constraint functions rather than total energy.
- Constraints are enforced by modifying the inverse mass matrix, with a "modified conjugate gradient" that filters constrained directions at each iteration, so constraints hold exactly regardless of iteration count.
- Collision handling uses position alterations inside the implicit step.
- Adaptive time stepping.
- Reported results: typically 2 to 3 time steps per 30 Hz frame, about 50 to 100 CG iterations for a 6,000 node system, run time roughly O(n^1.5), largely insensitive to stiffness. A 2,602 node cloth took about 2.2 seconds per frame, garments with several thousand nodes about 7 to 15 seconds per frame (1998 hardware).

Relevance to formfabric: this is the baseline CPU solver and a reference oracle for testing. It is stable for stiff cloth, which matters because real garment fabrics are very stiff in stretch. Its weakness is that Newton and linear solves per step are heavy and hard to run well on a GPU compared with local methods (section 2.6 to 2.8).

### 2.2 Mass-spring versus FEM

- Mass-spring: cheap and easy, but results depend on mesh topology and resolution, shear behavior is poorly controlled, and parameters do not map to measured material constants. [unverified, general knowledge]
- Baraff and Witkin's formulation is itself a triangle-based "condition function" energy (stretch, shear per triangle in the (u, v) texture frame, bend per edge pair), which is closer to a membrane FEM than to a spring network. [verified from the paper summary above: quadratic energies on a triangular mesh, anisotropic stretch]
- FEM membranes (St. Venant-Kirchhoff, co-rotational, neo-Hookean, orthotropic) give resolution-independent behavior and parameters that can be matched to measurements. The projective dynamics paper explicitly derives continuum-mechanics-inspired potentials for cloth and shells that work with its solver. [verified, see 2.7]
- The C-IPC paper shows a strain limiting barrier that works for isotropic and anisotropic cloth, relevant because real fabric stiffens sharply near its yarn lock-up strain. [verified, see 2.9]

Recommendation: triangle-based membrane FEM in the 2D pattern (material) space with an orthotropic constitutive model (warp, weft, shear moduli plus Poisson coupling), co-rotational or StVK for implementation simplicity, plus strain limiting. Do not use a plain spring network as the final model; keep it only as a fast preview mode if needed.

### 2.3 Discrete bending

- Bridson, Marino, Fedkiw, "Simulation of Clothing with Folds and Wrinkles", SCA 2003, https://graphics.stanford.edu/papers/cloth2003/ [verified]. Contributions listed on the page: a mixed explicit/implicit time integration scheme; a physically based bending model that can use nonzero rest angles to pre-shape wrinkles; an interface forecasting technique that adds detail in contact regions; a post-processing method for cloth-character collisions that preserves folds; a dynamic constraint mechanism for large-scale folding.
- Grinspun, Hirani, Desbrun, Schroeder, "Discrete Shells", SCA 2003 (pages 62 to 67, DOI 10.2312/SCA03/062-067), https://diglib.eg.org/handle/10.2312/SCA.SCA03.062-067 [verified]. A geometrically derived discrete shell model for triangle meshes, added to a standard cloth simulator, for thin flexible objects with a naturally curved rest shape. Authors: Eitan Grinspun, Anil N. Hirani, Mathieu Desbrun, Peter Schroeder.

Practical choice: the hinge (dihedral angle) energy on edges, with a rest angle taken from the pattern (flat, so zero) or from a pre-shaped target (for seam allowance, darts, pleats). Bending stiffness should be anisotropic where fabric measurement says so (warp bend differs from weft bend, and bias bend differs again). A quadratic isometric bending form (cotangent-Laplacian-based) is a good fit for projective-style solvers because it is constant per rest mesh; this is from general knowledge [unverified], so confirm before implementation.

### 2.4 PBD and XPBD

Source: Macklin, Mueller, Chentanez, "XPBD: Position-Based Simulation of Compliant Constrained Dynamics", MIG 2016, fetched from https://mmacklin.com/xpbd.pdf. [verified]
- Extends position-based dynamics with compliance (inverse stiffness) so constraints correspond to well-defined energy potentials.
- Time-step and iteration-count independent stiffness, derived from an implicit time discretization with a Lagrange multiplier per constraint.
- Per-constraint scalar gives force estimates (useful for stress visualization, which garment designers use heavily, and for breakable seams).
- Limitation stated: with zero compliance it reduces to PBD with k = 1, and runs that stop before convergence show artificial compliance.

Assessment: XPBD is easy to parallelize (graph-colored Gauss-Seidel), robust under large time steps, and a good fit for interactive manipulation, pinning, and seam pulling. Its accuracy under early termination is weaker than a true implicit solver, so it is better as the interactive mode than as the final "fit check" mode.

### 2.5 Seam constraints in the solver

Seam sewing is typically modeled as zero-rest-length stitch constraints or merged vertices between pattern edges. I did not fetch a primary source for MD's or CLO's exact handling, so the following is design guidance [unverified]:
- Pull-together phase: start with 2D pieces placed around an avatar, apply attractive seam springs or XPBD distance constraints with rest length 0 and a ramped stiffness, with collision on.
- Weld phase: once seam edge vertices are within a tolerance, merge or rigidly weld them into shared vertices (or keep a stiff constraint) so the garment behaves as one shell with a seam fold line.
- Seam-allowance and fold-over behavior: use a rest dihedral angle at seam hinge edges (the nonzero rest angle idea from Bridson et al., verified above).

### 2.6 Projective Dynamics

Source: Bouaziz, Martin, Liu, Kavan, Pauly, "Projective Dynamics: Fusing Constraint Projections for Fast Simulation", ACM TOG 33(4), SIGGRAPH 2014, https://users.cs.utah.edu/~ladislav/bouaziz14projective/bouaziz14projective.html [verified]
- A new implicit integration method bridging nodal FEM and position-based dynamics: simple, efficient, robust, accurate, supports many constraint types.
- Energies designed so an alternating optimization (local projection, global solve) works; continuum-mechanics-inspired potentials derived for the same solver.
- Applied to solids, cloth, shells, example-based simulation; compared favorably with Newton-based and PBD solvers.

Notes [unverified, general knowledge]: the global system matrix is constant while topology and stiffness are fixed, so it can be pre-factorized (sparse Cholesky) on CPU, giving very fast iterations; this is attractive for interactive CPU simulation of a few tens of thousands of vertices. Weaknesses: stiffness changes (for example when sewing) require refactorization; collision constraints change the matrix or must be handled as external forces; convergence is linear.

### 2.7 Vertex Block Descent (VBD, 2024)

Source: Chen, Liu, Yang, Yuksel, "Vertex Block Descent", arXiv:2403.06321 (cs.GR), v4 revised 1 June 2024, https://arxiv.org/abs/2403.06321 [verified]
- A block coordinate descent method for the implicit Euler variational formulation, using vertex-level Gauss-Seidel iterations.
- Each step updates individual vertex positions locally, lowering the global variational energy, with a high degree of parallelism.
- Claims: converges numerically, unconditionally stable, strong computational performance, iteration count can be capped to fit a compute budget while remaining stable, faster convergence than compared alternatives on elastic body dynamics.
- Demonstrated mainly on elastic body dynamics; the abstract also discusses particle-based simulation and rigid bodies. The abstract fetched does not claim cloth results specifically, so garment-specific quality needs prototype testing. [single source]

Why VBD matters for formfabric: it is a graph-colorable, per-vertex local method, which maps naturally to compute shaders (wgpu) or CUDA/Warp kernels, supports arbitrary hyperelastic energies including anisotropic membrane and hinge bending, and has a hard-bounded cost per frame (cap iterations). That matches the "interactive, always stable" requirement.

Related verified fact: NVIDIA Newton (Apache-2.0) lists cloth examples ("Cloth Poker Cards", "Cloth Style3D") and VBD appears in example names (for example "MuJoCo Franka VBD Cable ADMM") but the fetched README page did not describe the VBD solver itself, so I treat Newton's VBD cloth implementation as promising but needing a source-level check. https://github.com/newton-physics/newton [partly verified]

### 2.8 Multigrid and other accelerators

I did not fetch a primary multigrid source in this session. Multigrid for cloth (for example geometric or algebraic multigrid preconditioners for the implicit system, or for projective-dynamics global solves) is an active line of work, but I cannot cite a specific paper as verified. [unverified] Suggested action for a follow-up researcher: search ACM DL and arXiv for "multigrid cloth simulation" and "Chebyshev accelerated projective dynamics" before adopting.

Practical alternative that is within verified territory: CG with a diagonal or block-Jacobi preconditioner on GPU (Baraff-Witkin structure), and for VBD/XPBD use graph coloring plus Chebyshev or momentum acceleration (accelerations not verified here).

### 2.9 IPC and C-IPC for intersection-free contact

Source: Li, Kaufman, Jiang, "Codimensional Incremental Potential Contact" (C-IPC), ACM TOG (SIGGRAPH) 2021, https://ipc-sim.github.io/C-IPC/ [verified]
- Extends IPC to codimensional objects: cloth, shells, rods.
- Strain limiting: a C2 constitutive barrier enforces strain limits as an energy term, preserving the rest state; works for isotropic and anisotropic cloth.
- Thickness: distance offsets let shells keep a minimum separation equal to modeled thickness even when thickness is tiny.
- Collision detection: additive continuous collision detection (ACCD) using conservative advancement to find time of impact.
- Claims: intersection-free, stable, accurate, controllable strain limiting, thickness, frictional contact; large time steps (table cloth demo uses h = 0.01 s).
- Authors: Minchen Li (UCLA, UPenn, Adobe Research), Danny M. Kaufman (Adobe Research), Chenfanfu Jiang (UCLA, UPenn).

Cost caveat [unverified, general knowledge]: IPC solves a nonlinear problem with Newton and line search plus CCD each iteration, so it is far from interactive for garments with tens of thousands of vertices. It is the right reference for a final-quality "settle" mode, not the live mode.

Reusable code: IPC Toolkit, MIT license, https://github.com/ipc-sim/ipc-toolkit [verified]. Features listed: IPC barrier function and derivatives plus adaptive barrier stiffness; broad and narrow phase CCD of linear and nonlinear trajectories; distance computation and derivatives between edges in 2D and triangles in 3D; distance barrier potential. It is a C++ library, so it needs a Rust FFI wrapper or a C++ sidecar module. [language of toolkit is from background knowledge, unverified]

### 2.10 Continuous collision detection, self-collision, BVH

- CCD: C-IPC's ACCD (conservative advancement; verified above) and IPC Toolkit's CCD (verified above) are permissive and ready to reuse.
- BVH and collision queries in Rust: parry (Apache-2.0, "2D and 3D collision-detection library for Rust", verified license; the fetched page did not confirm BVH or time-of-impact features, so check the docs) https://github.com/dimforge/parry [license verified; features unverified].
- Self-collision for garments: common practice is a triangle BVH rebuilt or refit each step with refit, vertex-face and edge-edge proximity pairs, plus a repulsion force at a thickness offset and CCD for the final safety net. [unverified general knowledge]
- Cloth-avatar collision: the avatar is a fixed or animated triangle mesh; use a signed distance field or BVH with thickness offset. Pre-computing an SDF grid on GPU makes the avatar queries O(1). [unverified design suggestion]

### 2.11 Untangling

Source: Baraff, Witkin, Kass, "Untangling Cloth", SIGGRAPH 2003. I confirmed through search results that the paper exists and that Pixar and the SIGGRAPH history archive list it (https://history.siggraph.org/?p=115940), but I could not fetch the paper page (redirected to a generic Pixar page) so I cannot quote its abstract. [unverified beyond existence]

Background knowledge, flagged unverified: the method does a global analysis of intersection curves between cloth surfaces, determines a consistent set of "sides" for each region, and pushes regions apart so that tangled cloth can be recovered rather than hitting the failure mode where static intersections persist. For formfabric this is a rescue operation: if a user drags a garment through the body or two layers cross, untangling can recover; the preferred strategy remains prevention through CCD and IPC-style barriers.

### 2.12 CLO Virtual Fashion researchers and related garment papers

- CLO Virtual Fashion is listed as an exhibitor at SIGGRAPH 2011, 2019, 2023 and 2024 and describes itself as working on garment simulation and 3D fashion design software. https://history.siggraph.org/?p=182606 [verified for exhibitor listing only]
- I did not find a verified peer-reviewed paper by CLO researchers describing the MD or CLO solver. A search returned a patent listing page for "CLO virtual fashion" (https://patents.justia.com/assignee/clo-virtual-fashion), which I did not open, so patent contents are unreviewed. [unverified]
- Papers I checked that are often confused with CLO work but are not shown to be from CLO:
  - "Inverse Garment and Pattern Modeling with a Differentiable Simulator", Boyang Yu, Frederic Cordier, Hyewon Seo (Computer Graphics Forum), https://diglib.eg.org/handle/10.1111/cgf15249 (arXiv 2403.06841). The fetched page lists no CLO affiliation. [verified authors]
  - "Automatic Digital Garment Initialization From Sewing Patterns", listed in the SIGGRAPH history archive with blank author and affiliation fields, https://history.siggraph.org/?p=164201. A hybrid of AI classification, heuristics and numerical optimization. Authorship not shown. [single source]
  - "DiffAvatar: Simulation-Ready Garment Optimization with Differentiable Simulation", Y. Li, H. Chen, E. Larionov, N. Sarafianos, W. Matusik, T. Stuyck, CVPR 2024, appeared in an author publication index; no CLO affiliation shown. [single source]
- Recommendation: treat MD and CLO internals as closed; do not claim parity by method. Parity is judged by behavior (drape quality, speed, stability).

---

## 3. Anisotropic fabric models fit to measured data

### 3.1 Data-driven elastic models (Wang, O'Brien, Ramamoorthi 2011)

- The paper "Data-driven elastic models for cloth: modeling and measurement" by Huamin Wang, James F. O'Brien and Ravi Ramamoorthi is listed in the SIGGRAPH history archive, https://history.siggraph.org/?p=114658, and in Crossref under DOI 10.1145/2010324.1964966 (search result listing). The ACM page itself failed to fetch and the author page on graphics.berkeley.edu blocked fetching (robots disallowed). [existence verified from search listing; abstract not fetched]
- A document I fetched from cdfg.mit.edu (file named DataDrivenCloth-EG2012.pdf) describes a "piecewise linear elastic model" that approximates nonlinear anisotropic stretching and bending, fit by optimization, with inexpensive, simple measurement setups and ten cloth materials measured. The file name suggests it is the Eurographics 2012 paper "Data Driven Estimation of Cloth Simulation Models" (which appears in another search result listing on academia.edu), not the 2011 paper, and the fetch did not state authors or venue. Therefore I cannot safely attribute the "ten materials" claim to Wang et al. 2011. [unverified attribution; content described is real as fetched]
- Design takeaway that holds either way: modern data-driven approaches replace single scalar moduli with nonlinear stress-strain curves in warp, weft and bias directions, and similar for bending, then fit a runtime model to those curves. This is the right target for formfabric's fabric library: store measured curves, not just Young's modulus.

### 3.2 KES (Kawabata Evaluation System) and FAST

Source: NC State Textiles, https://textiles.ncsu.edu/tpacc/comfort-performance/kawabata-evaluation-system [verified]
- KES measures fabric properties at low forces (handling-level) related to hand and comfort.
- KES-FB1 tensile-shear tester: extensibility (EMT), tensile resilience (RT), shear stiffness (G), which relates to drape.
- KES-FB2 bending tester: bending rigidity (B).
- KES-FB3 compression tester: compressibility (EMC), resilience (RC), thickness.
- KES-FB4 surface tester: friction (MIU) and geometric roughness (SMD).
- FAST (Fabric Assurance by Simple Testing): I did not fetch a source for FAST in this session, so any FAST detail is [unverified]. It is a lower-cost alternative to KES that measures compression, bending, extensibility and dimensional stability. Confirm before documenting.

Mapping to simulation parameters:
- Warp and weft stretch stiffness from FB1 tensile curves.
- Shear stiffness from FB1 shear (G).
- Bending stiffness from FB2 (B), directional.
- Thickness (collision offset) from FB3.
- Friction (cloth-body and cloth-cloth) from FB4 (MIU).
- Density (weight per area) is a separate scale measurement.

Product implication: formfabric should define an open fabric preset JSON schema with per-direction curves and optionally KES-style scalar summaries, plus a "fit from KES scalars" function that builds a default curve family. A weakness: public, licensable KES datasets of many fabrics are not verified; the cdfg-hosted data (above) has unknown license terms. [unverified]

---

## 4. Pattern triangulation, seams and 2D to 3D placement

### 4.1 Triangulation of 2D pattern pieces

Requirements: the mesh must respect the polygon boundary, internal lines (grain lines, darts, internal edges), seam edges with matching vertex spacing across sewn pairs, and produce well-shaped triangles (near-equilateral) because stretch/shear energies and bending hinges behave best on them.

Options (all verified for license in section 7):
- spade (Rust): 2D constrained Delaunay triangulation (CDT) and Delaunay refinement, dual-licensed Apache-2.0 or MIT. https://github.com/Stoeoef/spade [verified]. Best fit for a Rust native module.
- CDT (C++, artem-ogre): MPL-2.0, constrained or conforming Delaunay, header-only or compiled, robust predicates, Windows/Linux/macOS on x64 and arm64. https://github.com/artem-ogre/CDT [verified]. MPL-2.0 is file-level copyleft; compatible with distribution in an Apache-2.0 app if CDT files remain MPL and modifications to them are published.
- Triangle (Shewchuk): NOT usable. The author page says it "may not be sold or included in commercial products without a license." https://www.cs.cmu.edu/~quake/triangle.html [verified]. Only its robust geometric predicates are public domain, so predicates may be reused (the page says "These geometric predicates are in the public domain (though Triangle is not)").

Pipeline suggestion:
1. Resample boundary edges to a target edge length with matched vertex counts across seam pairs (so sewn edges align vertex to vertex when desired).
2. Run CDT with boundary and constraint edges.
3. Refine with Ruppert-style refinement to a minimum angle (spade offers refinement, verified).
4. Optionally run a particle-distance or Lloyd-style relaxation for uniform triangle size.
5. Keep per-vertex rest 2D coordinates (material coordinates) for the FEM membrane energy.

### 4.2 Remeshing during simulation

Adaptive remeshing for cloth (refine in wrinkles, coarsen elsewhere) is a published area (for example Narain et al. adaptive anisotropic remeshing, SIGGRAPH Asia 2012). I did not fetch it in this session, so I cannot cite details. [unverified] Recommend a uniform mesh for v1, and put adaptive remeshing on the roadmap.

### 4.3 Quad remeshing and UV

- Instant Meshes (Jakob, Tarini, Panozzo, Sorkine-Hornung): BSD-style three-clause license with an extra "Enhancements" clause. The LICENSE.txt I fetched opens "Copyright (c) 2015 Wenzel Jakob, Daniele Panozzo, Marco Tarini, and Olga Sorkine-Hornung. All rights reserved." [verified, but the Enhancements clause text was not read in full; legal review needed].
- QuadriFlow (hjwdzh): MIT. https://github.com/hjwdzh/QuadriFlow [verified].
- xatlas (jpcy): MIT. https://github.com/jpcy/xatlas [verified]. For garments the UV layout is the pattern itself, so xatlas matters only for avatars, accessories and baked textures.

### 4.4 2D-to-3D placement

Design guidance, not from a fetched source [unverified]:
- Each pattern piece carries a 2D polygon with a local frame. The user (or an auto-arrange function) places the piece in 3D around the avatar at body-relative anchor points (for example "front, chest level, offset 5 cm"), with optional curvature pre-bend (arrangement points as in Marvelous Designer).
- Initial condition: a flat piece positioned near the body, not intersecting it, then pulled by seams.
- Avoid initial interpenetration with the avatar; if unavoidable, run an untangling or push-out step (section 2.11).

---

## 5. GPU options in a desktop app

### 5.1 WebGPU compute in the webview

Facts:
- MDN marks WebGPU as "Limited availability" and not Baseline; it needs a secure context (HTTPS). https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API [verified]. MDN's page, as fetched, did not include the browser compatibility table.
- Tauri uses WebView2 (Chromium-based) on Windows, WKWebView (WebKit) on macOS, and webkit2gtk on Linux. https://v2.tauri.app/reference/webview-versions/ [verified]
- WebKit announced WebGPU "is now shipping in Safari 26 beta for macOS, iOS, iPadOS, and visionOS." https://webkit.org/blog/16993/news-from-wwdc25-web-technology-coming-this-fall-in-safari-26-beta/ [verified for Safari; the page does not say whether WKWebView in embedded apps (Tauri) exposes it].
- Not verified: WebGPU in webkit2gtk (Linux), and status in WebView2 versions.

Conclusion: the webview engine differs per OS, so WebGPU in the webview has uneven support; I could not verify it on two of three target platforms. Use WebGPU only for rendering previews that can fall back to WebGL, not as the simulation engine. Also, running the solver in the webview couples simulation to the UI process and loses access to native threads and pinned memory.

### 5.2 Native wgpu (Rust)

Source: https://github.com/gfx-rs/wgpu [verified]
- License: Apache-2.0 and MIT (dual).
- Backends: Vulkan (first class on Windows and Linux/Android; on macOS/iOS via MoltenVK), Metal (first class on macOS/iOS), DX12 (first class on Windows), OpenGL (downlevel), WebGPU (first class on web/wasm).
- Runs natively on Vulkan, Metal, D3D12 and OpenGL, and on WebGL2 and WebGPU on wasm.

Implication: one WGSL compute shader code base for Windows, Linux and macOS, in a Rust native module callable from Tauri. Compute shaders can implement graph-colored XPBD, VBD, Jacobi/CG, BVH refit and SDF sampling. Caveats [unverified general knowledge]: WGSL lacks f64 and atomic float add on some backends, which affects accumulation-based assembly (use coloring or fixed-point instead), and debug tooling is thinner than CUDA's.

### 5.3 CUDA and NVIDIA Warp

- NVIDIA Warp: Apache-2.0. CUDA works on Windows x86-64 and Linux wheels and requires an NVIDIA GPU; macOS wheels support CPU only, "The macOS wheels support CPU execution but not Metal acceleration." https://github.com/NVIDIA/warp [verified]
- Warp's LICENSE.md header is "Apache License Version 2.0, January 2004", copyright NVIDIA 2022. [verified]
- ZOZO ppf-contact-solver: Apache-2.0, a cloth/shell/solid contact solver using FEM on GPU. Requirements: NVIDIA GPU with CUDA 12.8 or newer, x86 only (arm64 not supported), Docker or Windows 10/11 native, Blender 5+ add-on optional. https://github.com/st-tech/ppf-contact-solver [verified]. Excellent as a reference implementation and as an optional "pro GPU" backend, but cannot be the only backend since it excludes macOS and Apple Silicon.

### 5.4 Portability matrix

| Option | Windows | Linux | macOS | Notes |
| --- | --- | --- | --- | --- |
| wgpu native (Rust) | yes (DX12, Vulkan) | yes (Vulkan) | yes (Metal) | Single WGSL code base [verified backends] |
| WebGPU in Tauri webview | WebView2, not verified | webkit2gtk, not verified | WKWebView, not verified | Per-OS engine differences [verified engines] |
| CUDA / Warp GPU | NVIDIA only | NVIDIA only | no | Warp macOS is CPU only [verified] |
| Warp CPU | yes | yes | yes | Good fallback and test oracle [verified] |
| CPU Rust (rayon + SIMD) | yes | yes | yes | Baseline; always available [design] |

Recommendation: wgpu as the primary GPU backend, a pure-Rust CPU path as fallback and reference, and an optional CUDA backend (Warp or custom) for NVIDIA users later.

---

## 6. Where Python (FastAPI sidecar) fits

- Do not put the inner loop in Python. Use the FastAPI engine for orchestration: project files, fabric library, pattern import/export (DXF, SVG, glTF), asset conversion, job queue, and batch/headless simulation.
- Put the solver in a Rust crate (native module) exposed to the Tauri shell by IPC and to Python via pyo3 or a local socket, so the same solver is usable from the app, CLI and tests.
- Warp (Python plus CUDA/CPU) is a good prototyping tool for solver research: write VBD or XPBD in Warp first, validate against the Baraff-Witkin reference, then port the kernels to WGSL. Warp is Apache-2.0 (verified), so its kernels can be adapted with attribution.

---

## 7. Library license table

All licenses below were taken from the cited repo pages or license files fetched in this session unless marked otherwise. "OK" means usable in an Apache-2.0 distributed desktop app; the notes list obligations. This is engineering research, not legal advice; have counsel review before release.

| Library | Purpose | License (as verified) | Status for formfabric | Source |
| --- | --- | --- | --- | --- |
| wgpu | GPU API (Rust) | Apache-2.0 and MIT | OK | https://github.com/gfx-rs/wgpu |
| nalgebra | Linear algebra (Rust) | Apache-2.0 | OK | https://github.com/dimforge/nalgebra |
| parry | Collision detection (Rust) | Apache-2.0 | OK (features to be checked) | https://github.com/dimforge/parry |
| rapier | Rigid body physics (Rust) | Apache-2.0 | OK (avatar, props, not cloth) | https://github.com/dimforge/rapier |
| spade | 2D CDT and refinement (Rust) | Apache-2.0 or MIT | OK, preferred triangulator | https://github.com/Stoeoef/spade |
| CDT (artem-ogre) | 2D CDT (C++) | MPL-2.0 | OK with MPL file-level obligations | https://github.com/artem-ogre/CDT |
| Triangle (Shewchuk) | 2D mesh generator (C) | Copyright, not for commercial products without a license | NOT USABLE | https://www.cs.cmu.edu/~quake/triangle.html |
| libigl | Geometry processing (C++) | Repo shows MPL-2.0 and GPL-3.0 license files; described as "MPL-2.0-licensed" | CAUTION: use only MPL-2.0 parts, avoid anything GPL, legal review | https://github.com/libigl/libigl |
| IPC Toolkit | Barrier, CCD, distances (C++) | MIT | OK | https://github.com/ipc-sim/ipc-toolkit |
| PolyFEM | FEM/IPC research code | MIT for PolyFEM itself; third-party deps vary | OK for reference; check each dependency | https://github.com/polyfem/polyfem |
| NVIDIA Warp | GPU kernels in Python | Apache-2.0 | OK (CUDA only for GPU) | https://github.com/NVIDIA/warp |
| NVIDIA Newton | Physics engine on Warp | Code Apache-2.0, docs CC-BY-4.0 | OK | https://github.com/newton-physics/newton |
| ZOZO ppf-contact-solver | GPU cloth contact solver | Apache-2.0 | OK (CUDA, x86 only) | https://github.com/st-tech/ppf-contact-solver |
| three.js | Rendering (JS) | MIT | OK | https://github.com/mrdoob/three.js |
| OpenSubdiv | Subdivision surfaces | Tomorrow Open Source Technology License 1.0 (Apache 2.0 based, modified trademark provision) | OK, notice obligations | https://github.com/PixarAnimationStudios/OpenSubdiv |
| OpenUSD | Scene interchange | Tomorrow Open Source Technology License 1.0, bundles third-party notices (RapidJSON, LZ4, pugixml and others) | OK, include third-party notices | https://github.com/PixarAnimationStudios/OpenUSD |
| Alembic | Cache interchange | BSD 3-Clause (Lucasfilm header) | OK | https://github.com/alembic/alembic |
| Assimp | Model import | Modified 3-clause BSD | OK; static link allowed | https://github.com/assimp/assimp |
| tinygltf | glTF I/O (C++) | MIT | OK | https://github.com/syoyo/tinygltf |
| ufbx | FBX import | MIT or public domain (choose) | OK | https://github.com/ufbx/ufbx |
| Autodesk FBX SDK | FBX I/O | Proprietary [unverified, not fetched] | NOT USABLE | not fetched |
| xatlas | UV unwrapping | MIT | OK | https://github.com/jpcy/xatlas |
| Instant Meshes | Quad remesh | BSD-style 3-clause with extra Enhancements clause | OK likely; read the extra clause with counsel | https://raw.githubusercontent.com/wjakob/instant-meshes/master/LICENSE.txt |
| QuadriFlow | Quad remesh | MIT | OK | https://github.com/hjwdzh/QuadriFlow |
| Blender (including cloth) | DCC and cloth solver | GPL 2-or-later default; binaries GPLv3-or-later; add-ons using the API must be GPL-compatible; Cycles is Apache-2.0 | NOT USABLE (cloth code is GPL) | https://www.blender.org/about/license/ |

Additional verification notes:
- libigl: the GitHub About text reads "GPL-3.0, MPL-2.0 licenses found" and the README tagline says "Simple MPL-2.0-licensed C++ geometry processing library". The repo includes LICENSE.GPL and LICENSE.MPL2. I could not determine from fetched pages which modules (if any) are GPL-only. Treat libigl as MPL-2.0 for the core, verify each source file header you use, and prefer not to depend on it unless needed. [verified labels; module mapping unverified]
- Blender: the license page says the source is "default being licensed as GNU GPL Version 2 or later", exceptions exist (Cycles Apache-2.0), and combined distribution is GPLv3 or later. The cloth solver is part of the GPL source, so reading it for ideas is fine but copying code into formfabric is not. [verified]
- OpenSubdiv and OpenUSD: the fetched LICENSE text begins with a note that it "differs from the original Apache License 2.0" with a different Section 6 (Trademarks). It is Apache-derived and permissive. [verified]
- Instant Meshes: license text was only partly retrieved. [partly verified]
- Autodesk FBX SDK: not fetched. Do not use. Rely on ufbx for reading FBX. Writing FBX has no permissive full option verified here; recommend glTF, USD, Alembic and OBJ as primary export formats, FBX export deferred or via ufbx-adjacent solutions after a separate check. [unverified]
- Rust crates: always check the license of transitive dependencies with `cargo deny`, and for JavaScript with a license checker, as a CI gate.

Flag summary for "do not use":
- Triangle (non-commercial copyright)
- Blender source (GPL)
- Autodesk FBX SDK (proprietary, unverified here)
- Any GPL-only module of libigl
- Anything AGPL (none of the verified libraries above is AGPL)

---

## 8. Recommended solver architecture

### 8.1 Goals (parity with Marvelous Designer)

- Interactive drape: cloth responds in real time while the user drags pieces, pins, and changes fabric parameters.
- Stable under user abuse: no explosion with large time steps, extreme stiffness, or fast avatar motion.
- Convincing fabric differences (denim versus silk) driven by measured parameters.
- Robust seams and layering with minimal interpenetration.
- Deterministic enough for reproducible exports.
- Portable GPU acceleration on Windows, Linux, macOS.

### 8.2 Layers

1. Data layer (Python FastAPI sidecar and Rust types):
   - Pattern pieces (2D polygons with internal lines), seams (edge pairs with direction and ease), fabrics (curve-based presets), avatar mesh, collision proxy.
2. Meshing layer (Rust):
   - Boundary resampling with seam-matched vertex counts, spade CDT plus refinement, per-triangle rest-state matrices (Dm inverse) for the membrane.
3. Physics model (Rust, shared by CPU and GPU backends):
   - Orthotropic membrane (warp, weft, shear, with strain limiting), hinge bending with rest angle, stitch constraints, gravity, air drag, friction.
   - Implicit Euler variational formulation: minimize inertia plus elastic energy per step.
4. Solvers (selectable, same model):
   - Interactive mode: VBD (vertex-level Gauss-Seidel, graph-colored) with capped iterations, with XPBD-style constraints available for pins and stitches. Reasons: parallel, bounded cost, unconditionally stable per the VBD abstract [verified, but cloth garments need prototype validation].
   - Fallback or small-mesh CPU mode: Projective Dynamics with pre-factorized global matrix (sparse Cholesky on CPU), good for single-threaded determinism [design].
   - Reference mode: Baraff-Witkin implicit Euler with CG (verified algorithm, section 2.1), used as a ground-truth oracle in tests.
   - Final-quality mode (v2): C-IPC style barrier solve with IPC Toolkit's distance and CCD (MIT) for guaranteed intersection-free results, run as a background "refine" job.
5. Collision (Rust, GPU where possible):
   - Avatar: SDF grid or BVH with thickness offset (parry or custom).
   - Cloth-cloth: triangle BVH refit each step, proximity pairs, repulsion, plus ACCD/CCD safety net before position commit.
   - Rescue: untangling pass (Baraff-Witkin-Kass idea) as v2 if users hit tangles.
6. GPU backend (Rust, wgpu):
   - WGSL kernels for per-color vertex updates, BVH refit, SDF queries, normals. CPU fallback with rayon.
7. Rendering and UI (TypeScript, three.js, MIT):
   - Streams vertex positions from the native module via shared memory or a binary channel; WebGPU is optional for rendering, WebGL is the safe default.

### 8.3 Interactivity budget (targets, not measured)

- Target mesh sizes: 20k to 60k vertices for a full garment at interactive rate, finer in "quality" mode. These are design targets, not benchmarks from sources. [unverified]
- Iteration cap per frame is a tunable; VBD's claim of bounded budget while staying stable supports this approach. [verified claim from the abstract]

### 8.4 Fabric parameterization

- Store per-fabric: density, warp/weft/bias stretch curves, shear curve, warp/weft bending stiffness (and bias), thickness, friction, damping, strain-limit threshold.
- Provide a "from KES scalars" importer mapping EMT, G, B, thickness and MIU (verified KES quantities) to default curves, and a "measurement CSV" importer for curve data.

### 8.5 Testing and validation

- Cross-solver tests: same scene through Baraff-Witkin CG reference and VBD/PD; compare energy and final drape within tolerance.
- Standard benchmark: cantilever bending test (overhang length), circular drape (drape coefficient) with KES-derived fabrics, a dress-on-mannequin scene.
- Fuzz tests for seam welds and extreme parameters; invariants for no NaNs and bounded energy.
- License CI: `cargo deny`, JS license checker, and a banned list (Triangle, FBX SDK, GPL).

### 8.6 Risks

1. VBD for cloth: the fetched abstract emphasises elastic bodies; cloth quality at very stiff stretch needs validation. Mitigation: keep PD and CG implementations and strain-limit constraints.
2. wgpu limits (f32 only, no float atomics on some backends): mitigate with graph coloring and CPU fallback.
3. Fabric data: free KES datasets are not verified; mitigate by building a measurement import path and a small in-house dataset.
4. IPC Toolkit is C++: needs an FFI wrapper (cxx or bindgen); alternatively reimplement the point-triangle and edge-edge barrier derivatives in Rust (they are well specified in the IPC papers).
5. Pattern import format (DXF/AAMA, CLO or MD project files) is a separate research slice and not covered here.

### 8.7 Phased plan

- Phase 0: pattern to mesh (spade), CPU XPBD with stitches, avatar SDF collision. Proves seam workflow.
- Phase 1: orthotropic membrane plus hinge bending, CG reference solver, VBD CPU.
- Phase 2: wgpu VBD, BVH self-collision, fabric library with KES import.
- Phase 3: IPC-based refine mode with IPC Toolkit; untangling rescue; adaptive remeshing.
- Phase 4: optional CUDA backend (Warp or ppf-contact-solver integration) for NVIDIA users.

---

## 9. Open questions and unverified items

- CLO or MD solver internals: no verified peer-reviewed source found.
- Wang, O'Brien, Ramamoorthi 2011 abstract and the attribution of the "piecewise linear, ten materials" details.
- FAST test specifics.
- Multigrid cloth papers.
- Untangling Cloth abstract (exists; not fetched).
- WebGPU in WKWebView, webkit2gtk, and WebView2.
- parry BVH and time-of-impact features (license verified only).
- libigl GPL-covered modules.
- Autodesk FBX SDK license text.
- Instant Meshes "Enhancements" clause text.
- Newton's VBD cloth implementation details.

---

## 10. Sources (URLs fetched or returned by search in this session)

Fetched successfully:
- https://github.com/gfx-rs/wgpu
- https://github.com/dimforge/nalgebra
- https://github.com/dimforge/parry
- https://github.com/dimforge/rapier
- https://github.com/Stoeoef/spade
- https://github.com/artem-ogre/CDT
- https://github.com/libigl/libigl
- https://github.com/ufbx/ufbx
- https://github.com/ipc-sim/ipc-toolkit
- https://github.com/jpcy/xatlas
- https://github.com/assimp/assimp
- https://github.com/syoyo/tinygltf
- https://github.com/mrdoob/three.js
- https://github.com/NVIDIA/warp
- https://github.com/NVIDIA/warp/blob/main/LICENSE.md
- https://github.com/polyfem/polyfem
- https://github.com/hjwdzh/QuadriFlow
- https://github.com/newton-physics/newton
- https://github.com/st-tech/ppf-contact-solver
- https://raw.githubusercontent.com/wjakob/instant-meshes/master/LICENSE.txt
- https://raw.githubusercontent.com/PixarAnimationStudios/OpenSubdiv/dev/LICENSE.txt
- https://raw.githubusercontent.com/alembic/alembic/master/LICENSE.txt
- https://raw.githubusercontent.com/PixarAnimationStudios/OpenUSD/dev/LICENSE.txt
- https://www.cs.cmu.edu/~quake/triangle.html
- https://www.blender.org/about/license/
- https://arxiv.org/abs/2403.06321
- https://ipc-sim.github.io/C-IPC/
- https://mmacklin.com/xpbd.pdf
- https://www.cs.cmu.edu/~baraff/papers/sig98.pdf
- https://users.cs.utah.edu/~ladislav/bouaziz14projective/bouaziz14projective.html
- https://graphics.stanford.edu/papers/cloth2003/
- https://diglib.eg.org/handle/10.2312/SCA.SCA03.062-067
- https://cdfg.mit.edu/assets/files/DataDrivenCloth-EG2012.pdf
- https://history.siggraph.org/?p=114658
- https://history.siggraph.org/?p=182606
- https://history.siggraph.org/?p=164201
- https://arxiv.org/abs/2403.06841 (content returned was for the Yu, Cordier, Seo inverse garment paper)
- https://textiles.ncsu.edu/tpacc/comfort-performance/kawabata-evaluation-system
- https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API
- https://v2.tauri.app/reference/webview-versions/
- https://webkit.org/blog/16993/news-from-wwdc25-web-technology-coming-this-fall-in-safari-26-beta/

Fetch failed or redirected (not used as evidence):
- http://graphics.berkeley.edu/papers/Wang-DDE-2011-12/ (robots disallowed)
- https://www.cs.cmu.edu/~baraff/papers/sig03.pdf (client error)
- https://graphics.pixar.com/library/UntanglingCloth and its paper.pdf (redirected to a generic Pixar page)
- https://dl.acm.org/doi/10.1145/1964921.1964966 (client error)

Seen only as search-result listings (existence, not content):
- https://history.siggraph.org/?p=115940 (Untangling Cloth listing)
- https://patents.justia.com/assignee/clo-virtual-fashion
- https://diglib.eg.org/handle/10.1111/cgf15249
- https://api.crossref.org/works/10.1145%2F2010324.1964966
