# 16. Fabric Data: Conversions, Measured Table, Presets, Fabric Lab Formulas

Scope: SI parameter data for the formfabric fabric model (crate ffb-fabric). Every number is either (a) quoted from a fetched approved source (peer-reviewed paper or textbook overview), (b) converted from one with a shown derivation, or (c) derived with a stated assumption and tagged DERIVED. Tags: [single source] means one independent source only. [units inferred] means the source did not print units and standard KES-F units were assumed. [suspect] means the number conflicts with physics or another row and is not used for a preset.

Approved-source note: the Wang, O'Brien, Ramamoorthi 2011 and Miguel et al. 2012 papers were reachable only as publication listings or a partial text; their per-fabric stiffness parameter tables live in supplemental material that could not be fetched, so only their fabric list and areal densities are used (see S-1). ASTM D1388, ISO 13934 and ISO 9073 standard texts were not fetched (paywalled or not retrieved); formulas attributed to D1388 are verified through the peer-reviewed papers that apply it (S-4, S-7, S-9, S-11).

Language: warp is mapped to formfabric "warp" and weft to "weft". For knits, wale maps to warp and course maps to weft. Bias is 45 degrees.

---

## 1. Unit conversions with derivations

Constants: 1 gf = 9.80665e-3 N. 1 cm = 1e-2 m. 1 cm2 = 1e-4 m2.

### 1.1 KES-F bending rigidity B (gf cm2 / cm) to uN m per metre width

B is moment per unit width per unit curvature, units force x length (gf cm2/cm = gf x cm).

1 gf cm2/cm = 9.80665e-3 N x 1e-4 m2 / 1e-2 m = 9.80665e-5 N m = 98.0665 uN m.

Formula: B[uN m] = 98.0665 x B_KES[gf cm2/cm].

Cross-check: S-7 states KES-F values are reported in 1e-4 N m2/m (that is 1e-4 N m = 100 uN m, matching 98.07 to within the rounding of g). S-3 table: KES B 30.01 uN m for sample ST8 warp equals 0.306 gf cm2/cm, in the normal range for wool suiting.

### 1.2 KES-F tensile (EMT, LT, WT) to tensile stiffness N/m

KES-FB1 tensile test runs to a maximum load of 500 gf/cm of specimen width (S-12, Adamu 2021: "tensile test to 500 gf/cm").

Fmax = 500 x 9.80665e-3 N / 1e-2 m = 490.33 N/m.

Definitions: EMT = strain at Fmax (percent; fraction e = EMT/100). WT = tensile energy per unit area, gf cm/cm2. LT = WT / (Fmax x e / 2) (dimensionless linearity; LT = 1 is a straight line).

WT conversion: 1 gf cm/cm2 = 9.80665e-3 N x 1e-2 m / 1e-4 m2 = 0.980665 N m/m2, which is numerically N/m. (S-6 Table 3 prints WT already in N m/m2.)

Secant stiffness to Fmax (call it k_500):

- With EMT: k_500 = 490.33 / (EMT/100) [N/m].
- Without EMT (only LT and WT): e = 2 WT / (Fmax LT), so k_500 = Fmax^2 x LT / (2 WT) = 240394 x LT / (2 WT[N/m]) [N/m].

Important limit: woven fabrics have LT below 1 (stiffening curve), so the true stiffness at tiny strain (crimp region, the "low-strain region" in the formfabric model) is LOWER than k_500, and stiffness at Fmax is higher. k_500 is the only stiffness that KES summary parameters support. For the low-strain value use FAST E5 (section 1.4) or the Fabric Lab strip test (section 4). Under a power-law curve F = a e^n, LT = 2/(n+1), the tangent at strain e is n x secant, and the secant at strain e is the Fabric Lab measurement.

### 1.3 KES G (gf / cm degree) to shear stiffness N/m

KES-FB1 reports shear force per unit length per shear angle: F_s[gf/cm] = G x phi[deg] (S-12 gives G in gf/cm.deg).

G_SI[N/m per radian] = G x 9.80665e-3 N / 1e-2 m x (180/pi deg/rad) = G x 0.980665 x 57.2958 = 56.188 x G.

Shear strain gamma = tan(phi), approximately phi in radians for the KES angle range, so G_SI is the shear stiffness in N/m per unit shear strain.

Example: G = 0.52 gf/cm.deg gives 29.2 N/m.

### 1.4 FAST (CSIRO) conversions

From S-5 (ScienceDirect overview of FAST) and S-8:

