use serde::Serialize;

use crate::PanelMesh;

pub const BINS: usize = 6;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Quality {
    pub vertices: usize,
    pub triangles: usize,
    pub min_angle: f64,
    pub mean_min_angle: f64,
    pub below_20: usize,
    pub histogram: [usize; BINS],
    pub edge_min: f64,
    pub edge_mean: f64,
    pub edge_max: f64,
    pub area: f64,
}

fn angle(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let u = [b[0] - a[0], b[1] - a[1]];
    let v = [c[0] - a[0], c[1] - a[1]];
    let cross = u[0] * v[1] - u[1] * v[0];
    let dot = u[0] * v[0] + u[1] * v[1];
    cross.abs().atan2(dot).to_degrees()
}

fn edge(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

pub fn measure(mesh: &PanelMesh) -> Quality {
    let mut q = Quality { vertices: mesh.positions.len(), triangles: mesh.triangles.len(), min_angle: 180.0, edge_min: f64::MAX, ..Quality::default() };
    let mut sum_min = 0.0;
    let mut edge_sum = 0.0;
    for t in &mesh.triangles {
        let [a, b, c] = [mesh.positions[t[0] as usize], mesh.positions[t[1] as usize], mesh.positions[t[2] as usize]];
        let m = angle(a, b, c).min(angle(b, c, a)).min(angle(c, a, b));
        q.min_angle = q.min_angle.min(m);
        sum_min += m;
        q.below_20 += usize::from(m < 20.0);
        let bin = ((m / 10.0) as usize).min(BINS - 1);
        q.histogram[bin] += 1;
        for e in [edge(a, b), edge(b, c), edge(c, a)] {
            q.edge_min = q.edge_min.min(e);
            q.edge_max = q.edge_max.max(e);
            edge_sum += e;
        }
        q.area += 0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs();
    }
    if q.triangles == 0 {
        q.min_angle = 0.0;
        q.edge_min = 0.0;
    } else {
        q.mean_min_angle = sum_min / q.triangles as f64;
        q.edge_mean = edge_sum / (3 * q.triangles) as f64;
    }
    q
}
