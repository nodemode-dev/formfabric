use std::collections::BTreeMap;

use crate::math::*;
use crate::measure::{self, Side, Slicer};
use crate::skin::{self, Frames};
use crate::Body;

#[derive(Debug, Clone, Copy, Default)]
pub struct Targets {
    pub height: Option<f64>,
    pub inseam: Option<f64>,
    pub chest: Option<f64>,
    pub waist: Option<f64>,
    pub hip: Option<f64>,
    pub shoulder_width: Option<f64>,
    pub arm_length: Option<f64>,
}

impl Targets {
    pub fn any(&self) -> bool {
        self.height.is_some() || self.inseam.is_some() || self.chest.is_some() || self.waist.is_some() || self.hip.is_some() || self.shoulder_width.is_some() || self.arm_length.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub before: BTreeMap<String, f64>,
    pub after: BTreeMap<String, f64>,
    pub iterations: u32,
}

fn rest_frames(body: &Body) -> Frames {
    skin::bind_frames(body)
}

fn crotch_map(u: f32, c: f32, k: f32, w: f32) -> f32 {
    let y0 = c - w;
    let h = if u <= y0 {
        u
    } else if u >= c + w {
        c
    } else {
        let v = (u - y0) / (2.0 * w);
        y0 + (u - y0) - 2.0 * w * (v * v * v - v * v * v * v * 0.5)
    };
    u + (k - 1.0) * h
}

struct ArmInfo {
    joint_attach: Vec<Option<usize>>,
    vertex_attach: Vec<Option<usize>>,
    vertex_weight: Vec<f32>,
}

fn arm_info(body: &Body, bind: &Frames) -> ArmInfo {
    let n = body.joints.len();
    let chains = measure::arm_chains(&body.joints, &bind.pos);
    let mut joint_attach: Vec<Option<usize>> = vec![None; n];
    for c in &chains {
        for (j, slot) in joint_attach.iter_mut().enumerate() {
            let mut at = Some(j);
            let mut guard = 0;
            while let Some(a) = at {
                if a == c.shoulder {
                    *slot = Some(c.shoulder);
                    break;
                }
                at = body.joints[a].parent.map(|p| p as usize).filter(|p| *p < n);
                guard += 1;
                if guard > n {
                    break;
                }
            }
        }
    }
    let mut vertex_attach = vec![None; body.positions.len()];
    let mut vertex_weight = vec![0.0f32; body.positions.len()];
    if body.weights.len() == body.positions.len() {
        for (vi, w) in body.weights.iter().enumerate() {
            let mut per: BTreeMap<usize, f32> = BTreeMap::new();
            for (j, x) in w {
                if let Some(Some(a)) = joint_attach.get(*j as usize) {
                    *per.entry(*a).or_default() += *x;
                }
            }
            if let Some((a, x)) = per.into_iter().max_by(|p, q| p.1.total_cmp(&q.1)) {
                vertex_attach[vi] = Some(a);
                vertex_weight[vi] = x.clamp(0.0, 1.0);
            }
        }
    }
    ArmInfo { joint_attach, vertex_attach, vertex_weight }
}

struct Shape {
    hip: f32,
    waist: f32,
    chest: f32,
    shoulder: f32,
    center: [f32; 2],
    s_hip: f32,
    s_waist: f32,
    s_chest: f32,
    kw: f32,
    ka: f32,
}

impl Shape {
    fn girth(&self, y: f32) -> f32 {
        let pts = [(self.hip - 300.0, 1.0), (self.hip, self.s_hip), (self.waist, self.s_waist), (self.chest, self.s_chest), (self.chest + 190.0, 1.0)];
        if y <= pts[0].0 || y >= pts[4].0 {
            return 1.0;
        }
        for w in pts.windows(2) {
            if y <= w[1].0 {
                let t = smoothstep((y - w[0].0) / (w[1].0 - w[0].0).max(1e-3));
                return w[0].1 + (w[1].1 - w[0].1) * t;
            }
        }
        1.0
    }

    fn width(&self, y: f32) -> f32 {
        let rise = smoothstep((y - self.chest) / (self.shoulder - 20.0 - self.chest).max(1.0));
        let fall = 1.0 - smoothstep((y - (self.shoulder + 10.0)) / 70.0);
        self.girth(y) * (1.0 + (self.kw - 1.0) * rise * fall)
    }

    fn radial(&self, p: V3) -> V3 {
        let g = self.girth(p[1]);
        let w = self.width(p[1]);
        [self.center[0] + (p[0] - self.center[0]) * w, p[1], self.center[1] + (p[2] - self.center[1]) * g]
    }

    fn apply(&self, p: V3, weight: f32, attach: Option<V3>) -> V3 {
        let q = self.radial(p);
        let Some(a) = attach else { return q };
        if weight <= 0.0 {
            return q;
        }
        let a2 = self.radial(a);
        let moved = [p[0] + (a2[0] - a[0]), p[1], p[2] + (a2[2] - a[2])];
        let mixed = lerp(q, moved, weight);
        let k = 1.0 + (self.ka - 1.0) * weight;
        add(a2, scale(sub(mixed, a2), k))
    }
}

fn deform(shape: &Shape, base: &[V3], jbase: &[V3], info: &ArmInfo, jfix: &[V3]) -> (Vec<V3>, Vec<V3>) {
    let verts = base
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let attach = info.vertex_attach[i].map(|a| jbase[a]);
            shape.apply(*p, info.vertex_weight[i], attach)
        })
        .collect();
    let joints = jbase
        .iter()
        .enumerate()
        .map(|(i, p)| match info.joint_attach[i] {
            Some(a) => shape.apply(*p, 1.0, Some(jfix[a])),
            None => shape.apply(*p, 0.0, None),
        })
        .collect();
    (verts, joints)
}

