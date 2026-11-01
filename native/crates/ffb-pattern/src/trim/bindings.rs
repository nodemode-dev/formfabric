use std::collections::BTreeMap;

use super::locate::{locate, Barycentric, TrimMesh};
use super::style::{flag_param, param};
use super::zipper::zipper_bindings;
use crate::sew::{Binding, MeshView};
use crate::{Garment, Hardware, StyleKind};

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareBinding {
    pub hardware: u32,
    pub style: u32,
    pub kind: StyleKind,
    pub at: Barycentric,
    pub rotation: f64,
    pub width: f64,
    pub thickness: f64,
    pub weight: f64,
    pub collide: bool,
    pub off_mesh: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fastening {
    pub button: u32,
    pub buttonhole: u32,
    pub a: Barycentric,
    pub b: Barycentric,
    pub rest_length: f64,
    pub stiffness: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrimBindings {
    pub hardware: Vec<HardwareBinding>,
    pub fastenings: Vec<Fastening>,
    pub zippers: Vec<Binding>,
}

fn attach(garment: &Garment, meshes: &BTreeMap<u32, TrimMesh>, h: &Hardware) -> Option<HardwareBinding> {
    let style = garment.styles.iter().find(|s| s.id == h.style)?;
    let mesh = meshes.get(&h.anchor.panel)?;
    let (at, off_mesh) = locate(h.anchor.panel, mesh, h.anchor.point)?;
    Some(HardwareBinding {
        hardware: h.id,
        style: style.id,
        kind: style.kind,
        at,
        rotation: h.rotation,
        width: param(style, "width", 12.0),
        thickness: param(style, "thickness", 3.0),
        weight: param(style, "weight", 1.5),
        collide: flag_param(style, "collision", true),
        off_mesh,
    })
}

pub fn trim_bindings(garment: &Garment, meshes: &BTreeMap<u32, TrimMesh>) -> TrimBindings {
    let mut out = TrimBindings::default();
    for h in &garment.hardware {
        if garment.panels.iter().any(|p| p.id == h.anchor.panel && p.state.disabled) {
            continue;
        }
        if let Some(binding) = attach(garment, meshes, h) {
            out.hardware.push(binding);
        }
    }
    for b in out.hardware.iter().filter(|b| b.kind == StyleKind::Button) {
        let Some(hole_id) = garment.hardware.iter().find(|h| h.id == b.hardware).and_then(|h| h.fastened_to) else { continue };
        let Some(hole) = out.hardware.iter().find(|x| x.hardware == hole_id && x.kind == StyleKind::Buttonhole) else { continue };
        let stiffness = garment.styles.iter().find(|s| s.id == b.style).map_or(1.0, |s| param(s, "stiffness", 1.0));
        out.fastenings.push(Fastening { button: b.hardware, buttonhole: hole.hardware, a: b.at, b: hole.at, rest_length: 0.0, stiffness });
    }
    let views: BTreeMap<u32, MeshView> = meshes.iter().map(|(id, m)| (*id, m.view())).collect();
    out.zippers = zipper_bindings(garment, &views);
    out
}
