use std::collections::{BTreeMap, HashMap};

use ffb_fabric::Physical;
use ffb_mesh::hash::Fnv;
use ffb_mesh::layout;
use ffb_mesh::PanelMesh;
use ffb_pattern::sew::{self, Binding, MeshView};
use ffb_pattern::Garment;
use ffb_sim::{Cloth, Collider, PinPoint, SeamBind, SeamPair};

use super::adapter;
use super::drape::placement_signature;
use super::seams;
use super::states::{Anchor, Extras};
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
    pub extras: Extras,
}

#[derive(Default)]
struct Derived {
    list: Vec<(u16, u32, u16)>,
}

impl Derived {
    fn scaled(&mut self, cloth: &mut Cloth, base: u16, factor: f32) -> u16 {
        let bits = factor.to_bits();
        if let Some((_, _, made)) = self.list.iter().find(|(b, f, _)| *b == base && *f == bits) {
            return *made;
        }
        let made = adapter::stiffened(&cloth.materials[base as usize], factor);
        cloth.materials.push(made);
        let index = (cloth.materials.len() - 1) as u16;
        self.list.push((base, bits, index));
        index
    }
}

fn physical_for(garment: &Garment, fabric: Option<u32>) -> (u32, Physical) {
    let wanted = fabric.or(garment.default_fabric);
    let found = wanted.and_then(|id| garment.fabrics.iter().find(|f| f.id == id)).or_else(|| garment.fabrics.first());
    match found {
        Some(f) => (f.id, f.physical.clone()),
        None => (0, Physical::default()),
    }
}

pub(crate) fn assemble(garment: &Garment, mesh: &mut MeshState, bodies: &[ffb_body::Body]) -> Result<Assembled, String> {
    let mut cloth = Cloth::default();
    let mut slices: Vec<Slice> = Vec::new();
    let mut meshes: BTreeMap<u32, PanelMesh> = BTreeMap::new();
    let mut materials: Vec<(u32, [u32; 4])> = Vec::new();
    let mut folds = Vec::new();
    let mut extras = Extras::default();
    let mut tri_ranges: HashMap<u32, (usize, usize)> = HashMap::new();
    for panel in &garment.panels {
        if panel.state.disabled {
            continue;
        }
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
        let positions: Vec<[f32; 3]> = match panel.place3d.as_ref().and_then(|p| p.pose.as_ref()).filter(|pose| pose.len() == m.positions.len()) {
            Some(pose) => pose.clone(),
            None => layout::positions_3d(m, panel).chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
        };
        let (fabric_id, physical) = physical_for(garment, panel.fabric);
        let pressure = if panel.pressure.is_finite() { panel.pressure as f32 } else { 0.0 };
        h.u64(u64::from(pressure.to_bits()));
        let state = &panel.state;
        let key = [pressure.to_bits(), (state.stiffen as f32).to_bits(), (state.shrink_weft as f32).to_bits(), (state.shrink_warp as f32).to_bits()];
        let material = match materials.iter().position(|(id, k)| *id == fabric_id && *k == key) {
            Some(i) => i,
            None => {
                let mut made = adapter::stiffened(&adapter::material(&physical), state.stiffen as f32);
                made.pressure = pressure;
                made.shrink_weft = state.shrink_weft as f32;
                made.shrink_warp = state.shrink_warp as f32;
                cloth.materials.push(made);
                materials.push((fabric_id, key));
                materials.len() - 1
            }
        };
        let grain = if flip { 180.0 - panel.grain } else { panel.grain };
        cloth.positions.extend(positions);
        cloth.rest.extend(m.positions.iter().map(|p| [if flip { -p[0] as f32 } else { p[0] as f32 }, p[1] as f32]));
        let base = start as u32;
        let tri_start = cloth.triangles.len();
        cloth.triangles.extend(layout::indices(m, flip).chunks_exact(3).map(|t| [base + t[0], base + t[1], base + t[2]]));
        tri_ranges.insert(panel.id, (tri_start, cloth.triangles.len() - tri_start));
        cloth.triangle_material.resize(cloth.triangles.len(), material as u16);
        cloth.triangle_grain.resize(cloth.triangles.len(), grain.to_radians() as f32);
        cloth.vertex_panel.resize(cloth.positions.len(), panel.id);
        folds.extend(seams::inner_folds(panel, m, base));
        slices.push(Slice { id: panel.id, start, count: m.positions.len(), sig: h.finish() });
        if state.frozen {
            extras.frozen.push((start, m.positions.len()));
        } else if state.hold_shape {
            extras.hold.push((start, m.positions.len()));
        }
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
    let sewn = sew::seam_bindings(garment, &views).into_iter().filter(|b| {
        let (Binding::Pair { seam, .. } | Binding::Edge { seam, .. }) = b;
        !garment.ripped.contains(seam)
    });
    for binding in sewn.chain(adapter::trim_bindings(garment, &meshes)) {
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
    let mut derived = Derived::default();
    for (id, factors) in adapter::stiffen_bands(garment, &meshes) {
        let Some(&(from, count)) = tri_ranges.get(&id) else { continue };
        for (k, factor) in factors.iter().enumerate().take(count) {
            if *factor > 1.0001 {
                let base = cloth.triangle_material[from + k];
                cloth.triangle_material[from + k] = derived.scaled(&mut cloth, base, *factor);
            }
        }
    }
    let vertex_at = |end: &ffb_pattern::TackEnd| -> Option<usize> {
        let id = end.panel?;
        let local = nearest_vertex(meshes.get(&id)?, end.point)?;
        Some(start_of(id)? + local)
    };
    let link_k = cloth.materials.iter().map(|m| m.stretch_weft.max(m.stretch_warp)).fold(0.0, f32::max).max(1.0);
    for tack in &garment.tacks {
        match (vertex_at(&tack.a), vertex_at(&tack.b), tack.a.body_point.or(tack.b.body_point)) {
            (Some(a), Some(b), _) if a != b => cloth.seams.push(SeamPair { a: a as u32, b: b as u32, rest_length: tack.length as f32, stiffness: link_k }),
            (Some(v), None, Some(p)) | (None, Some(v), Some(p)) => {
                extras.anchors.push(Anchor { vertex: v as u32, point: [p[0] as f32, p[1] as f32, p[2] as f32], length: tack.length as f32, stiffness: link_k });
            }
            _ => {}
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
    Ok(Assembled { cloth, slices, colliders, extras })
}

pub(crate) fn nearest_vertex(mesh: &PanelMesh, point: [f64; 2]) -> Option<usize> {
    mesh.positions
        .iter()
        .enumerate()
        .map(|(i, p)| (i, (p[0] - point[0]).powi(2) + (p[1] - point[1]).powi(2)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}
