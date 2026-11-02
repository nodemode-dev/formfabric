use std::sync::OnceLock;

use ffb_fabric::Physical;
use ffb_mesh::layout;
use ffb_pattern::Garment;
use ffb_sim::collide::Prepared;
use ffb_sim::{Collider, Material};

use crate::{CommandError, Session};

pub(crate) struct PanelScene {
    pub id: u32,
    pub rest: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
    pub material: Material,
    pub density: f32,
    pub thickness: f32,
    pub positions: Vec<[f32; 3]>,
}

pub(crate) struct Scene {
    pub version: u64,
    pub panels: Vec<PanelScene>,
    pub offset: f32,
    pub shell: Option<f32>,
    bodies: Vec<Collider>,
    prepared: OnceLock<Vec<Prepared>>,
}

impl Scene {
    pub fn bodies(&self) -> &[Prepared] {
        self.prepared.get_or_init(|| self.bodies.iter().map(Prepared::from_collider).collect())
    }

    pub fn has_bodies(&self) -> bool {
        !self.bodies.is_empty()
    }
}

fn physical_for(garment: &Garment, fabric: Option<u32>) -> Physical {
    let wanted = fabric.or(garment.default_fabric);
    let found = wanted.and_then(|id| garment.fabrics.iter().find(|f| f.id == id)).or_else(|| garment.fabrics.first());
    found.map(|f| f.physical.clone()).unwrap_or_default()
}

fn stiffened(m: &Material, k: f32) -> Material {
    let k = if k.is_finite() && k > 0.0 { k } else { 1.0 };
    Material { stretch_weft: m.stretch_weft * k, stretch_warp: m.stretch_warp * k, shear: m.shear * k, ..*m }
}

pub(crate) fn build(s: &mut Session, version: u64) -> Result<Scene, CommandError> {
    let ids: Vec<u32> = s.garment.panels.iter().filter(|p| !p.state.disabled).map(|p| p.id).collect();
    let mut panels = Vec::new();
    for id in ids {
        let Some(index) = s.garment.index_of(id) else { continue };
        let panel = s.garment.panels[index].clone();
        let Ok(built) = s.mesh.ensure(&panel, false).built else { continue };
        let m = &built.mesh;
        if m.positions.is_empty() || m.triangles.is_empty() {
            continue;
        }
        let positions = crate::sim_cmds::current_positions(s, id)?;
        if positions.len() != m.positions.len() {
            continue;
        }
        let flip = layout::mirrored_flat(&panel);
        let grain = (if flip { 180.0 - panel.grain } else { panel.grain }).to_radians();
        let (cg, sg) = (grain.cos(), grain.sin());
        let physical = physical_for(&s.garment, panel.fabric);
        let mut material = stiffened(&Material::from_physical(&physical), panel.state.stiffen as f32);
        let shrink = |x: f64| if x.is_finite() && x > 1.0e-6 { x } else { 1.0 };
        let (sw, sp) = (shrink(panel.state.shrink_weft), shrink(panel.state.shrink_warp));
        material.shrink_weft = sw as f32;
        material.shrink_warp = sp as f32;
        let rest = m
            .positions
            .iter()
            .map(|p| {
                let x = if flip { -p[0] } else { p[0] };
                [(x * cg + p[1] * sg) * sw, (-x * sg + p[1] * cg) * sp]
            })
            .collect();
        panels.push(PanelScene {
            id,
            rest,
            triangles: layout::indices(m, flip).as_chunks::<3>().0.to_vec(),
            material,
            density: physical.density as f32,
            thickness: if physical.thickness > 0.0 { physical.thickness as f32 } else { 0.3 },
            positions,
        });
    }
    if panels.is_empty() {
        return Err(CommandError::new("invalid_geometry", "no panel has a mesh"));
    }
    let bodies = s.bodies.iter().filter(|b| b.visible && !b.triangles.is_empty()).map(ffb_body::collider).collect();
    let offset = s.sim.params.lock().map_or(ffb_sim::Params::default().skin_offset, |p| p.skin_offset);
    let shell = s.garment.output.thickness.enabled.then_some(s.garment.output.thickness.amount).filter(|a| a.is_finite() && *a > 0.0);
    Ok(Scene { version, panels, offset, shell, bodies, prepared: OnceLock::new() })
}
