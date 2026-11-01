pub mod bezier;
pub mod dart;
pub mod edit;
pub mod garment;
pub mod geom;
pub mod grain;
pub mod link;
pub mod measure2;
pub mod notch;
pub mod offset;
pub mod path;
pub mod place;
pub mod polygon;
pub mod shapes;
pub mod slash;
pub mod snap;
pub mod spiral;
pub mod trace;
pub mod unfold;
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
    pub fabrics: Vec<Fabric>,
    pub default_fabric: Option<u32>,
    pub colorways: Vec<Colorway>,
    pub active_colorway: Option<u32>,
    pub seams: Vec<Seam>,
    pub tacks: Vec<Tack>,
    pub pins: Vec<Pin>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SeamEnd {
    pub panel: u32,
    pub inner: Option<u32>,
    pub segment: usize,
    pub t0: f64,
    pub t1: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeamFold {
    #[default]
    Turned,
    Flat,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeamSide {
    #[default]
    Auto,
    Front,
    Back,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Seam {
    pub id: u32,
    pub a: Vec<SeamEnd>,
    pub b: Vec<SeamEnd>,
    pub reversed: bool,
    pub fold: SeamFold,
    pub fold_angle: f64,
    pub fold_strength: f64,
    pub ratio: f64,
    pub tension: f64,
    pub side: SeamSide,
}

impl Default for Seam {
    fn default() -> Self {
        Seam {
            id: 0,
            a: Vec::new(),
            b: Vec::new(),
            reversed: false,
            fold: SeamFold::Turned,
            fold_angle: 180.0,
            fold_strength: 0.0,
            ratio: 1.0,
            tension: 1.0,
            side: SeamSide::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TackEnd {
    pub panel: Option<u32>,
    pub point: [f64; 2],
    pub body_point: Option<[f64; 3]>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Tack {
    pub id: u32,
    pub a: TackEnd,
    pub b: TackEnd,
    pub length: f64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pin {
    pub id: u32,
    pub panel: u32,
    pub point: [f64; 2],
    pub target: Option<[f64; 3]>,
}

impl Default for Garment {
    fn default() -> Self {
        Garment {
            units: Units::Mm,
            panels: Vec::new(),
            selection: Value::Object(Default::default()),
            fabrics: Vec::new(),
            default_fabric: None,
            colorways: Vec::new(),
            active_colorway: None,
            seams: Vec::new(),
            tacks: Vec::new(),
            pins: Vec::new(),
        }
    }
}

pub use ffb_fabric::{Colorway, Fabric};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CornerStyle {
    #[default]
    Miter,
    Square,
    Round,
    Mirror,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeamAllowance {
    pub width: f64,
    pub corners: CornerStyle,
    pub per_segment: Vec<Option<f64>>,
}

impl Default for SeamAllowance {
    fn default() -> Self {
        SeamAllowance { width: 10.0, corners: CornerStyle::Miter, per_segment: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotchKind {
    #[default]
    Slit,
    V,
    T,
    U,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Notch {
    pub id: u32,
    pub segment: usize,
    pub t: f64,
    pub kind: NotchKind,
    pub depth: f64,
    pub width: f64,
    pub angle: f64,
}

impl Default for Notch {
    fn default() -> Self {
        Notch { id: 0, segment: 0, t: 0.5, kind: NotchKind::Slit, depth: 6.0, width: 3.0, angle: 90.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    #[default]
    Mirror,
    Instance,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelLink {
    pub kind: LinkKind,
    pub source: u32,
    pub axis: Option<[[f64; 2]; 2]>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Place3D {
    pub origin: [f64; 3],
    pub rotation: [f64; 4],
    pub bend: f64,
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
    pub seam_allowance: Option<SeamAllowance>,
    pub notches: Vec<Notch>,
    pub grain_line: Option<[[f64; 2]; 2]>,
    pub link: Option<PanelLink>,
    pub fabric: Option<u32>,
    pub place3d: Option<Place3D>,
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
            seam_allowance: None,
            notches: Vec::new(),
            grain_line: None,
            link: None,
            fabric: None,
            place3d: None,
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
    pub role: InnerRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InnerRole {
    #[default]
    Shape,
    Fold,
    Dart,
    Reference,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            id: 0,
            kind: InnerKind::Line,
            points: Vec::new(),
            closed: false,
            fold_angle: 180.0,
            fold_strength: 0.0,
            role: InnerRole::Shape,
        }
    }
}