- E5, E20, E100 are warp and weft extensibility (percent) at 5, 20 and 100 gf/cm, equal to 4.90, 19.61 and 98.07 N/m (5 x 9.80665e-3 / 1e-2 = 4.903; 19.613; 98.067).
- Stiffness from FAST extensibility: k_E100 [N/m] = 98.07 / (E100/100). Likewise k_E5 = 4.903 / (E5/100) and k_E20 = 19.61 / (E20/100). k_E5 is the closest published proxy for the low-strain stiffness formfabric wants.
- Bias extension EB5 is measured at 5 gf/cm. Shear rigidity G[N/m] = 123 / EB5[percent] (S-5; the same relation is used in S-9 and S-10). G here is already in N/m per unit shear strain.
- FAST-2 bending length BL (mm) is measured at 41.5 degrees. Bending rigidity BR[uN m] = 9.8e-6 x W[g/m2] x BL[mm]^3 (S-5; S-7 prints 9.807e-6; S-9 prints B = W x C^3 x 9.807e-6 in uN m).
- FAST B vs KES B (S-7, 80 wool/other outerwear fabrics, both in uN m): heavy wool outerwear B_KES = 1.37 B_FAST - 3.27 (R2 0.99, W 211 to 398 g/m2); light shirting B_KES = 0.88 B_FAST - 0.60 (R2 0.94, W 96 to 170 g/m2, quoted there from Yick et al., second-hand). Check: ST8 warp FAST 25.00 predicts KES 30.98, measured 30.01 (S-3).

### 1.5 Cantilever bending length C (ASTM D1388, Peirce) to bending rigidity

Peirce cantilever: a strip overhangs until its tip reaches the 41.5 degree line; overhang length l; bending length c = l/2 (S-7). The general Peirce relation (S-4) is c^3 = G / (W g), and for overhang L and tip angle theta, c^3 = (L^3/8) cos(theta/2) / tan(theta). At theta = 41.5 degrees the trig factor is cos(20.75)/tan(41.5) = 0.9351/0.8847 = 1.057, so the exact c is 1.9 percent above L/2; the standard (and the FAST instrument) uses c = L/2.

Flexural rigidity per unit width: G_bend = W_area x g x c^3, with W_area the mass per unit area.

SI: B[N m] = rho_A[kg/m2] x 9.80665 x c[m]^3.

Practical units: W in g/m2 and c in mm give B[uN m] = 9.807e-6 x W x c^3. Derivation: W g/m2 = 1e-3 W kg/m2; c mm = 1e-3 c m; B = 1e-3 W x 9.807 x 1e-9 c^3 = 9.807e-12 W c^3 N m = 9.807e-6 W c^3 uN m. Matches S-5, S-7, S-9.

With c in cm and B in mN mm (= uN m): B = 9.8e-3 x W x C^3 (S-11, Shirley tester). Do not mix up: in S-11 C is bending length; if a Shirley instrument scale actually reads overhang length l, B is overstated by 2^3 = 8. See [suspect] flags in section 2.

Areal weight conversion: 1 oz/yd2 = 33.906 g/m2 (exact unit conversion, 28.3495 g / 0.836127 m2).

Bias bending from warp and weft: B45 is close to the geometric mean sqrt(B_warp x B_weft). Evidence: S-11 mean of 84 fabrics warp 89.6, weft 33.98, bias 54.75 uN m (geometric mean 55.2); maximum row 791.6, 366.3, 477.4 (538.5); minimum row 3.24, 1.16, 1.88 (1.94). [single source]

Effective bending modulus (sanity check): E_b = 12 B / T^3 [Pa], with B in N m and T in m. S-11 Eq. 2 is the same relation with T measured at 100 gf/cm2 (their formula text is garbled in extraction).

---

## 2. Measured data table

Units normalized to SI. Weight in g/m2, thickness in mm, B in uN m, shear in N/m, tensile k_500 in N/m (computed per 1.2 from printed LT and WT). Directions: Wp warp, Wf weft (for knits: wale/course). Where I converted from the printed units, the printed value is in brackets.

### 2.1 Woven, cotton

