use std::collections::HashMap;

use crate::math::*;
use crate::skin::{self, Bone};
use crate::{Body, Joint};

pub const FORM_NAME: &str = "Basic Form";
pub const FORM_LICENSE: &str = "formfabric original, Apache-2.0";
const CELL: f32 = 14.6;

struct Seg {
    a: V3,
    d: V3,
    s: V3,
    f: V3,
    len: f32,
    ra: [f32; 2],
    rb: [f32; 2],
    cap: f32,
    lo: V3,
    hi: V3,
}

impl Seg {
    fn new(a: V3, b: V3, ra: [f32; 2], rb: [f32; 2], cap: f32) -> Seg {
        let ab = sub(b, a);
        let len = length(ab).max(1e-3);
        let d = scale(ab, 1.0 / len);
        let x = [1.0, 0.0, 0.0];
        let mut s = sub(x, scale(d, dot(x, d)));
        if length(s) < 1e-3 {
            s = sub([0.0, 0.0, 1.0], scale(d, d[2]));
        }
        let s = normalize(s);
        let f = cross(d, s);
        let reach = ra[0].max(ra[1]).max(rb[0]).max(rb[1]) * cap.max(1.0) * 1.05;
        let lo = [a[0].min(b[0]) - reach, a[1].min(b[1]) - reach, a[2].min(b[2]) - reach];
        let hi = [a[0].max(b[0]) + reach, a[1].max(b[1]) + reach, a[2].max(b[2]) + reach];
        Seg { a, d, s, f, len, ra, rb, cap, lo, hi }
    }

    fn bound(&self, p: V3) -> f32 {
        let mut q = 0.0f32;
        for (i, c) in p.iter().enumerate() {
            let e = (self.lo[i] - c).max(c - self.hi[i]).max(0.0);
            q += e * e;
        }
        q.sqrt()
    }

    fn eval(&self, p: V3) -> f32 {
        let rel = sub(p, self.a);
        let along = dot(rel, self.d);
        let t = (along / self.len).clamp(0.0, 1.0);
        let u = along - t * self.len;
        let q = sub(rel, scale(self.d, along));
        let x = dot(q, self.s);
        let z = dot(q, self.f);
        let rx = self.ra[0] + (self.rb[0] - self.ra[0]) * t;
        let rz = self.ra[1] + (self.rb[1] - self.ra[1]) * t;
        let ru = (rx + rz) * 0.5 * self.cap;
        let w = ((x / rx).powi(2) + (z / rz).powi(2) + (u / ru).powi(2)).sqrt();
        (w - 1.0) * rx.min(rz)
    }
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

#[derive(Default)]
struct Group {
    segs: Vec<Seg>,
    k: f32,
}

impl Group {
    fn eval(&self, p: V3, limit: f32) -> f32 {
        let mut v = f32::MAX;
        for s in &self.segs {
            if v < f32::MAX && s.bound(p) > v + self.k + limit {
                continue;
            }
            v = smin(v, s.eval(p), self.k);
        }
        v
    }

    fn lo(&self) -> V3 {
        let mut lo = [f32::MAX; 3];
        for s in &self.segs {
            for (i, c) in lo.iter_mut().enumerate() {
                *c = c.min(s.lo[i]);
            }
        }
        lo
    }

