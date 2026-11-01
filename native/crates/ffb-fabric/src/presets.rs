use serde::Serialize;

use crate::{Fabric, Look, LookKind, Physical, Surface};

pub const DEFAULT_ID: &str = "cotton_poplin_shirting";

pub const SOURCES: [(&str, &str); 15] = [
    ("S-1", "Wang, O'Brien, Ramamoorthi, Data-driven elastic models for cloth, ACM TOG 2011 (fabric list and areal densities)"),
    ("S-2", "KES-F table of cotton knit and woven fabrics, Fibre2Fashion hosted paper (units inferred)"),
    ("S-3", "Ancutiene, Strazdiene, Nesterova, Fabrics bending rigidity by KES-F and FAST, Materials Science 16(4), 2010"),
    ("S-4", "Peirce relations for the bending of flax reinforcements, arXiv 1312.3796"),
    ("S-5", "Fabric Assurance by Simple Testing overview, ScienceDirect Topics (E5, E20, E100, bending rigidity and shear formulas)"),
    ("S-6", "Naujokaitye, Strazdiene, Effect of finishing upon textile mechanical properties at low loading, Materials Science 13(3), 2007"),
    ("S-7", "Same paper as S-3, formulas section (cantilever bending, FAST to KES regression)"),
    ("S-8", "NCSU Textiles KES overview and KES parameter table"),
    ("S-9", "Haghighat et al., Initial shear modulus of twill woven denim fabrics, Fibres and Textiles in Eastern Europe 20(4), 2012"),
    ("S-10", "Bekampiene, Domskiene, Fabric specimen aspect ratio during bias tension, Materials Science 15(2), 2009"),
    ("S-11", "Dehghan-Manshadi et al., Funnel shaped nozzle extraction method, Fibres and Textiles in Eastern Europe 22(2), 2014"),
    ("S-12", "Adamu, Denim fabric low-stress tensile and shear properties by KES, JTATM 12(2), 2021"),
    ("S-13", "Ijaz et al., Bending length, rigidity and modulus of woven and knitted fabrics, Proc. Pakistan Acad. Sci. A"),
    ("S-14", "Cotton single-jersey KES-FB table, Turkish J. Engineering and Environmental Sciences 27(3)"),
    ("S-15", "Choi, Lee, Yoneyama, Effect of knit structure on objective hand values, Material Technology 32(1), 2014"),
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub physical: Physical,
    pub look: Look,
    pub sources: &'static [&'static str],
    pub status: &'static str,
    pub provisional: bool,
    pub notes: &'static str,
}

fn phys(density: f64, thickness: f64, stretch: [f64; 2], shear: f64, bending: [f64; 3], friction: f64) -> Physical {
    Physical {
        stretch_weft: stretch[0],
        stretch_warp: stretch[1],
        shear,
        bending_weft: bending[0],
        bending_warp: bending[1],
        bending_bias: bending[2],
        density,
        thickness,
        friction,
        ..Physical::default()
    }
}

fn look(kind: LookKind, color: [f32; 3], roughness: f32, sheen: f32, opacity: f32) -> Look {
    let front = Surface { color: [color[0], color[1], color[2], 1.0], roughness, sheen, opacity, ..Surface::default() };
    Look { kind, front, back: None, side: None }
}

