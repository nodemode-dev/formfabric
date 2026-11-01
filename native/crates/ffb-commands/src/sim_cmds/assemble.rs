use std::collections::{BTreeMap, HashMap};

use ffb_fabric::Physical;
use ffb_mesh::hash::Fnv;
use ffb_mesh::layout;
use ffb_mesh::PanelMesh;
use ffb_pattern::sew::{self, Binding, MeshView};
use ffb_pattern::Garment;
use ffb_sim::{Cloth, Collider, PinPoint, SeamBind, SeamPair};

use super::adapter;
use super::drape::{placement_signature, DrapeCache};
use super::seams;
use crate::mesh_cmds::MeshState;

#[derive(Debug, Clone)]
pub(crate) struct Slice {
    pub id: u32,
    pub start: usize,
    pub count: usize,
    pub sig: u64,
}

pub(crate) struct Assembled {
    pub cloth: Cloth,
    pub slices: Vec<Slice>,
    pub colliders: Vec<Collider>,
}

fn physical_for(garment: &Garment, fabric: Option<u32>) -> (u32, Physical) {
    let wanted = fabric.or(garment.default_fabric);
    let found = wanted.and_then(|id| garment.fabrics.iter().find(|f| f.id == id)).or_else(|| garment.fabrics.first());
    match found {
        Some(f) => (f.id, f.physical.clone()),
        None => (0, Physical::default()),
    }
}

pub(crate) fn assemble(garment: &Garment, mesh: &mut MeshState, bodies: &[ffb_body::Body], cache: &DrapeCache) -> Result<Assembled, String> {
    let mut cloth = Cloth::default();
    let mut slices: Vec<Slice> = Vec::new();
    let mut meshes: HashMap<u32, PanelMesh> = HashMap::new();
    let mut materials: Vec<(u32, u32)> = Vec::new();
    let mut folds = Vec::new();
    for panel in &garment.panels {
        let ensured = mesh.ensure(panel, false);
        let Ok(built) = ensured.built else { continue };
        let m = &built.mesh;
        if m.positions.is_empty() || m.triangles.is_empty() {
            continue;
        }
        let flip = layout::mirrored_flat(panel);
        let start = cloth.positions.len();
        let place_sig = placement_signature(panel);
        let mut h = Fnv::default();
        h.u64(ensured.key);
        h.u64(place_sig);
        let flat = layout::positions_3d(m, panel);
        let positions: Vec<[f32; 3]> = match cache.get(panel.id, place_sig, m.positions.len()) {
            Some(cached) => cached.as_ref().clone(),
            None => flat.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
        };
        let (fabric_id, physical) = physical_for(garment, panel.fabric);
        let pressure = if panel.pressure.is_finite() { panel.pressure as f32 } else { 0.0 };
        h.u64(u64::from(pressure.to_bits()));
        let material = match materials.iter().position(|(id, bits)| *id == fabric_id && *bits == pressure.to_bits()) {
            Some(i) => i,
            None => {
                let mut made = adapter::material(&physical);
                made.pressure = pressure;
                cloth.materials.push(made);
                materials.push((fabric_id, pressure.to_bits()));
                materials.len() - 1
            }
        };
        let grain = if flip { 180.0 - panel.grain } else { panel.grain };
        cloth.positions.extend(positions);
        cloth.rest.extend(m.positions.iter().map(|p| [if flip { -p[0] as f32 } else { p[0] as f32 }, p[1] as f32]));
        let base = start as u32;
        cloth.triangles.extend(layout::indices(m, flip).chunks_exact(3).map(|t| [base + t[0], base + t[1], base + t[2]]));
        cloth.triangle_material.resize(cloth.triangles.len(), material as u16);
        cloth.triangle_grain.resize(cloth.triangles.len(), grain.to_radians() as f32);
        cloth.vertex_panel.resize(cloth.positions.len(), panel.id);
        folds.extend(seams::inner_folds(panel, m, base));
        slices.push(Slice { id: panel.id, start, count: m.positions.len(), sig: h.finish() });
        meshes.insert(panel.id, m.clone());
    }
    if slices.is_empty() {
        return Err("no panel has a mesh to simulate".into());
    }
    let start_of = |id: u32| slices.iter().find(|s| s.id == id).map(|s| s.start);
    let views: BTreeMap<u32, MeshView> = meshes
        .iter()
        .map(|(id, m)| (*id, MeshView { positions: &m.positions, boundary: &m.boundary, boundary_param: &m.boundary_param, inner_edges: &m.inner_edges }))
        .collect();
    for binding in sew::seam_bindings(garment, &views) {
        match binding {
            Binding::Pair { a, b, rest_length, stiffness, .. } => {
                if let (Some(sa), Some(sb)) = (start_of(a.panel), start_of(b.panel)) {
                    cloth.seams.push(SeamPair { a: (sa + a.vertex as usize) as u32, b: (sb + b.vertex as usize) as u32, rest_length: rest_length as f32, stiffness: stiffness as f32 });
                }
            }
            Binding::Edge { a, b0, b1, w, rest_length, stiffness, .. } => {
                if let (Some(sa), Some(s0), Some(s1)) = (start_of(a.panel), start_of(b0.panel), start_of(b1.panel)) {
                    cloth.binds.push(SeamBind {
                        a: (sa + a.vertex as usize) as u32,
                        b: [(s0 + b0.vertex as usize) as u32, (s1 + b1.vertex as usize) as u32],
                        w: w as f32,
                        stiffness: stiffness as f32,
                        rest_length: rest_length as f32,
                    });
                }
            }
        }
    }
    for pin in &garment.pins {
        let (Some(m), Some(start)) = (meshes.get(&pin.panel), start_of(pin.panel)) else { continue };
        let Some(local) = nearest_vertex(m, pin.point) else { continue };
        let vertex = start + local;
        let held = cloth.positions[vertex];
        cloth.pins.push(PinPoint { vertex: vertex as u32, target: pin.target.map_or(held, |t| [t[0] as f32, t[1] as f32, t[2] as f32]) });
    }
    cloth.folds = folds;
    let mut colliders = adapter::body_colliders(bodies);
    colliders.push(adapter::floor());
    Ok(Assembled { cloth, slices, colliders })
}

pub(crate) fn nearest_vertex(mesh: &PanelMesh, point: [f64; 2]) -> Option<usize> {
    mesh.positions
        .iter()
        .enumerate()
        .map(|(i, p)| (i, (p[0] - point[0]).powi(2) + (p[1] - point[1]).powi(2)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}
