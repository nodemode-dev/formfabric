use glam::DVec3;

use crate::bvh::{barycentric, closest_on_triangle};
use crate::selfcol::SpatialHash;
use crate::solver::Solver;

const MAX_PUSH: f64 = 3.0;
const SPEED_CAP: f64 = 1200.0;

thread_local! {
    static PUSH: std::cell::RefCell<Vec<DVec3>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub struct Contact<'a> {
    pub tris: &'a [[u32; 3]],
    pub neighbors: &'a [Vec<u32>],
    pub inv_mass: &'a [f64],
    pub thick: &'a [f64],
    pub layer: &'a [i32],
    pub xn: &'a [DVec3],
    pub base: f64,
    pub reach: f64,
    pub margin: f64,
    pub seam: &'a [bool],
}

impl Contact<'_> {
    fn sep(&self, verts: &[u32], dl: i32) -> f64 {
        let mean = verts.iter().map(|&v| self.thick[v as usize]).sum::<f64>() / verts.len() as f64;
        let s0 = self.base.max(mean);
        s0 * f64::from(dl.abs().clamp(1, 3))
    }

    fn linked(&self, v: u32, a: u32) -> bool {
        let nb = &self.neighbors[v as usize];
        let two = v == a || nb.binary_search(&a).is_ok() || nb.iter().any(|&w| self.neighbors[w as usize].binary_search(&a).is_ok());
        if two {
            return true;
        }
        nb.iter().any(|&w| self.neighbors[w as usize].iter().any(|&u| self.neighbors[u as usize].binary_search(&a).is_ok()))
    }

    fn near(&self, v: u32, t: &[u32; 3]) -> bool {
        t.iter().any(|&i| self.linked(v, i))
    }

    fn tri_normal(&self, x: &[DVec3], t: &[u32; 3]) -> DVec3 {
        (x[t[1] as usize] - x[t[0] as usize]).cross(x[t[2] as usize] - x[t[0] as usize]).normalize_or_zero()
    }
}

fn seg_seg(p1: DVec3, q1: DVec3, p2: DVec3, q2: DVec3) -> (f64, f64, DVec3, DVec3) {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r = p1 - p2;
    let a = d1.dot(d1);
    let e = d2.dot(d2);
    let f = d2.dot(r);
    let (mut s, mut t);
    if a <= 1.0e-18 && e <= 1.0e-18 {
        return (0.0, 0.0, p1, p2);
    }
    if a <= 1.0e-18 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if e <= 1.0e-18 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(d2);
            let denom = a * e - b * b;
            s = if denom > 1.0e-18 { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
            t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
        }
    }
    (s, t, p1 + d1 * s, p2 + d2 * t)
}