pub fn all() -> Vec<Preset> {
    vec![
        Preset {
            id: "cotton_poplin_shirting",
            name: "Cotton poplin",
            category: "woven",
            description: "Plain weave cotton shirting.",
            physical: phys(125.0, 0.61, [7400.0, 7400.0], 29.0, [5.5, 6.7, 6.1], 0.3),
            look: look(LookKind::Matte, [0.86, 0.89, 0.94], 0.9, 0.05, 1.0),
            sources: &["S-2:M-1", "S-9:M-3", "R2"],
            status: "partial",
            provisional: true,
            notes: "Density, thickness, tensile and shear from M-1 (single source, units inferred). Bending derived from the effective bending modulus of the twill M-3 (rule R2) with the weft to warp ratio of M-3 and the geometric mean for bias. Tensile is a k_500 secant and overstates low-strain stiffness; override with Fabric Lab.",
        },
        Preset {
            id: "cotton_twill",
            name: "Cotton twill",
            category: "woven",
            description: "Cotton twill 3/1, chino and workwear weight.",
            physical: phys(196.0, 0.99, [7400.0, 7400.0], 62.0, [23.4, 28.7, 25.9], 0.3),
            look: look(LookKind::Matte, [0.76, 0.69, 0.55], 0.92, 0.05, 1.0),
            sources: &["S-9:M-3", "S-2:M-1", "S-11:bias", "R3"],
            status: "measured core",
            provisional: false,
            notes: "Density, thickness, bending and shear measured (M-3). Bias bending is the geometric mean of warp and weft. Tensile borrowed from poplin M-1 (rule R3).",
        },
        Preset {
            id: "denim_heavy_finished",
            name: "Denim",
            category: "woven",
            description: "Finished heavy cotton denim twill.",
            physical: phys(398.0, 1.10, [7400.0, 7400.0], 210.0, [115.0, 98.0, 106.0], 0.2),
            look: look(LookKind::Matte, [0.17, 0.25, 0.42], 0.93, 0.0, 1.0),
            sources: &["S-9:M-7", "S-9:M-6", "S-12:M-10", "S-2:M-1", "S-11:bias"],
            status: "measured core",
            provisional: false,
            notes: "Density, thickness and bending are the means of six finished denims (M-7). Shear is the twill S4 value scaled by density (398/207) because the measured 688 N/m is suspect (note a of the data file). Friction 0.2 is a KES sensor value against steel, not cloth on cloth. Tensile borrowed from M-1.",
        },
        Preset {
            id: "linen",
            name: "Linen",
            category: "woven",
            description: "Mechanically softened plain weave linen.",
            physical: phys(170.0, 0.49, [11500.0, 7700.0], 29.0, [17.0, 22.0, 19.0], 0.3),
            look: look(LookKind::Matte, [0.82, 0.76, 0.64], 0.95, 0.0, 1.0),
            sources: &["S-6:L-3", "S-2:M-1", "S-11:bias", "R3"],
            status: "measured core",
            provisional: false,
            notes: "Density, thickness, bending and tensile from L-3. The source gives no numeric shear, so 29 N/m comes from M-1. Unfinished linen is much stiffer.",
        },
        Preset {
            id: "wool_suiting_worsted",
            name: "Wool suiting",
            category: "woven",
            description: "Worsted wool and polyamide suiting twill.",
            physical: phys(244.0, 1.01, [9500.0, 9500.0], 62.0, [11.6, 17.9, 14.4], 0.3),
            look: look(LookKind::Matte, [0.24, 0.25, 0.28], 0.9, 0.1, 1.0),
            sources: &["S-3:W-2", "S-6:L-rows", "S-9:M-3", "S-11:bias", "R3"],
            status: "measured core",
            provisional: false,
            notes: "Density, thickness and bending from W-2. No wool tensile or shear table was found: tensile is the median of the linen k_500 values and shear is taken from the twill M-3.",
        },
        Preset {
            id: "wool_coating",
            name: "Wool coating",
            category: "woven",
            description: "Heavy wool and polyester outerwear coating.",
            physical: phys(398.0, 2.28, [9500.0, 9500.0], 62.0, [41.3, 73.5, 55.1], 0.3),
            look: look(LookKind::Matte, [0.62, 0.47, 0.32], 0.97, 0.15, 1.0),
            sources: &["S-3:W-4", "S-3:W-3", "S-6:L-rows", "S-9:M-3", "S-11:bias", "R3"],
            status: "measured core",
            provisional: false,
            notes: "Density, thickness and bending from W-4. Tensile and shear derived as for wool suiting. Wool flannel is not measured; W-3 is the nearest light thick row.",
        },
        Preset {
            id: "silk_charmeuse_crepe",
            name: "Silk crepe",
            category: "woven",
            description: "Fluid silk crepe or charmeuse.",
            physical: phys(145.0, 0.37, [9500.0, 9500.0], 29.0, [1.5, 1.5, 1.5], 0.3),
            look: look(LookKind::Satin, [0.90, 0.82, 0.72], 0.35, 0.6, 1.0),
            sources: &["S-13:X-1", "S-9:M-3", "R1", "R2", "R3"],
            status: "derived",
            provisional: true,
            notes: "No measured silk bending, tensile or shear in the sources. Density from X-1 (silk and polyester, 145 g/m2; true charmeuse is lighter). Thickness by rule R1 with the densest sourced bulk density, bending by rule R2, probably a lower bound. Measure with Fabric Lab.",
        },
        Preset {
            id: "chiffon_georgette",
            name: "Chiffon",
            category: "woven",
            description: "Sheer lightweight chiffon or georgette.",
            physical: phys(46.0, 0.12, [9500.0, 9500.0], 29.0, [0.4, 0.4, 0.4], 0.3),
            look: look(LookKind::Matte, [0.93, 0.78, 0.80], 0.7, 0.1, 0.55),
            sources: &["S-11:X-4", "R1", "R2", "R3"],
            status: "derived",
            provisional: true,
            notes: "Density from the lightest woven in S-11 (fibre unknown). Bending is the geometric mean of the rule R2 value (0.05) and the printed value (3.2), isotropic. Opacity 0.55 is a look choice, not sourced.",
        },
        Preset {
            id: "polyester_taffeta",
            name: "Polyester taffeta",
            category: "woven",
            description: "Crisp polyester dress taffeta.",
            physical: phys(113.0, 0.29, [9500.0, 9500.0], 29.0, [0.7, 0.7, 0.7], 0.3),
            look: look(LookKind::Shiny, [0.45, 0.12, 0.22], 0.4, 0.35, 1.0),
            sources: &["S-1:X-2", "R1", "R2", "R3"],
            status: "derived",
            provisional: true,
            notes: "Density from X-2, a polyester dress fabric that is not confirmed taffeta. Thickness by rule R1, bending by rule R2, isotropic assumption.",
        },
        Preset {
            id: "cotton_jersey_knit",
            name: "Cotton jersey",
            category: "knit",
            description: "Single jersey cotton knit, course as weft and wale as warp.",
            physical: phys(176.0, 1.29, [2360.0, 2360.0], 25.0, [3.2, 1.5, 2.2], 0.3),
            look: look(LookKind::Matte, [0.62, 0.63, 0.65], 0.95, 0.05, 1.0),
            sources: &["S-14:K-2", "S-2:K-5", "S-11:bias", "R1"],
            status: "measured core",
            provisional: false,
            notes: "Density, bending and shear from K-2. Thickness by rule R1 with the bulk density of K-5, tensile from K-5 (k_500). Knit tensile is strongly nonlinear and k_500 overstates the low-strain stiffness by a large factor.",
        },
        Preset {
            id: "rib_knit",
            name: "Rib knit",
            category: "knit",
            description: "Cotton and elastane rib knit.",
            physical: phys(276.0, 2.0, [2360.0, 2360.0], 25.0, [12.0, 12.0, 12.0], 0.3),
            look: look(LookKind::Matte, [0.55, 0.62, 0.52], 0.96, 0.05, 1.0),
            sources: &["S-1:K-7", "S-2:K-5", "S-14:K-2", "S-15", "R1", "R2", "R3"],
            status: "derived",
            provisional: true,
            notes: "Density from K-7. Thickness by rule R1, bending by rule R2 with the jersey modulus, tensile and shear from jersey. S-15 says rib shears more easily and stretches more in the course direction, so shear and weft tensile are probably too high. Low confidence.",
        },
        Preset {
            id: "fleece",
            name: "Fleece",
            category: "knit",
            description: "Polyester fleece or blanket knit.",
            physical: phys(228.0, 1.67, [2360.0, 2360.0], 25.0, [6.9, 6.9, 6.9], 0.4),
            look: look(LookKind::Velvet, [0.85, 0.85, 0.82], 1.0, 0.4, 1.0),
            sources: &["S-1:X-5", "S-2:K-5", "S-14:K-2", "R1", "R2", "R3"],
            status: "derived",
            provisional: true,
            notes: "Density from X-5, a polyester blanket used as a proxy. Thickness by rule R1, bending by rule R2 with the jersey modulus, tensile and shear from jersey. Friction 0.4 is a formfabric choice for a napped surface, not sourced. Low confidence.",
        },
        Preset {
            id: "spandex_stretch_jersey",
            name: "Stretch jersey",
            category: "knit",
            description: "Nylon and elastane swim or active knit.",
            physical: phys(204.0, 1.49, [2360.0, 2360.0], 25.0, [4.9, 4.9, 4.9], 0.3),
            look: look(LookKind::Shiny, [0.10, 0.12, 0.30], 0.5, 0.2, 1.0),
            sources: &["S-1:K-10", "S-2:K-5", "S-14:K-2", "R1", "R2", "R3"],
            status: "derived",
            provisional: true,
            notes: "Density from K-10. Thickness by rule R1, bending by rule R2, tensile and shear borrowed from jersey. A real elastane knit extends far more at the same load: lower the tensile values from a Fabric Lab measurement before use.",
        },
        Preset {
            id: "leather_garment",
            name: "Garment leather",
            category: "leather",
            description: "Soft lambskin or garment leather.",
            physical: phys(650.0, 0.9, [20000.0, 20000.0], 1500.0, [300.0, 300.0, 300.0], 0.5),
            look: look(LookKind::Leather, [0.20, 0.12, 0.08], 0.55, 0.1, 1.0),
            sources: &[],
            status: "estimate",
            provisional: true,
            notes: "The data file has no leather data. These values are an engineering estimate by the fabric builder, not sourced: density 650 g/m2 and thickness 0.9 mm for lambskin, bending from an assumed bending modulus of 5 MPa through B = E T^3 / 12, tensile from an assumed 20 MPa times the thickness, shear kept under a tenth of the tensile stiffness, friction 0.5 for a grippy surface. Replace with a Fabric Lab measurement.",
        },
        Preset {
            id: "canvas_cotton",
            name: "Cotton canvas",
            category: "woven",
            description: "Heavy cotton basket weave canvas.",
            physical: phys(369.0, 1.12, [10600.0, 10500.0], 196.0, [52.0, 77.0, 63.0], 0.3),
            look: look(LookKind::Matte, [0.72, 0.66, 0.52], 0.95, 0.0, 1.0),
            sources: &["S-6:M-12", "S-9:M-6", "S-11:bias"],
            status: "measured core",
            provisional: false,
            notes: "Density, thickness, bending and tensile from M-12 (basket weave, dyed and washed). Shear is the twill S4 value scaled by density, a crude estimate. Plain duck canvas is not separately measured.",
        },
    ]
}

pub fn find(id: &str) -> Option<Preset> {
    all().into_iter().find(|p| p.id == id)
}

pub fn instantiate(preset: &Preset, id: u32, name: Option<&str>) -> Fabric {
    Fabric { id, name: name.unwrap_or(preset.name).to_string(), preset: Some(preset.id.to_string()), physical: preset.physical.clone(), look: preset.look.clone() }
}

pub fn default_preset() -> Preset {
    find(DEFAULT_ID).unwrap_or_else(|| all().remove(0))
}
