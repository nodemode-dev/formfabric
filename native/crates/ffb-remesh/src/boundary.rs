use std::collections::BTreeMap;

use ffb_mesh::resample::{arc_length, flatten_tolerance, natural_count};
use ffb_mesh::{build_panel, MeshOptions, PanelMesh};
use ffb_pattern::{path, sew, Garment, Panel};

use crate::geom::signed_area;
use crate::types::{fail, Params, RemeshError, V2};

pub const REFINE: usize = 3;
const MAX_FINE_VERTICES: f64 = 400_000.0;

pub struct Work {
    pub mesh: PanelMesh,
    pub loop_ids: Vec<u32>,
    pub rim: Vec<(usize, usize, usize)>,
    pub h: f64,
}

pub fn segment_length(panel: &Panel, segment: usize) -> f64 {
    let tol = flatten_tolerance(panel.particle_distance.max(1e-3));
    arc_length(&path::seg(&panel.outline, segment), tol)
}

pub fn natural_counts(panel: &Panel, edge: f64) -> BTreeMap<usize, usize> {
    let mut out = BTreeMap::new();
    for s in 0..path::seg_count(panel.outline.len(), true) {
        let length = segment_length(panel, s);
        if length >= 1e-6 {
            out.insert(s, natural_count(length, edge));
        }
    }
    out
}

pub fn seam_counts(garment: &Garment, edge_of: &dyn Fn(&Panel) -> f64) -> BTreeMap<u32, BTreeMap<usize, usize>> {
    let mut out: BTreeMap<u32, BTreeMap<usize, usize>> = BTreeMap::new();
    for seam in &garment.seams {
        if (seam.ratio - 1.0).abs() > 1e-9 || seam.a.len() != 1 || seam.b.len() != 1 {
            continue;
        }
        let (ea, eb) = (&seam.a[0], &seam.b[0]);
        if !sew::is_full(ea) || !sew::is_full(eb) {
            continue;
        }
        let find = |id: u32, segment: usize| -> Option<(&Panel, usize)> {
            let p = garment.panels.iter().find(|p| p.id == id)?;
            if segment >= path::seg_count(p.outline.len(), true) || !p.particle_distance.is_finite() || p.particle_distance <= 0.0 {
                return None;
            }
            let length = segment_length(p, segment);
            (length >= 1e-6).then(|| (p, natural_count(length, edge_of(p))))
        };
        let (Some((pa, na)), Some((pb, nb))) = (find(ea.panel, ea.segment), find(eb.panel, eb.segment)) else { continue };
        let n = na.max(nb);
        for (panel, segment) in [(pa.id, ea.segment), (pb.id, eb.segment)] {
            let slot = out.entry(panel).or_default().entry(segment).or_insert(n);
            *slot = (*slot).max(n);
        }
    }
    out
}

pub fn work_mesh(panel: &Panel, params: &Params) -> Result<Work, RemeshError> {
    let edge = params.edge;
    if !edge.is_finite() || edge <= 0.0 {
        return fail("the target edge length must be a positive number");
    }
    if panel.outline.len() < 3 {
        return fail("the outline needs at least 3 points");
    }
    let h = edge / REFINE as f64;
    let mut counts = natural_counts(panel, edge);
    for (s, c) in &params.counts {
        if counts.contains_key(s) {
            counts.insert(*s, (*c).max(1));
        }
    }
    let flat = path::flatten(&panel.outline, true, (h * 0.05).clamp(0.01, 1.0));
    let area = signed_area(&flat).abs();
    if area / (0.433 * h * h) > MAX_FINE_VERTICES {
        return fail(format!("panel {} needs too many vertices at an edge length of {edge:.2} mm; raise the edge length or lower the density", panel.name));
    }
    let forced: BTreeMap<usize, usize> = counts.iter().map(|(s, c)| (*s, c * REFINE)).collect();
    let mut fine = panel.clone();
    fine.particle_distance = h;
    fine.inner.clear();
    let options = MeshOptions { segment_counts: forced, ..MeshOptions::default() };
    let mesh = build_panel(&fine, &options).map_err(|e| RemeshError(e.to_string()))?;

    let nb = mesh.boundary.len();
    let mut loop_ids: Vec<u32> = Vec::new();
    let mut rim: Vec<(usize, usize, usize)> = Vec::new();
    let mut i = 0;
    while i < nb {
        let segment = mesh.boundary_param[i].0;
        let mut j = i;
        while j < nb && mesh.boundary_param[j].0 == segment {
            j += 1;
        }
        let group = j - i;
        if group % REFINE != 0 {
            return fail("the working mesh boundary does not follow the requested sample counts");
        }
        for k in (0..group).step_by(REFINE) {
            loop_ids.push(mesh.boundary[i + k]);
            rim.push((segment, k / REFINE, group / REFINE));
        }
        i = j;
    }
    if loop_ids.len() < 3 {
        return fail("the outline has too few boundary samples");
    }
    let pts: Vec<V2> = loop_ids.iter().map(|v| mesh.positions[*v as usize]).collect();
    if signed_area(&pts) < 0.0 {
        loop_ids.reverse();
        rim.reverse();
    }
    Ok(Work { mesh, loop_ids, rim, h })
}
