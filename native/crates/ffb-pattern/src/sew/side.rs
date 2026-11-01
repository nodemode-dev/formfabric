use crate::{path, polygon, Garment, Panel, Seam, SeamSide};

fn turn(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
    let [x, y, z, w] = if n < 1e-12 || !n.is_finite() { [0.0, 0.0, 0.0, 1.0] } else { [q[0] / n, q[1] / n, q[2] / n, q[3] / n] };
    let t = [2.0 * (y * v[2] - z * v[1]), 2.0 * (z * v[0] - x * v[2]), 2.0 * (x * v[1] - y * v[0])];
    [v[0] + w * t[0] + (y * t[2] - z * t[1]), v[1] + w * t[1] + (z * t[0] - x * t[2]), v[2] + w * t[2] + (x * t[1] - y * t[0])]
}

fn face_normal(panel: &Panel) -> [f64; 3] {
    match &panel.place3d {
        Some(p) => turn(p.rotation, [0.0, 0.0, 1.0]),
        None => [0.0, 0.0, if panel.mirror { -1.0 } else { 1.0 }],
    }
}

fn centre(panel: &Panel) -> [f64; 3] {
    let local = polygon::centroid(&path::flatten(&panel.outline, true, 0.5));
    match &panel.place3d {
        Some(p) => {
            let r = turn(p.rotation, [local[0], local[1], 0.0]);
            [p.origin[0] + r[0], p.origin[1] + r[1], p.origin[2] + r[2]]
        }
        None => {
            let w = panel.placement().to_world(local);
            [w[0], w[1], 0.0]
        }
    }
}

pub fn resolve_side(garment: &Garment, seam: &Seam) -> SeamSide {
    if seam.side != SeamSide::Auto {
        return seam.side;
    }
    let find = |id: u32| garment.panels.iter().find(|p| p.id == id);
    let (Some(a), Some(b)) = (seam.a.first().and_then(|e| find(e.panel)), seam.b.first().and_then(|e| find(e.panel))) else {
        return SeamSide::Front;
    };
    let (ca, cb, n) = (centre(a), centre(b), face_normal(a));
    let dot = n[0] * (cb[0] - ca[0]) + n[1] * (cb[1] - ca[1]) + n[2] * (cb[2] - ca[2]);
    if dot < -1e-6 {
        SeamSide::Back
    } else {
        SeamSide::Front
    }
}
