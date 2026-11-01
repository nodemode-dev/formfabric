use ffb_mesh::hash::Fnv;
use ffb_pattern::Panel;

pub(crate) fn placement_signature(panel: &Panel) -> u64 {
    let mut h = Fnv::default();
    match &panel.place3d {
        Some(p) => {
            h.u64(1);
            for v in p.origin.iter().chain(p.rotation.iter()).chain(std::iter::once(&p.bend)) {
                h.f64(*v);
            }
            if let Some(pose) = &p.pose {
                h.u64(pose.len() as u64);
                for v in pose.iter().flatten() {
                    h.u64(u64::from(v.to_bits()));
                }
            }
        }
        None => {
            h.u64(0);
            h.pair(panel.position);
            h.f64(panel.rotation);
            h.u64(u64::from(panel.mirror));
        }
    }
    h.finish()
}

type M3 = [[f64; 3]; 3];

fn det(m: &M3) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn inverse_transpose(m: &M3) -> Option<M3> {
    let d = det(m);
    if d.abs() < 1e-18 {
        return None;
    }
    let c = |r1: usize, c1: usize, r2: usize, c2: usize| m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1];
    let cof = [
        [c(1, 1, 2, 2), -c(1, 0, 2, 2), c(1, 0, 2, 1)],
        [-c(0, 1, 2, 2), c(0, 0, 2, 2), -c(0, 0, 2, 1)],
        [c(0, 1, 1, 2), -c(0, 0, 1, 2), c(0, 0, 1, 1)],
    ];
    let mut out = [[0.0; 3]; 3];
    for r in 0..3 {
        for k in 0..3 {
            out[r][k] = cof[r][k] / d;
        }
    }
    Some(out)
}

fn quaternion(r: &M3) -> [f64; 4] {
    let t = r[0][0] + r[1][1] + r[2][2];
    let q = if t > 0.0 {
        let s = (t + 1.0).sqrt() * 2.0;
        [(r[2][1] - r[1][2]) / s, (r[0][2] - r[2][0]) / s, (r[1][0] - r[0][1]) / s, 0.25 * s]
    } else if r[0][0] > r[1][1] && r[0][0] > r[2][2] {
        let s = (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt() * 2.0;
        [0.25 * s, (r[0][1] + r[1][0]) / s, (r[0][2] + r[2][0]) / s, (r[2][1] - r[1][2]) / s]
    } else if r[1][1] > r[2][2] {
        let s = (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt() * 2.0;
        [(r[0][1] + r[1][0]) / s, 0.25 * s, (r[1][2] + r[2][1]) / s, (r[0][2] - r[2][0]) / s]
    } else {
        let s = (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt() * 2.0;
        [(r[0][2] + r[2][0]) / s, (r[1][2] + r[2][1]) / s, 0.25 * s, (r[1][0] - r[0][1]) / s]
    };
    ffb_mesh::layout::quaternion(q)
}

pub(crate) fn rigid_fit(rest: &[[f32; 2]], pos: &[[f32; 3]], triangles: &[[u32; 3]]) -> Option<([f64; 3], [f64; 4])> {
    let n = rest.len();
    if n < 3 || pos.len() != n {
        return None;
    }
    let inv = 1.0 / n as f64;
    let rc = rest.iter().fold([0.0; 2], |a, p| [a[0] + f64::from(p[0]) * inv, a[1] + f64::from(p[1]) * inv]);
    let pc = pos.iter().fold([0.0; 3], |a, p| [a[0] + f64::from(p[0]) * inv, a[1] + f64::from(p[1]) * inv, a[2] + f64::from(p[2]) * inv]);
    let mut m: M3 = [[0.0; 3]; 3];
    let mut scale = 0.0;
    for (q, p) in rest.iter().zip(pos) {
        let q = [f64::from(q[0]) - rc[0], f64::from(q[1]) - rc[1], 0.0];
        let p = [f64::from(p[0]) - pc[0], f64::from(p[1]) - pc[1], f64::from(p[2]) - pc[2]];
        for r in 0..3 {
            for c in 0..3 {
                m[r][c] += p[r] * q[c];
            }
        }
        scale += q[0] * q[0] + q[1] * q[1];
    }
    let mut normal = [0.0f64; 3];
    for t in triangles {
        let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
        let u = [f64::from(b[0] - a[0]), f64::from(b[1] - a[1]), f64::from(b[2] - a[2])];
        let v = [f64::from(c[0] - a[0]), f64::from(c[1] - a[1]), f64::from(c[2] - a[2])];
        normal[0] += u[1] * v[2] - u[2] * v[1];
        normal[1] += u[2] * v[0] - u[0] * v[2];
        normal[2] += u[0] * v[1] - u[1] * v[0];
    }
    let len = normal.iter().map(|x| x * x).sum::<f64>().sqrt();
    if len < 1e-12 || scale < 1e-12 {
        return None;
    }
    let weight = m.iter().flatten().map(|x| x * x).sum::<f64>().sqrt() / std::f64::consts::SQRT_2;
    for r in 0..3 {
        m[r][2] += normal[r] / len * weight;
    }
    let mut rot = m;
    for _ in 0..40 {
        let it = inverse_transpose(&rot)?;
        let mut next = rot;
        let mut delta = 0.0;
        for r in 0..3 {
            for c in 0..3 {
                next[r][c] = 0.5 * (rot[r][c] + it[r][c]);
                delta += (next[r][c] - rot[r][c]).abs();
            }
        }
        rot = next;
        if delta < 1e-12 {
            break;
        }
    }
    if det(&rot) < 0.0 {
        for row in rot.iter_mut() {
            row[2] = -row[2];
        }
    }
    let origin = [
        pc[0] - (rot[0][0] * rc[0] + rot[0][1] * rc[1]),
        pc[1] - (rot[1][0] * rc[0] + rot[1][1] * rc[1]),
        pc[2] - (rot[2][0] * rc[0] + rot[2][1] * rc[1]),
    ];
    Some((origin, quaternion(&rot)))
}
