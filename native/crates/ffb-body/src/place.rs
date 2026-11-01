use std::cell::Cell;
use std::collections::HashMap;

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

    pub fn fit(&self, body: &Body, volume: &PlacementVolume, point: &PlacementPoint, clearance: f64) -> PointFrame {
        let base = self.point(volume, point);
        let f = self.volume(volume);
        let mean = ((f.radius[0] * f.radius[0] + f.radius[1] * f.radius[1]) * 0.5).sqrt();
        let positions = skin::posed_positions(body);
        let center = add(f.start, scale(f.axis, point.along * f.height));
        let cap = 1.6 * f.radius[0].max(f.radius[1]);
        let theta = point.around.to_radians();
        let want = (f.radius[0] * theta.sin()).atan2(f.radius[1] * theta.cos());
        let Some(ring) = Ring::slice(&positions, &body.triangles, &f, center, cap, true) else { return base };
        let equivalent = ring.perimeter / std::f64::consts::TAU;
        if !(equivalent > mean * 0.4 && equivalent < mean * 2.0) {
            return base;
        }
        let (dist, tangent) = ring.hit(want);
        let dir = add(scale(f.side, want.sin()), scale(f.front, want.cos()));
        let mut normal = unit(add(scale(f.side, tangent[1]), scale(f.front, -tangent[0])));
        if dot(normal, dir) < 0.0 {
            normal = scale(normal, -1.0);
        }
        let surface = add(ring.center, scale(dir, dist));
        let position = add(surface, scale(normal, point.offset));
        let gap = point.offset.max(body.skin_offset).max(0.0) + clearance;
        PointFrame { center: ring.center, surface, position, normal, up: base.up, bend_radius: equivalent + gap }
    }
}

const RING_BINS: usize = 72;
const RING_SAMPLE: f64 = 1.5;

struct Ring {
    center: V,
    perimeter: f64,
    radii: Vec<f64>,
}

impl Ring {
    fn slice(positions: &[V3], triangles: &[[u32; 3]], f: &VolumeFrame, center: V, cap: f64, recentre: bool) -> Option<Ring> {
        let d: Vec<f64> = positions.iter().map(|p| dot(sub(v64(*p), center), f.axis)).collect();
        let mut points: Vec<V> = Vec::new();
        for t in triangles {
            let ds = [d[t[0] as usize], d[t[1] as usize], d[t[2] as usize]];
            if ds.iter().all(|x| *x > 0.0) || ds.iter().all(|x| *x <= 0.0) {
                continue;
            }
            let mut hits: Vec<V> = Vec::with_capacity(2);
            for k in 0..3 {
                let (a, b) = (t[k] as usize, t[(k + 1) % 3] as usize);
                if (d[a] > 0.0) != (d[b] > 0.0) {
                    let (pa, pb) = (v64(positions[a]), v64(positions[b]));
                    hits.push(add(pa, scale(sub(pb, pa), d[a] / (d[a] - d[b]))));
                }
            }
            if hits.len() == 2 {
                let pieces = (length(sub(hits[1], hits[0])) / RING_SAMPLE).ceil().max(1.0) as usize;
                for k in 0..=pieces {
                    points.push(add(hits[0], scale(sub(hits[1], hits[0]), k as f64 / pieces as f64)));
                }
            }
        }
        let mut c = center;
        let mut radii = Self::star(&points, c, f, cap)?;
        for _ in 0..(if recentre { 2 } else { 0 }) {
            let (cx, cy) = Self::centroid(&radii);
            c = add(c, add(scale(f.side, cx), scale(f.front, cy)));
            radii = Self::star(&points, c, f, cap)?;
        }
        let n = radii.len();
        let mut perimeter = 0.0;
        for k in 0..n {
            let (a, b) = (Self::vertex(&radii, k), Self::vertex(&radii, (k + 1) % n));
            perimeter += (a[0] - b[0]).hypot(a[1] - b[1]);
        }
        Some(Ring { center: c, perimeter, radii })
    }

