use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightKind {
    #[default]
    Directional,
    Point,
    Spot,
    Area,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Light {
    pub id: u32,
    pub kind: LightKind,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub shadows: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Stage {
    pub lights: Vec<Light>,
    pub hdri: Option<String>,
    pub hdri_rotation: f32,
    pub exposure: f32,
    pub ground: bool,
    pub background: Value,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedView {
    pub id: u32,
    pub name: String,
    pub camera: Value,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Fur {
    pub id: u32,
    pub panel: u32,
    pub length: f32,
    pub density: f32,
    pub thickness: f32,
    pub curl: f32,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Viz {
    pub stage: Stage,
    pub views: Vec<SavedView>,
    pub fur: Vec<Fur>,
    pub map: Option<String>,
    pub compare: Value,
}