pub fn vertex_triangle_pass(x: &mut [DVec3], hash: &SpatialHash, c: &Contact) -> u32 {
    let mut cand = Vec::new();
    let mut count = 0;
    for v in 0..x.len() {
        let wv = c.inv_mass[v];
        hash.query(x[v], &mut cand);
        for &ti in &cand {
            let t = c.tris[ti as usize];
            let (a, b, cc) = (x[t[0] as usize], x[t[1] as usize], x[t[2] as usize]);
            let (cp, _) = closest_on_triangle(x[v], a, b, cc);
            let dvec = x[v] - cp;
            let dist = dvec.length();
            if dist > c.reach + c.margin {
                continue;
            }
            let tn = (b - a).cross(cc - a).normalize_or_zero();
            if tn == DVec3::ZERO || c.near(v as u32, &t) {
                continue;
            }
            let dl = c.layer[v] - c.layer[t[0] as usize];
            let s_req = if dl > 0 {
                1.0
            } else if dl < 0 {
                -1.0
            } else {
                let pn = c.tri_normal(c.xn, &t);
                let sp = (c.xn[v] - c.xn[t[0] as usize]).dot(pn);
                if sp > 1.0e-9 {
                    1.0
                } else if sp < -1.0e-9 || dvec.dot(tn) < 0.0 {
                    -1.0
                } else {
                    1.0
                }
            };
            let sep = c.sep(&[v as u32, t[0], t[1], t[2]], dl);
            if dist > sep + c.margin {
                continue;
            }
            let bary = barycentric(cp, a, b, cc);
            let interior = bary.iter().all(|w| *w > 1.0e-6);
            let plane = dvec.dot(tn) * s_req;
            let sd = if interior {
                plane
            } else if plane >= 0.0 {
                dist
            } else {
                -dist
            };
            if sd >= sep {
                continue;
            }
            let n = if interior || plane < 0.0 || dist < 1.0e-12 { tn * s_req } else { dvec / dist };
            let wt = [c.inv_mass[t[0] as usize], c.inv_mass[t[1] as usize], c.inv_mass[t[2] as usize]];
            let wsum = wv + bary[0] * bary[0] * wt[0] + bary[1] * bary[1] * wt[1] + bary[2] * bary[2] * wt[2];
            if wsum < 1.0e-18 {
                continue;
            }
            if c.seam[v] || t.iter().any(|&i| c.seam[i as usize]) {
                continue;
            }
            let lambda = (sep - sd).min(MAX_PUSH * sep) / wsum;
            x[v] += n * (lambda * wv);
            for k in 0..3 {
                x[t[k] as usize] -= n * (lambda * wt[k] * bary[k]);
            }
            count += 1;
        }
    }
    count
}

pub fn edge_edge_pass(x: &mut [DVec3], hash: &SpatialHash, c: &Contact) -> u32 {
    let mut cand = Vec::new();
    let mut count = 0;
    for t in c.tris {
        for e in 0..3 {
            let (p, q) = (t[e], t[(e + 1) % 3]);
            if p > q {
                continue;
            }
            hash.query((x[p as usize] + x[q as usize]) * 0.5, &mut cand);
            for &tj in &cand {
                let u = c.tris[tj as usize];
                for f in 0..3 {
                    let (r, s) = (u[f], u[(f + 1) % 3]);
                    if r > s || (p, q) >= (r, s) {
                        continue;
                    }
                    if p == r || p == s || q == r || q == s {
                        continue;
                    }
                    let (sp, st, c1, c2) = seg_seg(x[p as usize], x[q as usize], x[r as usize], x[s as usize]);
                    let delta = c1 - c2;
                    let dist = delta.length();
                    if [p, q, r, s].iter().any(|&i| c.seam[i as usize]) {
                        continue;
                    }
                    if dist >= c.reach + c.margin || c.linked(p, r) || c.linked(p, s) || c.linked(q, r) || c.linked(q, s) {
                        continue;
                    }
                    let dl = c.layer[p as usize] - c.layer[r as usize];
                    let sep = c.sep(&[p, q, r, s], dl);
                    let (n, sd) = if dl == 0 {
                        if dist < 1.0e-12 {
                            continue;
                        }
                        (delta / dist, dist)
                    } else {
                        let (lo, sign) = if dl > 0 { (&u, 1.0) } else { (t, -1.0) };
                        let tn = c.tri_normal(x, lo) * sign;
                        if tn == DVec3::ZERO {
                            continue;
                        }
                        (tn, delta.dot(tn))
                    };
                    if sd >= sep {
                        continue;
                    }
                    let w = [c.inv_mass[p as usize], c.inv_mass[q as usize], c.inv_mass[r as usize], c.inv_mass[s as usize]];
                    let k = [1.0 - sp, sp, 1.0 - st, st];
                    let wsum: f64 = (0..4).map(|i| w[i] * k[i] * k[i]).sum();
                    if wsum < 1.0e-18 {
                        continue;
                    }
                    let lambda = (sep - sd).min(MAX_PUSH * sep) / wsum;
                    x[p as usize] += n * (lambda * w[0] * k[0]);
                    x[q as usize] += n * (lambda * w[1] * k[1]);
                    x[r as usize] -= n * (lambda * w[2] * k[2]);
                    x[s as usize] -= n * (lambda * w[3] * k[3]);
                    count += 1;
                }
            }
        }
    }
    count
}