| Row | Fabric | Source | W | T | B Wp / Wf / bias | Shear | Tensile k_500 | Notes |
|---|---|---|---|---|---|---|---|---|
| M-1 | Cotton woven plain shirting (C-08208) | S-2 | 125.2 [12.52 mg/cm2, units inferred] | 0.610 | not reported (2HB 0.0277 only) | 29.2 [G 0.52] | 7385 (LT 0.597, WT 9.91; EMT calc 6.6 percent) | direction not given; [single source] [units inferred] |
| M-2 | Cotton woven plain suiting (C-08209) | S-2 | 246 [24.60] | 1.022 | not reported | 88.8 [G 1.58] | 7787 (LT 0.667, WT 10.50; EMT calc 6.3 percent) | [single source] [units inferred] |
| M-3 | Cotton twill 3/1, S3 (49 tex, 16 picks/cm) | S-9 | 196 | 0.99 | 28.7 / 23.4 / bias n.a. | 62.4 (FAST) | n.a. | FAST-2 and FAST-3 |
| M-4 | Cotton twill 3/1, S1 | S-9 | 172 | 0.93 | 19.1 / 12.5 | 27.6 | n.a. | |
| M-5 | Cotton twill 3/1, S2 | S-9 | 173 | 0.92 | 15.9 / 14.9 | 32.6 | n.a. | |
| M-6 | Cotton twill 3/1, S4 | S-9 | 207 | 0.94 | 34.4 / 35.7 | 110.2 | n.a. | |
| M-7 | Denim twill, finished, D15 to D20 (mean of 6) | S-9 | 398 (392 to 402) | 1.099 (1.048 to 1.162) | 97.9 / 114.9 (range 88.8 to 103.1 / 90.4 to 127.4) | 688 (615 to 820) [suspect, see note a] | n.a. | FAST |
| M-8 | Denim twill, unfinished, D5 to D14 | S-9 | 346 to 384 | 1.09 to 1.31 | 215.7 to 373.7 / 161.2 to 267.1 | 1230 to 1836 [suspect] | n.a. | |
| M-9 | Denim 11 oz black (99 cotton, 1 spandex) | S-1 | 324 [0.324 kg/m2] | n.a. | n.a. | n.a. | n.a. | areal density only; Wang 2011 |
| M-10 | Denim 3/1 Z twill, weft MIU (friction coefficient vs KES sensor) | S-12 | n.a. | n.a. | n.a. | n.a. | n.a. | MIU weft 0.164 to 0.258, warp 0.134 to 0.218 depending on weft yarn; printed order inconsistent in the paper [single source] |
| M-11 | Cotton basket weave canvas-like, IM_6 gray | S-6 | 379.9 | 1.33 | 48 / 50 [0.48e-4, 0.50e-4 N m2/m] | n.a. | 5623 (Wp), 14248 (Wf) from LT 0.69/0.78, WT 14.75/6.58 | KES |
| M-12 | Cotton basket weave, IM_6 dyed + washed | S-6 | 368.8 | 1.12 | 77 / 52 | n.a. | 10535 (Wp), 10603 (Wf); EMT calc 4.65 / 4.62 percent | KES |
| M-13 | Cotton plain, A-1 | S-13 | 155 | n.a. | bar chart only | n.a. | n.a. | |
| M-14 | Plain woven cotton voile-like | S-10 | 72.0 | 0.28 | n.a. | n.a. | bias Pmax 530 N/m at 36.6 percent (aspect 2) | |

### 2.2 Woven, linen (flax), KES, plain unless noted (S-6)

| Row | Code and finish | W | T | B Wp / Wf | Tensile k_500 Wp / Wf | EMT calc Wp / Wf (percent) |
|---|---|---|---|---|---|---|
| L-1 | IM_1 dyed + chemical softening | 158.7 | 0.41 | 41 / 32 | 9754 / 9199 | 5.0 / 5.3 |
| L-2 | IM_2 mechanically softened | 161.3 | 0.48 | 20 / 12 | 9086 / 12270 | 5.4 / 4.0 |
| L-3 | IM_3 dyed + mechanical softening | 170.2 | 0.49 | 22 / 17 | 7726 / 11483 | 6.4 / 4.3 |
| L-4 | IM_4 thick, dyed + mechanical | 433.6 | 1.31 | 49 / 51 | 8760 / 3623 | 5.6 / 13.5 |
| L-5 | IM_5 weft rib, chem + mech | 216.6 | 0.93 | 36 / 20 | 12957 / 3114 | 3.8 / 15.7 |
| L-6 | IM_1 to IM_5 gray (unfinished) | 152 to 355 | 0.53 to 1.30 | 31 to 413 / 43 to 524 (several marked as manually calculated, bending curve did not reach 0.5 cm^-1) | not used | not used |

Printed shear: the paper gives only relative changes (softening cut G by 81 to 94 percent); no numeric G. Printed tensile gray LT of IM_4 and IM_5 are identical in the source (possible transcription error).

### 2.3 Woven, wool and wool blend (S-3, Ancutiene et al. 2010; KES B converted from 1e-4 N m2/m; all broken twill)

