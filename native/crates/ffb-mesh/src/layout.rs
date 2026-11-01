use ffb_pattern::{Panel, Place3D};

use crate::PanelMesh;

pub fn quaternion(q: [f64; 4]) -> [f64; 4] {
    let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
    if n < 1e-12 || !n.is_finite() {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
    }
}

pub fn rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let [x, y, z, w] = q;
    let t = [2.0 * (y * v[2] - z * v[1]), 2.0 * (z * v[0] - x * v[2]), 2.0 * (x * v[1] - y * v[0])];
    [
        v[0] + w * t[0] + (y * t[2] - z * t[1]),
        v[1] + w * t[1] + (z * t[0] - x * t[2]),
        v[2] + w * t[2] + (x * t[1] - y * t[0]),
    ]
}

pub fn centre_x(mesh: &PanelMesh) -> f64 {
    let (lo, hi) = mesh.positions.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p[0]), hi.max(p[0])));
    if lo > hi {
        0.0
    } else {
        (lo + hi) * 0.5
    }
}

pub fn bent(x: f64, cx: f64, radius: f64) -> (f64, f64) {
    if radius.abs() < 1e-6 {
        return (x, 0.0);
    }
    let angle = (x - cx) / radius;
    (cx + radius * angle.sin(), radius * (angle.cos() - 1.0))
}

pub fn mirrored_flat(panel: &Panel) -> bool {
    panel.place3d.is_none() && panel.mirror
}

pub fn positions_3d(mesh: &PanelMesh, panel: &Panel) -> Vec<f32> {
    let mut out = Vec::with_capacity(mesh.positions.len() * 3);
    match &panel.place3d {
        Some(Place3D { origin, rotation, bend }) => {
            let q = quaternion(*rotation);
            let cx = centre_x(mesh);
            for p in &mesh.positions {
                let (bx, bz) = bent(p[0], cx, *bend);
                let r = rotate(q, [bx, p[1], bz]);
                out.extend_from_slice(&[(origin[0] + r[0]) as f32, (origin[1] + r[1]) as f32, (origin[2] + r[2]) as f32]);
            }
        }
        None => {
            let place = panel.placement();
            for p in &mesh.positions {
                let w = place.to_world(*p);
                out.extend_from_slice(&[w[0] as f32, w[1] as f32, 0.0]);
            }
        }
    }
    out
}

pub fn indices(mesh: &PanelMesh, flip: bool) -> Vec<u32> {
    let mut out = Vec::with_capacity(mesh.triangles.len() * 3);
    for t in &mesh.triangles {
        if flip {
            out.extend_from_slice(&[t[0], t[2], t[1]]);
        } else {
            out.extend_from_slice(t);
        }
    }
    out
}

pub fn panel_ids(mesh: &PanelMesh) -> Vec<u16> {
    vec![(mesh.panel & 0xffff) as u16; mesh.triangles.len()]
}