fn segment_hits(a: DVec3, b: DVec3, t0: DVec3, t1: DVec3, t2: DVec3) -> bool {
    let d = b - a;
    let e1 = t1 - t0;
    let e2 = t2 - t0;
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1.0e-14 {
        return false;
    }
    let inv = 1.0 / det;
    let s = a - t0;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let qv = s.cross(e1);
    let v = d.dot(qv) * inv;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = e2.dot(qv) * inv;
    (0.0..=1.0).contains(&t)
}

fn triangles_intersect(x: &[DVec3], t: &[u32; 3], u: &[u32; 3]) -> bool {
    let p = [x[t[0] as usize], x[t[1] as usize], x[t[2] as usize]];
    let q = [x[u[0] as usize], x[u[1] as usize], x[u[2] as usize]];
    (0..3).any(|i| segment_hits(p[i], p[(i + 1) % 3], q[0], q[1], q[2])) || (0..3).any(|i| segment_hits(q[i], q[(i + 1) % 3], p[0], p[1], p[2]))
}

impl Solver {
    fn contact_hash(&self, margin: f64) -> SpatialHash {
        let cell = (self.model.rest_edge * 1.5).max(margin * 2.0);
        SpatialHash::build(&self.x, &self.model.triangles, cell, margin)
    }

    pub(crate) fn seam_flags(&self) -> Vec<bool> {
        let mut seam = vec![false; self.x.len()];
        for l in &self.model.links {
            for &v in &l.v[..l.n] {
                seam[v as usize] = true;
            }
        }
        for _ in 0..2 {
            let s0 = seam.clone();
            for (i, nb) in self.model.neighbors.iter().enumerate() {
                if nb.iter().any(|&w| s0[w as usize]) {
                    seam[i] = true;
                }
            }
        }
        seam
    }

    pub(crate) fn resolve_self_contact(&mut self) {
        let seam = self.seam_flags();
        let cap = SPEED_CAP * self.substep_time();
        for ((x, xn), pinned) in self.x.iter_mut().zip(&self.xn).zip(&self.pinned) {
            let d = *x - *xn;
            let l = d.length();
            if l > cap && !*pinned {
                *x = *xn + d * (cap / l);
            }
        }
        let base = self.params.collision_thickness.max(0.0) as f64;
        let mean_thick = self.model.thickness.iter().copied().fold(0.0, f64::max);
        let reach = base.max(mean_thick) * 3.0;
        if reach <= 0.0 {
            return;
        }
        let mean = self.x.iter().zip(&self.xn).map(|(a, b)| *a - *b).sum::<DVec3>() / self.x.len().max(1) as f64;
        let moved = self.x.iter().zip(&self.xn).map(|(a, b)| (*a - *b - mean).length()).fold(0.0, f64::max);
        let margin = (moved + 0.5 * reach).min(self.model.rest_edge.max(1.0));
        let hash = self.contact_hash(reach + margin);
        let c = Contact {
            tris: &self.model.triangles,
            neighbors: &self.model.neighbors,
            inv_mass: &self.inv_mass,
            thick: &self.model.thickness,
            layer: &self.model.layer,
            xn: &self.xn,
            base,
            reach,
            margin,
            seam: &seam,
        };
        let before = self.x.clone();
        vertex_triangle_pass(&mut self.x, &hash, &c);
        edge_edge_pass(&mut self.x, &hash, &c);
        vertex_triangle_pass(&mut self.x, &hash, &c);
        self.project_links();
        let delta: Vec<DVec3> = self.x.iter().zip(&before).map(|(a, b)| *a - *b).collect();
        PUSH.with(|p| *p.borrow_mut() = delta);
    }

