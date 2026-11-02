use std::collections::HashSet;

use glam::DVec3;

use crate::collide::resolve;
use crate::solver::Solver;
use crate::Untangle;

const ROUNDS: usize = 3;
const FRAME_CAP: f64 = 2.0;
const GAIN: f64 = 0.5;
const EPS: f64 = 1.0e-9;

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    prev: Vec<DVec3>,
    pairs: HashSet<(u32, u32)>,
    pub(crate) left: usize,
    pub(crate) moved: f64,
}

impl State {
    pub(crate) fn clear(&mut self) {
        self.prev.clear();
        self.pairs.clear();
        self.left = 0;
        self.moved = 0.0;
    }
}

fn normal(x: &[DVec3], t: &[u32; 3]) -> DVec3 {
    (x[t[1] as usize] - x[t[0] as usize]).cross(x[t[2] as usize] - x[t[0] as usize]).normalize_or_zero()
}

fn sign(v: f64) -> f64 {
    if v >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

impl Solver {
    pub fn untangle_report(&self) -> (usize, f64) {
        (self.tangle.left, self.tangle.moved)
    }

    fn wanted_side(&self, mode: Untangle, a: &[u32; 3], b: &[u32; 3], known: bool, na: DVec3) -> f64 {
        let (la, lb) = (self.model.layer[a[0] as usize], self.model.layer[b[0] as usize]);
        if la != lb {
            let up = if mode == Untangle::Flipped { -1.0 } else { 1.0 };
            return if lb > la { up } else { -up };
        }
        if known {
            let prev = &self.tangle.prev;
            let np = normal(prev, a);
            if np != DVec3::ZERO {
                let d: f64 = b.iter().map(|&v| (prev[v as usize] - prev[a[0] as usize]).dot(np)).sum();
                if d.abs() > EPS {
                    return sign(d) * np.dot(na).signum();
                }
            }
        }
        sign(b.iter().map(|&v| (self.x[v as usize] - self.x[a[0] as usize]).dot(na)).sum())
    }

    pub(crate) fn untangle_frame(&mut self) {
        let mode = self.params.untangle;
        let n = self.x.len();
        if mode == Untangle::Off || n == 0 {
            self.tangle.clear();
            return;
        }
        if self.tangle.prev.len() != n {
            self.tangle.prev = self.x.clone();
        }
        let seam = self.seam_flags();
        let clear = f64::from(self.params.collision_thickness).max(0.5);
        let offset = f64::from(self.params.skin_offset.max(0.0));
        let mut total = vec![DVec3::ZERO; n];
        let mut pairs = self.self_intersections();
        for _ in 0..ROUNDS {
            if pairs.is_empty() {
                break;
            }
            let mut shift = vec![DVec3::ZERO; n];
            let mut hits = vec![0u32; n];
            for &(ti, tj) in &pairs {
                let known = !self.tangle.pairs.contains(&(ti, tj));
                let (t, u) = (self.model.triangles[ti as usize], self.model.triangles[tj as usize]);
                if !known && self.model.layer[t[0] as usize] == self.model.layer[u[0] as usize] {
                    continue;
                }
                for (a, b) in [(t, u), (u, t)] {
                    let na = normal(&self.x, &a);
                    if na == DVec3::ZERO {
                        continue;
                    }
                    let side = self.wanted_side(mode, &a, &b, known, na);
                    for &v in &b {
                        let vi = v as usize;
                        if self.pinned[vi] || seam[vi] {
                            continue;
                        }
                        let d = (self.x[vi] - self.x[a[0] as usize]).dot(na);
                        if d * side >= clear {
                            continue;
                        }
                        shift[vi] += na * (side * clear - d);
                        hits[vi] += 1;
                    }
                }
            }
            let mut any = false;
            for i in 0..n {
                if hits[i] == 0 {
                    continue;
                }
                let mut s = shift[i] / f64::from(hits[i]) * GAIN;
                let room = FRAME_CAP - total[i].length();
                if room <= 0.0 {
                    continue;
                }
                if s.length() > room {
                    s = s.normalize_or_zero() * room;
                }
                self.x[i] += s;
                total[i] += s;
                let dir = s.normalize_or_zero();
                let into = self.v[i].dot(dir);
                if into < 0.0 {
                    self.v[i] -= dir * into;
                }
                any = true;
            }
            if !any {
                break;
            }
            for (i, h) in hits.iter().enumerate() {
                if *h > 0 && !self.pinned[i] {
                    self.x[i] = resolve(&self.colliders, self.x[i], offset).0;
                }
            }
            self.project_links();
            pairs = self.self_intersections();
        }
        let moved = total.iter().map(|d| d.length()).fold(0.0, f64::max);
        self.xn.clone_from(&self.x);
        self.tangle.prev.clone_from(&self.x);
        self.tangle.pairs = pairs.iter().copied().collect();
        self.tangle.left = pairs.len();
        self.tangle.moved = moved;
    }
}
