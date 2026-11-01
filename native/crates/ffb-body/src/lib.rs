use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Body {
    pub id: u32,
    pub name: String,
    pub source: Option<String>,
    pub license: Option<String>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub joints: Vec<Joint>,
    pub weights: Vec<[(u16, f32); 4]>,
    pub pose: BTreeMap<String, [f32; 4]>,
    pub measurements: BTreeMap<String, f64>,
    pub volumes: Vec<PlacementVolume>,
    pub points: Vec<PlacementPoint>,
    pub skin_offset: f64,
    pub friction: f64,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Joint {
    pub name: String,
    pub parent: Option<u16>,
    pub rest: [f32; 3],
    pub rest_rotation: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VolumeKind {
    #[default]
    Cylinder,
    Box,
    Plane,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PlacementVolume {
    pub id: u32,
    pub name: String,
    pub kind: VolumeKind,
    pub joints: [Option<u16>; 2],
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub radius: [f64; 2],
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PlacementPoint {
    pub id: u32,
    pub name: String,
    pub volume: u32,
    pub around: f64,
    pub along: f64,
    pub offset: f64,
    pub wrap: bool,
    pub mirror_of: Option<u32>,
}
