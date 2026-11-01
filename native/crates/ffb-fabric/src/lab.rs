use serde::{Deserialize, Serialize};

use crate::units;
use crate::validate;
use crate::Physical;

pub const FAST_BENDING: f64 = 9.807e-6;
pub const FAST_SHEAR: f64 = 123.0;
pub const MODULUS_RANGE: [f64; 2] = [0.02e6, 2.3e6];
pub const SHEAR_TO_TENSILE: f64 = 0.1;

const BENDING_SOURCES: [&str; 4] = ["16-FABRIC-DATA:1.5", "S-5", "S-7", "S-9"];

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tensile {
    pub force: f64,
    pub strain: f64,
    #[serde(default = "strip_width")]
    pub width: f64,
}

fn strip_width() -> f64 {
    50.0
}

fn frame_length() -> f64 {
    100.0
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShearTest {
    pub angle: f64,
    pub force: f64,
    #[serde(default = "frame_length")]
    pub length: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Directions<T> {
    #[serde(default = "none")]
    pub warp: Option<T>,
    #[serde(default = "none")]
    pub weft: Option<T>,
    #[serde(default = "none")]
    pub bias: Option<T>,
}

fn none<T>() -> Option<T> {
    None
}

impl<T> Default for Directions<T> {
    fn default() -> Self {
        Directions { warp: None, weft: None, bias: None }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct LabInput {
    pub density: Option<f64>,
    pub thickness: Option<f64>,
    pub bending_length: Directions<f64>,
    pub tensile: Directions<Tensile>,
    pub shear: Option<ShearTest>,
    pub bias_extension: Option<f64>,
    pub friction: Option<f64>,
    pub damping: Option<f64>,
    pub buckling_ratio: Option<f64>,
    pub buckling_stiffness: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Formula {
    pub field: &'static str,
    pub expression: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub sources: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KesBending {
    pub direction: &'static str,
    pub value: f64,
    pub expression: &'static str,
    pub sources: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabResult {
    pub physical: Physical,
    pub computed: Vec<&'static str>,
    pub kept: Vec<&'static str>,
    pub formulas: Vec<Formula>,
    pub kes_equivalent: Vec<KesBending>,
    pub warnings: Vec<String>,
}

fn range(x: f64, lo: f64, hi: f64, unit: &str, path: &str) -> Result<f64, String> {
    if !x.is_finite() || x < lo || x > hi {
        return Err(format!("{path} must be between {lo} and {hi} {unit}, got {x}"));
    }
    Ok(x)
}

fn significant(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let scale = 10f64.powi(5 - x.abs().log10().floor() as i32);
    (x * scale).round() / scale
}

struct Book {
    physical: Physical,
    formulas: Vec<Formula>,
    computed: Vec<&'static str>,
}

impl Book {
    fn put(&mut self, key: &'static str, value: f64, expression: &'static str, sources: &[&'static str]) {
        let value = significant(value);
        units::set(&mut self.physical, key, value);
        if !self.computed.contains(&key) {
            self.computed.push(key);
        }
        let unit = units::field(key).map_or("", |f| f.unit);
        self.formulas.retain(|f| f.field != key);
        self.formulas.push(Formula { field: key, expression, value, unit, sources: sources.to_vec() });
    }
}

fn opt(x: Option<f64>, lo: f64, hi: f64, unit: &str, path: &str) -> Result<Option<f64>, String> {
    x.map(|x| range(x, lo, hi, unit, path)).transpose()
}

fn kes_equivalent(b: f64, density: f64) -> Option<(f64, &'static str)> {
    if density > 200.0 {
        Some(((1.37 * b - 3.27).max(0.0), "B_KES = 1.37 * B - 3.27 (outerwear heavier than 200 g/m2)"))
    } else if density < 170.0 {
        Some(((0.88 * b - 0.60).max(0.0), "B_KES = 0.88 * B - 0.60 (shirting lighter than 170 g/m2)"))
    } else {
        None
    }
}

pub fn run(input: &LabInput, base: &Physical) -> Result<LabResult, String> {
    let mut book = Book { physical: base.clone(), formulas: Vec::new(), computed: Vec::new() };

    let density = opt(input.density, 1.0, 3000.0, "g/m2", "density")?;
    if let Some(w) = density {
        book.put("density", w, "density = measured areal density", &[]);
    }
    if let Some(t) = opt(input.thickness, 0.01, 10.0, "mm", "thickness")? {
        book.put("thickness", t, "thickness = measured thickness", &[]);
    }

    let w = density.unwrap_or(book.physical.density);
    let bending = |length_cm: f64| w * (length_cm * 10.0).powi(3) * FAST_BENDING;
    let warp_len = opt(input.bending_length.warp, 0.1, 50.0, "cm", "bending_length.warp")?;
    let weft_len = opt(input.bending_length.weft, 0.1, 50.0, "cm", "bending_length.weft")?;
    let bias_len = opt(input.bending_length.bias, 0.1, 50.0, "cm", "bending_length.bias")?;
    let expression = "B = 9.807e-6 * W * c^3, W in g/m2, c in mm (bending length c, half the overhang at 41.5 degrees), B in uN*m";
    if let Some(c) = warp_len {
        book.put("bending_warp", bending(c), expression, &BENDING_SOURCES);
    }
    if let Some(c) = weft_len {
        book.put("bending_weft", bending(c), expression, &BENDING_SOURCES);
    }
    match (bias_len, warp_len.is_some() || weft_len.is_some()) {
        (Some(c), _) => book.put("bending_bias", bending(c), expression, &BENDING_SOURCES),
        (None, true) => {
            let v = (book.physical.bending_warp * book.physical.bending_weft).sqrt();
            book.put("bending_bias", v, "B_bias = sqrt(B_warp * B_weft) when no bias strip was measured", &["16-FABRIC-DATA:1.5", "S-11"]);
        }
        _ => {}
    }

    for (label, key, test) in [("warp", "stretch_warp", input.tensile.warp), ("weft", "stretch_weft", input.tensile.weft)] {
        if let Some(t) = test {
            let path = format!("tensile.{label}");
            let force = range(t.force, 0.001, 10_000.0, "N", &format!("{path}.force"))?;
            let strain = range(t.strain, 0.01, 100.0, "percent", &format!("{path}.strain"))?;
            let width = range(t.width, 1.0, 500.0, "mm", &format!("{path}.width"))?;
            book.put(key, (force / (width * 1e-3)) / (strain / 100.0), "k = (F / w) / strain, secant at the test strain; use 1 to 2 percent for the low-strain region", &["16-FABRIC-DATA:4.3", "S-5"]);
        }
    }

    if let Some(s) = input.shear {
        let angle = range(s.angle, 0.1, 60.0, "deg", "shear.angle")?;
        let force = range(s.force, 0.0001, 10_000.0, "N", "shear.force")?;
        let length = range(s.length, 1.0, 1000.0, "mm", "shear.length")?;
        book.put("shear", (force / (length * 1e-3)) / angle.to_radians().tan(), "k_s = (F_s / L_s) / tan(phi), picture frame test", &["16-FABRIC-DATA:4.4"]);
    }
    if let Some(eb5) = opt(input.bias_extension, 0.01, 100.0, "percent", "bias_extension")? {
        if input.shear.is_none() {
            book.put("shear", FAST_SHEAR / eb5, "G = 123 / EB5, EB5 the bias extension in percent at 5 gf/cm", &["S-5", "S-9"]);
        }
    }

    for (key, value) in [("friction", input.friction), ("damping", input.damping), ("buckling_ratio", input.buckling_ratio), ("buckling_stiffness", input.buckling_stiffness)] {
        if let Some(x) = value {
            let f = units::field(key).ok_or("unknown field")?;
            let x = range(x, f.min, f.max, f.unit, key)?;
            book.put(key, x, "taken as given, no measurement formula", &[]);
        }
    }

    let p = book.physical.clone();
    validate::check_physical(&p)?;

    let mut warnings = Vec::new();
    let t = p.thickness * 1e-3;
    for (label, key, b) in [("warp", "bending_warp", p.bending_warp), ("weft", "bending_weft", p.bending_weft), ("bias", "bending_bias", p.bending_bias)] {
        if book.computed.contains(&key) {
            let modulus = 12.0 * b * 1e-6 / t.powi(3);
            if !(MODULUS_RANGE[0]..=MODULUS_RANGE[1]).contains(&modulus) {
                warnings.push(format!("effective bending modulus {label} is {:.3} MPa, outside the sourced 0.02 to 2.3 MPa range; re-check the bending length, density and thickness", modulus / 1e6));
            }
        }
    }
    if book.computed.contains(&"shear") {
        let mean = (p.stretch_warp + p.stretch_weft) / 2.0;
        if p.shear > SHEAR_TO_TENSILE * mean {
            warnings.push(format!("shear stiffness {:.0} N/m is more than a tenth of the mean tensile stiffness {:.0} N/m; re-check the units", p.shear, mean));
        }
    }

    let mut kes = Vec::new();
    for (direction, key, b) in [("warp", "bending_warp", p.bending_warp), ("weft", "bending_weft", p.bending_weft), ("bias", "bending_bias", p.bending_bias)] {
        if book.computed.contains(&key) {
            if let Some((value, expression)) = kes_equivalent(b, w) {
                kes.push(KesBending { direction, value, expression, sources: vec!["S-7"] });
            }
        }
    }

    let kept = units::FIELDS.iter().map(|f| f.key).filter(|k| !book.computed.contains(k)).collect();
    Ok(LabResult { physical: p, computed: book.computed, kept, formulas: book.formulas, kes_equivalent: kes, warnings })
}
