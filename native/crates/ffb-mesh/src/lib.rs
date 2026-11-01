use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelMesh {
    pub panel: u32,
    pub positions: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
    pub boundary: Vec<u32>,
    pub boundary_param: Vec<(usize, f64)>,
    pub inner_edges: Vec<(u32, Vec<u32>)>,
}
