use std::collections::BTreeMap;

use ffb_mesh::resample::{arc_length, natural_count};
use ffb_mesh::PanelMesh;
use ffb_pattern::sew::{self, Binding, MeshView};
use ffb_pattern::{path, Garment};

use crate::{Edit, Session};

fn forced_counts(garment: &Garment) -> BTreeMap<u32, BTreeMap<usize, usize>> {
    let mut out: BTreeMap<u32, BTreeMap<usize, usize>> = BTreeMap::new();
    for seam in &garment.seams {
        if (seam.ratio - 1.0).abs() > 1e-9 || seam.a.len() != 1 || seam.b.len() != 1 {
            continue;
        }
        let (ea, eb) = (&seam.a[0], &seam.b[0]);
        if !sew::is_full(ea) || !sew::is_full(eb) {
            continue;
        }
        let natural = |panel: u32, segment: usize| -> Option<usize> {
            let p = garment.panels.iter().find(|p| p.id == panel)?;
            if segment >= path::seg_count(p.outline.len(), true) || !p.particle_distance.is_finite() || p.particle_distance <= 0.0 {
                return None;
            }
            let tol = ffb_mesh::resample::flatten_tolerance(p.particle_distance);
            Some(natural_count(arc_length(&path::seg(&p.outline, segment), tol), p.particle_distance))
        };
        let (Some(na), Some(nb)) = (natural(ea.panel, ea.segment), natural(eb.panel, eb.segment)) else { continue };
        let n = na.max(nb);
        for (panel, segment) in [(ea.panel, ea.segment), (eb.panel, eb.segment)] {
            let slot = out.entry(panel).or_default().entry(segment).or_insert(n);
            *slot = (*slot).max(n);
        }
    }
    out
}

fn sync_counts(s: &mut Session) {
    let mut wanted = forced_counts(&s.garment);
    let known: Vec<u32> = s.mesh.forced.keys().copied().collect();
    for panel in known {
        wanted.entry(panel).or_default();
    }
    for (panel, counts) in wanted {
        if s.mesh.forced.get(&panel).cloned().unwrap_or_default() != counts {
            s.mesh.set_segment_counts(panel, counts);
        }
    }
}

fn repair(s: &mut Session, ops: Vec<sew::Op>) {
    if s.garment.seams.is_empty() && s.garment.tacks.is_empty() {
        return;
    }
    let (seams, tacks) = (s.garment.seams.clone(), s.garment.tacks.clone());
    sew::apply_pending(&mut s.garment, ops);
    sew::prune(&mut s.garment);
    if s.garment.seams == seams && s.garment.tacks == tacks {
        return;
    }
    let after = s.garment.clone();
    let mut before = after.clone();
    before.seams = seams;
    before.tacks = tacks;
    s.history.amend_last(Edit::Garment { label: "Seam Update".into(), before: Box::new(before), after: Box::new(after) });
}

pub(crate) fn after_change(s: &mut Session, command: &str) {
    let ops = sew::take_pending();
    if !command.starts_with("history.") && !command.starts_with("file.") {
        repair(s, ops);
    }
    sync_counts(s);
}

pub fn seam_bindings(garment: &Garment, meshes: &BTreeMap<u32, PanelMesh>) -> Vec<Binding> {
    let views: BTreeMap<u32, MeshView> = meshes
        .iter()
        .map(|(id, m)| (*id, MeshView { positions: &m.positions, boundary: &m.boundary, boundary_param: &m.boundary_param, inner_edges: &m.inner_edges }))
        .collect();
    sew::seam_bindings(garment, &views)
}

pub fn session_bindings(s: &mut Session) -> Vec<Binding> {
    let Session { garment, mesh, .. } = s;
    let mut views: BTreeMap<u32, PanelMesh> = BTreeMap::new();
    for panel in &garment.panels {
        if let Ok(built) = mesh.ensure(panel, false).built {
            views.insert(panel.id, built.mesh.clone());
        }
    }
    seam_bindings(garment, &views)
}
