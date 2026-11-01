pub mod bezier;
pub mod edit;
pub mod garment;
pub mod geom;
pub mod path;
pub mod place;
pub mod polygon;
pub mod shapes;
pub mod snap;
pub mod xform;

use ffb_core::Units;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Garment {
    pub units: Units,
    pub panels: Vec<Panel>,
    pub selection: Value,
}

impl Default for Garment {
    fn default() -> Self {
        Garment { units: Units::Mm, panels: Vec::new(), selection: Value::Object(Default::default()) }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Panel {
    pub id: u32,
    pub name: String,
    pub outline: Vec<Point>,
    pub inner: Vec<Inner>,
    pub position: [f64; 2],
    pub rotation: f64,
    pub mirror: bool,
    pub grain: f64,
    pub particle_distance: f64,
    pub layer: i32,
}

impl Default for Panel {
    fn default() -> Self {
        Panel {
            id: 0,
            name: String::new(),
            outline: Vec::new(),
            inner: Vec::new(),
            position: [0.0, 0.0],
            rotation: 0.0,
            mirror: false,
            grain: 0.0,
            particle_distance: 20.0,
            layer: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PointKind {
    #[default]
    Corner,
    Curve,
    Smooth,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Point {
    pub p: [f64; 2],
    pub kind: PointKind,
    pub h_in: Option<[f64; 2]>,
    pub h_out: Option<[f64; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InnerKind {
    #[default]
    Line,
    Polyline,
    Rectangle,
    Ellipse,
    Polygon,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Inner {
    pub id: u32,
    pub kind: InnerKind,
    pub points: Vec<Point>,
    pub closed: bool,
    pub fold_angle: f64,
    pub fold_strength: f64,
}

impl Default for Inner {
    fn default() -> Self {
        Inner { id: 0, kind: InnerKind::Line, points: Vec::new(), closed: false, fold_angle: 180.0, fold_strength: 0.0 }
    }
}
