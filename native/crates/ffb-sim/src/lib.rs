use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct Cloth {
    pub positions: Vec<[f32; 3]>,
    pub rest: Vec<[f32; 2]>,
    pub triangles: Vec<[u32; 3]>,
    pub triangle_material: Vec<u16>,
    pub triangle_grain: Vec<f32>,
    pub vertex_panel: Vec<u32>,
    pub materials: Vec<Material>,
    pub seams: Vec<SeamPair>,
    pub folds: Vec<Fold>,
    pub pins: Vec<PinPoint>,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Params {
    pub time_step: f32,
    pub substeps: u32,
    pub iterations: u32,
    pub gravity: [f32; 3],
    pub air_damping: f32,
    pub skin_offset: f32,
    pub collision_thickness: f32,
    pub friction: f32,
    pub self_collision: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            time_step: 1.0 / 30.0,
            substeps: 5,
            iterations: 10,
            gravity: [0.0, -9800.0, 0.0],
            air_damping: 0.02,
            skin_offset: 3.0,
            collision_thickness: 1.5,
            friction: 0.3,
            self_collision: false,
        }
    }
}
