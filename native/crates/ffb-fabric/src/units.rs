use serde::Serialize;

use crate::Physical;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scale {
    Linear,
    Log,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub min: f64,
    pub max: f64,
    pub scale: Scale,
    pub description: &'static str,
}

pub const SLIDER_MAX: f64 = 99.0;

pub const FIELDS: [Field; 12] = [
    Field { key: "stretch_weft", label: "Stretch weft", unit: "N/m", min: 1.0, max: 1_000_000.0, scale: Scale::Log, description: "Tensile stiffness along the weft yarns, force per unit width per unit strain." },
    Field { key: "stretch_warp", label: "Stretch warp", unit: "N/m", min: 1.0, max: 1_000_000.0, scale: Scale::Log, description: "Tensile stiffness along the warp yarns, force per unit width per unit strain." },
    Field { key: "shear", label: "Shear", unit: "N/m", min: 0.1, max: 100_000.0, scale: Scale::Log, description: "In-plane shear stiffness, force per unit width per unit shear strain." },
    Field { key: "bending_weft", label: "Bending weft", unit: "uN*m", min: 0.01, max: 100_000.0, scale: Scale::Log, description: "Bending rigidity per unit width across the weft direction, micro newton metres." },
    Field { key: "bending_warp", label: "Bending warp", unit: "uN*m", min: 0.01, max: 100_000.0, scale: Scale::Log, description: "Bending rigidity per unit width across the warp direction, micro newton metres." },
    Field { key: "bending_bias", label: "Bending bias", unit: "uN*m", min: 0.01, max: 100_000.0, scale: Scale::Log, description: "Bending rigidity per unit width along the 45 degree bias, micro newton metres." },
    Field { key: "buckling_ratio", label: "Buckling ratio", unit: "ratio", min: 0.0, max: 1.0, scale: Scale::Linear, description: "Share of the bending curvature at which folds start to form; 0 buckles at once, 1 never buckles." },
    Field { key: "buckling_stiffness", label: "Buckling stiffness", unit: "ratio", min: 0.0, max: 1.0, scale: Scale::Linear, description: "Bending rigidity left inside a fold as a fraction of the bending values." },
    Field { key: "density", label: "Density", unit: "g/m2", min: 1.0, max: 3000.0, scale: Scale::Log, description: "Areal mass of the cloth, grams per square metre." },
    Field { key: "friction", label: "Friction", unit: "ratio", min: 0.0, max: 1.0, scale: Scale::Linear, description: "Coulomb friction coefficient between cloth layers and against bodies." },
    Field { key: "thickness", label: "Thickness", unit: "mm", min: 0.01, max: 10.0, scale: Scale::Log, description: "Cloth thickness used for collision and thick rendering, millimetres." },
    Field { key: "damping", label: "Damping", unit: "ratio", min: 0.0, max: 1.0, scale: Scale::Linear, description: "Internal damping of the cloth motion; 0 is bouncy, 1 is sluggish." },
];

pub fn field(key: &str) -> Option<&'static Field> {
    FIELDS.iter().find(|f| f.key == key)
}

pub fn get(p: &Physical, key: &str) -> Option<f64> {
    Some(match key {
        "stretch_weft" => p.stretch_weft,
        "stretch_warp" => p.stretch_warp,
        "shear" => p.shear,
        "bending_weft" => p.bending_weft,
        "bending_warp" => p.bending_warp,
        "bending_bias" => p.bending_bias,
        "buckling_ratio" => p.buckling_ratio,
        "buckling_stiffness" => p.buckling_stiffness,
        "density" => p.density,
        "friction" => p.friction,
        "thickness" => p.thickness,
        "damping" => p.damping,
        _ => return None,
    })
}

pub fn set(p: &mut Physical, key: &str, value: f64) -> bool {
    let slot = match key {
        "stretch_weft" => &mut p.stretch_weft,
        "stretch_warp" => &mut p.stretch_warp,
        "shear" => &mut p.shear,
        "bending_weft" => &mut p.bending_weft,
        "bending_warp" => &mut p.bending_warp,
        "bending_bias" => &mut p.bending_bias,
        "buckling_ratio" => &mut p.buckling_ratio,
        "buckling_stiffness" => &mut p.buckling_stiffness,
        "density" => &mut p.density,
        "friction" => &mut p.friction,
        "thickness" => &mut p.thickness,
        "damping" => &mut p.damping,
        _ => return false,
    };
    *slot = value;
    true
}

pub fn to_slider(f: &Field, value: f64) -> f64 {
    let t = match f.scale {
        Scale::Linear => (value - f.min) / (f.max - f.min),
        Scale::Log => (value.max(f.min) / f.min).ln() / (f.max / f.min).ln(),
    };
    (t.clamp(0.0, 1.0) * SLIDER_MAX).round()
}

pub fn from_slider(f: &Field, slider: f64) -> f64 {
    let t = (slider / SLIDER_MAX).clamp(0.0, 1.0);
    match f.scale {
        Scale::Linear => f.min + t * (f.max - f.min),
        Scale::Log => f.min * (f.max / f.min).powf(t),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct SiPhysical {
    pub stretch_weft: f64,
    pub stretch_warp: f64,
    pub shear: f64,
    pub bending_weft: f64,
    pub bending_warp: f64,
    pub bending_bias: f64,
    pub buckling_ratio: f64,
    pub buckling_stiffness: f64,
    pub density: f64,
    pub friction: f64,
    pub thickness: f64,
    pub damping: f64,
}

impl Physical {
    pub fn si(&self) -> SiPhysical {
        SiPhysical {
            stretch_weft: self.stretch_weft,
            stretch_warp: self.stretch_warp,
            shear: self.shear,
            bending_weft: self.bending_weft * 1e-6,
            bending_warp: self.bending_warp * 1e-6,
            bending_bias: self.bending_bias * 1e-6,
            buckling_ratio: self.buckling_ratio,
            buckling_stiffness: self.buckling_stiffness,
            density: self.density * 1e-3,
            friction: self.friction,
            thickness: self.thickness * 1e-3,
            damping: self.damping,
        }
    }
}
