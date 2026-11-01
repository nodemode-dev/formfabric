use std::collections::{BTreeMap, HashMap};

use ffb_mesh::PanelMesh;
use ffb_pattern::Garment;
use ffb_sim::Fold;

use super::fx::{self, Mode};
use super::geo::{self, V2};

fn polyline(points: &[[f32; 2]]) -> Vec<V2> {
    points.iter().map(|p| [f64::from(p[0]), f64::from(p[1])]).collect()
}

pub fn styleline_stiffen(garment: &Garment, meshes: &BTreeMap<u32, PanelMesh>) -> BTreeMap<u32, Vec<f32>> {
    let mut out: BTreeMap<u32, Vec<f32>> = BTreeMap::new();
    for line in &garment.style_lines {
        let f = fx::line_fx(garment, line.id);
        if f.mode != Mode::Stiffen || f.stiffen <= 1.0 {
            continue;
        }
        let Some(mesh) = meshes.get(&line.panel) else { continue };
        if garment.panels.iter().any(|p| p.id == line.panel && p.state.disabled) {
            continue;
        }
        let segs = geo::segments(&polyline(&line.points), line.closed);
        let half = f.width * 0.5;
        let slot = out.entry(line.panel).or_insert_with(|| vec![1.0; mesh.triangles.len()]);
        if slot.len() != mesh.triangles.len() {
            continue;
        }
        for (k, t) in mesh.triangles.iter().enumerate() {
            let c: Vec<V2> = t.iter().filter_map(|i| mesh.positions.get(*i as usize).copied()).collect();
            if c.len() != 3 {
                continue;
            }
            let centre = [(c[0][0] + c[1][0] + c[2][0]) / 3.0, (c[0][1] + c[1][1] + c[2][1]) / 3.0];
            if c.iter().chain(std::iter::once(&centre)).any(|p| geo::dist_to(&segs, *p) <= half) {
                slot[k] = slot[k].max(f.stiffen as f32);
            }
        }
    }
    out.retain(|_, v| v.iter().any(|x| *x > 1.0));
    out
}

pub fn styleline_folds(garment: &Garment, meshes: &BTreeMap<u32, PanelMesh>, starts: &BTreeMap<u32, u32>) -> Vec<Fold> {
    let mut out = Vec::new();
    for line in &garment.style_lines {
        let f = fx::line_fx(garment, line.id);
        if f.mode != Mode::Fold {
            continue;
        }
        let (Some(mesh), Some(&start)) = (meshes.get(&line.panel), starts.get(&line.panel)) else { continue };
        if garment.panels.iter().any(|p| p.id == line.panel && p.state.disabled) {
            continue;
        }
        let segs = geo::segments(&polyline(&line.points), line.closed);
        let mut edges: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
        for t in &mesh.triangles {
            for (a, b, o) in [(t[0], t[1], t[2]), (t[1], t[2], t[0]), (t[2], t[0], t[1])] {
                edges.entry((a.min(b), a.max(b))).or_default().push(o);
            }
        }
        let mut keys: Vec<_> = edges.keys().copied().collect();
        keys.sort_unstable();
        for (a, b) in keys {
            let Some(opp) = edges.get(&(a, b)).filter(|o| o.len() == 2) else { continue };
            let (pa, pb) = (mesh.positions[a as usize], mesh.positions[b as usize]);
            let len = (pb[0] - pa[0]).hypot(pb[1] - pa[1]);
            if len < 1e-9 {
                continue;
            }
            let mid = [(pa[0] + pb[0]) * 0.5, (pa[1] + pb[1]) * 0.5];
            let reach = (f.width * 0.5).max(len * 0.5);
            let Some((d, dir)) = segs
                .iter()
                .map(|s| (geo::seg_closest(mid, s.0, s.1).0, [s.1[0] - s.0[0], s.1[1] - s.0[1]]))
                .min_by(|x, y| x.0.total_cmp(&y.0))
            else {
                continue;
            };
            if d > reach {
                continue;
            }
            let dl = dir[0].hypot(dir[1]);
            if dl < 1e-9 {
                continue;
            }
            let cos = ((pb[0] - pa[0]) * dir[0] + (pb[1] - pa[1]) * dir[1]).abs() / (len * dl);
            if cos < 0.8 {
                continue;
            }
            out.push(Fold { edge: [start + a, start + b], opposite: [start + opp[0], start + opp[1]], angle: f.fold_angle.to_radians() as f32, strength: f.fold_strength as f32 });
        }
    }
    out
}