    fn vertex(radii: &[f64], k: usize) -> [f64; 2] {
        let a = (k as f64 + 0.5) * std::f64::consts::TAU / radii.len() as f64;
        [radii[k] * a.sin(), radii[k] * a.cos()]
    }

    fn star(points: &[V], c: V, f: &VolumeFrame, cap: f64) -> Option<Vec<f64>> {
        let mut r = vec![f64::MAX; RING_BINS];
        for p in points {
            let q = sub(*p, c);
            let (x, y) = (dot(q, f.side), dot(q, f.front));
            let rho = x.hypot(y);
            if rho > cap || rho < 1e-6 {
                continue;
            }
            let a = x.atan2(y).rem_euclid(std::f64::consts::TAU);
            let b = ((a / std::f64::consts::TAU * RING_BINS as f64) as usize).min(RING_BINS - 1);
            r[b] = r[b].min(rho);
        }
        let filled = r.iter().filter(|x| **x < f64::MAX).count();
        if filled < RING_BINS / 2 {
            return None;
        }
        for k in 0..RING_BINS {
            if r[k] < f64::MAX {
                continue;
            }
            let mut lo = 1;
            while r[(k + RING_BINS - lo) % RING_BINS] == f64::MAX {
                lo += 1;
            }
            let mut hi = 1;
            while r[(k + hi) % RING_BINS] == f64::MAX {
                hi += 1;
            }
            let (a, b) = (r[(k + RING_BINS - lo) % RING_BINS], r[(k + hi) % RING_BINS]);
            let t = lo as f64 / (lo + hi) as f64;
            r[k] = a + (b - a) * t;
        }
        for _ in 0..2 {
            let prev = r.clone();
            for k in 0..RING_BINS {
                r[k] = 0.25 * prev[(k + RING_BINS - 1) % RING_BINS] + 0.5 * prev[k] + 0.25 * prev[(k + 1) % RING_BINS];
            }
        }
        Some(r)
    }

    fn centroid(radii: &[f64]) -> (f64, f64) {
        let n = radii.len();
        let (mut area, mut cx, mut cy) = (0.0, 0.0, 0.0);
        for k in 0..n {
            let (a, b) = (Self::vertex(radii, k), Self::vertex(radii, (k + 1) % n));
            let cross = a[0] * b[1] - b[0] * a[1];
            area += cross;
            cx += (a[0] + b[0]) * cross;
            cy += (a[1] + b[1]) * cross;
        }
        if area.abs() < 1e-9 {
            return (0.0, 0.0);
        }
        (cx / (3.0 * area), cy / (3.0 * area))
    }

    fn hit(&self, angle: f64) -> (f64, [f64; 2]) {
        let n = self.radii.len();
        let at = |a: f64| -> f64 {
            let u = a.rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU * n as f64 - 0.5;
            let k = u.floor();
            let t = u - k;
            let i = (k as i64).rem_euclid(n as i64) as usize;
            self.radii[i] + (self.radii[(i + 1) % n] - self.radii[i]) * t
        };
        let step = std::f64::consts::TAU / n as f64;
        let point = |a: f64| -> [f64; 2] {
            let r = at(a);
            [r * a.sin(), r * a.cos()]
        };
        let (p, q) = (point(angle - step), point(angle + step));
        (at(angle), [q[0] - p[0], q[1] - p[1]])
    }
}

struct Slice {
    center: V,
    radii: Vec<f64>,
    verts: Vec<[f64; 2]>,
    cum: Vec<f64>,
    start: f64,
}

pub struct Wrapper<'a> {
    positions: Vec<V3>,
    triangles: &'a [[u32; 3]],
    frame: VolumeFrame,
    center: V,
    cap: f64,
    want: f64,
    sign: f64,
    along: f64,
    cache: HashMap<i64, Option<Slice>>,
    origin: Cell<Option<V>>,
}

const WRAP_STEP: f64 = 4.0;

