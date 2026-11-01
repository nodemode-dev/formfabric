use crate::geom::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seg {
    pub p: [V2; 4],
    pub curved: bool,
}

const MAX_DEPTH: u32 = 18;
const LENGTH_TOL: f64 = 0.002;

impl Seg {
    pub fn line(a: V2, b: V2) -> Seg {
        Seg { p: [a, lerp(a, b, 1.0 / 3.0), lerp(a, b, 2.0 / 3.0), b], curved: false }
    }

    pub fn at(&self, t: f64) -> V2 {
        let [p0, p1, p2, p3] = self.p;
        let u = 1.0 - t;
        let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        [a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0], a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1]]
    }

    pub fn tangent(&self, t: f64) -> V2 {
        let [p0, p1, p2, p3] = self.p;
        let u = 1.0 - t;
        let (d0, d1, d2) = (sub(p1, p0), sub(p2, p1), sub(p3, p2));
        add(add(mul(d0, 3.0 * u * u), mul(d1, 6.0 * u * t)), mul(d2, 3.0 * t * t))
    }

    pub fn split(&self, t: f64) -> (Seg, Seg) {
        let [p0, p1, p2, p3] = self.p;
        let (a, b, c) = (lerp(p0, p1, t), lerp(p1, p2, t), lerp(p2, p3, t));
        let (d, e) = (lerp(a, b, t), lerp(b, c, t));
        let m = lerp(d, e, t);
        (Seg { p: [p0, a, d, m], curved: self.curved }, Seg { p: [m, e, c, p3], curved: self.curved })
    }

    fn flat(&self, tol: f64) -> bool {
        !self.curved || dist_to_line(self.p[1], self.p[0], self.p[3]).max(dist_to_line(self.p[2], self.p[0], self.p[3])) <= tol
    }

    pub fn flatten_into(&self, tol: f64, out: &mut Vec<V2>) {
        self.flatten_depth(tol.max(1e-6), MAX_DEPTH, out);
    }

    fn flatten_depth(&self, tol: f64, depth: u32, out: &mut Vec<V2>) {
        if depth == 0 || self.flat(tol) {
            out.push(self.p[3]);
        } else {
            let (l, r) = self.split(0.5);
            l.flatten_depth(tol, depth - 1, out);
            r.flatten_depth(tol, depth - 1, out);
        }
    }

    pub fn polyline(&self, tol: f64) -> Vec<V2> {
        let mut out = vec![self.p[0]];
        self.flatten_into(tol, &mut out);
        out
    }

    pub fn length(&self) -> f64 {
        if !self.curved {
            return dist(self.p[0], self.p[3]);
        }
        self.polyline(LENGTH_TOL).windows(2).map(|w| dist(w[0], w[1])).sum()
    }

    pub fn length_to(&self, t: f64) -> f64 {
        self.split(t).0.length()
    }

    pub fn t_at_length(&self, target: f64) -> f64 {
        let total = self.length();
        if total < EPS {
            return 0.0;
        }
        let target = target.clamp(0.0, total);
        if !self.curved {
            return target / total;
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..48 {
            let mid = 0.5 * (lo + hi);
            if self.length_to(mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    pub fn nearest(&self, q: V2) -> (f64, V2, f64) {
        if !self.curved {
            let (t, p) = closest_on_segment(q, self.p[0], self.p[3]);
            return (t, p, dist(q, p));
        }
        const N: usize = 32;
        let mut best = (0.0, f64::MAX);
        for i in 0..=N {
            let t = i as f64 / N as f64;
            let d = dist(self.at(t), q);
            if d < best.1 {
                best = (t, d);
            }
        }
        let step = 1.0 / N as f64;
        let (mut lo, mut hi) = ((best.0 - step).max(0.0), (best.0 + step).min(1.0));
        for _ in 0..40 {
            let m1 = lo + (hi - lo) / 3.0;
            let m2 = hi - (hi - lo) / 3.0;
            if dist(self.at(m1), q) < dist(self.at(m2), q) {
                hi = m2;
            } else {
                lo = m1;
            }
        }
        let t = 0.5 * (lo + hi);
        let p = self.at(t);
        (t, p, dist(p, q))
    }

    pub fn bounds(&self) -> Bounds {
        let mut b = [
            [self.p[0][0].min(self.p[3][0]), self.p[0][1].min(self.p[3][1])],
            [self.p[0][0].max(self.p[3][0]), self.p[0][1].max(self.p[3][1])],
        ];
        if !self.curved {
            return b;
        }
        let [p0, p1, p2, p3] = self.p;
        let (d0, d1, d2) = (sub(p1, p0), sub(p2, p1), sub(p3, p2));
        for axis in 0..2 {
            let a = d0[axis] - 2.0 * d1[axis] + d2[axis];
            let bq = 2.0 * (d1[axis] - d0[axis]);
            let c = d0[axis];
            let mut roots = Vec::new();
            if a.abs() < EPS {
                if bq.abs() > EPS {
                    roots.push(-c / bq);
                }
            } else {
                let disc = bq * bq - 4.0 * a * c;
                if disc >= 0.0 {
                    let r = disc.sqrt();
                    roots.push((-bq + r) / (2.0 * a));
                    roots.push((-bq - r) / (2.0 * a));
                }
            }
            for t in roots.into_iter().filter(|t| *t > 0.0 && *t < 1.0) {
                b = grow(b, self.at(t));
            }
        }
        b
    }
}
