# 13b. Science and license gap-fill for formfabric

Scope: closes the items left unverified in 13-cloth-science-oss.md. Fetched in this session from primary sources only. Tags: [verified] = read on a primary page or file this session; [single source] = one primary page only; [unverified] = could not be confirmed, do not rely on it. Where a fetch tool summarized a page, the summary is treated as the reading of that page, and exact legal text was pulled with curl from raw repo files.

Search note: the WebSearch budget for this turn ran out after two searches (Untangling Cloth, Wang 2011), so several items (CLO academic papers, FAST, bending formula, WebKitGTK WebGPU) could not be searched further. They are flagged below and listed in the Open items section.

## Summary table

| # | Item | Result | Status |
|---|------|--------|--------|
| 1 | Untangling Cloth (2003) | History-free global intersection analysis plus "collision flypapering"; Pixar PDF itself redirects away, abstract read from SIGGRAPH history archive | verified (abstract level) |
| 2 | Wang et al. 2011 | Piecewise linear anisotropic stretch (24 params) and bending (15 params), measured with cheap hanging/loaded-sample rigs, 10 materials, data downloadable, no license stated | verified |
| 3 | CLO papers and patents | No academic paper by CLO staff found (search budget exhausted); 6 published applications and 4 granted patents identified | patents verified, papers unverified |
| 4 | WebGPU in webviews | Safari 26.0 ships WebGPU (macOS 26 era); WKWebView applicability not stated by Apple/WebKit pages; WebView2 follows Chromium (WebGPU since Chrome 113 on D3D12); WebKitGTK no WebGPU mention in 2.52 or 2.54 notes | partly verified |
| 5 | VBD reference code license | Gaia repo, Apache-2.0 | verified |
| 6 | Instant Meshes, QuadriFlow | BSD 3-clause plus an extra "Enhancements" grant clause, both repos identical text | verified |
| 7 | xatlas, ufbx, OpenUSD, Alembic | MIT; MIT or Unlicense dual; Tomorrow Open Source Technology License 1.0 (Apache 2.0 variant); BSD 3-clause | verified |
| 8 | FAST, KES-F | KES-F tensile and shear definitions verified from maker PDFs; bending page lacks formulas; FAST not reachable | partly verified |

## 1. Baraff, Witkin, Kass 2003, "Untangling Cloth"