fn check(name: &str, target: f64, current: Option<f64>) -> Result<f64, String> {
    let c = current.ok_or_else(|| format!("{name} could not be measured on this body"))?;
    if c <= 1.0 {
        return Err(format!("{name} could not be measured on this body"));
    }
    let ratio = target / c;
    if !(0.4..=2.5).contains(&ratio) {
        return Err(format!("{name} {target:.0} mm is too far from the current {c:.0} mm (limit 0.4x to 2.5x)"));
    }
    Ok(ratio)
}

pub fn resize(body: &mut Body, targets: &Targets) -> Result<Report, String> {
    if body.positions.is_empty() {
        return Err("the body has no mesh".into());
    }
    let bind = rest_frames(body);
    let before = measure::measure_geometry(&body.positions, &body.triangles, &body.joints, &bind);
    let mut verts = body.positions.clone();
    let mut jpos = bind.pos.clone();
    let jrot = bind.rot.clone();
    let floor = verts.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let get = |m: &BTreeMap<String, f64>, k: &str| m.get(k).copied();

    if let Some(t) = targets.inseam {
        let k = check("inseam", t, get(&before, "inseam"))? as f32;
        let c = get(&before, "inseam").unwrap_or(0.0) as f32;
        let map = |p: &mut V3| p[1] = floor + crotch_map(p[1] - floor, c, k, 70.0);
        verts.iter_mut().for_each(map);
        jpos.iter_mut().for_each(map);
    }
    if let Some(t) = targets.height {
        let mid = measure::measure_geometry(&verts, &body.triangles, &body.joints, &Frames { pos: jpos.clone(), rot: jrot.clone() });
        let k = check("height", t, get(&mid, "height"))? as f32;
        let map = |p: &mut V3| p[1] = floor + (p[1] - floor) * k;
        verts.iter_mut().for_each(map);
        jpos.iter_mut().for_each(map);
    }

    let slicer = Slicer::new(&verts, &body.triangles);
    let shoulder = measure::shoulder_height(&body.joints, &jpos);
    let lm = measure::landmarks(&slicer, shoulder);
    let mut shape = Shape { hip: lm.hip, waist: lm.waist, chest: lm.chest, shoulder: lm.shoulder, center: lm.center, s_hip: 1.0, s_waist: 1.0, s_chest: 1.0, kw: 1.0, ka: 1.0 };
    drop(slicer);
    let staged = Body { positions: verts.clone(), ..body.clone() };
    let info = arm_info(&staged, &bind);
    let wants = targets.chest.is_some() || targets.waist.is_some() || targets.hip.is_some() || targets.shoulder_width.is_some() || targets.arm_length.is_some();
    let mut iterations = 0;
    let (mut out_v, mut out_j) = (verts.clone(), jpos.clone());
    if wants {
        let base_v = verts.clone();
        let base_j = jpos.clone();
        for it in 0..12 {
            iterations = it + 1;
            let (v, j) = deform(&shape, &base_v, &base_j, &info, &base_j);
            let m = measure::measure_geometry(&v, &body.triangles, &body.joints, &Frames { pos: j.clone(), rot: jrot.clone() });
            out_v = v;
            out_j = j;
            let mut worst = 0.0f64;
            let mut step = |target: Option<f64>, key: &str, factor: &mut f32| -> Result<(), String> {
                if let Some(t) = target {
                    let now = get(&m, key);
                    let r = check(key, t, now)? as f32;
                    worst = worst.max((t - now.unwrap_or(t)).abs());
                    *factor = (*factor * r).clamp(0.3, 3.0);
                }
                Ok(())
            };
            step(targets.chest, "chest", &mut shape.s_chest)?;
            step(targets.waist, "waist", &mut shape.s_waist)?;
            step(targets.hip, "hip", &mut shape.s_hip)?;
            step(targets.shoulder_width, "shoulder_width", &mut shape.kw)?;
            step(targets.arm_length, "arm_length", &mut shape.ka)?;
            if worst < 0.2 {
                break;
            }
        }
        let (v, j) = deform(&shape, &base_v, &base_j, &info, &base_j);
        out_v = if iterations > 0 { v } else { out_v };
        out_j = if iterations > 0 { j } else { out_j };
    }
    body.positions = out_v;
    skin::set_from_world(&mut body.joints, &out_j, &jrot);
    body.normals = skin::compute_normals(&body.positions, &body.triangles);
    body.measurements = measure::measure(body);
    Ok(Report { before, after: body.measurements.clone(), iterations })
}

pub fn side_name(s: Side) -> &'static str {
    match s {
        Side::Left => "left",
        Side::Right => "right",
    }
}