    fn hi(&self) -> V3 {
        let mut hi = [f32::MIN; 3];
        for s in &self.segs {
            for (i, c) in hi.iter_mut().enumerate() {
                *c = c.max(s.hi[i]);
            }
        }
        hi
    }
}

fn chain(group: &mut Group, stations: &[(V3, [f32; 2])], cap: f32) {
    for w in stations.windows(2) {
        group.segs.push(Seg::new(w[0].0, w[1].0, w[0].1, w[1].1, 0.2));
    }
    if let (Some(first), Some(last)) = (stations.first(), stations.last()) {
        let dir = normalize(sub(stations[1].0, first.0));
        group.segs.push(Seg::new(first.0, add(first.0, scale(dir, 0.01)), first.1, first.1, cap));
        let dir = normalize(sub(last.0, stations[stations.len() - 2].0));
        group.segs.push(Seg::new(add(last.0, scale(dir, -0.01)), last.0, last.1, last.1, cap));
    }
}

struct Layout {
    torso: Group,
    arm: [Group; 2],
    leg: [Group; 2],
}

fn layout() -> Layout {
    let mut torso = Group { k: 2.0, ..Group::default() };
    let t: [(f32, f32, f32, f32); 11] = [
        (840.0, 150.0, 100.0, -8.0),
        (900.0, 178.0, 114.0, -10.0),
        (960.0, 172.0, 110.0, -6.0),
        (1030.0, 132.0, 90.0, 0.0),
        (1100.0, 152.0, 100.0, 2.0),
        (1180.0, 168.0, 108.0, 8.0),
        (1250.0, 168.0, 112.0, 14.0),
        (1320.0, 176.0, 112.0, 10.0),
        (1390.0, 166.0, 100.0, 5.0),
        (1440.0, 125.0, 84.0, 0.0),
        (1480.0, 58.0, 56.0, 0.0),
    ];
    let stations: Vec<(V3, [f32; 2])> = t.iter().map(|(y, rx, rz, cz)| ([0.0, *y, *cz], [*rx, *rz])).collect();
    torso.segs.push(Seg::new(stations[0].0, stations[1].0, stations[0].1, stations[1].1, 0.45));
    chain(&mut torso, &stations[1..], 1.0);
    chain(&mut torso, &[([0.0, 1470.0, 2.0], [56.0, 54.0]), ([0.0, 1540.0, 10.0], [50.0, 50.0])], 1.0);
    torso.segs.push(Seg::new([0.0, 1580.0, 12.0], [0.0, 1588.0, 12.0], [78.0, 92.0], [78.0, 92.0], 1.3));
    torso.segs.push(Seg::new([0.0, 1535.0, 22.0], [0.0, 1555.0, 26.0], [60.0, 66.0], [60.0, 66.0], 0.9));
    for sx in [1.0f32, -1.0] {
        torso.segs.push(Seg::new([sx * 50.0, 1470.0, -4.0], [sx * 185.0, 1432.0, 0.0], [38.0, 40.0], [44.0, 46.0], 1.0));
    }
    let mut arm = [Group { k: 2.0, ..Group::default() }, Group { k: 2.0, ..Group::default() }];
    let mut leg = [Group { k: 2.0, ..Group::default() }, Group { k: 2.0, ..Group::default() }];
    for (i, sx) in [1.0f32, -1.0].into_iter().enumerate() {
        let (sh, el, wr, hd) = (SHOULDER, ELBOW, WRIST, HAND);
        let m = |p: V3| [sx * p[0], p[1], p[2]];
        chain(&mut arm[i], &[(m(sh), [46.0, 46.0]), (m(el), [37.0, 36.0]), (m(wr), [26.0, 22.0])], 1.0);
        arm[i].segs.push(Seg::new(m(wr), m(hd), [17.0, 38.0], [13.0, 28.0], 0.8));
        let leg_stations: [(f32, f32, f32, f32, f32); 7] = [
            (98.0, 880.0, 0.0, 82.0, 88.0),
            (98.0, 760.0, 3.0, 76.0, 80.0),
            (96.0, 600.0, 5.0, 64.0, 68.0),
            (94.0, 490.0, 5.0, 52.0, 55.0),
            (94.0, 380.0, -12.0, 51.0, 58.0),
            (95.0, 250.0, -5.0, 42.0, 45.0),
            (95.0, 100.0, 0.0, 33.0, 35.0),
        ];
        let st: Vec<(V3, [f32; 2])> = leg_stations.iter().map(|(x, y, z, rx, rz)| ([sx * x, *y, *z], [*rx, *rz])).collect();
        chain(&mut leg[i], &st, 1.0);
        let foot = [([sx * 95.0, 70.0, 0.0], [34.0, 34.0]), ([sx * 95.0, 38.0, -40.0], [36.0, 38.0]), ([sx * 97.0, 27.0, 80.0], [45.0, 27.0]), ([sx * 99.0, 17.0, 130.0], [34.0, 17.0])];
        chain(&mut leg[i], &foot, 1.0);
    }
    Layout { torso, arm, leg }
}

const SHOULDER: V3 = [185.0, 1432.0, 0.0];
const ELBOW: V3 = [335.0, 1172.2, 0.0];
const WRIST: V3 = [442.8, 941.1, 0.0];
const HAND: V3 = [480.8, 859.5, 0.0];

struct Eval<'a> {
    l: &'a Layout,
}