Citation: David Baraff, Andrew Witkin, Michael Kass, "Untangling Cloth", ACM SIGGRAPH 2003 (ACM TOG 22(3)). The year and TOG volume come from background knowledge and the Stanford colloquium abstract (which says the paper was slated for SIGGRAPH '03); the SIGGRAPH history archive page does not state them. [single source for year]

What the abstract says (SIGGRAPH history archive page, read via fetch summary, paraphrased) [verified]:
- Cloth-cloth collision response is called the most serious shortcoming of most cloth simulation systems.
- Earlier methods used collision history to decide whether regions of cloth had interpenetrated, so one wrong decision could leave persistent tangles, especially where a character body pinches cloth at elbows, knees or hands.
- The paper proposes a "history-free cloth collision response algorithm" that performs global intersection analysis of the cloth meshes at every simulation step. It untangles pinched cloth as soon as geometry permits and also repairs tangled initial conditions.
- It also introduces "collision flypapering", a cloth-solid collision method that removes flutter and other artifacts during pinches.
- The authors state both methods were used extensively in a production animation environment.

Method summary (secondary: a university course reading summary of the paper, not a primary source; treat as [single source, secondary]):
1. Find the intersection curves between pairs of cloth meshes.
2. For each intersection curve, flood fill the mesh regions on both sides; the smaller region marks the side the cloth should be pushed toward.
3. Use those global classifications to constrain which local attract or repel response is applied, so separation is topologically consistent and needs no collision history.
4. Pinch handling: cloth pinched between stationary objects should stay still, sit midway, and avoid abrupt changes when pinching starts or ends.

Corroboration: the Stanford Broad Area Colloquium abstract by Michael Kass describes the algorithms as untangling cloth in non-physical states (characters pushed through each other) for Monsters, Inc. [verified]

Access caveat: graphics.pixar.com/library/UntanglingCloth and its paper.pdf return a 302 redirect to pixar.com/technology-libraries. The full paper text was NOT read; algorithm details beyond the abstract (curve extraction cost, exact response forces) are unverified.

Implication for formfabric: a rescue pass for user-induced or imported tangled starts (dragging cloth through a body, imported layered garments). Prevention (CCD, barrier methods) remains the primary strategy. No code license is attached to the paper; implement from the paper, not from any copied source.

## 2. Wang, O'Brien, Ramamoorthi 2011, "Data-driven elastic models for cloth: modeling and measurement"

Citation: Huamin Wang, James F. O'Brien, Ravi Ramamoorthi, ACM Transactions on Graphics 30(4), Article 71, SIGGRAPH 2011. UC Berkeley. Read from the eScholarship full-text PDF. [verified]

This is the 2011 SIGGRAPH paper. It is distinct from the later Eurographics 2012 paper "Data Driven Estimation of Cloth Simulation Models" (cdfg.mit.edu file in the earlier report), so the earlier report's attribution warning was correct: the "ten materials" figure belongs to the 2011 paper, per the PDF below.

Model [verified]:
- Stretching: generalized Hooke's law with a 3x3 stiffness matrix. For orthotropic cloth this reduces to four parameters (c11, c12, c22, c33). Strain is expressed as principal strains plus a strain angle, and stiffness is piecewise linear, interpolated over six sample points in that polar strain space, giving 24 stretching parameters.
- Bending: extends the bending force formulation of Bridson et al. Bending stiffness is a piecewise linear function of a mesh-independent curvature variable, sampled at five points for each of three bias angles (0, 45, 90 degrees), giving 15 bending parameters.
- Total per material: 24 + 15 = 39 fitted parameters (derived by addition from the two counts above).

Measurement [verified]:
- Stretching: three 400 mm x 400 mm samples cut at bias angles 0, 45 and 90 degrees. Top edge fixed between wooden slats, other edges loaded with weights. 105 tests per material. A calibrated camera records the deformed shape; labeled point features are matched against a finite element simulation, and parameters are fit with a quasi-Newton (BFGS) optimizer.
- Bending: three 4 cm wide samples at the same bias angles, clamped at top and allowed to drape under their own weight. A side-view camera records the curves, which are matched against simulated curves the same way.
- Devices are described as inexpensive and simple, not specialist tensile or bending testers.
- Materials: ten, including Ivory Rib Knit, Pink Ribbon Brown, Navy Sparkle Sweat, Gray Interlock, 11oz Black Denim, White Swim Solid, Tango Red Jet Set, Royal Target.

Data release [verified, license unknown]: the paper states raw image measurements, fitted parameters and image textures are downloadable from a Berkeley graphics group URL (graphics.berkeley.edu/papers/Wang-DDE-2011-08). The text read does not state a license. The project page itself could not be fetched (robots disallowed), so license terms are unverified. Do not bundle the data in an Apache-2.0 repo until the terms are confirmed or the authors grant permission.

Implication for formfabric: a preset schema with per-bias-angle stretch and bend curves is directly supported by this paper. A user-side "fit from photos" tool following this recipe needs only a camera, slats, weights and a clamp.

## 3. CLO Virtual Fashion: papers and patents

### 3a. Academic papers

No peer-reviewed cloth simulation paper by CLO Virtual Fashion staff or founders was found or verified. The search budget ran out before this could be searched properly (ACM DL, DBLP, Google Scholar listing pages were not queried). [unverified: absence is not proof]. Suggested follow-up: query DBLP and ACM DL for author affiliations containing "CLO Virtual Fashion" or "Marvelous Designer", and look at the inventor names on the patents below, who are the most likely authors.

### 3b. Patents and applications

Source for the list: Justia assignee page for "CLO virtual fashion" (six published applications, no pagination shown) [verified], with family data and claim text from Google Patents pages. Claims were read as page summaries, not full claim text; confirm against the official claims before citing. Names under other assignee spellings (for example Korean entity names) may exist and were not searched.

| Application | Granted | Title | One-line content | Status of reading |
|-------------|---------|-------|------------------|-------------------|
| US 2016/0314604 | not determined | Method and apparatus for creating digital clothing | Filed 2015-10-23, published 2016-10-27. Page could not be fetched (server error). | title only |
| US 2018/0025539 | US 10,810,794 B2 | Method and apparatus for 3D clothing draping simulation | Sewing-line designation, then a sublayer input, then applying a layer limitation so two designated sewing lines are layered in a chosen front-behind order, avoiding intersection; KR priority 10-2016-0092791 (2016-07-21), US filing 2017-04-26, app 15/497,957 | claim 1 of the granted patent summarized |
| US 2018/0087196 | US 11,280,036 B2 (app 15/474,091, published 2022-03-22) | Method and apparatus for 3D clothing draping simulation | Pattern/block database: input 2D pattern and pattern block, match them, store; later select matching blocks and automatically sew the 2D patterns using stored sewing information. KR priority 10-2016-0125040 (2016-09-28). Estimated expiry July 2040 per the Google Patents page | claim 1 of the grant summarized |
| US 2022/0267938 | US 12,371,830 B2 (app 17/667,819, filed 2022-02-09, grant date shown 2025-07-29) | Method and apparatus for 3D clothing draping simulation | Divisional of the above; claim 1 recites a clothing pattern DB generation apparatus: pattern input unit, pattern block input unit, pattern/block matching unit, pattern/block database. A further continuation US 2025/0333888 A1 (filed 2025-07-07) is pending | claim 1 shown on page |
| US 2019/0304182 | US 10,789,774 B2 (granted 2020-09-29, app 16/368,897); continuation US 11,348,312 B2 | Method of generating transferred pattern of garment draped on avatar | Generates target-avatar patterns by directly deforming source patterns: avatar deformation values give target strain tensors; pattern strain tensors derived from those and source strain tensors; multiplied with source patterns to give target patterns, instead of flattening a 3D target garment | abstract level |
| US 2019/0304200 | not determined | Apparatus and method for transferring garment draping between avatars | Filed 2019-03-29, published 2019-10-03. Page could not be fetched | title only |

Observations:
- None of the six covers the core solver (time integration, constitutive model, collision). The portfolio found concerns workflow: layered sewing order, automatic sewing from a pattern-block database, and grading or transferring patterns between avatars.
- Implication for formfabric (not legal advice): features that resemble the claims above (automatic sewing from a stored pattern-to-block database; sewing-line-based layer ordering via guideline and operation symbol; strain-tensor-based pattern transfer between avatars) deserve a freedom-to-operate review by counsel before shipping. 10,810,794 and 11,280,036 are in force by the Google Patents status shown; 12,371,830 is recent. Standard physical simulation (XPBD, VBD, IPC) is not claimed in what was read.
- US filings may have additional CLO-assigned grants not listed on Justia. Not exhaustive.

## 4. WebGPU in Tauri webviews

Tauri uses the OS webview, so WebGPU availability equals the OS webview's availability.

### macOS: WKWebView (Safari engine)
- WebKit blog "News from WWDC25": WebGPU "is now shipping in Safari 26 beta for macOS, iOS, iPadOS, and visionOS". It had been enabled in Safari Technology Preview for over a year. [verified]
- WebKit features in Safari 26.0: WebGPU added, "supersedes WebGL on macOS, iOS, iPadOS, and visionOS", adds compute shaders. [verified]
- Apple release notes: Safari 26.0, build 20622.1.22, release date 2025-09-15. The notes page body (WebGPU entry, OS requirements) could not be read through the fetch. [verified for date]
- NOT verified: that WKWebView (as embedded by Tauri/wry, not Safari the app) exposes navigator.gpu by default on macOS 26, and the minimum macOS version. Neither WebKit page mentions WKWebView in connection with WebGPU; the only WKWebView statement found was about the Digital Credentials API being unsupported there. The WWDC25 post mentions macOS Sequoia and Sonoma only as Safari 26 beta host OSes, which concerns the browser app, not the system WebKit used by WKWebView. Treat WebGPU in Tauri-on-macOS as "likely only on macOS 26 and later; test with navigator.gpu on a real build" [unverified].

### Windows: WebView2 (Chromium)
- Chrome 113 shipped WebGPU by default, initially on ChromeOS, macOS and Windows; Windows requires Direct3D 12 support; Linux and Android were deferred. [verified, Chrome developer blog]
- WebView2 Runtime release notes: newest stable listed 153.0.4234.32 (2026-09-11), preview 154.0.4258.9. The page says the Runtime shares code and binaries with Edge and version numbers follow the Edge/Chromium major version. It does not mention WebGPU. [verified for versions]
- Inference: any current Evergreen Runtime (well past 113) has the WebGPU code path, subject to GPU driver and D3D12 support and any flags WebView2 hosts disable. Not directly confirmed that WebView2 enables WebGPU by default. [unverified inference]

### Linux: webkit2gtk (WebKitGTK)
- Latest stable on the news page: WebKitGTK 2.54.2 (2026-10-23). The 2.54 highlights (2026-09-16) and 2.52 highlights (2026-03-18) were read and neither mentions WebGPU; the 2.54 post mentions a new compositor and states that it no longer stands in the way of Vulkan-based rendering "in the future". [verified: no WebGPU mention; this is absence of evidence only]
- Conclusion: WebGPU in Tauri on Linux should be considered unavailable or experimental. [unverified]

Recommendation for formfabric (unchanged but now firmer): keep the simulation engine in native Rust with wgpu. Use webview WebGPU only as an optional renderer, with a WebGL fallback, and feature-detect navigator.gpu at runtime.

## 5. Vertex Block Descent (Chen, Liu, Yang, Yuksel, SIGGRAPH 2024)

- Paper: ACM Transactions on Graphics 43(4), Proceedings of SIGGRAPH 2024. Authors Anka He Chen, Ziheng Liu, Yin Yang, Cem Yuksel. [verified, University of Utah project page]
- Project page links code at https://github.com/AnkaChan/Gaia and says VBD is included in Anka He Chen's Gaia engine. The project page itself states no code license (shows only a University of Utah copyright notice). [verified]
- Gaia repo LICENSE: "Apache License, Version 2.0, January 2004", unmodified text with the appendix template (no holder named in LICENSE). The README states the code is licensed under Apache 2.0 with copyright Anka He Chen (2023) per the fetch summary of the repo page. [verified; the README line was seen in one fetch summary only, so the copyright holder is [single source]]
- Repo contents: C++ engine; both VBD and XPBD simulators at Gaia/Simulator/VBDDynamics and Gaia/Simulator/PBDDynamics; README says tested only on Windows with Visual Studio. Dependencies: MeshFrame2, CuMatrix (submodules), polyscope (GUI option), plus OneTBB, Eigen3 and Embree 3.x (not Embree 4) installed separately. Dependency licenses were not audited. [verified]
- Implication: license is compatible with an Apache-2.0 project. Reference implementation is C++ (and CUDA via CuMatrix, which was not verified); formfabric needs a Rust/WGSL reimplementation anyway. Check git history for third-party headers before copying code verbatim.

## 6. Instant Meshes and QuadriFlow licenses

### Instant Meshes (wjakob/instant-meshes, LICENSE.txt, fetched raw with curl) [verified]
Copyright (c) 2015 Wenzel Jakob, Daniele Panozzo, Marco Tarini, and Olga Sorkine-Hornung. All rights reserved. BSD 3-clause (conditions 1 source retains notice, 2 binary reproduces notice, 3 no endorsement using names) with the standard "AS IS" disclaimer, followed by this extra clause, quoted in full:

    You are under no obligation whatsoever to provide any bug fixes, patches, or
    upgrades to the features, functionality or performance of the source code
    ("Enhancements") to anyone; however, if you choose to make your Enhancements
    available either publicly, or directly to the authors of this software, without
    imposing a separate written license agreement for such Enhancements, then you
    hereby grant the following license: a non-exclusive, royalty-free perpetual
    license to install, use, modify, prepare derivative works, incorporate into
    other computer software, distribute, and sublicense such enhancements or
    derivative works thereof, in binary and source code form.

Reading: this is a BSD-3-clause variant with a grant-back for enhancements. It does not restrict use, but if formfabric publishes improvements to Instant Meshes code (for example a public fork or a PR) without a separate written license, it grants everyone a broad perpetual license to those enhancements. Compatible with Apache-2.0 distribution of unmodified or modified use; the grant-back applies only to enhancements made available publicly or to the authors. OSI classification of this variant is not asserted here. Counsel should confirm whether a separate written license on enhancements (for instance Apache-2.0 headers on a fork) satisfies the "without imposing a separate written license agreement" carve-out.

### QuadriFlow (hjwdzh/QuadriFlow, LICENSE.txt on the master branch) [verified]
Copyright (c) 2018 Jingwei Huang, Yichao Zhou, Matthias Niessner, Jonathan Shewchuk and Leonidas Guibas. All rights reserved. The text is the same BSD 3-clause plus the identical "Enhancements" grant-back paragraph, word for word as above. The file path is LICENSE.txt (LICENSE and LICENSE.md return 404). The default branch name is master for this file path; main returned 404.

Caveat that matters: the QuadriFlow copyright list includes Jonathan Shewchuk. The earlier report found that Triangle (Shewchuk) is not licensable for redistribution; QuadriFlow's own license file nonetheless grants BSD terms for the QuadriFlow source. Whether QuadriFlow bundles any Triangle-derived code was not checked. Audit the repo's third-party directories (for example 3rd folder) before vendoring. [unverified]

## 7. xatlas, ufbx, OpenUSD, Alembic

### xatlas (jpcy/xatlas) [verified]
- LICENSE: MIT License, "Copyright (c) 2018-2020 Jonathan Young". Standard MIT text.
- README badge says MIT. README states it is an independent fork of thekla_atlas (used by The Witness). Parts derived from thekla_atlas/NVTT may carry their own notices; the repo-level LICENSE file does not list them. [unverified: check source headers]
- Note: mosra/xatlas is not the right repo (404); the upstream is jpcy/xatlas.

### ufbx (ufbx/ufbx) [verified]
- LICENSE: "available under 2 licenses -- choose whichever you prefer": Alternative A, MIT License, Copyright (c) 2020 Samuli Raivio; Alternative B, Public Domain (Unlicense). The README License section repeats the dual license.
- Read/import only (it is a loader). FBX export is out of scope for ufbx.

### OpenUSD (PixarAnimationStudios/OpenUSD, LICENSE.txt on release branch) [verified]
- License: "TOMORROW OPEN SOURCE TECHNOLOGY LICENSE 1.0". The file states it differs from Apache License 2.0 only in Section 6 (Trademarks). Section 6 reads: "This License does not grant permission to use the trade names, trademarks, service marks, or product names of the Licensor and its affiliates, except as required to comply with Section 4(c) of the License and to reproduce the content of the NOTICE file."
- The same LICENSE.txt is 895 lines because it also reproduces third-party notices (for example RapidJSON, whose text includes a note about the JSON-licensed jsonchecker directory under bin/jsonchecker). The OpenUSD distribution therefore carries multiple licenses; when linking pxr libraries, ship the third-party notices for the components actually built.
- Compatible with Apache-2.0 redistribution (Apache 2.0 patent grant retained; only trademark section differs). C++ library with Python bindings; a Rust binding would need FFI or a USD file writer. Alternatively, write USDA/USDC through a minimal own writer to avoid linking. [design note, unverified]

### Alembic (alembic/alembic, LICENSE.txt) [verified]
- File header: "SPDX-License-Identifier: BSD 3-Clause". Copyright lines: Lucasfilm Entertainment Company Ltd. (2009-2015) and Sony Pictures Imageworks Inc. (2009-2015); the redistribution clause forbids using the name of Industrial Light & Magic or contributors for endorsement. Standard BSD 3-clause disclaimer. No extra clause seen in the head and tail of the file. The middle of the file was not read in full. [verified for head and tail]

## 8. FAST and KES-F

### 8a. KES-F (Kawabata Evaluation System for Fabrics), maker documentation (Kato Tech)

Instruments: KES-FB1 tensile and shear, KES-FB2 bending, KES-FB3 compression, KES-FB4 surface. Only the tensile/shear (FB1-A) and bending (FB2) product pages were read; compression and surface were not. [verified for FB1 and FB2, unverified for FB3 and FB4]

Tensile (KES-FB1-A "How to read data: Tensile" PDF) [verified]:
- Sample 20 cm x 5 cm, gripped at the 20 cm edges, stretched along the 5 cm length (suppresses lateral contraction near the grips). Force F is per unit width in gf/cm. Strain epsilon = elongation / initial length x 100 (percent). Maximum force Fm = 500 gf/cm under standard conditions. Strain rate not given in the PDF.
- LT (linearity) = area(a + b) / area of triangle ABC, a dimensionless ratio of the forward curve area to the triangle spanned by the origin, the endpoint (Fm, epsilon_m) and the strain axis. Smaller LT means softer initial response.
- WT (tensile energy per unit area) = area under the forward curve to Fm.
- RT (resilience) = area b / (area a + b) x 100 percent (recovery work over tensile work).
- epsilon_m (the PDF symbol) is the elongation at Fm. The common KES name EMT was not defined in the PDF.

Shear (KES-FB1-A "How to read data: Shear" PDF) [verified for definitions; formulas not given]:
- G: shear stiffness, units gf/cm per degree; larger means stiffer in shear.
- 2HG: shear hysteresis at 0.5 degrees (gf/cm), larger means poorer recovery after small shear.
- 2HG5: shear hysteresis at 5 degrees (gf/cm).
- Test tension W = 10 gf/cm. Angles ±5 and ±8 degrees appear on the diagram. The PDF does not give an equation for G. In the KES literature G is the mean slope of the shear force vs. angle curve between 0.5 and 5 degrees [unverified here, background knowledge].

Bending (KES-FB2 product page) [verified for scope only]:
- Reproduces the hand movement of bending, yields bending rigidity and recoverability. Standard sample 200 mm x 200 mm; maximum width 200 mm, thickness 1 mm; 50 gf·cm upper measurement limit noted. The page does not define B or 2HB and gives no curvature range.
- From KES literature (background knowledge, unverified in this session): B is bending rigidity per unit width, the slope of bending moment M (gf·cm/cm) vs curvature K (1/cm) over roughly K = 0.5 to 1.5 cm^-1, in gf·cm^2/cm; 2HB is the hysteresis width at K = 0.5 cm^-1.

### 8b. Formulas mapping KES-F to simulation parameters

Derived from the verified definitions above plus unit conversions; the mapping choices are design proposals, not published results.

Unit conversion: 1 gf = 9.80665e-3 N; 1 cm = 1e-2 m.

Tensile stiffness (membrane stretch, force per unit width per unit strain, N/m):
- Secant stiffness at maximum load: K_t,secant = Fm / (epsilon_m / 100) in gf/cm, then multiply by 0.980665 to get N/m (since 1 gf/cm = 0.980665 N/m).
- Algebra from the verified definitions: WT = LT x Fm x epsilon_m / 2 (energy under the curve equals LT times the triangle area, with epsilon_m as a fraction when computing in force-per-width units). So LT and WT carry the same information once Fm and epsilon_m are known; the initial tangent stiffness can only be estimated from the curve itself, not from LT alone.
- Warp and weft are measured separately; the bias direction needs a separate bias test (which is what Wang et al. do in Section 2). KES-FB1 tensile in 0 and 90 degrees plus shear G give an orthotropic membrane (c11, c22, c33 equivalents via G) but not c12; Poisson coupling must be assumed or fit.

Shear stiffness: G in gf/cm per degree converts to N/m per radian by G x 0.980665 x (180/pi) = G x 56.19 (derived).

Bending rigidity (per unit width):
- B in gf·cm^2/cm converts to N·m by B x 9.80665e-3 N/gf x 1e-4 m^2/cm^2 x 1e2 cm/m = B x 9.80665e-5 N·m (derived from the unit definition and the background-knowledge unit of B).
- For a shell of thickness h, B_per_width = E h^3 / (12 (1 - nu^2)) links bending rigidity to an equivalent modulus; this is standard plate theory, not from KES documentation. In practice formfabric should use B directly as the hinge-energy stiffness per unit width.

### 8c. FAST (Fabric Assurance by Simple Testing, CSIRO)

Not verified. The CSIRO page fetch failed and the search budget was exhausted. From the first-pass report: FAST is a lower-cost alternative to KES measuring compression, bending, extensibility and dimensional stability. From background knowledge (all unverified here): FAST-1 compression meter (thickness at two loads, surface layer thickness, released thickness), FAST-2 bending meter (bending length, from which bending rigidity B = 9.8 x 10^-6 x W x C^3 mN·m per metre... where W is fabric mass per unit area and C bending length; unverified, check the CSIRO manual), FAST-3 extension meter (extensibility at 5, 20 and 100 gf/cm, with shear rigidity estimated from bias extension at 45 degrees), FAST-4 dimensional stability (relaxation shrinkage and hygral expansion). These must be confirmed against the CSIRO FAST manual before being written into product docs. The cantilever bending-length formula for rigidity per unit width, B = W x c^3 with c = l/2 for the Peirce cantilever test, is textbook (Peirce 1930) but was not source-checked here.

## Open items (could not close in this session)

1. Academic papers by CLO Virtual Fashion staff: not searched (budget).
2. Full text of Untangling Cloth: Pixar library redirects; try the ACM DL page or an author-hosted PDF.
3. WKWebView WebGPU exposure and minimum macOS version: needs a real test on macOS 26 (navigator.gpu in a Tauri window) or a WebKit statement about system WebKit.
4. WebView2 WebGPU default-on confirmation: check the Microsoft WebView2 docs on supported web platform features.
5. WebKitGTK WebGPU: confirm in WebKit tracking pages; nothing in 2.52 or 2.54 notes.
6. Wang 2011 dataset license: ask the authors or read the download page.
7. FAST formulas and KES bending formulas: obtain CSIRO FAST manual and Kawabata's KES-FB2 reference.
8. Patent claims: only page summaries were read; US 2016/0314604 and US 2019/0304200 did not load.
9. QuadriFlow bundled third-party code (Shewchuk co-author, possible Triangle-derived or other bundled libraries).
10. Alembic and OpenUSD third-party notices: audit exactly which components get built.

## Sources

- SIGGRAPH History Archive, "Untangling cloth" by Baraff, Witkin and Kass: https://history.siggraph.org/?p=115940
- Stanford Broad Area Colloquium abstract (Michael Kass, spring 2003): https://graphics.stanford.edu/courses/ba-colloquium/previous/spring03/abst-kass.html
- Pixar Graphics library entry (redirects, not read): https://graphics.pixar.com/library/UntanglingCloth
- Course reading summary of Untangling Cloth (secondary, University of Wisconsin): https://research.cs.wisc.edu/graphics/Courses/777-s2006/TextWeb/Summaries/Week11/krill-untangle.txt
- Wang, O'Brien, Ramamoorthi 2011 full text on eScholarship: https://escholarship.org/content/qt7g35q2x7/qt7g35q2x7.pdf
- SIGGRAPH History Archive entry for Wang et al. 2011: https://history.siggraph.org/?p=114658
- Justia, patents assigned to CLO virtual fashion: https://patents.justia.com/assignee/clo-virtual-fashion
- Google Patents: https://patents.google.com/patent/US20220267938A1/en , https://patents.google.com/patent/US20180025539A1/en , https://patents.google.com/patent/US20190304182A1/en , https://patents.google.com/patent/US10810794B2/en , https://patents.google.com/patent/US10789774B2/en , https://patents.google.com/patent/US11280036B2/en
- WebKit blog, News from WWDC25 (Safari 26 beta): https://webkit.org/blog/16993/news-from-wwdc25-web-technology-coming-this-fall-in-safari-26-beta/
- WebKit blog, WebKit features in Safari 26.0: https://webkit.org/blog/17333/webkit-features-in-safari-26-0/
- Apple Safari 26 release notes: https://developer.apple.com/documentation/safari-release-notes/safari-26-release-notes
- Chrome developers, WebGPU release: https://developer.chrome.com/blog/webgpu-release
- Microsoft, WebView2 Runtime release notes: https://learn.microsoft.com/en-us/microsoft-edge/webview2/release-notes/runtime/
- WebKitGTK news: https://webkitgtk.org/news.html , https://webkitgtk.org/2026/09/16/webkitgtk-2.54-highlights.html , https://webkitgtk.org/2026/03/18/webkitgtk-2.52-highlights.html
- University of Utah, Vertex Block Descent project page: https://graphics.cs.utah.edu/research/projects/vbd/
- Gaia repository and LICENSE: https://github.com/AnkaChan/Gaia , https://raw.githubusercontent.com/AnkaChan/Gaia/main/LICENSE
- Instant Meshes LICENSE: https://raw.githubusercontent.com/wjakob/instant-meshes/master/LICENSE.txt
- QuadriFlow LICENSE: https://raw.githubusercontent.com/hjwdzh/QuadriFlow/master/LICENSE.txt
- xatlas LICENSE and README: https://raw.githubusercontent.com/jpcy/xatlas/master/LICENSE , https://raw.githubusercontent.com/jpcy/xatlas/master/README.md
- ufbx LICENSE and README: https://raw.githubusercontent.com/ufbx/ufbx/master/LICENSE , https://raw.githubusercontent.com/ufbx/ufbx/master/README.md
- OpenUSD LICENSE: https://raw.githubusercontent.com/PixarAnimationStudios/OpenUSD/release/LICENSE.txt
- Alembic LICENSE: https://raw.githubusercontent.com/alembic/alembic/master/LICENSE.txt
- Kato Tech KES-FB1-A page: https://english.keskato.co.jp/products/kes-fb/
- Kato Tech KES-FB2 page: https://english.keskato.co.jp/products/kes-fb2/
- Kato Tech tensile data guide: https://english.keskato.co.jp/contents/EN_FB1-A_Tensile.pdf
- Kato Tech shear data guide: https://english.keskato.co.jp/contents/EN_FB1-A_Shear.pdf