| Row | Fabric | W | T | B KES Wp / Wf | B FAST Wp / Wf |
|---|---|---|---|---|---|
| W-1 | ST8 80 wool / 20 polyamide | 251 | 1.16 | 30.01 / 22.48 | 25.00 / 18.58 |
| W-2 | ST10 80 wool / 20 polyamide | 244 | 1.01 | 17.93 / 11.57 | 17.11 / 11.19 |
| W-3 | SM 50 wool / 50 acetate | 211 | 1.55 | 12.85 / 8.22 | 10.60 / 7.62 |
| W-4 | R 80 wool / 20 polyester (heavy outerwear, coating weight) | 398 | 2.28 | 73.53 / 41.34 | 55.64 / 32.20 |

No tensile or shear data for these rows in S-3 (the paper states this). Consistency: FAST B from W-2: c = (17.11 / (244 x 9.807e-6))^(1/3) = 19.3 mm.

### 2.4 Silk, chiffon, taffeta, fleece, leather

No fetched approved source gave measured bending, tensile or shear for these. Only areal densities of related fabrics:

| Row | Fabric | Source | W | Note |
|---|---|---|---|---|
| X-1 | Silk 60 / polyester 40 plain woven, 145 gsm, 120 x 135 threads | S-13 | 145 | bending length reported only as bar chart, rank highest of six fabrics, no numbers |
| X-2 | Tango Red Jet Set, 100 percent polyester fashion dress fabric | S-1 | 113 [0.113 kg/m2] | not specifically taffeta |
| X-3 | White dots on black, 100 percent polyester tablecloth | S-1 | 128 | |
| X-4 | Lightest woven in an 84-fabric set (row I2, plain, fibre not stated) | S-11 | 45.9 | B 3.24 / 1.16 / 1.88 uN m (warp / weft / bias), but see note b |
| X-5 | Pink Ribbon Brown 100 percent polyester blanket (fleece-like) | S-1 | 228 | |

Leather: no measured data in any fetched source. No density, thickness or bending data found.

### 2.5 Knits

| Row | Fabric | Source | W | T | B Course (Wf) / Wale (Wp) | Shear | Tensile k_500 |
|---|---|---|---|---|---|---|---|
| K-1 | Cotton single jersey TF 11, dry (20 Ne ring) | S-14 | 152 | n.a. | 1.49 / 1.13 [0.0152 / 0.0115 gf cm2/cm] | G 0.368 / 0.300 gf/cm.deg = 20.7 / 16.9 N/m | n.a. |
| K-2 | same, TF 13, wash and dry | S-14 | 176 | n.a. | 3.19 / 1.54 [0.0325 / 0.0157] | 25.7 / 24.7 N/m [0.458 / 0.440] | n.a. |
| K-3 | same, TF 14, wash and dry | S-14 | 193 | n.a. | 4.34 / 1.59 [0.0443 / 0.0162] | 29.8 / 29.3 [0.530 / 0.522] | n.a. |
| K-4 | same, TF 16, wash and dry | S-14 | 211 | n.a. | [0.1656 / 0.0255; course value looks like a typo, suspect] / 2.50 | 40.8 / 37.3 [0.726 / 0.664] | n.a. |
| K-5 | Cotton single jersey, innerwear (C-08206) | S-2 | 125.5 [12.55, units inferred] | 0.919 | n.a. | 26.4 [G 0.47] | 2363 (LT 0.632, WT 32.78; EMT calc 20.7 percent) |
| K-6 | 95 cotton / 5 PU single jersey outerwear (C-08207) | S-2 | 158.6 [15.86] | 0.943 | n.a. | 27.0 [G 0.48] | 15487 (LT 0.849, WT 6.72; EMT calc 3.2 percent) [suspect: WT is 5x lower than K-5 for a jersey with elastane] |
| K-7 | Ivory rib knit 95 cotton / 5 spandex | S-1 | 276 | n.a. | n.a. | n.a. | n.a. |
| K-8 | Gray interlock 60 cotton / 40 polyester | S-1 | 187 | n.a. | n.a. | n.a. | n.a. |
| K-9 | Navy sparkle sweat 96 polyester / 4 spandex | S-1 | 224 | n.a. | n.a. | n.a. | n.a. |
| K-10 | White swim solid 87 nylon / 13 spandex | S-1 | 204 | n.a. | n.a. | n.a. | n.a. |

Qualitative only (no numbers printed): wool knits plain, rib, purl (S-15): rib has the highest wale bending, is most easily sheared and has high course EMT; purl has the highest wale EMT.

Wang et al. 2011 (S-1) fitted parameters (c11, c12, c22, c33 as functions of strain and angle; bending curve per bias angle) exist only as plots with unlabeled stretching units and a bending axis labeled N/m (ambiguous); they are not usable as table values here. Method summary (for Fabric Lab design): biaxial test with weights up to 600 g (bottom) and 400 g (side), 400 x 400 mm samples at 0, 45, 90 degrees; bending by cantilever drape test on 4 cm wide strips at 0, 45, 90 degrees.

