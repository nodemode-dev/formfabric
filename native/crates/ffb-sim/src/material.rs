use ffb_fabric::Physical;

use crate::Material;

pub const FORCE_PER_N_PER_M: f64 = 1.0e3;
pub const BENDING_PER_UNM: f64 = 1.0e3;
pub const AREAL_DENSITY_SCALE: f64 = 1.0e-6;
pub const DAMPING_SECONDS: f32 = 0.005;
pub const DEFAULT_DENSITY: f32 = 1.3e-4;

impl Material {
    pub fn from_physical(p: &Physical) -> Material {
        Material {
            stretch_weft: (p.stretch_weft * FORCE_PER_N_PER_M) as f32,
            stretch_warp: (p.stretch_warp * FORCE_PER_N_PER_M) as f32,
            shear: (p.shear * FORCE_PER_N_PER_M) as f32,
            bending_weft: (p.bending_weft * BENDING_PER_UNM) as f32,
            bending_warp: (p.bending_warp * BENDING_PER_UNM) as f32,
            bending_bias: (p.bending_bias * BENDING_PER_UNM) as f32,
            buckling_ratio: p.buckling_ratio as f32,
            buckling_stiffness: p.buckling_stiffness as f32,
            density: (p.density * AREAL_DENSITY_SCALE) as f32,
            friction: p.friction as f32,
            thickness: p.thickness as f32,
            damping: p.damping as f32,
            shrink_weft: 1.0,
            shrink_warp: 1.0,
            pressure: 0.0,
        }
    }

    pub fn effective(&self) -> Material {
        let mut m = *self;
        if m.density <= 0.0 {
            m.density = DEFAULT_DENSITY;
        }
        if m.shrink_weft <= 1.0e-6 {
            m.shrink_weft = 1.0;
        }
        if m.shrink_warp <= 1.0e-6 {
            m.shrink_warp = 1.0;
        }
        if m.bending_bias <= 0.0 {
            m.bending_bias = (m.bending_weft.max(0.0) * m.bending_warp.max(0.0)).sqrt();
        }
        m
    }

    pub fn kd(&self) -> f64 {
        (self.damping.max(0.0) * DAMPING_SECONDS) as f64
    }
}