    pub(crate) fn project_links(&mut self) {
        for _ in 0..2 {
            for l in &self.model.links {
                let mut d = DVec3::ZERO;
                for k in 0..l.n {
                    d += self.x[l.v[k] as usize] * l.c[k];
                }
                let len = d.length();
                if len < 1.0e-9 || len <= l.rest {
                    continue;
                }
                let dir = d / len;
                let w: Vec<f64> = (0..l.n).map(|k| if self.pinned[l.v[k] as usize] { 0.0 } else { self.inv_mass[l.v[k] as usize] }).collect();
                let wsum: f64 = (0..l.n).map(|k| w[k] * l.c[k] * l.c[k]).sum();
                if wsum < 1.0e-18 {
                    continue;
                }
                let lambda = 0.5 * (len - l.rest) / wsum;
                for (k, wk) in w.iter().enumerate() {
                    self.x[l.v[k] as usize] -= dir * (lambda * wk * l.c[k]);
                }
            }
        }
    }

    pub(crate) fn settle_self_contact(&mut self) {
        let push = PUSH.with(|p| std::mem::take(&mut *p.borrow_mut()));
        if push.len() != self.x.len() {
            return;
        }
        for ((xn, p), pinned) in self.xn.iter_mut().zip(&push).zip(&self.pinned) {
            if !*pinned {
                *xn += *p;
            }
        }
    }

    pub fn self_intersections(&self) -> Vec<(u32, u32)> {
        let hash = self.contact_hash(0.0);
        let mut out = Vec::new();
        let mut cand = Vec::new();
        let tris = &self.model.triangles;
        for (ti, t) in tris.iter().enumerate() {
            let mid = (self.x[t[0] as usize] + self.x[t[1] as usize] + self.x[t[2] as usize]) / 3.0;
            hash.query(mid, &mut cand);
            for &tj in &cand {
                if (tj as usize) <= ti {
                    continue;
                }
                let u = &tris[tj as usize];
                let nb = &self.model.neighbors;
                if t.iter().any(|&a| u.contains(&a) || u.iter().any(|b| nb[a as usize].binary_search(b).is_ok())) {
                    continue;
                }
                if triangles_intersect(&self.x, t, u) {
                    out.push((ti as u32, tj));
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn untangle(&mut self, steps: u32) -> usize {
        let base = self.params.collision_thickness.max(0.0) as f64;
        let push = base.max(self.model.thickness.iter().copied().fold(0.0, f64::max)).max(0.5);
        let mut left = 0;
        for _ in 0..steps.max(1) {
            let pairs = self.self_intersections();
            left = pairs.len();
            if pairs.is_empty() {
                break;
            }
            for (ti, tj) in pairs {
                let (t, u) = (self.model.triangles[ti as usize], self.model.triangles[tj as usize]);
                let ct = (self.x[t[0] as usize] + self.x[t[1] as usize] + self.x[t[2] as usize]) / 3.0;
                let cu = (self.x[u[0] as usize] + self.x[u[1] as usize] + self.x[u[2] as usize]) / 3.0;
                let mut n = (self.x[t[1] as usize] - self.x[t[0] as usize]).cross(self.x[t[2] as usize] - self.x[t[0] as usize]).normalize_or_zero();
                if n == DVec3::ZERO {
                    n = (cu - ct).normalize_or_zero();
                }
                if n.dot(cu - ct) < 0.0 {
                    n = -n;
                }
                let lt = self.model.layer[t[0] as usize];
                let lu = self.model.layer[u[0] as usize];
                let (ft, fu) = if lt > lu { (1.0, 0.25) } else if lt < lu { (0.25, 1.0) } else { (0.5, 0.5) };
                for &i in &t {
                    if !self.pinned[i as usize] {
                        self.x[i as usize] -= n * (push * ft);
                    }
                }
                for &i in &u {
                    if !self.pinned[i as usize] {
                        self.x[i as usize] += n * (push * fu);
                    }
                }
            }
            self.xn.clone_from(&self.x);
        }
        left
    }
}
