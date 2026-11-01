use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod ffab;
pub mod lab;
pub mod presets;
pub mod units;
pub mod validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fabric {
    pub id: u32,
    pub name: String,
    pub preset: Option<String>,
    pub physical: Physical,
    pub look: Look,
}

impl Default for Fabric {
    fn default() -> Self {
        Fabric { id: 0, name: "Fabric".into(), preset: None, physical: Physical::default(), look: Look::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Physical {
    pub stretch_weft: f64,
    pub stretch_warp: f64,
    pub shear: f64,
    pub bending_weft: f64,
    pub bending_warp: f64,
    pub bending_bias: f64,
    pub buckling_ratio: f64,
    pub buckling_stiffness: f64,
    pub density: f64,
    pub friction: f64,
    pub thickness: f64,
    pub damping: f64,
}

impl Default for Physical {
    fn default() -> Self {
        Physical {
            stretch_weft: 5000.0,
            stretch_warp: 6000.0,
            shear: 500.0,
            bending_weft: 6.0,
            bending_warp: 8.0,
            bending_bias: 7.0,
            buckling_ratio: 0.5,
            buckling_stiffness: 0.5,
            density: 130.0,
            friction: 0.3,
            thickness: 0.35,
            damping: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LookKind {
    #[default]
    Matte,
    Shiny,
    Satin,
    Velvet,
    Leather,
    Metal,
    Plastic,
    Glass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Look {
    pub kind: LookKind,
    pub front: Surface,
    pub back: Option<Surface>,
    pub side: Option<Surface>,
}

impl Default for Look {
    fn default() -> Self {
        Look { kind: LookKind::Matte, front: Surface::default(), back: None, side: None }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Surface {
    pub color: [f32; 4],
    pub roughness: f32,
    pub metalness: f32,
    pub sheen: f32,
    pub opacity: f32,
    pub maps: Maps,
    pub map_scale: [f64; 2],
    pub map_rotation: f64,
    pub map_offset: [f64; 2],
}

impl Default for Surface {
    fn default() -> Self {
        Surface {
            color: [0.78, 0.78, 0.78, 1.0],
            roughness: 0.85,
            metalness: 0.0,
            sheen: 0.0,
            opacity: 1.0,
            maps: Maps::default(),
            map_scale: [100.0, 100.0],
            map_rotation: 0.0,
            map_offset: [0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Maps {
    pub base_color: Option<String>,
    pub normal: Option<String>,
    pub roughness: Option<String>,
    pub metalness: Option<String>,
    pub opacity: Option<String>,
    pub displacement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Colorway {
    pub id: u32,
    pub name: String,
    pub fabric_colors: BTreeMap<u32, [f32; 4]>,
}
