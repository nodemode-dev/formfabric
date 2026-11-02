use glam::{DMat3, DVec3};

use crate::collide::{pair_friction, CONTACT_STIFFNESS, FRICTION_EPS};
use crate::energy::{NonLinear, TRI_PAIRS};
use crate::solver::{Solver, GRAB_STIFFNESS};

use super::pcg::PLANE;
use super::State;

fn put(a: &mut [f64], slot: u32, m: &DMat3, k: f64) {
    let blk = &mut a[9 * slot as usize..9 * slot as usize + 9];
    let c = m.to_cols_array();
    for r in 0..3 {
        for q in 0..3 {
            blk[3 * r + q] += k * c[3 * q + r];
        }
    }
}

fn put_blk(a: &mut [f64], slot: u32, m: &[f64; 9]) {
    let blk = &mut a[9 * slot as usize..9 * slot as usize + 9];
    for (o, v) in blk.iter_mut().zip(m.iter()) {
        *o += *v;
    }
}

fn put_blk_t(a: &mut [f64], slot: u32, m: &[f64; 9]) {
    let blk = &mut a[9 * slot as usize..9 * slot as usize + 9];
    blk[0] += m[0];
    blk[1] += m[3];
    blk[2] += m[6];
    blk[3] += m[1];
    blk[4] += m[4];
    blk[5] += m[7];
    blk[6] += m[2];
    blk[7] += m[5];
    blk[8] += m[8];
}

fn put_outer(a: &mut [f64], slot: u32, u: DVec3, v: DVec3, k: f64) {
    let blk = &mut a[9 * slot as usize..9 * slot as usize + 9];
    let uk = u * k;
    blk[0] += uk.x * v.x;
    blk[1] += uk.x * v.y;
    blk[2] += uk.x * v.z;
    blk[3] += uk.y * v.x;
    blk[4] += uk.y * v.y;
    blk[5] += uk.y * v.z;
    blk[6] += uk.z * v.x;
    blk[7] += uk.z * v.y;
    blk[8] += uk.z * v.z;
}

fn sub_grad(b: &mut [f64], i: u32, g: DVec3) {
    let o = 3 * i as usize;
    b[o] -= g.x;
    b[o + 1] -= g.y;
    b[o + 2] -= g.z;
}

