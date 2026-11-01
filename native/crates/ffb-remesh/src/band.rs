use std::collections::{HashMap, HashSet};

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::geom::{cross, dist, dot, inside, len, sub, Bucket};
use crate::types::V2;

pub struct Assembled {
    pub positions: Vec<V2>,
    pub quads: Vec<[u32; 4]>,
    pub tris: Vec<[u32; 3]>,
    pub lattice_quads: usize,
    pub band_quads: usize,
    pub band_tris: usize,
}

fn in_quad(q: [V2; 4], p: V2) -> bool {
    (0..4).all(|k| cross(sub(q[(k + 1) % 4], q[k]), sub(p, q[k])) >= -1e-9)
}

fn merged_score(c: V2, a: V2, d: V2, b: V2) -> Option<f64> {
    let p = [c, a, d, b];
    let mut worst = 0.0f64;
    let mut lmin = f64::MAX;
    let mut lmax = 0.0f64;
    for k in 0..4 {
        let (x, y, z) = (p[(k + 3) % 4], p[k], p[(k + 1) % 4]);
        let u = sub(x, y);
        let v = sub(z, y);
        let (lu, lv) = (len(u), len(v));
        if lu < 1e-9 || lv < 1e-9 {
            return None;
        }
        lmin = lmin.min(lu).min(lv);
        lmax = lmax.max(lu).max(lv);
        let ang = cross(v, u).atan2(dot(u, v)).to_degrees();
        if ang <= 8.0 || ang >= 172.0 {
            return None;
        }
        worst = worst.max((ang - 90.0).abs());
    }
    Some(worst + 20.0 * (lmax / lmin - 1.0).min(3.0))
}

pub fn assemble(loop_pts: &[V2], sites: &[V2], lattice: &[[u32; 4]]) -> Assembled {
    let nb = loop_pts.len();
    let mut remap: HashMap<u32, u32> = HashMap::new();
    let mut positions: Vec<V2> = loop_pts.to_vec();
    for q in lattice {
        for v in q {
            remap.entry(*v).or_insert_with(|| {
                positions.push(sites[*v as usize]);
                positions.len() as u32 - 1
            });
        }
    }
    let quads_l: Vec<[u32; 4]> = lattice.iter().map(|q| [remap[&q[0]], remap[&q[1]], remap[&q[2]], remap[&q[3]]]).collect();

    let mut count: HashMap<(u32, u32), u32> = HashMap::new();
    for q in &quads_l {
        for k in 0..4 {
            let (a, b) = (q[k], q[(k + 1) % 4]);
            *count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    let mut edges: Vec<[usize; 2]> = (0..nb).map(|i| [i, (i + 1) % nb]).collect();
    let mut seen_edge: HashSet<(u32, u32)> = HashSet::new();
    for (e, c) in &count {
        if *c == 1 && seen_edge.insert(*e) {
            edges.push([e.0 as usize, e.1 as usize]);
        }
    }
    edges.sort_unstable();

    let points: Vec<Point2<f64>> = positions.iter().map(|p| Point2::new(p[0], p[1])).collect();
    let mut tris_all: Vec<[u32; 3]> = Vec::new();
    if let Ok(cdt) = ConstrainedDelaunayTriangulation::<Point2<f64>>::try_bulk_load_cdt(points, edges, |_| {}) {
        let cell = loop_pts.windows(2).map(|w| dist(w[0], w[1])).sum::<f64>() / nb.max(1) as f64;
        let centres: Vec<V2> = quads_l.iter().map(|q| [(positions[q[0] as usize][0] + positions[q[2] as usize][0]) * 0.5, (positions[q[0] as usize][1] + positions[q[2] as usize][1]) * 0.5]).collect();
        let bucket = Bucket::new(&centres, (cell * 2.0).max(1e-6));
        for face in cdt.inner_faces() {
            let v = face.vertices();
            let idx = [v[0].fix().index() as u32, v[1].fix().index() as u32, v[2].fix().index() as u32];
            let (a, b, c) = (positions[idx[0] as usize], positions[idx[1] as usize], positions[idx[2] as usize]);
            let centroid = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
            if !inside(loop_pts, centroid) {
                continue;
            }
            let covered = bucket.near(centroid, cell * 4.0).into_iter().any(|qi| {
                let q = quads_l[qi as usize];
                in_quad([positions[q[0] as usize], positions[q[1] as usize], positions[q[2] as usize], positions[q[3] as usize]], centroid)
            });
            if !covered {
                tris_all.push(idx);
            }
        }
    }
    for t in tris_all.iter_mut() {
        let (a, b, c) = (positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]);
        if cross(sub(b, a), sub(c, a)) < 0.0 {
            t.swap(1, 2);
        }
    }
    tris_all.retain(|t| {
        let (a, b, c) = (positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]);
        cross(sub(b, a), sub(c, a)) > 1e-12
    });

    let mut directed: HashMap<(u32, u32), usize> = HashMap::new();
    for (i, t) in tris_all.iter().enumerate() {
        for k in 0..3 {
            directed.insert((t[k], t[(k + 1) % 3]), i);
        }
    }
    let mut pairs: Vec<(f64, usize, usize, [u32; 4])> = Vec::new();
    for (i, t) in tris_all.iter().enumerate() {
        for k in 0..3 {
            let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
            let Some(&j) = directed.get(&(b, a)) else { continue };
            if j < i {
                continue;
            }
            let u = &tris_all[j];
            let d = *u.iter().find(|x| **x != a && **x != b).unwrap_or(&a);
            let (pa, pb, pc, pd) = (positions[a as usize], positions[b as usize], positions[c as usize], positions[d as usize]);
            if let Some(score) = merged_score(pc, pa, pd, pb) {
                pairs.push((score, i, j, [c, a, d, b]));
            }
        }
    }
    pairs.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
    let mut taken = vec![false; tris_all.len()];
    let mut band_quads: Vec<[u32; 4]> = Vec::new();
    for (score, i, j, q) in &pairs {
        if *score > 60.0 || taken[*i] || taken[*j] {
            continue;
        }
        taken[*i] = true;
        taken[*j] = true;
        band_quads.push(*q);
    }
    for (score, i, j, q) in &pairs {
        if *score > 130.0 || taken[*i] || taken[*j] {
            continue;
        }
        taken[*i] = true;
        taken[*j] = true;
        band_quads.push(*q);
    }
    let tris: Vec<[u32; 3]> = tris_all.iter().enumerate().filter(|(i, _)| !taken[*i]).map(|(_, t)| *t).collect();
    let (lattice_quads, band_count, band_tris) = (quads_l.len(), band_quads.len(), tris.len());
    let mut quads = quads_l;
    quads.extend(band_quads);
    Assembled { positions, quads, tris, lattice_quads, band_quads: band_count, band_tris }
}