Notes
- a. FAST shear in S-9 for denim (615 to 1836 N/m) and the mean 1436 N/m in S-11 are an order of magnitude above the same instrument's value for cotton twill S4 (110 N/m) and above KES G for heavy fabrics (a KES G of 3 gf/cm.deg is 169 N/m). G = 123/EB5 would need EB5 of 0.18 percent for 688 N/m, which is implausibly small for bias extension at 5 gf/cm. These are not used for presets.
- b. S-11 prints B = W C^3 x 9.8e-3 with C in cm from a Shirley tester. Mean C of 4.0 cm for a 124 g/m2 fabric (about 2.0 cm if C is l/2) suggests C may be the overhang length l, in which case B is overstated 8x. The warp : weft : bias ratios are still usable (1 : 0.38 : 0.61 for the mean row).

---

## 3. Recommended formfabric presets

Fields: tensile weft / warp / shear in N/m; bending weft / warp / bias in uN m; areal density g/m2; thickness mm. Buckling, friction and damping: see end of section. Tensile numbers are k_500 secant stiffness (section 1.2) unless marked; they overstate the stiffness in the low-strain region for woven fabrics, so use the Fabric Lab low-strain measurement (section 4) to override whenever available.

Derivation rules used when no measured row exists:
- R1 thickness from bulk density: T = W / rho_b. Sourced bulk densities: 205 kg/m3 (M-1 poplin), 387 kg/m3 (L-1 finished linen), 137 kg/m3 (K-5 jersey). Higher value used for filament or dense fabrics, stated per preset.
- R2 bending from effective bending modulus: B = E_b T^3 / 12. E_b sourced: 0.355 MPa (M-3 twill), 0.209 MPa (W-2 suiting), 0.885 MPa (finished denim M-7), 0.658 MPa (M-12 canvas), 2.24 MPa (L-3 linen), 17.8 kPa (K-2 jersey, T from K-5 bulk).
- R3 tensile and shear default: nearest measured fabric of the same structure class.

