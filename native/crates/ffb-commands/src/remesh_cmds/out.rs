use std::collections::BTreeMap;

use ffb_mesh::layout;
use ffb_pattern::output::MeshKind;
use ffb_pattern::{path, sew, Panel};
use ffb_remesh::{boundary, measure, remesh_panel, Params, QuadMesh};
use serde_json::{json, Value};

use super::state::{guides_of, key_of, Entry};
use crate::Session;

#[derive(Debug, Clone, Default)]
pub struct QuadPanel {
    pub panel: u32,
    pub name: String,
    pub positions2d: Vec<[f64; 2]>,
    pub positions: Vec<[f32; 3]>,
    pub quads: Vec<[u32; 4]>,
    pub tris: Vec<[u32; 3]>,
    pub boundary: Vec<(u32, usize, usize, usize)>,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now_ms() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() * 1000.0
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn now_ms() -> f64 {
    0.0
}

pub(crate) fn round(x: f64, places: i32) -> f64 {
    let k = 10f64.powi(places);
    (x * k).round() / k
}

pub(crate) fn pure_flag(s: &Session) -> bool {
    s.garment.output.retopo.get("pure_quads").and_then(Value::as_bool).unwrap_or(false)
}

pub(crate) fn density_of(s: &Session) -> f64 {
    let d = f64::from(s.garment.output.quad_density);
    if d.is_finite() && d > 0.0 {
        d
    } else {
        1.0
    }
}

pub(crate) fn edge_for(panel: &Panel, density: f64) -> f64 {
    panel.particle_distance / density
}

fn counts_for(s: &Session, density: f64) -> BTreeMap<u32, BTreeMap<usize, usize>> {
    boundary::seam_counts(&s.garment, &|p| edge_for(p, density))
}

pub(crate) fn summary(id: u32, e: &Entry) -> Value {
    let m = &e.metrics;
    let mut per_segment: BTreeMap<usize, usize> = BTreeMap::new();
    for r in &e.mesh.rim {
        per_segment.insert(r.segment, r.count);
    }
    json!({
        "panel": id,
        "vertices": m.vertices,
        "quads": m.quads,
        "triangles": m.triangles,
        "percent_quads": round(m.percent_quads, 2),
        "interior_vertices": m.interior_vertices,
        "irregular_interior": m.irregular_interior,
        "valence_histogram": {"valence": [0, 1, 2, 3, 4, 5, 6, 7, "8+"], "counts": m.valence_histogram},
        "max_angle_deviation": round(m.max_angle_deviation, 2),
        "mean_angle_deviation": round(m.mean_angle_deviation, 2),
        "boundary_vertices": m.boundary_vertices,
        "edge": {"min": round(m.edge_min, 3), "mean": round(m.edge_mean, 3), "max": round(m.edge_max, 3), "target": round(e.mesh.edge, 3)},
        "boundary_counts": per_segment,
        "report": e.mesh.report,
        "ms": round(e.ms, 1),
        "error": null
    })
}

pub(crate) fn run(s: &mut Session, ids: &[u32], density: f64, pure: bool) -> Vec<Value> {
    let seam = counts_for(s, density);
    let mut rows = Vec::with_capacity(ids.len());
    for id in ids {
        let Some(index) = s.garment.index_of(*id) else { continue };
        let panel = s.garment.panels[index].clone();
        let counts = seam.get(id).cloned().unwrap_or_default();
        let guides = guides_of(&s.garment, *id);
        let mut params = Params::new(edge_for(&panel, density));
        params.counts = counts.clone();
        params.guides = guides.clone();
        params.pure_quads = pure;
        let start = now_ms();
        match remesh_panel(&panel, &params) {
            Ok(mesh) => {
                let metrics = measure(&mesh);
                let entry = Entry { key: key_of(&panel, density, pure, &counts, &guides), mesh, metrics, ms: now_ms() - start };
                rows.push(summary(*id, &entry));
                s.remesh.meshes.insert(*id, entry);
            }
            Err(e) => {
                s.remesh.meshes.remove(id);
                rows.push(json!({"panel": id, "vertices": 0, "quads": 0, "triangles": 0, "error": e.to_string()}));
            }
        }
    }
    rows
}

struct Base {
    positions: Vec<[f64; 2]>,
    triangles: Vec<[u32; 3]>,
    values: Vec<[f64; 3]>,
    flip: bool,
}

fn base_3d(s: &mut Session, id: u32) -> Option<Base> {
    let Session { garment, mesh, .. } = s;
    let panel = garment.panels.iter().find(|p| p.id == id)?;
    let built = mesh.ensure(panel, false).built.ok()?;
    let n = built.mesh.positions.len();
    let flat: Vec<f32> = match panel.place3d.as_ref().and_then(|p| p.pose.as_ref()).filter(|p| p.len() == n) {
        Some(pose) => pose.iter().flat_map(|p| p.iter().copied()).collect(),
        None => layout::positions_3d(&built.mesh, panel),
    };
    let values: Vec<[f64; 3]> = flat.chunks_exact(3).map(|c| [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])]).collect();
    Some(Base { positions: built.mesh.positions.clone(), triangles: built.mesh.triangles.clone(), values, flip: layout::mirrored_flat(panel) })
}

