use std::collections::BTreeMap;

use ffb_body::math::V3;
use ffb_body::measure::{self, Side, Slicer};
use ffb_body::{skin, Body};
use ffb_sim::collide::Prepared;

pub type V = [f64; 3];

pub fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn mul(a: V, k: f64) -> V {
    [a[0] * k, a[1] * k, a[2] * k]
}
pub fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn len(a: V) -> f64 {
    dot(a, a).sqrt()
}
pub fn unit(a: V) -> V {
    let l = len(a);
    if l < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        mul(a, 1.0 / l)
    }
}
pub fn lerp(a: V, b: V, t: f64) -> V {
    add(mul(a, 1.0 - t), mul(b, t))
}
fn v64(a: V3) -> V {
    [f64::from(a[0]), f64::from(a[1]), f64::from(a[2])]
}

pub const RING_N: usize = 256;
const RING_STEP: f64 = 4.0;

#[derive(Clone)]
pub struct Ring {
    pub c: [f64; 2],
    pub r: Vec<f64>,
}

pub struct Arm {
    pub sign: f64,
    pub shoulder: V,
    pub axis: V,
    pub length: f64,
    pub profile: Vec<f64>,
    pub down: V,
    pub front: V,
}

impl Arm {
    pub fn root_radius(&self) -> f64 {
        self.radius(6.0) + 14.0
    }

    pub fn circle(&self, g: f64, radius: f64, axial: f64) -> V {
        let th = std::f64::consts::TAU * g;
        add(add(self.shoulder, mul(self.axis, axial)), add(mul(self.down, radius * th.cos()), mul(self.front, radius * th.sin())))
    }

    pub fn radius(&self, t: f64) -> f64 {
        if self.profile.is_empty() {
            return 45.0;
        }
        let f = (t / 5.0).clamp(0.0, (self.profile.len() - 1) as f64);
        let i = f.floor() as usize;
        let j = (i + 1).min(self.profile.len() - 1);
        let w = f - i as f64;
        self.profile[i] * (1.0 - w) + self.profile[j] * w
    }
}

pub struct Info {
    pub name: String,
    pub m: BTreeMap<String, f64>,
    pub y_floor: f64,
    pub y_crotch: f64,
    pub y_waist: f64,
    pub y_hip: f64,
    pub y_chest: f64,
    pub y_armpit: f64,
    pub y_neck: f64,
    pub y_knee: f64,
    pub y_sp: f64,
    pub neck_r: f64,
    pub arms: Vec<Arm>,
    pub skin: f64,
    pub prep: Prepared,
    rings: Vec<Ring>,
    ring_y0: f64,
}

fn hull(points: &mut Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();
    if points.len() < 3 {
        return points.clone();
    }
    let cr = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    let mut h: Vec<[f64; 2]> = Vec::new();
    for pass in 0..2 {
        let start = h.len();
        let it: Vec<[f64; 2]> = if pass == 0 { points.clone() } else { points.iter().rev().copied().collect() };
        for q in it {
            while h.len() >= start + 2 && cr(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
                h.pop();
            }
            h.push(q);
        }
        h.pop();
    }
    h
}

fn polar(poly: &[[f64; 2]]) -> Ring {
    let n = poly.len();
    let mut a = 0.0;
    let mut cx = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = poly[i];
        let q = poly[(i + 1) % n];
        let w = p[0] * q[1] - q[0] * p[1];
        a += w;
        cx += (p[0] + q[0]) * w;
        cz += (p[1] + q[1]) * w;
    }
    let c = if a.abs() > 1e-6 { [cx / (3.0 * a), cz / (3.0 * a)] } else { poly[0] };
    let mut r = Vec::with_capacity(RING_N);
    for k in 0..RING_N {
        let th = std::f64::consts::TAU * k as f64 / RING_N as f64;
        let d = [th.sin(), th.cos()];
        let mut best = 0.0f64;
        for i in 0..n {
            let p = [poly[i][0] - c[0], poly[i][1] - c[1]];
            let q = [poly[(i + 1) % n][0] - c[0], poly[(i + 1) % n][1] - c[1]];
            let e = [q[0] - p[0], q[1] - p[1]];
            let den = d[0] * e[1] - d[1] * e[0];
            if den.abs() < 1e-12 {
                continue;
            }
            let t = (p[0] * e[1] - p[1] * e[0]) / den;
            let u = (p[0] * d[1] - p[1] * d[0]) / den;
            if t > 0.0 && (-1e-9..=1.0 + 1e-9).contains(&u) {
                best = best.max(t);
            }
        }
        r.push(best);
    }
    Ring { c, r }
}

fn is_arm(name: &str) -> bool {
    let n = name.to_lowercase();
    ["upperarm", "lowerarm", "wrist", "hand", "finger", "metacarpal", "thumb"].iter().any(|k| n.contains(k))
}

