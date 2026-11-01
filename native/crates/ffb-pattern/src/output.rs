use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MeshKind {
    #[default]
    Triangle,
    Quad,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Thickness {
    pub enabled: bool,
    pub amount: f32,
    pub sides: bool,
}

impl Default for Thickness {
    fn default() -> Self {
        Self { enabled: false, amount: 1.0, sides: true }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UvIsland {
    pub panel: u32,
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: f32,
    pub tile: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Output {
    pub mesh: MeshKind,
    pub quad_density: f32,
    pub retopo: Value,
    pub thickness: Thickness,
    pub islands: Vec<UvIsland>,
    pub uv_padding: f32,
    pub presets: Value,
}