| # | Preset | W | T | Tensile weft / warp / shear | Bending weft / warp / bias | Status | Traceability |
|---|---|---|---|---|---|---|---|
| 1 | cotton_poplin_shirting | 125 | 0.61 | 7400 / 7400 / 29 | 5.5 / 6.7 / 6.1 | PARTIAL | W, T, tensile, shear from M-1 [single source, units inferred]. Bending DERIVED: R2 with E_b of M-3, T = 0.61: 0.355e6 x (0.61e-3)^3 / 12 = 6.7 uN m warp; weft = warp x (23.4/28.7 from M-3) = 5.5; bias = geometric mean 6.1. Typical KES shirting B is about 5 uN m, consistent. |
| 2 | cotton_twill | 196 | 0.99 | 7400 / 7400 / 62 | 23.4 / 28.7 / 25.9 | MEASURED (tensile DERIVED) | M-3 (W, T, B, shear). Bias = sqrt(28.7 x 23.4) (rule from S-11). Tensile borrowed from M-1 (R3). |
| 3 | denim_heavy_finished | 398 | 1.10 | 7400 / 7400 / 210 | 115 / 98 / 106 | MEASURED W, T, B; shear and tensile DERIVED | M-7 means of D15 to D20. Shear: S4 value 110.2 scaled by W (398 / 207) = 212, used instead of the suspect measured 688 (note a). Tensile from M-1. Friction 0.2 from M-10 (KES MIU, steel sensor, not fabric on fabric) [single source]. |
| 4 | linen | 170 | 0.49 | 11500 / 7700 / 29 | 17 / 22 / 19 | MEASURED W, T, B, tensile; shear DERIVED | L-3 (dyed + mechanically softened, 170.2 g/m2). Bias = sqrt(22 x 17). Shear 29 from M-1 (R3), because S-6 gives no numeric G. Unfinished (gray) linen is much stiffer (rows L-6). |
| 5 | wool_suiting_worsted | 244 | 1.01 | 9500 / 9500 / 62 | 11.6 / 17.9 / 14.4 | MEASURED W, T, B; tensile and shear DERIVED | W-2 (ST10). Bias = sqrt(17.93 x 11.57). Tensile 9500 = median of the 12 linen k_500 values (L rows), shear from M-3: no wool tensile or shear table was found in fetched sources. |
| 6 | wool_coating | 398 | 2.28 | 9500 / 9500 / 62 | 41.3 / 73.5 / 55.1 | MEASURED W, T, B; tensile and shear DERIVED | W-4 (R). Bias = sqrt(73.53 x 41.34). Wool flannel is not measured: SM (W-3: 211, 1.55, B 8.2 weft / 12.9 warp, bias 10.3) is the nearest brushed-wool-like thick light row (it is wool / acetate), used as proxy only. |
| 7 | silk_charmeuse_crepe | 145 | 0.37 | 9500 / 9500 / 29 | 1.5 / 1.5 / 1.5 | DERIVED | W from X-1 (silk / polyester 145 gsm; true charmeuse is lighter, set W from your own sample). T = 145 / 387 (R1, densest sourced) = 0.37. B: R2 with E_b of M-3: 0.355e6 x (0.37e-3)^3 / 12 = 1.5 uN m; likely a lower bound because X-1 had the highest bending length in S-13 among six fabrics. No measured silk B, tensile or shear found. |
| 8 | chiffon_georgette | 46 | 0.12 | 9500 / 9500 / 29 | 0.4 / 0.4 / 0.4 | DERIVED | W from X-4 (lightest woven in S-11, fibre unknown). T = 45.9 / 387 = 0.12 (R1). B bounds: R2 gives 0.05; X-4 as printed gives 3.2 (about 0.4 if C was overhang length, note b). Preset 0.4 is the geometric mean sqrt(0.05 x 3.2) = 0.40, isotropic. Measure your sample with Fabric Lab. |
| 9 | polyester_taffeta | 113 | 0.29 | 9500 / 9500 / 29 | 0.7 / 0.7 / 0.7 | DERIVED | W from X-2 (polyester dress fabric, not confirmed taffeta). T = 113 / 387 = 0.29. B: R2 with E_b of M-3 gives 0.72 uN m. Isotropic assumption. |
| 10 | cotton_jersey_knit | 176 | 1.29 | 2360 / 2360 / 25 | 3.2 / 1.5 / 2.2 | MEASURED W, B, shear; T and tensile from another sample | K-2 (W 176, B 3.19 course / 1.54 wale, G 25.2 N/m mean). T = 176 / 137 kg/m3 (K-5 bulk) = 1.29 (R1). Tensile from K-5 k_500 (EMT 20.7 percent). Bias = sqrt(3.19 x 1.54) = 2.2. Knit tensile stiffness is strongly nonlinear; k_500 overstates the low-strain stiffness by a large factor. |
| 11 | rib_knit | 276 | 2.0 | 2360 / 2360 / 25 | 12 / 12 / 12 | DERIVED | W from K-7 (95 cotton / 5 spandex). T = 276 / 137 = 2.0 (R1). B: R2 with jersey E_b (17.8 kPa): 17.8e3 x (2.0e-3)^3 / 12 = 11.9 uN m. Tensile and shear from K-5 / K-2 (R3); S-15 says rib shears more easily and extends more in course, so shear should be lower and weft tensile lower than jersey. Low confidence. |
| 12 | fleece | 228 | 1.67 | 2360 / 2360 / 25 | 6.9 / 6.9 / 6.9 | DERIVED | W from X-5 (polyester blanket, proxy for fleece). T = 228 / 137 = 1.67 (R1). B: R2 with jersey E_b = 6.9 uN m. Tensile and shear from jersey (R3). Low confidence. |
| 13 | spandex_stretch_jersey | 204 | 1.49 | 2360 / 2360 / 25 | 4.9 / 4.9 / 4.9 | DERIVED | W from K-10 (nylon / spandex swim, 204). T = 204 / 137 = 1.49. B: R2 jersey E_b = 4.9. Tensile and shear borrowed from jersey; a real elastane knit is much more extensible at the same load, so tensile must be lowered from a measured E (Fabric Lab) before use. |
| 14 | leather_garment | not available | not available | not available | not available | NO DATA | No density, thickness, bending or tensile data in any fetched source. Fabric Lab must supply all inputs. |
| 15 | canvas_cotton | 369 | 1.12 | 10600 / 10500 / 196 | 52 / 77 / 63 | MEASURED W, T, B, tensile; shear DERIVED | M-12 (basket weave 369 g/m2 dyed + washed). Shear: S4 110.2 x 369 / 207 = 196 (scaling by W, crude). Bias = sqrt(77 x 52). Plain duck canvas not separately measured; basket weave is the nearest. |

Counts: 7 presets MEASURED in the core columns (areal density, thickness, bending): twill, denim, linen, wool suiting, wool coating, jersey, canvas (in all of these tensile or shear or thickness is partly derived, see table). 1 PARTIAL (poplin: bending derived). 6 DERIVED (silk, chiffon, taffeta, rib, fleece, spandex). 1 NO DATA (leather). Tensile and shear fully measured on the same fabric: none; the closest are poplin (tensile and shear from M-1) and linen (tensile).