impl Info {
    pub fn new(body: &Body) -> Option<Info> {
        let pos = skin::posed_positions(body);
        let frames = skin::frames(&body.joints, Some(&body.pose));
        if pos.is_empty() || body.joints.is_empty() {
            return None;
        }
        let m = measure::measure(body);
        let full = Slicer::new(&pos, &body.triangles);
        let sh = measure::shoulder_height(&body.joints, &frames.pos);
        let lm = measure::landmarks(&full, sh);
        let chains = measure::arm_chains(&body.joints, &frames.pos);
        if chains.len() < 2 {
            return None;
        }
        let jp = |name: &str| body.joints.iter().position(|j| j.name.to_lowercase().starts_with(name)).map(|i| v64(frames.pos[i]));
        let y_neck = jp("neck01").or_else(|| jp("neck"))?[1];
        let y_knee = jp("lowerleg01").map(|p| p[1]).unwrap_or(f64::from(lm.crotch) * 0.6);
        let shoulder_y = f64::from(sh.unwrap_or(lm.shoulder));
        let mut y_armpit = shoulder_y - 70.0;
        let mut y = shoulder_y - 10.0;
        while y > f64::from(lm.crotch) {
            let n = full.loops(y as f32).iter().filter(|l| l.area > 300.0).count();
            if n >= 3 {
                y_armpit = y;
                break;
            }
            y -= 3.0;
        }
        let swidth = m.get("shoulder_width").copied().unwrap_or(320.0);
        let mut y_sp = shoulder_y;
        for p in &pos {
            let (x, yy) = (f64::from(p[0]).abs(), f64::from(p[1]));
            if (x - swidth * 0.5).abs() < 10.0 && yy < y_neck + 40.0 && yy > y_sp {
                y_sp = yy;
            }
        }
        let neck_girth = full.girth((y_neck + 25.0) as f32).map(f64::from).unwrap_or(330.0);
        let neck_r = neck_girth / std::f64::consts::TAU;
        let mask: Vec<bool> = (0..pos.len())
            .map(|i| {
                let w = &body.weights[i];
                let s: f32 = w.iter().filter(|(j, _)| body.joints.get(usize::from(*j)).is_some_and(|jj| is_arm(&jj.name))).map(|(_, x)| *x).sum();
                s > 0.5
            })
            .collect();
        let mut arms = Vec::new();
        for c in &chains {
            let sign = if c.side == Side::Left { 1.0 } else { -1.0 };
            let shoulder = v64(frames.pos[c.shoulder]);
            let elbow = v64(frames.pos[c.elbow]);
            let axis = unit(sub(elbow, shoulder));
            let length = len(sub(elbow, shoulder));
            let down = unit(sub([0.0, -1.0, 0.0], mul(axis, dot([0.0, -1.0, 0.0], axis))));
            let front = unit(cross(axis, down));
            let bins = (length / 5.0) as usize + 2;
            let mut rmax = vec![[0.0f64; 12]; bins];
            for (i, p) in pos.iter().enumerate() {
                if !mask[i] {
                    continue;
                }
                let v = sub(v64(*p), shoulder);
                if v[0] * sign < -20.0 {
                    continue;
                }
                let a = dot(v, axis);
                if a < 0.0 || a >= length {
                    continue;
                }
                let perp = sub(v, mul(axis, a));
                let r = len(perp);
                if r > 140.0 {
                    continue;
                }
                let ang = dot(perp, front).atan2(dot(perp, down));
                let k = (((ang + std::f64::consts::PI) / std::f64::consts::TAU * 12.0) as usize).min(11);
                let b = (a / 5.0) as usize;
                if r > rmax[b][k] {
                    rmax[b][k] = r;
                }
            }
            let mut profile: Vec<f64> = rmax
                .iter()
                .map(|row| {
                    let used: Vec<f64> = row.iter().copied().filter(|r| *r > 0.0).collect();
                    if used.is_empty() {
                        0.0
                    } else {
                        used.iter().sum::<f64>() / used.len() as f64
                    }
                })
                .collect();
            let fill = profile.iter().copied().filter(|r| *r > 0.0).fold(0.0, f64::max).max(40.0);
            for p in profile.iter_mut() {
                if *p <= 0.0 {
                    *p = fill * 0.8;
                }
            }
            let smooth: Vec<f64> = (0..profile.len())
                .map(|i| {
                    let lo = i.saturating_sub(3);
                    let hi = (i + 3).min(profile.len() - 1);
                    profile[lo..=hi].iter().sum::<f64>() / (hi - lo + 1) as f64
                })
                .collect();
            let fz = unit(sub([0.0, 0.0, 1.0], mul(axis, axis[2])));
            let fz = unit(sub(fz, mul(down, dot(fz, down))));
            arms.push(Arm { sign, shoulder, axis, length, profile: smooth, down, front: fz });
        }
        let y_cut = y_neck + 20.0;
        let y_lo = f64::from(lm.crotch) - 40.0;
        let build = |excl_below: f64| -> Vec<Ring> {
        let tris: Vec<[u32; 3]> = body
            .triangles
            .iter()
            .copied()
            .filter(|t| {
                t.iter().all(|&v| {
                    let yy = f64::from(pos[v as usize][1]);
                    yy <= y_cut && !(mask[v as usize] && yy < excl_below)
                })
            })
            .collect();
        let slicer = Slicer::new(&pos, &tris);
        let count = ((y_cut - y_lo) / RING_STEP) as usize + 1;
        let mut rings: Vec<Ring> = Vec::with_capacity(count);
        for k in 0..count {
            let yy = y_lo + k as f64 * RING_STEP;
            let mut pts: Vec<[f64; 2]> = Vec::new();
            for l in slicer.loops(yy as f32) {
                if l.pts.len() >= 4 {
                    pts.extend(l.pts.iter().map(|p| [f64::from(p[0]), f64::from(p[1])]));
                }
            }
            if pts.len() < 3 {
                if let Some(prev) = rings.last() {
                    let c = prev.clone();
                    rings.push(c);
                    continue;
                }
                pts = vec![[-50.0, -50.0], [50.0, -50.0], [50.0, 50.0], [-50.0, 50.0]];
            }
            let h = hull(&mut pts);
            rings.push(polar(&h));
        }
        let src = rings.clone();
        let last = src.len() - 1;
        #[allow(clippy::needless_range_loop)]
        for k in 0..src.len() {
            let (lo, hi) = (k.saturating_sub(2), (k + 2).min(last));
            let med = |mut v: Vec<f64>| {
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            rings[k].c = [med((lo..=hi).map(|j| src[j].c[0]).collect()), med((lo..=hi).map(|j| src[j].c[1]).collect())];
            for a in 0..RING_N {
                rings[k].r[a] = med((lo..=hi).map(|j| src[j].r[a]).collect());
            }
        }
        rings
        };
        let rt = build(y_cut + 1.0);
        let rf = build(y_armpit - 30.0);
        let rings: Vec<Ring> = rt
            .iter()
            .zip(&rf)
            .enumerate()
            .map(|(k, (a, b))| {
                let yy = y_lo + k as f64 * RING_STEP;
                let x = ((yy - (y_armpit - 5.0)) / 70.0).clamp(0.0, 1.0);
                let w = x * x * (3.0 - 2.0 * x);
                Ring { c: [a.c[0] * (1.0 - w) + b.c[0] * w, a.c[1] * (1.0 - w) + b.c[1] * w], r: a.r.iter().zip(&b.r).map(|(p, q)| p * (1.0 - w) + q * w).collect() }
            })
            .collect();
        let skin_offset = if body.skin_offset > 0.0 { body.skin_offset } else { 3.0 };
        Some(Info {
            name: body.name.clone(),
            m,
            y_floor: f64::from(lm.floor),
            y_crotch: f64::from(lm.crotch),
            y_waist: f64::from(lm.waist),
            y_hip: f64::from(lm.hip),
            y_chest: f64::from(lm.chest),
            y_armpit,
            y_neck,
            y_knee,
            y_sp,
            neck_r,
            arms,
            skin: skin_offset,
            prep: Prepared::from_collider(&ffb_body::collider(body)),
            rings,
            ring_y0: y_lo,
        })
    }

    pub fn measure(&self, key: &str) -> f64 {
        self.m.get(key).copied().unwrap_or(0.0)
    }

    pub fn y_range(&self) -> (f64, f64) {
        (self.ring_y0, self.ring_y0 + (self.rings.len() - 1) as f64 * RING_STEP)
    }

    pub fn ring(&self, y: f64) -> Ring {
        let (lo, hi) = self.y_range();
        let f = (y.clamp(lo, hi) - self.ring_y0) / RING_STEP;
        let i = (f.floor() as usize).min(self.rings.len() - 1);
        let j = (i + 1).min(self.rings.len() - 1);
        let w = f - i as f64;
        let (a, b) = (&self.rings[i], &self.rings[j]);
        Ring { c: [a.c[0] * (1.0 - w) + b.c[0] * w, a.c[1] * (1.0 - w) + b.c[1] * w], r: a.r.iter().zip(&b.r).map(|(p, q)| p * (1.0 - w) + q * w).collect() }
    }

    fn column_point(&self, y: f64, d: f64, arc: f64, back: bool) -> V {
        let pts = self.ring_pts(y, d);
        let a = at_arc(&pts, if back { RING_N / 2 } else { 0 }, arc);
        [a[0], y, a[1]]
    }

    pub fn march_up(&self, y0: f64, v_target: f64, arc: f64, back: bool, d_at: &dyn Fn(f64) -> f64) -> f64 {
        let mut yw = y0;
        let mut v = 0.0;
        let mut prev = self.column_point(yw, d_at(0.0), arc, back);
        while v < v_target - 1e-9 {
            let step = (v_target - v).min(8.0);
            let mut dy = step;
            for _ in 0..3 {
                let p = self.column_point(yw + dy, d_at(v + step), arc, back);
                let l = len(sub(p, prev)).max(1e-6);
                dy = (dy * step / l).clamp(0.2 * step, 1.6 * step);
            }
            prev = self.column_point(yw + dy, d_at(v + step), arc, back);
            yw += dy;
            v += step;
        }
        yw
    }

    pub fn march_len(&self, y0: f64, y1: f64, arc: f64, back: bool, d: f64) -> f64 {
        let mut yw = y0;
        let mut v = 0.0;
        let mut prev = self.column_point(yw, d, arc, back);
        let step = 6.0;
        while yw < y1 && v < 1500.0 {
            let mut dy = step;
            for _ in 0..3 {
                let p = self.column_point(yw + dy, d, arc, back);
                let l = len(sub(p, prev)).max(1e-6);
                dy = (dy * step / l).clamp(0.2 * step, 1.6 * step);
            }
            let next = self.column_point(yw + dy, d, arc, back);
            if yw + dy >= y1 {
                let t = (y1 - yw) / dy;
                return v + step * t;
            }
            prev = next;
            yw += dy;
            v += step;
        }
        v
    }

    pub fn ring_perimeter(&self, y: f64) -> f64 {
        let pts = self.ring_pts(y, 0.0);
        arc_total(&pts)
    }

    pub fn ring_pts(&self, y: f64, d: f64) -> Vec<[f64; 2]> {
        let ring = self.ring(y);
        let n = RING_N;
        let base: Vec<[f64; 2]> = (0..n)
            .map(|k| {
                let th = std::f64::consts::TAU * k as f64 / n as f64;
                [ring.c[0] + ring.r[k] * th.sin(), ring.c[1] + ring.r[k] * th.cos()]
            })
            .collect();
        if d.abs() < 1e-9 {
            return base;
        }
        (0..n)
            .map(|k| {
                let a = base[(k + n - 2) % n];
                let b = base[(k + 2) % n];
                let t = [b[0] - a[0], b[1] - a[1]];
                let l = (t[0] * t[0] + t[1] * t[1]).sqrt().max(1e-9);
                let mut nrm = [t[1] / l, -t[0] / l];
                let out = [base[k][0] - ring.c[0], base[k][1] - ring.c[1]];
                if nrm[0] * out[0] + nrm[1] * out[1] < 0.0 {
                    nrm = [-nrm[0], -nrm[1]];
                }
                [base[k][0] + nrm[0] * d, base[k][1] + nrm[1] * d]
            })
            .collect()
    }
}

pub fn arc_total(pts: &[[f64; 2]]) -> f64 {
    let n = pts.len();
    (0..n).map(|i| dist2(pts[i], pts[(i + 1) % n])).sum()
}

pub fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

pub fn at_arc(pts: &[[f64; 2]], start: usize, s: f64) -> [f64; 2] {
    let n = pts.len();
    let dir: isize = if s >= 0.0 { 1 } else { -1 };
    let mut left = s.abs();
    let mut i = start as isize;
    let total = arc_total(pts);
    if total > 0.0 && left > total {
        left %= total;
    }
    loop {
        let j = (i + dir).rem_euclid(n as isize) as usize;
        let a = pts[i.rem_euclid(n as isize) as usize];
        let b = pts[j];
        let l = dist2(a, b);
        if left <= l || l < 1e-12 {
            let t = if l < 1e-12 { 0.0 } else { left / l };
            return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        }
        left -= l;
        i += dir;
    }
}

pub fn arc_to_x(pts: &[[f64; 2]], start: usize, dir: isize, x: f64) -> f64 {
    let n = pts.len();
    let mut i = start as isize;
    let mut acc = 0.0;
    for _ in 0..n {
        let a = pts[i.rem_euclid(n as isize) as usize];
        let b = pts[(i + dir).rem_euclid(n as isize) as usize];
        let l = dist2(a, b);
        if (a[0] - x) * (b[0] - x) <= 0.0 && (a[0] - b[0]).abs() > 1e-12 {
            let t = (x - a[0]) / (b[0] - a[0]);
            return acc + l * t;
        }
        acc += l;
        i += dir;
    }
    acc
}
