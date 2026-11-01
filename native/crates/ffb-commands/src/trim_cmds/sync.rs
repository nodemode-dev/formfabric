use std::collections::BTreeMap;

use ffb_mesh::PanelMesh;
use ffb_pattern::sew::Op;
use ffb_pattern::trim::{self, TrimBindings, TrimMesh};
use ffb_pattern::Garment;

use crate::{Edit, Session};

fn views(meshes: &BTreeMap<u32, PanelMesh>) -> BTreeMap<u32, TrimMesh<'_>> {
    meshes
        .iter()
        .map(|(id, m)| (*id, TrimMesh { positions: &m.positions, triangles: &m.triangles, boundary: &m.boundary, boundary_param: &m.boundary_param, inner_edges: &m.inner_edges }))
        .collect()
}

pub fn trim_bindings(garment: &Garment, meshes: &BTreeMap<u32, PanelMesh>) -> TrimBindings {
    trim::trim_bindings(garment, &views(meshes))
}

pub fn stiffen_bands(garment: &Garment, meshes: &BTreeMap<u32, PanelMesh>) -> BTreeMap<u32, Vec<f32>> {
    trim::stiffen_bands(garment, &views(meshes))
}

pub fn session_meshes(s: &mut Session) -> BTreeMap<u32, PanelMesh> {
    let Session { garment, mesh, .. } = s;
    let mut out = BTreeMap::new();
    for panel in &garment.panels {
        if let Ok(built) = mesh.ensure(panel, false).built {
            out.insert(panel.id, built.mesh.clone());
        }
    }
    out
}

#[allow(dead_code)]
pub fn session_trim_bindings(s: &mut Session) -> TrimBindings {
    let meshes = session_meshes(s);
    trim_bindings(&s.garment, &meshes)
}

#[allow(dead_code)]
pub fn session_stiffen_bands(s: &mut Session) -> BTreeMap<u32, Vec<f32>> {
    let meshes = session_meshes(s);
    stiffen_bands(&s.garment, &meshes)
}

fn holds_trim_data(g: &Garment) -> bool {
    !(g.stitches.is_empty() && g.zippers.is_empty() && g.finishes.is_empty() && g.hardware.is_empty() && g.ripped.is_empty())
}

pub fn follow(s: &mut Session, ops: &[Op]) {
    if !holds_trim_data(&s.garment) {
        return;
    }
    let before = s.garment.clone();
    trim::follow(&mut s.garment, ops);
    if s.garment == before {
        return;
    }
    let after = s.garment.clone();
    s.history.amend_last(Edit::Garment { label: "Trim Update".into(), before: Box::new(before), after: Box::new(after) });
}

pub(crate) fn after_change(s: &mut Session, command: &str) {
    if command.starts_with("history.") || command.starts_with("file.") {
        return;
    }
    follow(s, &[]);
}