impl<'a> Wrapper<'a> {
    pub fn new(rig: &Rig, body: &'a Body, volume: &PlacementVolume, point: &PlacementPoint, up: V) -> Wrapper<'a> {
        let frame = rig.volume(volume);
        let theta = point.around.to_radians();
        let want = (frame.radius[0] * theta.sin()).atan2(frame.radius[1] * theta.cos());
        let center = add(frame.start, scale(frame.axis, point.along * frame.height));
        let cap = 1.6 * frame.radius[0].max(frame.radius[1]);
        let along = dot(up, frame.axis);
        Wrapper { positions: skin::posed_positions(body), triangles: &body.triangles, frame, center, cap, want, sign: if along >= 0.0 { 1.0 } else { -1.0 }, along, cache: HashMap::new(), origin: Cell::new(None) }
    }

    fn build(&self, key: i64, prev: Option<&Slice>) -> Option<Slice> {
        let radii = if key == 0 {
            let ring = Ring::slice(&self.positions, self.triangles, &self.frame, self.center, self.cap, true)?;
            self.origin.set(Some(ring.center));
            ring.radii
        } else {
            let c = add(self.origin.get()?, scale(self.frame.axis, key as f64 * WRAP_STEP));
            let ring = Ring::slice(&self.positions, self.triangles, &self.frame, c, self.cap, false);
            match (ring, prev) {
                (Some(r), Some(p)) => r.radii.iter().zip(&p.radii).map(|(a, b)| a.max(*b)).collect(),
                (Some(r), None) => r.radii,
                (None, Some(p)) => p.radii.clone(),
                (None, None) => return None,
            }
        };
        let centre = add(self.origin.get()?, scale(self.frame.axis, key as f64 * WRAP_STEP));
        Some(self.finish(centre, radii))
    }

    fn finish(&self, centre: V, radii: Vec<f64>) -> Slice {
        let n = radii.len();
        let verts: Vec<[f64; 2]> = (0..n).map(|k| Ring::vertex(&radii, k)).collect();
        let mut cum = vec![0.0; n + 1];
        for k in 0..n {
            let (a, b) = (verts[k], verts[(k + 1) % n]);
            cum[k + 1] = cum[k] + (a[0] - b[0]).hypot(a[1] - b[1]);
        }
        let step = std::f64::consts::TAU / n as f64;
        let u = self.want.rem_euclid(std::f64::consts::TAU) / step - 0.5;
        let k = u.floor();
        let i = (k as i64).rem_euclid(n as i64) as usize;
        let start = cum[i] + (u - k) * (cum[i + 1] - cum[i]);
        Slice { center: centre, radii, verts, cum, start }
    }

    pub fn at(&mut self, u: f64, v: f64, offset: f64) -> Option<V> {
        let key = (v * self.along / WRAP_STEP).round() as i64;
        let dir = if key >= 0 { 1 } else { -1 };
        let mut j = 0;
        loop {
            if !self.cache.contains_key(&j) {
                let made = {
                    let prev = if j == 0 { None } else { self.cache.get(&(j - dir)).and_then(Option::as_ref) };
                    self.build(j, prev)
                };
                self.cache.insert(j, made);
            }
            if j == key {
                break;
            }
            j += dir;
        }
        let s = self.cache.get(&key)?.as_ref()?;
        let n = s.verts.len();
        let total = s.cum[n];
        let off = offset.max(0.0);
        let k = total / (total + std::f64::consts::TAU * off);
        let arc = (s.start + self.sign * u * k).rem_euclid(total);
        let i = (0..n).rev().find(|i| s.cum[*i] <= arc).unwrap_or(0);
        let (a, b) = (s.verts[i], s.verts[(i + 1) % n]);
        let len = (s.cum[i + 1] - s.cum[i]).max(1e-9);
        let t = (arc - s.cum[i]) / len;
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        let tangent = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let flat = add(scale(self.frame.side, p[0] - tangent[1] * off), scale(self.frame.front, p[1] + tangent[0] * off));
        let lift = scale(self.frame.axis, v * self.along - key as f64 * WRAP_STEP);
        Some(add(add(s.center, flat), lift))
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
