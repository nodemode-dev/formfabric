use std::collections::BTreeMap;

use serde::Serialize;

pub type V2 = [f64; 2];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemeshError(pub String);

impl std::fmt::Display for RemeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RemeshError {}

pub fn fail<T>(message: impl Into<String>) -> Result<T, RemeshError> {
    Err(RemeshError(message.into()))
}

#[derive(Debug, Clone)]
pub struct Params {
    pub edge: f64,
    pub guides: Vec<Vec<V2>>,
    pub counts: BTreeMap<usize, usize>,
    pub pure_quads: bool,
    pub grain_weight: f64,
    pub cleanup: bool,
}

impl Params {
    pub fn new(edge: f64) -> Params {
        Params { edge, guides: Vec::new(), counts: BTreeMap::new(), pure_quads: false, grain_weight: 0.02, cleanup: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rim {
    pub vertex: u32,
    pub segment: usize,
    pub index: usize,
    pub count: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub fine_vertices: usize,
    pub lattice_quads: usize,
    pub band_quads: usize,
    pub band_tris: usize,
    pub singularities_found: usize,
    pub singularities_after: usize,
    pub valence2_removed: usize,
    pub diagonal_collapses: usize,
    pub field_iterations: usize,
    pub position_iterations: usize,
    pub ms: f64,
}

#[derive(Debug, Clone, Default)]
pub struct QuadMesh {
    pub panel: u32,
    pub positions: Vec<V2>,
    pub quads: Vec<[u32; 4]>,
    pub tris: Vec<[u32; 3]>,
    pub rim: Vec<Rim>,
    pub edge: f64,
    pub report: Report,
}
