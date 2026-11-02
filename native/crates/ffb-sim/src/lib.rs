use serde::{Deserialize, Serialize};

pub mod bvh;
pub mod collide;
pub mod contact;
pub mod energy;
pub mod external;
pub mod implicit;
pub mod material;
pub mod model;
pub mod par;
pub mod reference;
pub mod selfcol;
pub mod solver;
pub mod untangle;

pub use reference::{compare_positions, reference_step};
pub use solver::Solver;

#[derive(Debug, Clone, Default)]
pub struct Cloth {
    pub positions: Vec<[f32; 3]>,
    pub rest: Vec<[f32; 2]>,
    pub triangles: Vec<[u32; 3]>,
    pub triangle_material: Vec<u16>,
    pub triangle_grain: Vec<f32>,
    pub vertex_panel: Vec<u32>,
    pub vertex_layer: Vec<i32>,
    pub materials: Vec<Material>,
    pub seams: Vec<SeamPair>,
    pub folds: Vec<Fold>,
    pub pins: Vec<PinPoint>,
    pub binds: Vec<SeamBind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SeamBind {
    pub a: u32,
    pub b: [u32; 2],
    pub w: f32,
    pub stiffness: f32,
    pub rest_length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StepStats {
    pub ms: f32,
    pub iterations: u32,
    pub max_velocity: f32,
    pub contacts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Material {
    pub stretch_weft: f32,
    pub stretch_warp: f32,
    pub shear: f32,
    pub bending_weft: f32,
    pub bending_warp: f32,
    pub bending_bias: f32,
    pub buckling_ratio: f32,
    pub buckling_stiffness: f32,
    pub density: f32,
    pub friction: f32,
    pub thickness: f32,
    pub damping: f32,
    pub shrink_weft: f32,
    pub shrink_warp: f32,
    pub pressure: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SeamPair {
    pub a: u32,
    pub b: u32,
    pub rest_length: f32,
    pub stiffness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fold {
    pub edge: [u32; 2],
    pub opposite: [u32; 2],
    pub angle: f32,
    pub strength: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PinPoint {
    pub vertex: u32,
    pub target: [f32; 3],
}

#[derive(Debug, Clone)]
pub enum Collider {
    Plane { point: [f32; 3], normal: [f32; 3] },
    Sphere { center: [f32; 3], radius: f32 },
    Capsule { a: [f32; 3], b: [f32; 3], radius: f32 },
    Mesh { positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    #[default]
    Vbd,
    Implicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Untangle {
    #[default]
    Off,
    Normal,
    Flipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Params {
    pub untangle: Untangle,
    pub method: Method,
    pub newton: u32,
    pub cg_iterations: u32,
    pub cg_tolerance: f32,
    pub time_step: f32,
    pub substeps: u32,
    pub iterations: u32,
    pub gravity: [f32; 3],
    pub air_damping: f32,
    pub skin_offset: f32,
    pub collision_thickness: f32,
    pub friction: f32,
    pub self_collision: bool,
    pub nonlinear: bool,
    pub stiffening: f32,
    pub accel: f32,
    pub close_frames: u32,
    pub close_gap: f32,
    pub seam_boost: f32,
}

impl Params {
    pub fn preset(name: &str, method: Method) -> Option<Params> {
        let base = Params { method, ..Params::default() };
        let p = match (method, name) {
            (Method::Vbd, "normal") => Params { substeps: 3, iterations: 8, ..base },
            (Method::Vbd, "fast") => Params { substeps: 3, iterations: 6, ..base },
            (Method::Vbd, "quality" | "fine") => Params { substeps: 8, iterations: 15, self_collision: true, collision_thickness: 1.0, ..base },
            (Method::Vbd, "final") => Params { substeps: 12, iterations: 25, self_collision: true, collision_thickness: 0.8, ..base },
            (Method::Implicit, "normal") => Params { substeps: 3, newton: 2, cg_tolerance: 3.0e-3, ..base },
            (Method::Implicit, "fast") => Params { substeps: 2, newton: 2, cg_tolerance: 1.0e-2, ..base },
            (Method::Implicit, "quality" | "fine") => Params { substeps: 4, newton: 2, cg_tolerance: 1.0e-3, self_collision: true, collision_thickness: 1.0, ..base },
            (Method::Implicit, "final") => Params { substeps: 6, newton: 3, cg_iterations: 400, cg_tolerance: 3.0e-4, self_collision: true, collision_thickness: 0.8, ..base },
            _ => return None,
        };
        Some(p)
    }
}

impl Default for Params {
    fn default() -> Self {
        Params {
            untangle: Untangle::Off,
            method: Method::Implicit,
            newton: 2,
            cg_iterations: 200,
            cg_tolerance: 3.0e-3,
            time_step: 1.0 / 30.0,
            substeps: 3,
            iterations: 10,
            gravity: [0.0, -9800.0, 0.0],
            air_damping: 0.02,
            skin_offset: 3.0,
            collision_thickness: 1.5,
            friction: 0.3,
            self_collision: false,
            nonlinear: false,
            stiffening: 40.0,
            accel: 0.0,
            close_frames: 60,
            close_gap: 2.0,
            seam_boost: 25.0,
        }
    }
}