pub(crate) fn lifted(s: &mut Session, id: u32) -> Option<(Vec<[f64; 3]>, bool)> {
    let base = base_3d(s, id)?;
    let entry = s.remesh.meshes.get(&id)?;
    Some((ffb_remesh::lift(&base.positions, &base.triangles, &base.values, &entry.mesh.positions), base.flip))
}

fn fresh(s: &mut Session) {
    if s.garment.output.mesh != MeshKind::Quad {
        return;
    }
    let (density, pure) = (density_of(s), pure_flag(s));
    let seam = counts_for(s, density);
    let mut stale: Vec<u32> = Vec::new();
    for panel in &s.garment.panels {
        let counts = seam.get(&panel.id).cloned().unwrap_or_default();
        let key = key_of(panel, density, pure, &counts, &guides_of(&s.garment, panel.id));
        if s.remesh.meshes.get(&panel.id).is_none_or(|e| e.key != key) {
            stale.push(panel.id);
        }
    }
    if !stale.is_empty() {
        run(s, &stale, density, pure);
    }
    let live: Vec<u32> = s.garment.panels.iter().map(|p| p.id).collect();
    s.remesh.meshes.retain(|id, _| live.contains(id));
}

pub fn quad_meshes(s: &mut Session) -> Vec<QuadPanel> {
    fresh(s);
    if s.garment.output.mesh != MeshKind::Quad {
        return Vec::new();
    }
    let ids: Vec<u32> = s.garment.panels.iter().map(|p| p.id).collect();
    let mut out = Vec::new();
    for id in ids {
        let Some((points, flip)) = lifted(s, id) else { continue };
        let Some(entry) = s.remesh.meshes.get(&id) else { continue };
        let name = s.garment.panels.iter().find(|p| p.id == id).map(|p| p.name.clone()).unwrap_or_default();
        let m: &QuadMesh = &entry.mesh;
        let wind = |q: [u32; 4]| if flip { [q[3], q[2], q[1], q[0]] } else { q };
        let wind3 = |t: [u32; 3]| if flip { [t[2], t[1], t[0]] } else { t };
        out.push(QuadPanel {
            panel: id,
            name,
            positions2d: m.positions.clone(),
            positions: points.iter().map(|p| [p[0] as f32, p[1] as f32, p[2] as f32]).collect(),
            quads: m.quads.iter().copied().map(wind).collect(),
            tris: m.tris.iter().copied().map(wind3).collect(),
            boundary: m.rim.iter().map(|r| (r.vertex, r.segment, r.index, r.count)).collect(),
        });
    }
    out
}

