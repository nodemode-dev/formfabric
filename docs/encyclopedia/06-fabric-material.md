# Marvelous Designer / CLO: Fabrics, Physical Properties and Materials

Research slice 06 for the formfabric encyclopedia.

## Summary

Marvelous Designer (MD) and CLO share one engine and one manual lineage. The official MD manual (support.marvelousdesigner.com) was re-published in 2025/2026 and carries the same Fabric section as the CLO manual (support.clo3d.com). Both were used. Every claim below is from those official manuals unless tagged otherwise.

Key structural facts:

1. A Fabric is an object in the Object Browser Fabric tab. It bundles three things: Physical Property (simulation), Material (rendering, front/back/side), and Information (name, item no., supplier). It is saved as a .zfab file. Patterns reference a fabric; many patterns share one fabric.
2. Physical Property has two layers: a Preset layer (a small number of primary controls: Buckling Ratio, Density/Weight, and others) and a Detail layer (per-direction Stretch, Shear, Bending, Buckling Stiffness, Internal Damping, Friction, Thickness). Detail values can be typed either as a 0 to 99 slider scale or as a raw "constant value" 0 to 1,000,000,000.
3. The manual is deliberately thin on numbers: it almost never states defaults, units for slider scales, or formulas. The slider scale to real-world (N/m, gsm) mapping is NOT documented in the pages found. Anything about unit conversion below is flagged as unverified.
4. Material system is PBR (Base Color, Normal, Displacement, Opacity, Roughness, Reflection, Metalness) with a Type enum that selects a V-Ray-style shading model (Matte, Shiny, Silk/Satin, Velvet, Denim, Fur, Gem, Glass, Glitter, Light, Leather, Metal, Plastic, Skin/SSS, Iridescence), plus Substance (SBSAR) as an alternative material source.
5. Print on Fabric is a child object of a fabric (image with repeat type, spacing, face, transform) distinct from Graphic objects on patterns.

Confidence legend: [single source] means only one approved page; [unverified] means from general knowledge or a non-approved source, not confirmed in an approved page; [gap] means the manual does not state it.

---

## 1. Fabric object model

| Item | Detail | Source |
|---|---|---|
| Where fabrics live | Object Browser, Fabric tab. Select a fabric item and its properties appear in the Property Editor. | MD Set Fabric Textures; CLO Set Fabric Materials |
| Cross highlight | Clicking a fabric in the list highlights its patterns in 2D/3D; clicking a pattern highlights its fabric in the Fabric tab. | MD Set Fabric Textures |
| Create | Add button (top right of Fabric tab) creates a "Default Fabric". | CLO Add & Apply Fabric Style |
| Load from library | Open the Fabric Library and double-click a .zfab or drag it into the Fabric tab. | CLO Add & Apply Fabric Style |
| Assign | Select one or more patterns and click Assign beside the fabric; or drag the fabric onto a pattern in 2D or 3D. | CLO Add & Apply Fabric Style |
| Groups | Create New Folder (top left of Fabric tab) makes fabric groups; Add Fabric / Open Fabric Files buttons per group; rename via Edit Folder Name; drag fabrics between groups. | CLO Add & Apply Fabric Style |
| Delete | Only allowed if the fabric is not assigned to any pattern. Delete icon, right-click Delete, or Delete key. | CLO Add & Apply Fabric Style |
| Section topics (MD manual) | Fabric Style Window, Fabric List Window, Create Additional Fabric, Copy Fabric, Delete Fabric, Reset Fabric Property, Open/Save Fabric File, Apply Fabric, Preview Fabric Information, Rename Fabric, Front/Back/Side Setting, Set Fabric Textures, Apply Texture, Normal Map, Opacity Setting, Edit Texture, Edit Texture Size, Refresh Texture, Delete Texture, Express Cartoon, Print on Fabric. | MD Fabric section index |
| Physical topics (MD manual) | Physical Property Preset, Detail Setting, Stretch-Weft/Warp/Shear, Bending-Weft/Warp, Buckling Ratio, Buckling Stiffness, Internal Damping, Density/Weight (Ver 2024.1). | MD Fabric section index |

### 1.1 Fabric Information fields

| Field | Meaning |
|---|---|
| Information button (ver 6.1) | Opens the fabric file's revision history. |
| Name | Shown in the Fabric List. |
| Item No. | Free item number. |
| Classification | Free text for content and classification. |
| Supplier/Owner (ver 6.0) | Auto-filled from values entered when the fabric was created in Emulator mode. |

Previewable in the Library. Source: CLO Fabric Information.

### 1.2 Pattern level overrides tied to fabric

| Property | Location | Notes |
|---|---|---|
| Grain Direction | Pattern Properties, Fabric, Grain Direction | Exact grainline angle. "Link All Colorways" can be turned off to give a different grain direction per colorway. |
| Add'l Thickness-Rendering | Pattern selected, Property Editor, Simulation Properties (ver 2.3.0) | Per-pattern render thickness on top of fabric thickness. |
| Add'l Thickness-Collision | Same panel | Per-pattern collision thickness addition. |

Source: CLO Edit Texture, Adjust Rendering Thickness, Adjust Collision Thickness.

### 1.3 Fabric operations from the Edit Texture tool right-click