impl Eval<'_> {
    fn groups(&self, p: V3) -> [f32; 5] {
        [self.l.torso.eval(p, 0.0), self.l.arm[0].eval(p, 0.0), self.l.arm[1].eval(p, 0.0), self.l.leg[0].eval(p, 0.0), self.l.leg[1].eval(p, 0.0)]
    }

    fn value(&self, p: V3) -> f32 {
        let g = self.groups(p);
        let body = smin(smin(g[0], g[1], 26.0), g[2], 26.0);
        smin(smin(body, g[3], 22.0), g[4], 22.0)
    }
}

fn gradient(e: &Eval, p: V3) -> V3 {
    let h = 1.0;
    [
        e.value([p[0] + h, p[1], p[2]]) - e.value([p[0] - h, p[1], p[2]]),
        e.value([p[0], p[1] + h, p[2]]) - e.value([p[0], p[1] - h, p[2]]),
        e.value([p[0], p[1], p[2] + h]) - e.value([p[0], p[1], p[2] - h]),
    ]
}

fn project(e: &Eval, p: V3) -> V3 {
    let f = e.value(p);
    let g = gradient(e, p);
    let g2 = dot(g, g) / 4.0;
    if g2 < 1e-6 {
        return p;
    }
    let step = scale(g, f / (2.0 * g2));
    let l = length(step);
    let step = if l > CELL { scale(step, CELL / l) } else { step };
    sub(p, step)
}