fn seam_line(mesh: &QuadMesh, segment: usize) -> Option<Vec<u32>> {
    let mut mine: Vec<(usize, u32)> = mesh.rim.iter().filter(|r| r.segment == segment).map(|r| (r.index, r.vertex)).collect();
    if mine.is_empty() {
        return None;
    }
    mine.sort_unstable();
    let mut ids: Vec<u32> = mine.iter().map(|x| x.1).collect();
    let present: std::collections::BTreeSet<usize> = mesh.rim.iter().map(|r| r.segment).collect();
    let next = present.range(segment + 1..).next().or_else(|| present.iter().next()).copied()?;
    let end = mesh.rim.iter().find(|r| r.segment == next && r.index == 0)?;
    ids.push(end.vertex);
    Some(ids)
}

pub(crate) fn seam_report(s: &mut Session) -> Value {
    let seams = s.garment.seams.clone();
    let mut checked = 0usize;
    let mut mismatched = 0usize;
    let mut worst = 0.0f64;
    let mut rows: Vec<Value> = Vec::new();
    let mut cache: BTreeMap<u32, Vec<[f64; 3]>> = BTreeMap::new();
    for seam in &seams {
        if seam.a.len() != 1 || seam.b.len() != 1 || !sew::is_full(&seam.a[0]) || !sew::is_full(&seam.b[0]) {
            continue;
        }
        let (ea, eb) = (&seam.a[0], &seam.b[0]);
        if ea.segment >= path::seg_count(s.garment.panels.iter().find(|p| p.id == ea.panel).map_or(0, |p| p.outline.len()), true) {
            continue;
        }
        let (Some(ma), Some(mb)) = (s.remesh.meshes.get(&ea.panel), s.remesh.meshes.get(&eb.panel)) else { continue };
        let (Some(la), Some(lb)) = (seam_line(&ma.mesh, ea.segment), seam_line(&mb.mesh, eb.segment)) else { continue };
        let fractions = |m: &QuadMesh, line: &[u32]| -> Vec<f64> {
            let mut acc = vec![0.0];
            for w in line.windows(2) {
                let (p, q) = (m.positions[w[0] as usize], m.positions[w[1] as usize]);
                acc.push(acc[acc.len() - 1] + (p[0] - q[0]).hypot(p[1] - q[1]));
            }
            let total = acc[acc.len() - 1].max(1e-12);
            acc.iter().map(|x| x / total).collect()
        };
        let fa = fractions(&ma.mesh, &la);
        let mut fb = fractions(&mb.mesh, &lb);
        let mut lb = lb;
        if seam.reversed {
            lb.reverse();
            fb = fb.iter().rev().map(|x| 1.0 - x).collect();
        }
        checked += 1;
        let equal = la.len() == lb.len();
        let mut along = 0.0f64;
        let mut gap: Option<f64> = None;
        if equal {
            along = fa.iter().zip(&fb).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
            let placed = |id: u32| s.garment.panels.iter().find(|p| p.id == id).is_some_and(|p| p.place3d.is_some());
            if placed(ea.panel) && placed(eb.panel) {
                for panel in [ea.panel, eb.panel] {
                    if let std::collections::btree_map::Entry::Vacant(slot) = cache.entry(panel) {
                        if let Some((p, _)) = lifted(s, panel) {
                            slot.insert(p);
                        }
                    }
                }
                if let (Some(pa), Some(pb)) = (cache.get(&ea.panel), cache.get(&eb.panel)) {
                    let mut g = 0.0f64;
                    for (x, y) in la.iter().zip(&lb) {
                        let (u, v) = (pa[*x as usize], pb[*y as usize]);
                        g = g.max(((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt());
                    }
                    gap = Some(g);
                }
            }
        } else {
            mismatched += 1;
        }
        worst = worst.max(along);
        rows.push(json!({"seam": seam.id, "a": {"panel": ea.panel, "segment": ea.segment, "vertices": la.len()}, "b": {"panel": eb.panel, "segment": eb.segment, "vertices": lb.len()}, "equal_counts": equal, "max_arc_fraction_gap": round(along, 6), "max_gap_3d_mm": gap.map(|g| round(g, 3))}));
    }
    json!({"checked": checked, "count_mismatches": mismatched, "max_arc_fraction_gap": round(worst, 6), "ok": mismatched == 0 && worst < 0.02, "seams": rows})
}