Assign to New Fabric; Reset to Basic Fabric (restores the basic fabric's properties); Reset Fabric Angle; Align Grain (parallel to X, Y or custom axis); Match up Grain (ver 7.3, aligns grain so texture flows smoothly between a selected segment and the baseline; on symmetric patterns matches to the symmetric line); Delete Texture; Flip Texture Horizontally/Vertically. Source: CLO Edit Texture.

---

## 2. Physical Property: structure

### 2.1 Preset versus Detail

| Layer | Contents | Notes |
|---|---|---|
| Preset | Buckling Ratio, Density/Weight (labelled "Density" before 2024.1, "Weight" from 2024.1, unit gsm). The manual paths show Buckling Ratio and Density/Weight under "Physical Property > Preset". | MD Buckling Ratio and Density/Weight pages |
| Detail | Stretch (Weft, Warp, Shear), Bending (Weft, Warp), Buckling Ratio (Weft/Warp), Buckling Stiffness (Weft/Warp), Bias variants of stretch/bending/buckling (ver 5.1.0), Internal Damping, Friction, Density, Thickness (rendering and collision) | Detail exists "to express Physical Properties not included in Presets". |

Important conflict found: the MD page for Buckling Ratio gives its location as "Physical Property > Preset > Buckling Ratio", while the older CLO page and the CLO Detail article treat the group as Detail. The MD 2025 UI evidently shows a reduced Preset panel with the most common controls and a Detail expander. Treat both as the same underlying parameter. [gap on exact Preset panel membership and the list of named presets]

The list of built-in named presets (for example Cotton, Silk, Denim) is not published in any approved page fetched. The CLO Fabric Library ships .zfab files; the manual examples only name cotton, denim, silk, jersey, wool, leather as fabric types, not as preset names. A community post titled "The list of fabric presets has become cumbersome" exists on the MD support site (confirms a preset list exists and is long) but it is a forum post and was not used. [gap]

### 2.2 Input methods (CLO Physical Property Detail Setting)

| Method | Range | Use |
|---|---|---|
| Scroll bar (slider) | 0 to 99 | "The most commonly used property is split into 99 stages." Coarse adjustment. |
| Constant value (numeric field) | 0 to 1,000,000,000 | Precise settings, "actual measurement values". |

The page says properties interact: garment behavior comes from the combination of all detail values. Units and defaults are not stated. [gap]

Clone implication: store every parameter canonically as a raw numeric constant; derive a 0 to 99 slider as a monotonic (likely nonlinear) mapping. The exact curve is not documented. [gap]

---

## 3. Physical Property parameters in full

Menu path for all detail parameters: Object Browser > Fabric tab > Fabric Property Editor > Physical Property > Detail.

| Parameter | Meaning (manual wording, paraphrased) | Range | Default | Effect guidance | Source |
|---|---|---|---|---|---|
| Stretch Weft | Resistance to horizontal (weft yarn direction) stretch. Higher = stiffer, stretches less. | Slider 0 to 99; constant to 1e9 | [gap] | Match Shear ratio to Weft/Warp for crease-prone cotton/denim. | MD + CLO Stretch page |
| Stretch Warp | Resistance to vertical (warp yarn direction) stretch. | same | [gap] | Weft and Warp are independent so anisotropic fabrics work. | same |
| Shear | Resistance to diagonal stretch (in-plane shear). | same | [gap] | Raise toward Weft/Warp for materials that crease easily (cotton, denim). Lower for elastic, drapey materials (silk, jersey). | same |
| Stretch Bias | Bias (45 degree) stretch variant. Exists since ver 5.1.0 (named in the Bending page as a companion). | [gap] | [gap] | For bias-cut fabric. | CLO Bending/Buckling pages name "Bending-Bias" and "Buckling Ratio-Bias (ver 5.1.0)" |
| Bending Weft | Resistance to bending, sets stiffness. High = stiff (denim, leather). Low = drapes (silk). Weft and warp independent. | slider/constant | [gap] | | CLO Bending page |
| Bending Warp | As above, along warp. | same | [gap] | | same |
| Bending Bias | Separate bending control for bias direction, ver 5.1.0 or later. | [gap] | [gap] | | CLO Bending page |
| Buckling Ratio (Weft/Warp, plus Bias since 5.1.0) | How strongly the fabric "tries to keep its shape" when force bends it, i.e. onset of buckling/wrinkling. 0 to 100 percent. Near 100 percent: easily bendable (silk, jersey). Near 0 percent: less bendable (denim, wool). | 0 to 100 percent | [gap] | A user reported denim in the Library near 90 to 95, contradicting the description. Treat the manual's semantics as unreliable. | MD + CLO Buckling Ratio |
| Buckling Stiffness (Weft/Warp, Bias since 5.1.0) | Stiffness of folded corners as a percentage of Bending. Effective bending at folds = Buckling Stiffness percent times Bending. Example: 80 percent of Bending 60 = 48. Higher = corners harder to fold; lower = easier to fold. | percent; range [gap] | [gap] | | CLO Buckling Stiffness |
| Internal Damping | Resistance of the garment to its own bouncing motion. High = sluggish "underwater" motion; low = bouncy. Effect is visible only during animation, not when merely worn/static. | [gap] | [gap] | Tune during animation. | CLO Internal Damping |
| Density / Weight | Fabric weight per unit area. Unit: gsm (g/m2). Higher = heavier. Labelled Density before 2024.1, Weight from 2024.1 (Preset panel). Older CLO page says only "area in m2", weight unit unnamed. | [gap] | [gap] | | MD Density/Weight; CLO Adjust Density |
| Friction Coefficient | Friction between garment and garment (the CLO page explicitly says garment to garment). Path Detail > Friction. | [gap] | [gap] | A 2019 comment asking for optimum values is unanswered. Avatar friction is a separate avatar-side setting (not covered by these pages). | CLO Adjust Friction Coefficient |
| Thickness (Rendering) | Visual thickness measured from the garment center. Cosmetic only, does not affect simulation, so layers do not push apart. Needs Garment Rendering Style = Thick Textured Surface. | [gap] | 0.5 mm | Fabric-level field "Thickness" in Property Editor; per-pattern "Add'l Thickness-Rendering". | CLO Adjust Rendering Thickness |
| Thickness (Collision / Simulation) | Invisible collision limit that keeps simulation smooth. Now uses the same thickness direction as rendering thickness. Per-pattern "Add'l Thickness-Collision" under Simulation Properties. | [gap] | [gap] (avatar Skin Offset default 3 mm) | For form-fitting or fit measurement, also adjust avatar Skin Offset. | CLO Adjust Collision Thickness |

### 3.1 Parameter interplay and modelling hints for a clone

| Concept | What the manual implies | Implication |
|---|---|---|
| Anisotropy | Weft, Warp, Shear stretch and Weft, Warp bending are separate; bias variants added in 5.1.0. | The fabric model is orthotropic: 2 axes plus shear, plus an extra bias set for 45 degree behaviour. |
| Nonlinear bending | Buckling Ratio plus Buckling Stiffness define a two-regime bending curve: normal stiffness until a buckling threshold, then reduced stiffness (percentage of Bending) in folded zones. | Implement bending energy with a threshold angle (ratio) and post-buckle stiffness factor (stiffness percent). Inferred, mapping to the exact formula is [gap]. |
| Damping | Only visible in motion. | Rayleigh-like or velocity damping term; not active in static drape. |
| Rendering thickness vs collision thickness | Explicitly separated: render is cosmetic, collision is simulated. | Keep two independent fields. |

### 3.2 Competing product definitions (secondary, not CLO)

Style3D Studio documents a closely analogous parameter set (cited only for comparison, not an approved CLO source): Stretch (warp, weft, bias), Bending (warp, weft, bias), Bending Buckling Rate (warp, weft, bias), Bending Buckling Stiffness (reduction ratio of bending resistance in wrinkled areas), Dynamic Friction and Static Friction (unitless), Grammage in GSM, Thickness in mm. It also offers "General" (adjustable) and "Nonlinear" (non-adjustable, for knit and silk) simulation models, with no numeric ranges. Useful because it confirms CLO's Buckling Stiffness semantic as "reduction ratio in wrinkled areas". [single source, competitor]

---

## 4. Unit conversion to real-world values

### 4.1 What the approved pages say

| Quantity | Statement | Status |
|---|---|---|
| Density/Weight | Unit is gsm. | Verified (MD 2024.1 page) |
| Rendering Thickness | Millimetres, default 0.5 mm. | Verified |
| Avatar Skin Offset | Default 3 mm. | Verified |
| Slider scale | 0 to 99 for the most common property; constant value 0 to 1,000,000,000. | Verified |
| Stretch/Bending units | Not stated. The Fabric Kit page says Stretch and Bending are measured; no unit given. | [gap] |
| Fabric Kit accuracy | Fabrics made with the Fabric Kit are about 90 percent accurate; accuracy can drop if properties are hand-edited. | Verified (CLO page) |

### 4.2 What is NOT documented

The official pages do not publish: a formula from slider 0 to 99 to N/m, N/m to constant values, Kawabata KES-F or FAST mapping, nor the fabric-to-density conversion (gsm to the CLO density constant). Third-party community threads discuss these but are forum content and were excluded. A formfabric implementation must therefore define its own documented mapping, and should be wary of claiming CLO parity.

### 4.3 Peer-reviewed context for bending (helpful for designing the converter)

From a Materials Science (Kaunas) paper comparing FAST and KES-F bending rigidity (it does not mention CLO):

| Item | Value |
|---|---|
| FAST bending rigidity | B = W * c^3 * 9.807e-6 (W area density g/m2, c bending length mm); result in microNewton metres. Bending length is half the overhang when the specimen deflects to 41.5 degrees. |
| KES-F pure bending B | Slope of the hysteresis curve between curvature +/-0.5 and +/-1.5 per cm; reported in 1e-4 N m2/m (or 1e-6 N m2/m). |
| Heavy outerwear regression | B_KESF = 1.37 * B_FAST - 3.27 (R2 0.99). Lightweight shirting: 0.88 * B_FAST - 0.60. Mappings depend on fabric group and do not transfer across weights. |

[single source]. Use: a clone's "Fabric Emulator" can accept cantilever bending length and gsm, compute bending rigidity via the FAST formula, then map into its own solver units. Whether CLO does this internally is [unverified].

### 4.4 Suggested canonical clone representation (inference, not CLO fact)

| Field | Canonical unit | Notes |
|---|---|---|
| density | g/m2 (gsm) | Matches MD UI. |
| stretch_weft, stretch_warp, stretch_shear (and bias) | N/m (membrane stiffness) | Present as raw constant plus 0 to 99 UI scale. |
| bending_weft, bending_warp, bending_bias | N m (bending rigidity per width) or uN m | Matches FAST/KES-F. |
| buckling_ratio | 0 to 1 (UI shows 0 to 100 percent) | |
| buckling_stiffness | 0 to 1 fraction of bending | Per CLO formula 80 percent times 60 = 48. |
| internal_damping | dimensionless | |
| friction | unitless coefficient | |
| thickness_render, thickness_collision | mm | Default render 0.5 mm. |

---

## 5. Fabric Kit, Emulator, CLOFAB, digitization

| Topic | What is verified | Gap |
|---|---|---|
| Fabric Kit | A physical CLO product (hardware kit) for creating digital fabrics by measuring real materials ("emulate"). Measures Stretch and Bending plus "other Physical Properties". About 90 percent accuracy. Purchase via CLO Global Support. Page also said a fabric digitization service was being prepared. | Measurement protocol, units, which tests, the equation. [gap] |
| Emulator mode | Referenced by Fabric Information: the Supplier/Owner field is auto-filled from values entered when the fabric is created in Emulator mode (ver 6.0). So the app has an Emulator creation flow in the fabric editor with supplier metadata. | UI details, input fields. [gap] |
| CLO Fabric Digitization Service | An article exists titled "Non-testable Fabrics for CLO Fabric Digitization Service", implying a service that scans/tests physical swatches with restrictions on which fabrics can be tested. | Page content not fetched. [gap] |
| CLOFAB | An article titled "CLOFAB" exists in the Fabric section. | Fetch failed. [gap] |
| Knit Swatch Editor / Realistic Knitwear Designer | Articles exist for creating knit fabrics. | Not fetched. [gap] |
| Vizoo | A "Vizoo File Compatibility" and "VIZOO U3M Plugin Instructions" article exist: import of Vizoo scanned material files. | Details not fetched. [gap] |
| AxF import | "Import AxF file" article (X-Rite material format). | Not fetched. |
| DMIx cloud | "Connection to DMIx Cloud & Compatible with Material (ver 6.0)". | Not fetched. |

Clone implication: a first version can offer a manual "measure" wizard (gsm, cantilever bending length, stretch at a given load) rather than hardware. Document it as formfabric's own method.

---

## 6. File formats (.zfab and interchange)

| Format | Content | Source |
|---|---|---|
| .zfab | CLO/MD fabric file: the fabric item with physical properties and materials. Library double-click or drag loads it. Saving a fabric as ZFAB also saves its print properties. | CLO Add & Apply Fabric Style; Print on Fabric |
| .prt | Print properties saved/opened separately. | CLO Print on Fabric |
| glTF / GLB | Standard fabric file (ver 6.1). Stores the fabric PBR material and CLO physical property data. Export via File > Export > Fabric (GLTF/GLB) or right-click the fabric item; import via File > Import > GLTF/GLB or drag-drop, with object type set to Fabric. Substance (SBSAR) fabrics cannot be exported because of licensing. | CLO glTF page |
| SBSAR | Substance material source, applied as the fabric material. | MD Set Fabric Textures |
| AxF, U3M (Vizoo) | Imported scanned materials. | CLO section index |
| ACO, ASE, CCO, QTX | Color palette import; save as ACO or CCO (CCO is CLO-internal). | CLO Color Window |

Internal .zfab binary layout: not documented in any approved page. [gap] glTF is the documented open container, and the page confirms physical property data rides along, so a clone should adopt glTF with an extension block as its fabric exchange format and optionally write a CLO-compatible reader later only if a spec is found.

---

## 7. Material (rendering) properties

Menu path: Object Browser > Fabric tab > select fabric > Property Editor > Material. Tabs for Front, Back, Side.

### 7.1 Front/Back/Side

| Behaviour | Detail |
|---|---|
| Three face tabs | Front, Back, Side configured separately. |
| Back/Side inherit | On Back or Side, the toggle "Use Same Material as Front" is on by default. Turn it off to assign a different material. An option "Use Same Color as Front" matches only color. |
| Swap | After turning off "Use Same Material as Front" on Back, a Swap Front & Back button appears. |
| Surface colors | Color is per face tab via Base Color. |
| Back-texture editing | If Garment Rendering Style = Back Texture Surface, the Edit Texture tool edits the back of the texture. |

Source: CLO Set Fabric Materials; MD Set Fabric Textures; CLO Edit Texture.

### 7.2 Material types (Material > Type)

| Type | Description | Extra parameters |
|---|---|---|
| Fabric_Matte | General fabric | none listed |
| Fabric_Shiny | Shiny fabric | none listed |
| Fabric_Silk/Satin | Silk or satin | none listed |
| Fabric_Velvet | Velvet | none listed |
| Denim | Denim wet-wash effects | separate guide (Express Denim Wet Wash) |
| Fur (Render Only) | Realistic fur, V-Ray render | see section 9 |
| Fur_Strand (Beta, 2025.0) | Fur previewed in the 3D window; Windows only | see section 9 |
| Gem (Render Only) | Light reflecting in multiple colors like glass | Absorption (opacity relative to thickness), Abbe (increases/reduces reflection) |
| Glass | Glass | Absorption (opacity relative to thickness) |
| Glitter (Render Only) | Glitter | separate guide (Express Glitter, ver 5.0.0) |
| Light (Render Only) | Garments or objects act as light sources | Color Mult (brightness; higher colors shift from the set color) |
| Leather | Leather | none listed |
| Metal | Metallic materials | none listed |
| Plastic | Plastics | none listed |
| Skin (Render Only) | Subsurface scattering (human skin, leaves, fruit, candles) | section 8 |
| Iridescence (Render Only) | Angle dependent colored reflection | section 10 |

The MD manual notes MD has no render engine, so 3D viewport display of these types is not accurate; "Render Only" types appear only in the V-Ray render window. [Source: MD Set Fabric Textures; CLO Material Type.] Substance is an alternative to the PBR source (choose via the Material right menu).

### 7.3 PBR channel reference

| Channel | Controls | Notes and ranges |
|---|---|---|
| Base Color: Color | Color wheel picker or (None) | Opens Color Window (section 12). |
| Base Color Map | Texture image from Library/folder. A user setting can convert the image to sRGB on load. | |
| Recolor | Removes default color so other colors can be applied while keeping the pattern/texture. | Sub-modes: Desaturate (grayscale; adjust Shadow Intensity and Shadow Brightness), Switch / Color Switch (change color of solid fabric; Base Color Map should contain one color; the Texture Editor is recommended for multi-color maps). |
| Remove Texture Color | Removes default color to allow other colors while keeping texture. | |
| Normal Map: Automatic | Generates a normal map from the texture. Intensity: higher raises details, lower recesses them. | Range [gap] |
| Normal Map: Manual | Loads a normal map image. A second normal map can add extra surface pattern. | |
| Displacement Map | Thickness or bumpiness. Amount (max height; white = max), Shift (distance between map and garment surface), Clipping (deletes geometry below this value), Particle Distance (spacing of new mesh; avoid values below 1 over large areas due to memory), Keep Continuity (extends adjacent faces to remove boundary lines; Welded OBJ only). | Ranges [gap]. The ver 5.1.0 displacement article failed to fetch. |
| Opacity | Overall transparency value for sheer fabric. With a texture, set Garment Rendering Style to Thick Textured Surface. | Range [gap] |
| Alpha / Opacity Map | Grayscale image. RGB Mode: alpha encoded in RGB channels. Alpha Mode: alpha encoded in the alpha channel. | |
| Roughness (Gloss) Intensity | Gloss through roughness value. Range expanded in 2024.0; legacy values auto-converted (example: 50 becomes 100). | |
| Roughness Map and Map Intensity | Browse to add map; Map Intensity strength; range expanded 2024.2. | |
| Invert | Reverses gloss. Check for Specular maps from versions before 6 (MD text) or before 4.0.0 (CLO text); the two manuals disagree on the version. | |
| Reflection Intensity | Higher = more reflective. | |
| Metalness | Higher = more metallic. | |
| Metalness Map | Grayscale; white = metallic. | Tip: with a Metalness Map, set Roughness Intensity about 10 and Reflection Intensity about 90 (CLO example). For matte areas, use the same map as Roughness Map with Invert on. |
| Texture Mapping (Ver 11) / Repeat Type (Ver 10) | Repeat tiles the texture across the pattern. Unified Map maps by UV layout; Repeat U / Repeat V size the texture (1 = once across the 0 to 1 UV range). | |
| Substance | SBSAR file, Preset (from the SBSAR), Resolution, Custom Properties (file specific). High-Quality Render recommended. | |

Sources: MD Set Fabric Textures, CLO Set Fabric Materials, CLO Material Type.

### 7.4 PBR Map Generator (ver 2024.1) and Import All PBR Maps

| Tool | Behaviour |
|---|---|
| PBR Map Generator | Input: a base texture image. Select Texture Property Editor > each PBR map's properties > PBR Map Generator and these are auto-generated: Normal, Displacement, Opacity, Roughness, Metalness. No user options listed. The generation method is not stated (likely AI based). [gap] |
| Import All PBR Maps Once | Article exists in the CLO section index; not fetched. [gap] |
| Texture Generator (ver 7.1) | Article exists; not fetched. [gap] |

---

## 8. Subsurface scattering (Skin, Render Only)

Path: Fabric Property Editor > Material > Type > Skin (Render Only). Visible only in the V-Ray render window.

| Parameter | Meaning |
|---|---|
| Mix | Amount of light scattering. 0 = off (surface reflection only). Maximum = full scattering, no surface reflection. |
| Mix Map (ver 7.1) | Varies scattering across the surface. |
| Density Scale | Scales the scattering radii of all layers; lower spreads the effect wider. Change only overall size and keep radius ratios among layers consistent. |
| Shallow / Medium / Deep | Three scattering layers, each with Color, Color Map, Weight, Radius. |
| Color / Color Map | Color of scattered light; map sets area specific colors. |
| Weight | Layer intensity and color/map influence relative to others; 1 = highest. |
| Radius | How far each layer scatters beneath the surface. |

Known issue: overlapping meshes can render incorrectly. Avoid displacement maps with this type (V-Ray compatibility). Numeric ranges not stated except 0 to 1 implied for Mix and Weight. Source: CLO Subsurface Scattering Material; CLO Material Type.

---

## 9. Fur

Two material types: Fur (Render Only), and Fur_Strand (Beta, new in 2025.0, Windows only, previews in 3D window with Default, Quality and Interactive Render). Same parameter set. Presets for Fox, Mink, Angora, Sherpa.

| Group | Parameter | Meaning | Example values from manual |
|---|---|---|---|
| Fur | Melanin | Higher = browner, darker | |
| Fur | Pheomelanin | Higher = redder; no effect when Melanin is 0 | |
| Fur | Glossiness | Intrinsic shininess | |
| Fur | Glossiness Boost | Coated or waxed shine on top | |
| Fur | Softness | Higher = smoother, lower = rougher | |
| Fur | Compensate Energy | Compensates insufficient light bounces in dense hair | |
| Gradation Color | Color (mid, tip), Position, Interpolation | Enable via checkbox | |
| Base | Length | Longer at higher values | 50 vs 10 |
| Base | Thickness | Strand thickness | 10 vs 0.20 |
| Base | Bend | Bend under gravity | 1 vs 0.20 |
| Base | Taper | Thinner toward tips | 1.00 vs 0.00 |
| Base | Density | Denser vs scattered | 1 vs 0.20 |
| Base | Segments | Sections per strand; more for long strongly bent fur | |
| Maps | Monochrome maps for length, thickness, density, curl, bend, gravity (black = x0, white = x1); RGB maps for initial direction and bend direction (normal map like) | | |
| Cluster Generator | Length (Cropping Map: Mode, Level, Low/High Length), Trimming Intensity, Random, Bend Direction Cluster Intensity, Pores Density, Direction Intensity Random, Detail Cluster, Relax | | |
| Gravity | Force; Vector (per axis) | | |
| Curl | Radius, Number | | 5.00 vs 1.00; 5.00 vs 0.00 |
| Variance | Length, Thickness, Direction, Gravity (gravity only when Bend not 0) | Higher = more irregular | |
| Randomization (Fur_Strand) | Seed | | |

Fur_Strand display toggles: 3D Garment Display > Show Fur Strand, Random Fur Strand Color. User Settings > 3D > Fur Strand Shape: Cylinder (thickness and width), Plane (width only), Line (spline, no thickness/width). Export: File > Export > USD with "Include Fur Strand Data". No min/max ranges are documented. Source: CLO Fur (Render Only) and Fur_Strand pages.

---

## 10. Other special materials

| Material | Details |
|---|---|
| Iridescence (Render Only, ver 6.0) | Color 1 to 5 (angle dependent reflective areas), Roughness, Hue Shift (shifts all 5 colors), Weight (strength). Only visible in render window. |
| Glitter (Render Only, ver 5.0.0) | Article "Express Glitter" exists; not fetched. [gap] |
| Denim wet wash | "Express Denim Wet Wash" article exists; not fetched. [gap] |
| Express Cartoon | Listed in MD Fabric section; not fetched. [gap] |
| Gem | Absorption, Abbe. |
| Glass | Absorption. |
| Light | Color Mult. |

---

## 11. Texture placement and transform

### 11.1 Edit Texture tool (2D/3D toolbar or Main Menu > Materials > Texture > Edit Texture)

| Action | How |
|---|---|
| Move texture | Select the tool; left-click drag the texture image on the pattern. |
| Scale | Select a pattern with fabric; a gizmo appears at the top right of the 2D window. Drag diagonal lines for uniform scale; horizontal or vertical lines for single axis scale. Or type Width and Height in Property Editor > Material > Texture Transformation. |
| Rotate | Drag the gizmo circle; right-click to enter exact rotation and grainline values. |
| Rotate grainline | Hover the grain curve until a circle appears and drag; re-simulate to update drape. Exact angle: Pattern Properties > Fabric > Grain Direction. |
| Flip | Right-click Flip Texture Horizontally / Vertically. |
| Hotkeys | None listed. |

Texture rotation is tied to the grainline: rotating the texture also drives the simulation grain, so the fabric's Weft/Warp physical axes follow the pattern grain. Source: CLO Edit Texture.

### 11.2 Texture Editor (ver 7.x, via the More (...) icon next to a map)

| Area | Controls |
|---|---|
| Transformation | Pivot, Position (relative to pivot), Size/Angle, Reset (position, size, angle), Flip |
| Repeat | Repeat, Space/Shift (gap and offset between copies), Extend to Other Side (seamless wrap across grid edge) |
| Layers | Add (image; import AI or PDF artboards/layers), Copy, Delete, Lock, Show, Layer Order (drag), Rename (double-click) |
| Canvas | Size (width, height), Unit, PPI (Pixels), Transparent Background |
| Image properties | Paint Blend (blend with layer beneath), Color, Color Switch Number (number of colors extracted), Sub Color (edit extracted chips), Opacity, Desaturate |
| Preview | Show Grid, Occlude Opacity (opacity of repeated copies in preview) |
| Output | Apply & Close; Save Image as PNG, JPEG or TIFF, or merged layers |

No numeric limits are documented. Source: CLO Texture Editor.

### 11.3 Other texture commands

Apply Texture, Refresh Texture (reload after external edit), Delete Texture, Edit Texture Size (Property Editor), Open Textures in AI/PS (ver 6.1). Show/Hide Texture was removed after MD 10. Custom texture image articles exist: "Create Custom Texture Image" and for curved patterns. Not fetched.

---

## 12. Graphic and Print placement

Two distinct mechanisms:

| Mechanism | Scope | Notes |
|---|---|---|
| Print on Fabric | Child "print" item under a fabric in the Fabric tab; applies wherever that fabric is used. | Add Print (select an image), Copy Print, Print Properties. |
| Graphic (object) | Placed on specific pattern(s) as separate objects | The MD "Graphic Style Properties" page exists (support.marvelousdesigner.com article 47358146871065) but was not fetched. [gap] |

### 12.1 Print Properties

| Group | Field | Values |
|---|---|---|
| Information | Name | Rename by click, right-click Rename, or Property Editor. |
| Configuration | Face | Front, Back, or Both |
| Configuration | Repeat Type | Block, Half Drop, Brick, Diamond, Stripe |
| Configuration | Space | X and Y gap between images |
| Configuration | Shift | X and Y offset |
| Material | Use Same PBR Maps as Fabric | Automatically applies and blends fabric Normal, Displacement, Opacity, Roughness, Metalness with the print; individual map toggles are also available. |
| Type | Print image type | Option list not given. Can apply fabric normal map to print. |
| Transformation | Width, Height, Angle | Edit Texture tool edits fabric and print together; unavailable if the fabric has no texture. |
| Open/Save | .prt file | Saving the fabric as ZFAB also saves print properties. |

Source: CLO Print on Fabric.

---

## 13. Colorways and color

| Topic | Verified | Gap |
|---|---|---|
| Colorway concept | MD text: Material > Color "lets you define multiple colorways for each fabric." CLO Edit Texture: "Link All Colorways" can be disabled so each colorway has its own grain direction. | Colorway Window UI, creation, duplication, per-colorway overrides (which properties are per colorway: color, texture, print, graphic). [gap, the Color Window page says it "does not mention colorways"] |
| Color Window | Opens from the Color row of Fabric, Graphic, Button, Buttonhole, Topstitch, Puckering, Avatar, Trim, Zipper, imported OBJs. For avatars, shoes, hair, zippers and imported files select in 3D window and use Property Editor. | |
| Custom Palette | Pick from field or enter values; eyedropper (app or screen); add selected color or library color to the palette; import ACO, ASE, CCO, QTX; save ACO or CCO; reset; List/Detail/Icon views; swatch size; rename by double-click; reorder by drag; Delete key removes; search by name or code. | |
| Color Libraries | Default palette, Coloro 3500, Pantone, YKK Global Colors. | |
| Pantone libraries | Color Bridge Coated V5, Color Bridge Uncoated V5, Pastels and Neons Coated, Pastels and Neons Uncoated, FHI Polyester TSX, FHI Cotton TCX 2801, CMYK Coated, CMYK Uncoated, Solid Coated V5, Solid Uncoated V5, f+h nylon brights TN, METALLICS Coated. | Licensing and the numeric values are proprietary, a clone cannot bundle Pantone data. |
| Quick apply | Ver 7.1+: with the color window open, select another object and the color applies immediately. | |

Source: CLO Color Window.

Clone note: ship open color systems (sRGB, hex, Lab) plus importers for ACO and ASE (open-ish formats); treat Pantone, Coloro and YKK as user supplied.

---

## 14. Operating workflow summary (what a user does)

| Task | Steps |
|---|---|
| Apply a library fabric | Open Library > fabric folder > double-click .zfab (or drag to the Fabric tab) > select patterns > Assign (or drag onto pattern) > Simulate. |
| Tune drape | Select fabric > Physical Property > Preset/Detail > change Weight, Bending, Buckling Ratio/Stiffness, Shear > re-simulate. |
| Make it bouncy or sluggish | Detail > Internal Damping, test with animation. |
| Add texture | Fabric > Material > Base Color Map > explore > choose image; Normal Automatic Intensity; adjust repeat/scale in Texture Transformation or Edit Texture gizmo. |
| Realign pattern/grain | Edit Texture tool > rotate gizmo or Pattern Properties > Grain Direction; re-simulate. |
| Different back side | Material > Back tab > untick Use Same Material as Front. |
| Make metal | Type = Metal, or Metalness map + Roughness about 10 + Reflection about 90. |
| Sheer fabric | Opacity or Alpha map; set Garment Rendering Style to Thick Textured Surface. |
| Save fabric | Open/Save Fabric File to .zfab (also saves prints). |
| Exchange fabric | File > Export > Fabric (GLTF/GLB). |

---

## 15. Build guidance for formfabric (distilled, partly inference)

1. Data model: Fabric { info, physical{ stretch{weft,warp,shear,bias}, bending{weft,warp,bias}, buckling_ratio{weft,warp,bias}, buckling_stiffness{weft,warp,bias}, internal_damping, friction, density_gsm, thickness_render_mm=0.5, thickness_collision }, material{type, front, back, side}, prints[] }. Material face has base_color, base_color_map, recolor{mode}, normal{auto|manual, intensity}, displacement{amount, shift, clipping, particle_distance, keep_continuity}, opacity{value, map, channel_mode}, roughness{intensity, map, map_intensity, invert}, reflection, metalness{value, map}.
2. Two numeric entry modes per physical parameter (slider 0 to 99 and raw value) with a documented mapping of your own.
3. Fabric-level vs pattern-level overrides: grain direction, additional render and collision thickness per pattern.
4. Separate render thickness from collision thickness.
5. Back and Side faces default to inherit Front, with swap.
6. Prints are fabric children with repeat types Block, Half Drop, Brick, Diamond, Stripe, spacing, shift, face.
7. Interchange: glTF/GLB with physical property extension; a native JSON/zip fabric file; PBR map generator as a pluggable step.
8. Remaining numeric defaults of CLO cannot be sourced from the manual; plan to derive ranges empirically from your own solver or from user supplied .zfab files (analysis of .zfab binary is out of scope here and could not be verified).

---

## 16. Unresolved gaps (do not invent)

| Gap | Why it matters |
|---|---|
| Named preset list and each preset's numeric values | Needed for a built-in library. |
| Defaults for Stretch, Bending, Buckling, Damping, Friction, Density | Needed for Default Fabric. |
| Slider (0 to 99) to constant value curve and real units | Needed for unit conversion. |
| .zfab binary or XML layout | Needed for file compatibility. |
| Colorway data model | Needed for colorways. |
| Graphic object properties (MD article 47358146871065) | Needed for graphic placement. |
| Fabric Kit measurement procedure; CLOFAB; Digitization Service | Needed for Emulator. |
| Displacement Map article (fetch failed), Glitter, Denim Wet Wash, Express Cartoon, Knit Swatch Editor | Specialty materials. |

---

## Sources (URLs fetched)

Marvelous Designer official manual (support.marvelousdesigner.com):

- https://support.marvelousdesigner.com/hc/en-us/sections/51986752872217-Fabric
- https://support.marvelousdesigner.com/hc/en-us/articles/47358420149017-FABRIC-PHYSICAL-PROPERTIES-Adjust-Stretch-Weft-Warp-Shear
- https://support.marvelousdesigner.com/hc/en-us/articles/47358420908313-FABRIC-PHYSICAL-PROPERTIES-Adjust-Buckling-Ratio
- https://support.marvelousdesigner.com/hc/en-us/articles/47358364198425-Set-Fabric-Textures
- https://support.marvelousdesigner.com/hc/en-us/articles/47358460243737 (Density/Weight)

CLO Virtual Fashion official manual (support.clo3d.com):

- https://support.clo3d.com/hc/en-us/sections/115000714287-FABRIC and ?page=2 (article index)
- https://support.clo3d.com/hc/en-us/articles/115002685527-Adjust-Buckling-Stiffness
- https://support.clo3d.com/hc/en-us/articles/115002797808-Adjust-Buckling-Ratio
- https://support.clo3d.com/hc/en-us/articles/115000483087-Adjust-Stretch-Weft-Warp-Shear
- https://support.clo3d.com/hc/en-us/articles/115000483187-Adjust-Bending-Weft-Warp
- https://support.clo3d.com/hc/en-us/articles/115000512908-Adjust-Internal-Damping
- https://support.clo3d.com/hc/en-us/articles/115002797988-Adjust-Density
- https://support.clo3d.com/hc/en-us/articles/115002797948-Adjust-Friction-Coefficient
- https://support.clo3d.com/hc/en-us/articles/115002685867-Adjust-Rendering-Thickness
- https://support.clo3d.com/hc/en-us/articles/115002798068-Adjust-Collision-Thickness
- https://support.clo3d.com/hc/en-us/articles/115000483047-Physical-Property-Detail-Setting
- https://support.clo3d.com/hc/en-us/articles/115013660747 (Fabric Kit)
- https://support.clo3d.com/hc/en-us/articles/115015998728-Material-Type
- https://support.clo3d.com/hc/en-us/articles/360047317454-Set-Fabric-Materials
- https://support.clo3d.com/hc/en-us/articles/360053351314-Texture-Editor
- https://support.clo3d.com/hc/en-us/articles/115002680348-Edit-Texture
- https://support.clo3d.com/hc/en-us/articles/360053434434-Print-on-Fabric
- https://support.clo3d.com/hc/en-us/articles/360055193313-Subsurface-Scattering-Material
- https://support.clo3d.com/hc/en-us/articles/360001526568-How-to-Create-Fur-Material-Type-Fur-Render-Only
- https://support.clo3d.com/hc/en-us/articles/44205065227801-How-to-Create-Fur-Material-Type-Fur-Strand-Beta
- https://support.clo3d.com/hc/en-us/articles/360047316974-Color-Window-How-do-I-change-colors-in-my-project
- https://support.clo3d.com/hc/en-us/articles/360047319234-Fabric-Information
- https://support.clo3d.com/hc/en-us/articles/360056634954-Iridescence-Material-Expression-ver-6-0
- https://support.clo3d.com/hc/en-us/articles/900005384946-glTF-a-Standard-Fabric-File-Compatibility-ver-6-1
- https://support.clo3d.com/hc/en-us/articles/360047313154-Add-Apply-Fabric-Style
- https://support.clo3d.com/hc/en-us/articles/33852438561049-PBR-Map-Generator-ver-2024-1

Peer-reviewed and comparative:

- https://matsc.ktu.lt/index.php/MatSc/article/view/26098/13286 (FAST vs KES-F bending rigidity, Materials Science)
- https://help.style3d.com/studio/en/c74f/3365/ce60/664f (Style3D Studio physical property, comparison only, not an MD source)

Attempted but failed: CLO Displacement Map ver 5.1.0, CLO CLOFAB, PMC9773953 (captcha).
