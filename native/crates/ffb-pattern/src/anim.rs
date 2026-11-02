use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Interp {
    Step,
    #[default]
    Linear,
    Smooth,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Key {
    pub frame: u32,
    pub value: Value,
    pub interp: Interp,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Track {
    pub id: u32,
    pub target: String,
    pub keys: Vec<Key>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Motion {
    pub id: u32,
    pub name: String,
    pub source: String,
    pub body: u32,
    pub start: u32,
    pub frames: u32,
    pub fps: f32,
    pub lead_in: u32,
    pub retarget: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheKind {
    #[default]
    Pattern,
    Trim,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheLayer {
    pub id: u32,
    pub name: String,
    pub start: u32,
    pub frames: u32,
    pub visible: bool,
    pub entry: String,
    pub kind: CacheKind,
    pub locked: bool,
    pub offset: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Anim {
    pub fps: f32,
    pub start: u32,
    pub end: u32,
    pub current: u32,
    pub motions: Vec<Motion>,
    pub tracks: Vec<Track>,
    pub caches: Vec<CacheLayer>,
    pub active_cache: Option<u32>,
    pub auto_key: bool,
}

impl Default for Anim {
    fn default() -> Self {
        Self { fps: 30.0, start: 0, end: 120, current: 0, motions: Vec::new(), tracks: Vec::new(), caches: Vec::new(), active_cache: None, auto_key: false }
    }
}
