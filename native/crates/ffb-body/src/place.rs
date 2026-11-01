use crate::math::{qconj, qmul, qrot, V3};
use crate::skin::{self, Frames};
use crate::{Body, PlacementPoint, PlacementVolume};

pub type V = [f64; 3];

pub fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V, k: f64) -> V {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn length(a: V) -> f64 {
    dot(a, a).sqrt()
}

pub fn unit(a: V) -> V {
    let l = length(a);
    if l > 1e-12 {
        scale(a, 1.0 / l)
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn v64(a: V3) -> V {
    [f64::from(a[0]), f64::from(a[1]), f64::from(a[2])]
}

#[derive(Debug, Clone, Copy)]
pub struct VolumeFrame {
    pub start: V,
    pub end: V,
    pub axis: V,
    pub height: f64,
    pub front: V,
    pub side: V,
    pub radius: [f64; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct PointFrame {
    pub center: V,
    pub surface: V,
    pub position: V,
    pub normal: V,
    pub up: V,
    pub bend_radius: f64,
}

pub struct Rig {
    bind: Frames,
    posed: Frames,
}

impl Rig {
    pub fn new(body: &Body) -> Rig {
        Rig { bind: skin::bind_frames(body), posed: skin::frames(&body.joints, Some(&body.pose)) }
    }

    fn carry(&self, p: V, joint: Option<u16>) -> V {
        let Some(j) = joint.map(usize::from).filter(|j| *j < self.bind.pos.len()) else { return p };
        let delta = qmul(self.posed.rot[j], qconj(self.bind.rot[j]));
        let local = [(p[0] - f64::from(self.bind.pos[j][0])) as f32, (p[1] - f64::from(self.bind.pos[j][1])) as f32, (p[2] - f64::from(self.bind.pos[j][2])) as f32];
        add(v64(self.posed.pos[j]), v64(qrot(delta, local)))
    }

    pub fn volume(&self, v: &PlacementVolume) -> VolumeFrame {
        let axis_rest = unit(v.axis);
        let height = v.height.max(1e-3);
        let start_rest = sub(v.center, scale(axis_rest, height * 0.5));
        let end_rest = add(v.center, scale(axis_rest, height * 0.5));
        let j0 = v.joints[0].or(v.joints[1]);
        let j1 = v.joints[1].or(v.joints[0]);
        let start = self.carry(start_rest, j0);
        let end = self.carry(end_rest, j1);
        let span = sub(end, start);
        let (axis, height) = if length(span) > 1e-6 { (unit(span), length(span)) } else { (axis_rest, height) };
        let reference = if axis[2].abs() > 0.9 { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
        let front = unit(sub(reference, scale(axis, dot(reference, axis))));
        let side = cross(axis, front);
        VolumeFrame { start, end, axis, height, front, side, radius: [v.radius[0].max(1.0), v.radius[1].max(1.0)] }
    }

    pub fn point(&self, volume: &PlacementVolume, point: &PlacementPoint) -> PointFrame {
        let f = self.volume(volume);
        let theta = point.around.to_radians();
        let (sin, cos) = theta.sin_cos();
        let center = add(f.start, scale(f.axis, point.along * f.height));
        let surface = add(center, add(scale(f.side, f.radius[0] * sin), scale(f.front, f.radius[1] * cos)));
        let normal = unit(add(scale(f.side, sin / f.radius[0]), scale(f.front, cos / f.radius[1])));
        let up = if f.axis[1].abs() >= 0.2 {
            if f.axis[1] > 0.0 {
                f.axis
            } else {
                scale(f.axis, -1.0)
            }
        } else {
            scale(f.axis, -1.0)
        };
        let speed2 = f.radius[0] * f.radius[0] * cos * cos + f.radius[1] * f.radius[1] * sin * sin;
        let osculating = speed2 * speed2.sqrt() / (f.radius[0] * f.radius[1]);
        let mean = ((f.radius[0] * f.radius[0] + f.radius[1] * f.radius[1]) * 0.5).sqrt();
        let position = add(surface, scale(normal, point.offset));
        PointFrame { center, surface, position, normal, up, bend_radius: osculating.max(mean) + point.offset.max(0.0) }
    }
}

pub fn quaternion_from_columns(c0: V, c1: V, c2: V) -> [f64; 4] {
    let (m00, m10, m20) = (c0[0], c0[1], c0[2]);
    let (m01, m11, m21) = (c1[0], c1[1], c1[2]);
    let (m02, m12, m22) = (c2[0], c2[1], c2[2]);
    let tr = m00 + m11 + m22;
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s]
    };
    let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
    [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

pub fn rotate(q: [f64; 4], v: V) -> V {
    let [x, y, z, w] = q;
    let t = [2.0 * (y * v[2] - z * v[1]), 2.0 * (z * v[0] - x * v[2]), 2.0 * (x * v[1] - y * v[0])];
    [v[0] + w * t[0] + (y * t[2] - z * t[1]), v[1] + w * t[1] + (z * t[0] - x * t[2]), v[2] + w * t[2] + (x * t[1] - y * t[0])]
}
