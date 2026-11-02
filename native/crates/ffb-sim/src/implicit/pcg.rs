use glam::DVec3;

use super::coarse::Coarse;
use super::matrix::{inverse3, matvec, Pattern};

pub const FREE: u8 = 0;
pub const PINNED: u8 = 1;
pub const PLANE: u8 = 2;

#[derive(Debug, Clone, Default)]
pub struct Work {
    pub r: Vec<f64>,
    pub z: Vec<f64>,
    pub p: Vec<f64>,
    pub q: Vec<f64>,
    pub pre: Vec<f64>,
    pub bnorm: f64,
}

impl Work {
    pub fn new(n: usize) -> Work {
        Work { r: vec![0.0; 3 * n], z: vec![0.0; 3 * n], p: vec![0.0; 3 * n], q: vec![0.0; 3 * n], pre: vec![0.0; 9 * n], bnorm: 0.0 }
    }
}

pub struct Filter<'a> {
    pub kind: &'a [u8],
    pub nrm: &'a [DVec3],
    pub cons: &'a [u32],
}

impl Filter<'_> {
    pub fn apply(&self, v: &mut [f64]) {
        for &i in self.cons {
            let i = i as usize;
            let o = 3 * i;
            if self.kind[i] == PINNED {
                v[o] = 0.0;
                v[o + 1] = 0.0;
                v[o + 2] = 0.0;
            } else {
                let n = self.nrm[i];
                let d = v[o] * n.x + v[o + 1] * n.y + v[o + 2] * n.z;
                v[o] -= d * n.x;
                v[o + 1] -= d * n.y;
                v[o + 2] -= d * n.z;
            }
        }
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn build_preconditioner(pat: &Pattern, a: &[f64], f: &Filter, pre: &mut [f64]) {
    for i in 0..pat.n {
        let d = &a[9 * pat.diag[i] as usize..9 * pat.diag[i] as usize + 9];
        let out = &mut pre[9 * i..9 * i + 9];
        match f.kind[i] {
            PINNED => out.fill(0.0),
            PLANE => {
                let n = f.nrm[i];
                let nv = [n.x, n.y, n.z];
                let mut s = [[0.0f64; 3]; 3];
                for r in 0..3 {
                    for c in 0..3 {
                        s[r][c] = (if r == c { 1.0 } else { 0.0 }) - nv[r] * nv[c];
                    }
                }
                let mut t = [0.0f64; 9];
                for r in 0..3 {
                    for c in 0..3 {
                        let mut acc = 0.0;
                        for k in 0..3 {
                            for l in 0..3 {
                                acc += s[r][k] * d[3 * k + l] * s[l][c];
                            }
                        }
                        t[3 * r + c] = acc + nv[r] * nv[c];
                    }
                }
                match inverse3(&t) {
                    Some(inv) => out.copy_from_slice(&inv),
                    None => out.copy_from_slice(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
                }
            }
            _ => match inverse3(d) {
                Some(inv) => out.copy_from_slice(&inv),
                None => out.copy_from_slice(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
            },
        }
    }
}

fn apply_pre(pre: &[f64], r: &[f64], z: &mut [f64]) {
    for ((b, rv), zv) in pre.as_chunks::<9>().0.iter().zip(r.as_chunks::<3>().0).zip(z.as_chunks_mut::<3>().0) {
        zv[0] = b[0] * rv[0] + b[1] * rv[1] + b[2] * rv[2];
        zv[1] = b[3] * rv[0] + b[4] * rv[1] + b[5] * rv[2];
        zv[2] = b[6] * rv[0] + b[7] * rv[1] + b[8] * rv[2];
    }
}

#[allow(clippy::too_many_arguments)]
pub fn solve(pat: &Pattern, a: &[f64], f: &Filter, rhs: &[f64], dx: &mut [f64], w: &mut Work, tol: f64, abs: f64, max_it: usize, mut coarse: Option<&mut Coarse>) -> usize {
    f.apply(dx);
    build_preconditioner(pat, a, f, &mut w.pre);
    if let Some(c) = coarse.as_deref_mut() {
        c.factor(pat, a, f);
    }
    let mut pre = |pre: &[f64], r: &[f64], z: &mut [f64]| {
        apply_pre(pre, r, z);
        if let Some(c) = coarse.as_deref_mut() {
            if c.ok {
                c.apply(r, z);
                f.apply(z);
            }
        }
    };
    matvec(pat, a, dx, &mut w.q);
    f.apply(&mut w.q);
    w.r.copy_from_slice(rhs);
    f.apply(&mut w.r);
    let bnorm = dot(&w.r, &w.r).sqrt();
    w.bnorm = bnorm;
    for (r, q) in w.r.iter_mut().zip(&w.q) {
        *r -= *q;
    }
    if bnorm < 1.0e-30 {
        dx.fill(0.0);
        return 0;
    }
    let target = (tol * bnorm).max(abs);
    if dot(&w.r, &w.r).sqrt() <= target {
        return 0;
    }
    pre(&w.pre, &w.r, &mut w.z);
    w.p.copy_from_slice(&w.z);
    let mut rz = dot(&w.r, &w.z);
    let mut used = 0;
    for it in 0..max_it {
        matvec(pat, a, &w.p, &mut w.q);
        f.apply(&mut w.q);
        let pq = dot(&w.p, &w.q);
        if pq <= 0.0 || !pq.is_finite() {
            break;
        }
        let alpha = rz / pq;
        for ((x, r), (p, q)) in dx.iter_mut().zip(w.r.iter_mut()).zip(w.p.iter().zip(&w.q)) {
            *x += alpha * *p;
            *r -= alpha * *q;
        }
        used = it + 1;
        if dot(&w.r, &w.r).sqrt() <= target {
            break;
        }
        pre(&w.pre, &w.r, &mut w.z);
        let rz_new = dot(&w.r, &w.z);
        let beta = rz_new / rz;
        rz = rz_new;
        for (p, z) in w.p.iter_mut().zip(&w.z) {
            *p = *z + beta * *p;
        }
    }
    used
}