fn surface_nets(e: &Eval, lo: V3, hi: V3, h: f32) -> (Vec<V3>, Vec<[u32; 3]>) {
    let pad = 3.0 * h;
    let o = [lo[0] - pad, lo[1] - pad, lo[2] - pad];
    let n = [((hi[0] - lo[0] + 2.0 * pad) / h).ceil() as usize, ((hi[1] - lo[1] + 2.0 * pad) / h).ceil() as usize, ((hi[2] - lo[2] + 2.0 * pad) / h).ceil() as usize];
    let c = [n[0] + 1, n[1] + 1, n[2] + 1];
    let cidx = |i: usize, j: usize, k: usize| (k * c[1] + j) * c[0] + i;
    let mut field = vec![0.0f32; c[0] * c[1] * c[2]];
    for k in 0..c[2] {
        for j in 0..c[1] {
            for i in 0..c[0] {
                field[cidx(i, j, k)] = e.value([o[0] + i as f32 * h, o[1] + j as f32 * h, o[2] + k as f32 * h]);
            }
        }
    }
    let corner = |i: usize, j: usize, k: usize| [o[0] + i as f32 * h, o[1] + j as f32 * h, o[2] + k as f32 * h];
    const EDGES: [([usize; 3], [usize; 3]); 12] = [
        ([0, 0, 0], [1, 0, 0]),
        ([0, 1, 0], [1, 1, 0]),
        ([0, 0, 1], [1, 0, 1]),
        ([0, 1, 1], [1, 1, 1]),
        ([0, 0, 0], [0, 1, 0]),
        ([1, 0, 0], [1, 1, 0]),
        ([0, 0, 1], [0, 1, 1]),
        ([1, 0, 1], [1, 1, 1]),
        ([0, 0, 0], [0, 0, 1]),
        ([1, 0, 0], [1, 0, 1]),
        ([0, 1, 0], [0, 1, 1]),
        ([1, 1, 0], [1, 1, 1]),
    ];
    let mut cell_vertex: HashMap<usize, u32> = HashMap::new();
    let mut positions: Vec<V3> = Vec::new();
    let cell_key = |i: usize, j: usize, k: usize| (k * n[1] + j) * n[0] + i;
    for k in 0..n[2] {
        for j in 0..n[1] {
            for i in 0..n[0] {
                let mut acc = [0.0f32; 3];
                let mut count = 0.0f32;
                for (a, b) in EDGES {
                    let fa = field[cidx(i + a[0], j + a[1], k + a[2])];
                    let fb = field[cidx(i + b[0], j + b[1], k + b[2])];
                    if (fa < 0.0) != (fb < 0.0) {
                        let t = fa / (fa - fb);
                        let pa = corner(i + a[0], j + a[1], k + a[2]);
                        let pb = corner(i + b[0], j + b[1], k + b[2]);
                        acc = add(acc, lerp(pa, pb, t));
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    cell_vertex.insert(cell_key(i, j, k), positions.len() as u32);
                    positions.push(scale(acc, 1.0 / count));
                }
            }
        }
    }
    let mut quads: Vec<[u32; 4]> = Vec::new();
    for k in 1..n[2] {
        for j in 1..n[1] {
            for i in 1..n[0] {
                let f0 = field[cidx(i, j, k)];
                let inside = f0 < 0.0;
                let probes = [(1usize, 0usize, 0usize), (0, 1, 0), (0, 0, 1)];
                for (axis, (di, dj, dk)) in probes.into_iter().enumerate() {
                    let f1 = field[cidx(i + di, j + dj, k + dk)];
                    if (f1 < 0.0) == inside {
                        continue;
                    }
                    let cells: [(usize, usize, usize); 4] = match axis {
                        0 => [(i, j - 1, k - 1), (i, j, k - 1), (i, j, k), (i, j - 1, k)],
                        1 => [(i - 1, j, k - 1), (i - 1, j, k), (i, j, k), (i, j, k - 1)],
                        _ => [(i - 1, j - 1, k), (i, j - 1, k), (i, j, k), (i - 1, j, k)],
                    };
                    let mut q = [0u32; 4];
                    let mut ok = true;
                    for (s, (ci, cj, ck)) in cells.iter().enumerate() {
                        match cell_vertex.get(&cell_key(*ci, *cj, *ck)) {
                            Some(v) => q[s] = *v,
                            None => ok = false,
                        }
                    }
                    if ok {
                        if !inside {
                            q.reverse();
                        }
                        quads.push(q);
                    }
                }
            }
        }
    }
    let mut tris = Vec::with_capacity(quads.len() * 2);
    for q in quads {
        tris.push([q[0], q[1], q[2]]);
        tris.push([q[0], q[2], q[3]]);
    }
    (positions, tris)
}

fn smooth(e: &Eval, positions: &mut [V3], tris: &[[u32; 3]]) {
    let mut sets: Vec<Vec<u32>> = vec![Vec::new(); positions.len()];
    for t in tris {
        for k in 0..3 {
            let a = t[k] as usize;
            let b = t[(k + 1) % 3];
            let c = t[(k + 2) % 3];
            for v in [b, c] {
                if !sets[a].contains(&v) {
                    sets[a].push(v);
                }
            }
        }
    }
    for round in 0..8 {
        let before = positions.to_vec();
        for (i, nb) in sets.iter().enumerate() {
            if nb.is_empty() {
                continue;
            }
            let mut avg = [0.0f32; 3];
            for v in nb {
                avg = add(avg, before[*v as usize]);
            }
            avg = scale(avg, 1.0 / nb.len() as f32);
            let p = lerp(before[i], avg, if round < 6 { 0.5 } else { 0.0 });
            positions[i] = project(e, p);
        }
    }
    for p in positions.iter_mut() {
        *p = project(e, *p);
    }
}

struct Spec {
    name: &'static str,
    parent: Option<usize>,
    at: V3,
}

fn skeleton() -> Vec<Spec> {
    let sp = |name, parent, at| Spec { name, parent, at };
    let m = |p: V3| [-p[0], p[1], p[2]];
    vec![
        sp("root", None, [0.0, 0.0, 0.0]),
        sp("pelvis", Some(0), [0.0, 950.0, 0.0]),
        sp("spine", Some(1), [0.0, 1070.0, 0.0]),
        sp("chest", Some(2), [0.0, 1250.0, 8.0]),
        sp("neck", Some(3), [0.0, 1480.0, 0.0]),
        sp("head", Some(4), [0.0, 1555.0, 5.0]),
        sp("l_clavicle", Some(3), [25.0, 1445.0, 0.0]),
        sp("l_shoulder", Some(6), SHOULDER),
        sp("l_elbow", Some(7), ELBOW),
        sp("l_wrist", Some(8), WRIST),
        sp("r_clavicle", Some(3), m([25.0, 1445.0, 0.0])),
        sp("r_shoulder", Some(10), m(SHOULDER)),
        sp("r_elbow", Some(11), m(ELBOW)),
        sp("r_wrist", Some(12), m(WRIST)),
        sp("l_hip", Some(1), [95.0, 890.0, 0.0]),
        sp("l_knee", Some(14), [94.0, 490.0, 5.0]),
        sp("l_ankle", Some(15), [95.0, 80.0, 0.0]),
        sp("r_hip", Some(1), [-95.0, 890.0, 0.0]),
        sp("r_knee", Some(17), [-94.0, 490.0, 5.0]),
        sp("r_ankle", Some(18), [-95.0, 80.0, 0.0]),
    ]
}

fn bone(joint: usize, a: V3, b: V3) -> Bone {
    Bone { joint, a, b }
}

pub fn basic_form() -> Body {
    static CACHE: std::sync::OnceLock<Body> = std::sync::OnceLock::new();
    CACHE.get_or_init(build_form).clone()
}

fn build_form() -> Body {
    let lay = layout();
    let e = Eval { l: &lay };
    let mut lo = lay.torso.lo();
    let mut hi = lay.torso.hi();
    for g in lay.arm.iter().chain(lay.leg.iter()) {
        for i in 0..3 {
            lo[i] = lo[i].min(g.lo()[i]);
            hi[i] = hi[i].max(g.hi()[i]);
        }
    }
    let (mut positions, triangles) = surface_nets(&e, lo, hi, CELL);
    smooth(&e, &mut positions, &triangles);
    let floor = positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let mut specs = skeleton();
    let shift = [0.0, -floor, 0.0];
    for p in positions.iter_mut() {
        *p = add(*p, shift);
    }
    let weights = form_weights(&lay, &positions, &specs, floor);
    for s in specs.iter_mut() {
        s.at = add(s.at, shift);
    }
    let joints: Vec<Joint> = specs
        .iter()
        .map(|s| {
            let base = s.parent.map(|p| specs[p].at).unwrap_or([0.0; 3]);
            Joint { name: s.name.to_string(), parent: s.parent.map(|p| p as u16), rest: sub(s.at, base), rest_rotation: IDENTITY }
        })
        .collect();
    let normals = skin::compute_normals(&positions, &triangles);
    let mut body = Body {
        name: FORM_NAME.to_string(),
        source: Some("generated: ellipse cross-sections along a skeleton, smooth-union surface".to_string()),
        license: Some(FORM_LICENSE.to_string()),
        positions,
        normals,
        triangles,
        joints,
        weights,
        skin_offset: 3.0,
        friction: 0.3,
        visible: true,
        ..Body::default()
    };
    body.measurements = crate::measure::measure(&body);
    body
}

fn form_weights(lay: &Layout, positions: &[V3], specs: &[Spec], floor: f32) -> Vec<[(u16, f32); 4]> {
    let at = |n: &str| specs.iter().position(|s| s.name == n).unwrap();
    let pos = |n: &str| {
        let mut p = specs[at(n)].at;
        p[1] -= floor;
        p
    };
    let mut torso_bones = vec![
        bone(at("pelvis"), pos("pelvis"), pos("spine")),
        bone(at("pelvis"), pos("pelvis"), add(pos("pelvis"), [0.0, -110.0, 0.0])),
        bone(at("spine"), pos("spine"), pos("chest")),
        bone(at("chest"), pos("chest"), pos("neck")),
        bone(at("neck"), pos("neck"), pos("head")),
        bone(at("head"), pos("head"), add(pos("head"), [0.0, 130.0, 6.0])),
    ];
    let mut arm_bones: [Vec<Bone>; 2] = [Vec::new(), Vec::new()];
    let mut leg_bones: [Vec<Bone>; 2] = [Vec::new(), Vec::new()];
    for (i, p) in ["l_", "r_"].into_iter().enumerate() {
        let n = |s: &str| format!("{p}{s}");
        torso_bones.push(bone(at(&n("clavicle")), pos(&n("clavicle")), pos(&n("shoulder"))));
        let hand = if i == 0 { HAND } else { [-HAND[0], HAND[1], HAND[2]] };
        let hand = [hand[0], hand[1] - floor, hand[2]];
        arm_bones[i].push(bone(at(&n("shoulder")), pos(&n("shoulder")), pos(&n("elbow"))));
        arm_bones[i].push(bone(at(&n("elbow")), pos(&n("elbow")), pos(&n("wrist"))));
        arm_bones[i].push(bone(at(&n("wrist")), pos(&n("wrist")), hand));
        let sx = if i == 0 { 1.0 } else { -1.0 };
        leg_bones[i].push(bone(at(&n("hip")), pos(&n("hip")), pos(&n("knee"))));
        leg_bones[i].push(bone(at(&n("knee")), pos(&n("knee")), pos(&n("ankle"))));
        leg_bones[i].push(bone(at(&n("ankle")), pos(&n("ankle")), [sx * 97.0, 27.0 - floor, 100.0]));
    }
    let count = specs.len();
    let torso_w = skin::compute_weights(positions, count, &torso_bones, 16.0);
    let arm_w: Vec<Vec<[(u16, f32); 4]>> = arm_bones.iter().map(|b| skin::compute_weights(positions, count, b, 16.0)).collect();
    let leg_w: Vec<Vec<[(u16, f32); 4]>> = leg_bones.iter().map(|b| skin::compute_weights(positions, count, b, 16.0)).collect();
    let shift = [0.0, -floor, 0.0];
    let mut out = Vec::with_capacity(positions.len());
    for (vi, p) in positions.iter().enumerate() {
        let q = sub(*p, shift);
        let g = Eval { l: lay }.groups(q);
        let m = g.iter().copied().fold(f32::MAX, f32::min);
        let gamma: Vec<f32> = g
            .iter()
            .map(|v| {
                let x = (v - m) / 14.0;
                (-x * x).exp()
            })
            .collect();
        let sets = [&torso_w[vi], &arm_w[0][vi], &arm_w[1][vi], &leg_w[0][vi], &leg_w[1][vi]];
        let mut acc: Vec<f32> = vec![0.0; count];
        for (gi, set) in sets.iter().enumerate() {
            for (j, w) in set.iter() {
                acc[*j as usize] += gamma[gi] * w;
            }
        }
        let mut cand: Vec<(u16, f32)> = acc.iter().enumerate().filter(|(_, w)| **w > 1e-4).map(|(j, w)| (j as u16, *w)).collect();
        cand.sort_by(|a, b| b.1.total_cmp(&a.1));
        cand.truncate(4);
        let total: f32 = cand.iter().map(|c| c.1).sum();
        let mut slot = [(0u16, 0.0f32); 4];
        for (k, c) in cand.iter().enumerate() {
            slot[k] = (c.0, c.1 / total);
        }
        out.push(slot);
    }
    out
}