Buckling: the formfabric buckling parameter has no definition in the fetched sources (no measured buckling data other than critical bias-tension buckling loads for one 72 g/m2 cotton voile-like fabric in S-10: 7.1 N/m at aspect ratio 2). Leave at engine default; do not fit from this data.

Friction: only one number exists (M-10 denim KES MIU about 0.13 to 0.26, a fabric against a steel-wire sensor, not fabric on fabric). No sourced friction for other types; leave at engine default.

Damping: no measured data in any fetched source. Derive from KES hysteresis (2HB, 2HG) if needed: S-6 gives 2HB (N m/m x 1e-4) for linen, S-2 gives 2HB and 2HG for cotton; these are not converted to a damping coefficient here because the mapping depends on the formfabric integrator.

---

## 4. Fabric Lab formulas

Inputs: areal density W (g/m2; if oz/yd2 multiply by 33.906), thickness T (mm), cantilever bending length c_warp, c_weft, c_bias (mm; c = overhang length l / 2 at 41.5 degrees), strip tensile force F (N) at strain eps for strip width w (m) in warp and weft, shear force F_s (N) at angle phi on a specimen of length L_s (m).

Outputs and formulas:

1. Bending rigidity per direction: B_dir [uN m] = 9.807e-6 x W x c_dir^3 (S-5, S-7, S-9; Peirce 1930 via S-4). If bias bending length is not measured: B_bias = sqrt(B_warp x B_weft) (evidence in S-11).
2. Optional KES-equivalent B (for matching KES-based data): wool outerwear heavier than 200 g/m2: B_KES = 1.37 B - 3.27; light shirting under 170 g/m2: B_KES = 0.88 B - 0.60; clamp at zero (S-7). Cantilever B differs from KES B because the cantilever samples a larger curvature range under self-weight, so for a drape-matched simulator prefer the cantilever value.
3. Tensile stiffness per direction: k_dir [N/m] = (F / w) / eps (secant at the test strain). Use a small strain, such as 1 to 2 percent, to approximate the low-strain region. For FAST: k = (5, 20 or 100 gf/cm converted to 4.903, 19.61, 98.07 N/m) / (E/100). For ISO 13934-1 type strip data (standard text not fetched) use the same secant at a chosen strain rather than the force at break.
4. Shear stiffness: k_s [N/m] = (F_s / L_s) / tan(phi) for a picture-frame test (shear force per unit length per shear strain). For KES: 56.188 x G. For FAST: 123 / EB5 (EB5 in percent at 5 gf/cm).
5. Quality checks: effective bending modulus E_b = 12 B / T^3 (Pa; T in m, B in N m) should lie in the sourced range 0.02 to 2.3 MPa; tensile modulus E_t = k / T. Flag the sample if shear stiffness exceeds about 0.1 of the average tensile stiffness (the 688 N/m denim shear in S-9 fails this: 688 / 7400 = 0.09 is borderline; the 1836 value fails), as a prompt to re-check units.
6. Anisotropy: store warp, weft separately; bias bending per rule 1; shear stiffness is symmetric in warp and weft.
7. Optional bias-extension shear when no picture-frame rig: use FAST relation in item 4 with a 5 gf/cm (4.903 N/m) bias load and EB5 measured on a 50 mm wide strip; the factor 123 is quoted from S-5 and S-9 without a derivation in the fetched text.

Worked example (poplin-like input): W = 125 g/m2, c_warp = 20 mm: B = 9.807e-6 x 125 x 8000 = 9.8 uN m; E_b = 12 x 9.8e-6 / (0.61e-3)^3 = 0.52 MPa, within the sourced range.

---

## 5. Sources (fetched)

