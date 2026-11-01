use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::geom::{cross, dot, sub};
use crate::types::QuadMesh;

pub const VALENCE_BINS: usize = 9;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Metrics {
    pub vertices: usize,
    pub quads: usize,
    pub triangles: usize,
    pub percent_quads: f64,
    pub interior_vertices: usize,
    pub irregular_interior: usize,
    pub valence_histogram: [usize; VALENCE_BINS],
    pub max_angle_deviation: f64,
    pub mean_angle_deviation: f64,
    pub boundary_vertices: usize,
    pub edge_min: f64,
    pub edge_mean: f64,
    pub edge_max: f64,
}

fn deviation(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let u = sub(a, b);
    let v = sub(c, b);
    let ang = cross(v, u).atan2(dot(u, v)).to_degrees();
    (ang - 90.0).abs()
}

pub fn measure(mesh: &QuadMesh) -> Metrics {
    let p = &mesh.positions;
    let rim: HashSet<u32> = mesh.rim.iter().map(|r| r.vertex).collect();
    let mut nbr: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut edges: HashSet<(u32, u32)> = HashSet::new();
    let mut each = |f: &[u32]| {
        let n = f.len();
        for k in 0..n {
            let (a, b) = (f[k], f[(k + 1) % n]);
            nbr.entry(a).or_default().insert(b);
            nbr.entry(b).or_default().insert(a);
            edges.insert((a.min(b), a.max(b)));
        }
    };
    for q in &mesh.quads {
        each(q);
    }
    for t in &mesh.tris {
        each(t);
    }
    let mut m = Metrics { vertices: nbr.len(), quads: mesh.quads.len(), triangles: mesh.tris.len(), boundary_vertices: rim.len(), ..Metrics::default() };
    let faces = m.quads + m.triangles;
    m.percent_quads = if faces == 0 { 0.0 } else { 100.0 * m.quads as f64 / faces as f64 };
    for (v, set) in &nbr {
        if rim.contains(v) {
            continue;
        }
        m.interior_vertices += 1;
        let val = set.len();
        m.valence_histogram[val.min(VALENCE_BINS - 1)] += 1;
        if val != 4 {
            m.irregular_interior += 1;
        }
    }
    let mut sum = 0.0;
    let mut count = 0usize;
    for q in &mesh.quads {
        for k in 0..4 {
            let d = deviation(p[q[(k + 3) % 4] as usize], p[q[k] as usize], p[q[(k + 1) % 4] as usize]);
            m.max_angle_deviation = m.max_angle_deviation.max(d);
            sum += d;
            count += 1;
        }
    }
    if count > 0 {
        m.mean_angle_deviation = sum / count as f64;
    }
    if !edges.is_empty() {
        let mut lo = f64::MAX;
        let mut hi = 0.0f64;
        let mut total = 0.0;
        for (a, b) in &edges {
            let l = crate::geom::dist(p[*a as usize], p[*b as usize]);
            lo = lo.min(l);
            hi = hi.max(l);
            total += l;
        }
        m.edge_min = lo;
        m.edge_max = hi;
        m.edge_mean = total / edges.len() as f64;
    }
    m
}
