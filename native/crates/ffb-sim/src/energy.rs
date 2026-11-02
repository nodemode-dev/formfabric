use glam::{DMat3, DVec3};

pub const COMPRESSION: f64 = 0.05;

#[derive(Debug, Clone, Copy)]
pub struct NonLinear {
    pub on: bool,
    pub gain: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct TriElem {
    pub v: [u32; 3],
    pub alpha: [f64; 3],
    pub beta: [f64; 3],
    pub area: f64,
    pub ku: f64,
    pub kv: f64,
    pub ks: f64,
    pub kd: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct HingeElem {
    pub v: [u32; 4],
    pub coef: f64,
    pub theta0: f64,
    pub h_rest: [f64; 2],
    pub buckle_ratio: f64,
    pub buckle_floor: f64,
    pub kd: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct LinkElem {
    pub v: [u32; 3],
    pub c: [f64; 3],
    pub n: usize,
    pub k: f64,
    pub rest: f64,
}

fn outer(a: DVec3, b: DVec3) -> DMat3 {
    DMat3::from_cols(a * b.x, a * b.y, a * b.z)
}

pub struct TriEval {
    alpha: [f64; 3],
    beta: [f64; 3],
    ru: f64,
    rv: f64,
    nu: DVec3,
    nv: DVec3,
    mu: (f64, f64),
    mv: (f64, f64),
    shear_g: f64,
    shear_k: f64,
    fu: DVec3,
    fv: DVec3,
}

fn stretch_terms(f: DVec3, k: f64, area: f64, nl: NonLinear) -> (f64, DVec3, (f64, f64)) {
    let len = f.length();
    if len < 1.0e-12 {
        return (0.0, DVec3::X, (k * area, k * area));
    }
    let inv = 1.0 / len;
    let n = f * inv;
    let eps = len - 1.0;
    let (kg, kh) = if nl.on && eps > 0.0 {
        (k * (eps + nl.gain * eps * eps * eps), k * (1.0 + 3.0 * nl.gain * eps * eps))
    } else if eps < 0.0 {
        (k * COMPRESSION * eps, k * COMPRESSION)
    } else {
        (k * eps, k)
    };
    let tf = if eps > 0.0 { kg * inv } else { 0.0 };
    (kg * area, n, (kh * area, tf * area))
}

fn term_mat(n: DVec3, m: (f64, f64), w: f64) -> DMat3 {
    outer(n, n) * ((m.0 - m.1) * w) + DMat3::IDENTITY * (m.1 * w)
}

fn term_apply(n: DVec3, m: (f64, f64), v: DVec3) -> DVec3 {
    n * ((m.0 - m.1) * n.dot(v)) + v * m.1
}

impl TriElem {
    pub fn eval(&self, p: [DVec3; 3], nl: NonLinear) -> TriEval {
        let fu = p[0] * self.alpha[0] + p[1] * self.alpha[1] + p[2] * self.alpha[2];
        let fv = p[0] * self.beta[0] + p[1] * self.beta[1] + p[2] * self.beta[2];
        self.eval_f(fu, fv, nl)
    }

    pub fn eval_f(&self, fu: DVec3, fv: DVec3, nl: NonLinear) -> TriEval {
        let (ru, nu, mu) = stretch_terms(fu, self.ku, self.area, nl);
        let (rv, nv, mv) = stretch_terms(fv, self.kv, self.area, nl);
        TriEval { alpha: self.alpha, beta: self.beta, ru, rv, nu, nv, mu, mv, shear_g: fu.dot(fv), shear_k: self.ks * self.area, fu, fv }
    }

    pub fn energy(&self, p: [DVec3; 3], nl: NonLinear) -> f64 {
        let fu = p[0] * self.alpha[0] + p[1] * self.alpha[1] + p[2] * self.alpha[2];
        let fv = p[0] * self.beta[0] + p[1] * self.beta[1] + p[2] * self.beta[2];
        let one = |f: DVec3, k: f64| {
            let e = f.length() - 1.0;
            let q = if nl.on && e > 0.0 {
                e * e * (1.0 + 0.5 * nl.gain * e * e)
            } else if e < 0.0 {
                COMPRESSION * e * e
            } else {
                e * e
            };
            0.5 * k * q
        };
        let g = fu.dot(fv);
        self.area * (one(fu, self.ku) + one(fv, self.kv) + 0.5 * self.ks * g * g)
    }
}

impl TriEval {
    fn jac(&self, s: usize) -> DVec3 {
        self.fv * self.alpha[s] + self.fu * self.beta[s]
    }

    pub fn grad(&self, s: usize) -> DVec3 {
        self.nu * (self.alpha[s] * self.ru) + self.nv * (self.beta[s] * self.rv) + self.jac(s) * (self.shear_k * self.shear_g)
    }

    pub fn hess(&self, s: usize, t: usize) -> DMat3 {
        term_mat(self.nu, self.mu, self.alpha[s] * self.alpha[t]) + term_mat(self.nv, self.mv, self.beta[s] * self.beta[t]) + outer(self.jac(s), self.jac(t)) * self.shear_k
    }

    pub fn damp_d(&self, s: usize, du: DVec3, dv: DVec3) -> DVec3 {
        term_apply(self.nu, self.mu, du) * self.alpha[s] + term_apply(self.nv, self.mv, dv) * self.beta[s] + self.jac(s) * (self.shear_k * (self.fv.dot(du) + self.fu.dot(dv)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HingeEval {
    g: [DVec3; 4],
    c: f64,
    dtheta: f64,
}

impl HingeElem {
    pub fn eval(&self, p: [DVec3; 4]) -> HingeEval {
        let e = p[1] - p[0];
        let le = e.length();
        let n1v = e.cross(p[2] - p[0]);
        let n2v = (p[0] - p[1]).cross(p[3] - p[1]);
        let l1 = n1v.length_squared();
        let l2 = n2v.length_squared();
        if le < 1.0e-9 || l1 < 1.0e-18 || l2 < 1.0e-18 {
            return HingeEval { g: [DVec3::ZERO; 4], c: 0.0, dtheta: 0.0 };
        }
        let eh = e / le;
        let n1 = n1v / l1.sqrt();
        let n2 = n2v / l2.sqrt();
        let sin_t = n2.cross(n1).dot(eh);
        let cos_t = n1.dot(n2);
        let mut d = if self.theta0 == 0.0 && cos_t > 0.0 && sin_t.abs() < 0.4 {
            let s2 = sin_t * sin_t;
            sin_t * (1.0 + s2 * (1.0 / 6.0 + s2 * (3.0 / 40.0 + s2 * (5.0 / 112.0 + s2 * 35.0 / 1152.0))))
        } else {
            sin_t.atan2(cos_t) - self.theta0
        };
        while d > std::f64::consts::PI {
            d -= std::f64::consts::TAU;
        }
        while d < -std::f64::consts::PI {
            d += std::f64::consts::TAU;
        }
        let g2 = n1v * (le / l1);
        let g3 = n2v * (le / l2);
        let a1 = (p[2] - p[0]).dot(e) / (le * le);
        let a2 = (p[3] - p[0]).dot(e) / (le * le);
        let g0 = -g2 * (1.0 - a1) - g3 * (1.0 - a2);
        let g1 = -g2 * a1 - g3 * a2;
        let lambda = 0.5 * (l1.sqrt() / le / self.h_rest[0].max(1.0e-9) + l2.sqrt() / le / self.h_rest[1].max(1.0e-9));
        let compress = 1.0 - lambda;
        let f = if self.buckle_ratio > 0.0 && compress > 0.0 {
            1.0 - (1.0 - self.buckle_floor) * (compress * self.buckle_ratio / 0.05).min(1.0)
        } else {
            1.0
        };
        HingeEval { g: [g0, g1, g2, g3], c: self.coef * f, dtheta: d }
    }

    pub fn theta(&self, p: [DVec3; 4]) -> f64 {
        let e = p[1] - p[0];
        let n1 = e.cross(p[2] - p[0]).normalize_or_zero();
        let n2 = (p[0] - p[1]).cross(p[3] - p[1]).normalize_or_zero();
        n2.cross(n1).dot(e.normalize_or_zero()).atan2(n1.dot(n2))
    }
}

impl HingeEval {
    pub fn grad(&self, s: usize) -> DVec3 {
        self.g[s] * (self.c * self.dtheta)
    }

    pub fn hess(&self, s: usize, t: usize) -> DMat3 {
        outer(self.g[s], self.g[t]) * self.c
    }

    pub fn dtheta(&self) -> f64 {
        self.dtheta
    }

    pub fn coef(&self) -> f64 {
        self.c
    }

    pub fn gvec(&self, s: usize) -> DVec3 {
        self.g[s]
    }

    pub fn dot_g(&self, s: usize, v: DVec3) -> f64 {
        self.g[s].dot(v)
    }

    pub fn dot_all(&self, d: [DVec3; 4]) -> f64 {
        self.g[0].dot(d[0]) + self.g[1].dot(d[1]) + self.g[2].dot(d[2]) + self.g[3].dot(d[3])
    }

    pub fn scaled(&self, s: usize, t: f64) -> DVec3 {
        self.g[s] * (self.c * t)
    }

    pub fn damp(&self, s: usize, d: [DVec3; 4]) -> DVec3 {
        let sum = self.g[0].dot(d[0]) + self.g[1].dot(d[1]) + self.g[2].dot(d[2]) + self.g[3].dot(d[3]);
        self.g[s] * (self.c * sum)
    }

    pub fn energy(&self) -> f64 {
        0.5 * self.c * self.dtheta * self.dtheta
    }
}

pub struct LinkEval {
    c: [f64; 3],
    force: f64,
    n: DVec3,
    m: DMat3,
}

impl LinkElem {
    pub fn eval(&self, p: [DVec3; 3]) -> LinkEval {
        let mut d = DVec3::ZERO;
        for (q, c) in p.iter().zip(&self.c).take(self.n) {
            d += *q * *c;
        }
        let len = d.length();
        if len < 1.0e-9 {
            return LinkEval { c: self.c, force: 0.0, n: DVec3::X, m: DMat3::IDENTITY * self.k };
        }
        let n = d / len;
        let c = len - self.rest;
        let nn = outer(n, n);
        let tf = if c > 0.0 { self.k * c / len } else { 0.0 };
        let m = nn * self.k + (DMat3::IDENTITY - nn) * tf;
        LinkEval { c: self.c, force: self.k * c, n, m }
    }

    pub fn energy(&self, p: [DVec3; 3]) -> f64 {
        let mut d = DVec3::ZERO;
        for (q, c) in p.iter().zip(&self.c).take(self.n) {
            d += *q * *c;
        }
        let c = d.length() - self.rest;
        0.5 * self.k * c * c
    }
}

impl LinkEval {
    pub fn grad(&self, s: usize) -> DVec3 {
        self.n * (self.force * self.c[s])
    }

    pub fn hess(&self, s: usize, t: usize) -> DMat3 {
        self.m * (self.c[s] * self.c[t])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Acc {
    pub g: DVec3,
    pub h: [f64; 6],
}

impl Acc {
    pub fn new(diag: f64) -> Acc {
        Acc { g: DVec3::ZERO, h: [diag, 0.0, 0.0, diag, 0.0, diag] }
    }

    pub fn add_outer(&mut self, v: DVec3, k: f64) {
        let (kx, ky, kz) = (v.x * k, v.y * k, v.z * k);
        self.h[0] += kx * v.x;
        self.h[1] += kx * v.y;
        self.h[2] += kx * v.z;
        self.h[3] += ky * v.y;
        self.h[4] += ky * v.z;
        self.h[5] += kz * v.z;
    }

    pub fn add_diag(&mut self, k: f64) {
        self.h[0] += k;
        self.h[3] += k;
        self.h[5] += k;
    }

    pub fn add_mat(&mut self, m: DMat3) {
        self.h[0] += m.x_axis.x;
        self.h[1] += m.y_axis.x;
        self.h[2] += m.z_axis.x;
        self.h[3] += m.y_axis.y;
        self.h[4] += m.z_axis.y;
        self.h[5] += m.z_axis.z;
    }

    pub fn newton(&self) -> DVec3 {
        let [a, b, c, d, e, f] = self.h;
        let c0 = d * f - e * e;
        let c1 = c * e - b * f;
        let c2 = b * e - c * d;
        let det = a * c0 + b * c1 + c * c2;
        if det.abs() < 1.0e-30 || !det.is_finite() {
            return DVec3::ZERO;
        }
        let c3 = a * f - c * c;
        let c4 = b * c - a * e;
        let c5 = a * d - b * b;
        let r = -self.g;
        let inv = 1.0 / det;
        DVec3::new(c0 * r.x + c1 * r.y + c2 * r.z, c1 * r.x + c3 * r.y + c4 * r.z, c2 * r.x + c4 * r.y + c5 * r.z) * inv
    }
}

impl TriElem {
    #[allow(clippy::too_many_arguments)]
    pub fn accumulate(&self, fu: DVec3, fv: DVec3, du: DVec3, dv: DVec3, slot: usize, nl: NonLinear, r: f64, acc: &mut Acc) {
        let (a, b) = (self.alpha[slot], self.beta[slot]);
        let (ru, nu, mu) = stretch_terms(fu, self.ku, self.area, nl);
        let (rv, nv, mv) = stretch_terms(fv, self.kv, self.area, nl);
        let s1 = 1.0 + r;
        let sk = self.ks * self.area;
        let j = fv * a + fu * b;
        let mut g = nu * (a * ru) + nv * (b * rv) + j * (sk * fu.dot(fv));
        acc.add_outer(nu, (mu.0 - mu.1) * a * a * s1);
        acc.add_outer(nv, (mv.0 - mv.1) * b * b * s1);
        acc.add_diag((mu.1 * a * a + mv.1 * b * b) * s1);
        acc.add_outer(j, sk * s1);
        if r > 0.0 {
            let q = nu * ((mu.0 - mu.1) * nu.dot(du)) + du * mu.1;
            let p = nv * ((mv.0 - mv.1) * nv.dot(dv)) + dv * mv.1;
            g += (q * a + p * b + j * (sk * (fv.dot(du) + fu.dot(dv)))) * r;
        }
        acc.g += g;
    }
}

impl HingeEval {
    pub fn accumulate(&self, slot: usize, lin: f64, ds: f64, r: f64, acc: &mut Acc) {
        let g = self.g[slot];
        acc.g += g * (self.c * (lin + r * ds));
        acc.add_outer(g, self.c * (1.0 + r));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TriAsm {
    pub grad: [DVec3; 3],
    pub blk: [[f64; 9]; 6],
}

pub const TRI_PAIRS: [(usize, usize); 6] = [(0, 0), (0, 1), (0, 2), (1, 1), (1, 2), (2, 2)];

impl TriElem {
    pub fn assemble(&self, p: [DVec3; 3], d: [DVec3; 3], r: f64, nl: NonLinear, hess: bool) -> TriAsm {
        let fu = p[0] * self.alpha[0] + p[1] * self.alpha[1] + p[2] * self.alpha[2];
        let fv = p[0] * self.beta[0] + p[1] * self.beta[1] + p[2] * self.beta[2];
        let (ru, nu, mu) = stretch_terms(fu, self.ku, self.area, nl);
        let (rv, nv, mv) = stretch_terms(fv, self.kv, self.area, nl);
        let sk = self.ks * self.area;
        let sg = sk * fu.dot(fv);
        let (cu, cv) = (mu.0 - mu.1, mv.0 - mv.1);
        let mut grad = [DVec3::ZERO; 3];
        let (du, dv) = if r > 0.0 {
            (d[0] * self.alpha[0] + d[1] * self.alpha[1] + d[2] * self.alpha[2], d[0] * self.beta[0] + d[1] * self.beta[1] + d[2] * self.beta[2])
        } else {
            (DVec3::ZERO, DVec3::ZERO)
        };
        let qu = nu * (cu * nu.dot(du)) + du * mu.1;
        let qv = nv * (cv * nv.dot(dv)) + dv * mv.1;
        let sd = sk * (fv.dot(du) + fu.dot(dv));
        let mut jac = [DVec3::ZERO; 3];
        for s in 0..3 {
            let (a, b) = (self.alpha[s], self.beta[s]);
            jac[s] = fv * a + fu * b;
            let mut g = nu * (a * ru) + nv * (b * rv) + jac[s] * sg;
            if r > 0.0 {
                g += (qu * a + qv * b + jac[s] * sd) * r;
            }
            grad[s] = g;
        }
        let mut blk = [[0.0f64; 9]; 6];
        if hess {
            let s1 = 1.0 + r;
            let uu = [nu.x * nu.x, nu.x * nu.y, nu.x * nu.z, nu.y * nu.y, nu.y * nu.z, nu.z * nu.z];
            let vv = [nv.x * nv.x, nv.x * nv.y, nv.x * nv.z, nv.y * nv.y, nv.y * nv.z, nv.z * nv.z];
            for (k, &(s, t)) in TRI_PAIRS.iter().enumerate() {
                let (aa, bb) = (self.alpha[s] * self.alpha[t] * s1, self.beta[s] * self.beta[t] * s1);
                let (e, f, g) = (aa * cu, bb * cv, aa * mu.1 + bb * mv.1);
                let js = jac[s] * (sk * s1);
                let jt = jac[t];
                let b = &mut blk[k];
                b[0] = e * uu[0] + f * vv[0] + js.x * jt.x + g;
                b[1] = e * uu[1] + f * vv[1] + js.x * jt.y;
                b[2] = e * uu[2] + f * vv[2] + js.x * jt.z;
                b[3] = b[1] + js.y * jt.x - js.x * jt.y;
                b[4] = e * uu[3] + f * vv[3] + js.y * jt.y + g;
                b[5] = e * uu[4] + f * vv[4] + js.y * jt.z;
                b[6] = b[2] + js.z * jt.x - js.x * jt.z;
                b[7] = b[5] + js.z * jt.y - js.y * jt.z;
                b[8] = e * uu[5] + f * vv[5] + js.z * jt.z + g;
            }
        }
        TriAsm { grad, blk }
    }
}

pub fn solve3(h: DMat3, g: DVec3) -> DVec3 {
    let det = h.determinant();
    if det.abs() < 1.0e-30 || !det.is_finite() {
        return DVec3::ZERO;
    }
    h.inverse() * g
}