- S-1: Wang, O'Brien, Ramamoorthi, "Data-driven elastic models for cloth: modeling and measurement", ACM TOG / SIGGRAPH 2011, text via https://escholarship.org/content/qt7g35q2x7/qt7g35q2x7.pdf (fabric list and method; per-fabric stiffness values are in supplemental material not obtained). Miguel et al. 2012 (https://cdfg.mit.edu/publications/data-driven-estimation-cloth-simulation-models) was reachable only as a listing; no data extracted from it.
- S-2: Fibre2Fashion hosted paper with KES-F table (cotton knit and woven fabrics A to D; table extracted without units; journal not identified): https://static.fibre2fashion.com/articleresources/PdfFiles/60/5963.pdf
- S-3: Ancutiene, Strazdiene, Nesterova, "The Relationship between Fabrics Bending Rigidity Parameters Defined by KES-F and FAST Equipment", Materials Science (Medziagotyra) 16(4), 2010: https://matsc.ktu.lt/index.php/MatSc/article/view/26098/13286 and https://matsc.ktu.lt/index.php/MatSc/article/download/26098/21680
- S-4: Analysis of the bending behaviour of flax based reinforcements used in shape forming (Peirce relations), https://arxiv.org/pdf/1312.3796
- S-5: ScienceDirect Topics overview, Fabric Assurance by Simple Testing (E5, E20, E100 loads; BR and shear formulas), https://www.sciencedirect.com/topics/engineering/fabric-assurance-by-simple-testing. Also FAST background (abstract only): https://www.emeraldinsight.com/insight/content/doi/10.1108/09556229510087146/full/html (Minazio 1995, IJCST 7(2/3)).
- S-6: Naujokaitye, Strazdiene, "The Effect of Finishing upon Textile Mechanical Properties at Low Loading", Materials Science 13(3), 2007: https://matsc.ktu.lt/index.php/MatSc/article/view/26340/13480
- S-7: same as S-3 (formulas section).
- S-8: NCSU Textiles KES overview (B, G, WT definitions and units): https://textiles.ncsu.edu/tpacc/comfort-performance/kawabata-evaluation-system and https://fashionandtextiles.springeropen.com/articles/10.1186/s40691-024-00388-6/tables/2 (KES parameter table).
- S-9: Haghighat, Shaikhzadeh Najar, Ahmadloo, Rastgoo, "Estimation of the Initial Shear Modulus of Twill Woven Denim Fabrics...", Fibres and Textiles in Eastern Europe 20(4), 2012: https://yadda.icm.edu.pl/baztech/element/bwmeta1.element.baztech-article-BPW7-0023-0056/c/FTEE_93_51.pdf
- S-10: Bekampiene, Domskiene, "Analysis of Fabric Specimen Aspect Ratio and Deformation Mechanism during Bias Tension", Materials Science 15(2), 2009: https://matsc.ktu.lt/index.php/MatSc/article/view/26140/21785
- S-11: Dehghan-Manshadi, Alamdar-Yazdi, Bidoki, "New Funnel Shaped Nozzle for Evaluation of the Mechanical Properties of Woven Fabric by the Extraction Method", Fibres and Textiles in Eastern Europe 22(2), 2014: https://bibliotekanauki.pl/articles/233663.pdf
- S-12: Adamu, "Denim Fabric Surface, Low-stress Tensile and Shear Properties Evaluation based on KES", JTATM 12(2), 2021: https://jtatm.textiles.ncsu.edu/index.php/JTATM/article/download/18650/8785/71325
- S-13: Ijaz, Namood-e-Sahar, Tariq, "Evaluation of Bending Length, Rigidity and Modulus of Woven and Knitted Fabrics", Proc. Pakistan Acad. Sci. A: https://ppaspk.org/index.php/PPAS-A/article/download/1270/698/3896
- S-14: Cotton single-jersey KES-FB table (B, 2HB, G, 2HG5; 20 Ne ring yarn, Table 1), TUBITAK Turkish J. Engineering and Environmental Sciences 27(3): https://aj.tubitak.gov.tr/engineering/issues/muh-03-27-3/muh-27-3-4-0205-13.pdf
- S-15: Choi, Lee, Yoneyama, "Effect of Knit Structure on Objective Hand Values", Material Technology 32(1), 2014 (qualitative): https://bunka.repo.nii.ac.jp/record/194/files/00101_000137.pdf

Fetched but unused (no data): https://www.fibre2fashion.com/industry-article/7465/low-stress-mechanical-properties-of-woven-silk-fabrics-part-1 (silk, only effective gap 6 to 7 mm, no KES values), https://hrcak.srce.hr/index.php/en/clanak/35877, https://link.springer.com/10.1007/s12221-024-00499-3, https://link.springer.com/article/10.1007/s12221-013-2163-3, https://etheses.whiterose.ac.uk/5046/1/Final%20Thesis-3.pdf, https://ir.iitd.ac.in/bitstreams/146e86b7-6a86-40c1-94ef-ad5ee53fcba5/download. Blocked or failed: tandfonline, iastatedigitalpress, pmc.ncbi.nlm.nih.gov, 1library.net.

Known gaps to close with the next research pass: Matsudaira and Kawabata 1988 silk paper (J. Textile Inst.), published KES tables for chiffon, taffeta, fleece, leather; wool EMT, LT, WT and G; FAST E5 for woven fabrics; the supplemental parameter files of Wang 2011 and Miguel 2012.