pub(super) fn assemble(s: &Solver, st: &mut State, h: f64, nl: NonLinear, hess: bool) {
    let m = &s.model;
    let x = &s.x;
    let ih2 = 1.0 / (h * h);
    let ih = 1.0 / h;
    if hess {
        st.a.fill(0.0);
    }
    st.b.fill(0.0);
    let pat = &st.pat;
    let a = &mut st.a;
    let b = &mut st.b;
    for (i, xi) in x.iter().enumerate() {
        let w = m.mass[i] * ih2;
        if hess {
            let d = pat.diag[i] as usize * 9;
            a[d] += w;
            a[d + 4] += w;
            a[d + 8] += w;
        }
        sub_grad(b, i as u32, (*xi - s.y[i]) * w);
    }
    let tb = &mut st.tri_buf;
    let hb = &mut st.hev_buf;
    tb.resize(m.tris.len(), crate::energy::TriAsm { grad: [DVec3::ZERO; 3], blk: [[0.0; 9]; 6] });
    hb.resize(m.hinges.len(), crate::energy::HingeEval::default());
    let xn = &s.xn;
    crate::par::row_chunks(&mut tb[..], 1, 128, |first, out| {
        for (k, o) in out.iter_mut().enumerate() {
            let t = &m.tris[first + k];
            let q = t.v.map(|v| v as usize);
            let p = [x[q[0]], x[q[1]], x[q[2]]];
            let d = [p[0] - xn[q[0]], p[1] - xn[q[1]], p[2] - xn[q[2]]];
            *o = t.assemble(p, d, t.kd * ih, nl, hess);
        }
    });
    crate::par::row_chunks(&mut hb[..], 1, 128, |first, out| {
        for (k, o) in out.iter_mut().enumerate() {
            let q = m.hinges[first + k].v.map(|v| v as usize);
            *o = m.hinges[first + k].eval([x[q[0]], x[q[1]], x[q[2]], x[q[3]]]);
        }
    });
    for (ti, t) in m.tris.iter().enumerate() {
        let asm = &tb[ti];
        for k in 0..3 {
            sub_grad(b, t.v[k], asm.grad[k]);
        }
        if hess {
            let slots = &pat.tri[ti];
            for (k, &(si, tj)) in TRI_PAIRS.iter().enumerate() {
                put_blk(a, slots[si * 3 + tj], &asm.blk[k]);
                if tj != si {
                    put_blk_t(a, slots[tj * 3 + si], &asm.blk[k]);
                }
            }
        }
    }
    for (hi, hg) in m.hinges.iter().enumerate() {
        let q = hg.v.map(|v| v as usize);
        let ev = &hb[hi];
        let c = ev.coef();
        if c == 0.0 {
            continue;
        }
        let r = hg.kd * ih;
        let gv = [ev.gvec(0), ev.gvec(1), ev.gvec(2), ev.gvec(3)];
        let sd = if r > 0.0 {
            let mut acc = 0.0;
            for k in 0..4 {
                acc += gv[k].dot(x[q[k]] - s.xn[q[k]]);
            }
            acc
        } else {
            0.0
        };
        let slots = &pat.hinge[hi];
        let theta = ev.dtheta() + r * sd;
        for si in 0..4 {
            sub_grad(b, hg.v[si], gv[si] * (c * theta));
            if hess {
                for tj in 0..4 {
                    put_outer(a, slots[si * 4 + tj], gv[si], gv[tj], c * (1.0 + r));
                }
            }
        }
    }
    for (li, l) in m.links.iter().enumerate() {
        let ev = l.eval([x[l.v[0] as usize], x[l.v[1] as usize], x[l.v[2] as usize]]);
        let slots = &pat.link[li];
        for si in 0..l.n {
            sub_grad(b, l.v[si], ev.grad(si));
            if hess {
                for tj in 0..l.n {
                    put(a, slots[si * 3 + tj], &ev.hess(si, tj), 1.0);
                }
            }
        }
    }
    if let Some((gv, target)) = s.grab {
        let i = gv as usize;
        let k = m.mass[i] * ih2 * GRAB_STIFFNESS;
        if hess {
            let d = pat.diag[i] as usize * 9;
            a[d] += k;
            a[d + 4] += k;
            a[d + 8] += k;
        }
        sub_grad(b, gv, (x[i] - target) * k);
    }
    let mu_global = s.params.friction.max(0.0) as f64;
    for &ci in &st.cons {
        let i = ci as usize;
        if !hess {
            if st.fw[i] != 0.0 && st.kind[i] == PLANE {
                let n = st.nrm[i];
                let d = x[i] - s.xn[i];
                st.fg[i] = (d - n * d.dot(n)) * st.fw[i];
                sub_grad(b, ci, st.fg[i]);
            }
            continue;
        }
        st.fw[i] = 0.0;
        st.fg[i] = DVec3::ZERO;
        if st.kind[i] != PLANE {
            continue;
        }
        let mu = pair_friction(mu_global, m.friction[i]);
        if mu <= 0.0 {
            continue;
        }
        let n = st.nrm[i];
        let d = x[i] - s.xn[i];
        let ut = d - n * d.dot(n);
        let sm = (ut.length_squared() + FRICTION_EPS * FRICTION_EPS).sqrt();
        let kc = CONTACT_STIFFNESS * m.mass[i] * ih2;
        let w = (mu * st.lam[i] / sm).min(kc);
        let dg = pat.diag[i] as usize * 9;
        let nv = [n.x, n.y, n.z];
        for rr in 0..3 {
            for cc in 0..3 {
                a[dg + 3 * rr + cc] += w * ((if rr == cc { 1.0 } else { 0.0 }) - nv[rr] * nv[cc]);
            }
        }
        sub_grad(b, ci, ut * w);
        st.fw[i] = w;
        st.fg[i] = ut * w;
    }
}
