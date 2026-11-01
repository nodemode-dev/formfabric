use glam::{DMat3, DVec3};

use crate::collide::{contact_force, deepest, pair_friction, Contact};
use crate::energy::NonLinear;
use crate::solver::{Solver, GRAB_STIFFNESS};
use crate::StepStats;

pub const NEWTON_ITERATIONS: usize = 6;
pub const CG_MAX: usize = 2000;
pub const CG_TOLERANCE: f64 = 1.0e-6;

struct Bsr {
    start: Vec<usize>,
    cols: Vec<u32>,
    blocks: Vec<DMat3>,
}

impl Bsr {
    fn pattern(neighbors: &[Vec<u32>]) -> Bsr {
        let n = neighbors.len();
        let mut start = vec![0usize; n + 1];
        let mut cols = Vec::new();
        for i in 0..n {
            let mut row: Vec<u32> = neighbors[i].clone();
            row.push(i as u32);
            row.sort_unstable();
            cols.extend_from_slice(&row);
            start[i + 1] = cols.len();
        }
        let blocks = vec![DMat3::ZERO; cols.len()];
        Bsr { start, cols, blocks }
    }

    fn clear(&mut self) {
        for b in self.blocks.iter_mut() {
            *b = DMat3::ZERO;
        }
    }

    fn add(&mut self, i: u32, j: u32, m: DMat3) {
        let (s, e) = (self.start[i as usize], self.start[i as usize + 1]);
        if let Ok(k) = self.cols[s..e].binary_search(&j) {
            self.blocks[s + k] += m;
        }
    }

    fn mul(&self, v: &[DVec3], out: &mut [DVec3]) {
        for (i, o) in out.iter_mut().enumerate() {
            let mut acc = DVec3::ZERO;
            for k in self.start[i]..self.start[i + 1] {
                acc += self.blocks[k] * v[self.cols[k] as usize];
            }
            *o = acc;
        }
    }

    fn diag(&self, i: usize) -> DMat3 {
        let (s, e) = (self.start[i], self.start[i + 1]);
        match self.cols[s..e].binary_search(&(i as u32)) {
            Ok(k) => self.blocks[s + k],
            Err(_) => DMat3::IDENTITY,
        }
    }
}

fn dot(a: &[DVec3], b: &[DVec3]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x.dot(*y)).sum()
}

fn assemble(s: &Solver, h: f64, nl: NonLinear, a: &mut Bsr, grad: &mut [DVec3], contacts: &mut [Option<Contact>]) {
    let m = &s.model;
    let x = &s.x;
    let ih2 = 1.0 / (h * h);
    a.clear();
    for i in 0..x.len() {
        let w = m.mass[i] * ih2;
        grad[i] = (x[i] - s.y[i]) * w;
        a.add(i as u32, i as u32, DMat3::IDENTITY * w);
    }
    let d: Vec<DVec3> = x.iter().zip(&s.xn).map(|(p, q)| *p - *q).collect();
    for t in &m.tris {
        let ev = t.eval([x[t.v[0] as usize], x[t.v[1] as usize], x[t.v[2] as usize]], nl);
        let r = t.kd / h;
        for si in 0..3 {
            let mut g = ev.grad(si);
            for ti in 0..3 {
                let hm = ev.hess(si, ti);
                g += (hm * d[t.v[ti] as usize]) * r;
                a.add(t.v[si], t.v[ti], hm * (1.0 + r));
            }
            grad[t.v[si] as usize] += g;
        }
    }
    for hg in &m.hinges {
        let ev = hg.eval([x[hg.v[0] as usize], x[hg.v[1] as usize], x[hg.v[2] as usize], x[hg.v[3] as usize]]);
        let r = hg.kd / h;
        for si in 0..4 {
            let mut g = ev.grad(si);
            for ti in 0..4 {
                let hm = ev.hess(si, ti);
                g += (hm * d[hg.v[ti] as usize]) * r;
                a.add(hg.v[si], hg.v[ti], hm * (1.0 + r));
            }
            grad[hg.v[si] as usize] += g;
        }
    }
    for l in &m.links {
        let ev = l.eval([x[l.v[0] as usize], x[l.v[1] as usize], x[l.v[2] as usize]]);
        for si in 0..l.n {
            grad[l.v[si] as usize] += ev.grad(si);
            for ti in 0..l.n {
                a.add(l.v[si], l.v[ti], ev.hess(si, ti));
            }
        }
    }
    if let Some((gv, target)) = s.grab {
        let i = gv as usize;
        let k = m.mass[i] * ih2 * GRAB_STIFFNESS;
        grad[i] += (x[i] - target) * k;
        a.add(gv, gv, DMat3::IDENTITY * k);
    }
    for c in contacts.iter_mut() {
        *c = None;
    }
    if !s.colliders.is_empty() {
        let offset = s.params.skin_offset.max(0.0) as f64;
        let mu_global = s.params.friction.max(0.0) as f64;
        for i in 0..x.len() {
            if s.pinned[i] {
                continue;
            }
            if let Some(c) = deepest(&s.colliders, x[i], offset) {
                let cf = contact_force(c, m.mass[i] * ih2, pair_friction(mu_global, m.friction[i]), d[i]);
                grad[i] += cf.grad;
                a.add(i as u32, i as u32, cf.hess);
                contacts[i] = Some(c);
            }
        }
    }
}

fn cg(a: &Bsr, rhs: &[DVec3], free: &[bool], sol: &mut [DVec3]) -> usize {
    let n = rhs.len();
    let pre: Vec<DMat3> = (0..n).map(|i| if free[i] { let d = a.diag(i); if d.determinant().abs() > 1.0e-30 { d.inverse() } else { DMat3::IDENTITY } } else { DMat3::ZERO }).collect();
    for p in sol.iter_mut() {
        *p = DVec3::ZERO;
    }
    let mut r: Vec<DVec3> = (0..n).map(|i| if free[i] { rhs[i] } else { DVec3::ZERO }).collect();
    let bnorm = dot(&r, &r).sqrt();
    if bnorm < 1.0e-14 {
        return 0;
    }
    let mut z: Vec<DVec3> = (0..n).map(|i| pre[i] * r[i]).collect();
    let mut p = z.clone();
    let mut q = vec![DVec3::ZERO; n];
    let mut rz = dot(&r, &z);
    let mut used = 0;
    for it in 0..CG_MAX {
        a.mul(&p, &mut q);
        for i in 0..n {
            if !free[i] {
                q[i] = DVec3::ZERO;
            }
        }
        let pq = dot(&p, &q);
        if pq.abs() < 1.0e-30 {
            break;
        }
        let alpha = rz / pq;
        for i in 0..n {
            sol[i] += p[i] * alpha;
            r[i] -= q[i] * alpha;
        }
        used = it + 1;
        if dot(&r, &r).sqrt() <= CG_TOLERANCE * bnorm {
            break;
        }
        for i in 0..n {
            z[i] = pre[i] * r[i];
        }
        let rz_new = dot(&r, &z);
        let beta = rz_new / rz;
        rz = rz_new;
        for i in 0..n {
            p[i] = z[i] + p[i] * beta;
        }
    }
    used
}

pub fn reference_step(s: &mut Solver) -> StepStats {
    #[cfg(not(target_arch = "wasm32"))]
    let t0 = std::time::Instant::now();
    let substeps = s.params.substeps.max(1);
    let h = s.substep_time();
    let nl = s.nl();
    let n = s.x.len();
    let mut a = Bsr::pattern(&s.model.neighbors);
    let mut grad = vec![DVec3::ZERO; n];
    let mut rhs = vec![DVec3::ZERO; n];
    let mut dx = vec![DVec3::ZERO; n];
    let mut contacts: Vec<Option<Contact>> = vec![None; n];
    let mut cg_total = 0usize;
    let mut vmax = 0.0;
    for sub in 0..substeps {
        s.begin_substep(h, (sub + 1) as f64 / substeps as f64, false);
        s.project_all();
        let free: Vec<bool> = s.pinned.iter().map(|p| !p).collect();
        for _ in 0..NEWTON_ITERATIONS {
            assemble(s, h, nl, &mut a, &mut grad, &mut contacts);
            for i in 0..n {
                rhs[i] = -grad[i];
            }
            cg_total += cg(&a, &rhs, &free, &mut dx);
            for (p, d) in s.x.iter_mut().zip(&dx) {
                *p += *d;
            }
            let step = dx.iter().map(|d| d.length()).fold(0.0, f64::max);
            if step < 1.0e-3 {
                break;
            }
        }
        vmax = s.end_substep(h);
    }
    s.finish_frame();
    #[cfg(not(target_arch = "wasm32"))]
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    #[cfg(target_arch = "wasm32")]
    let ms = 0.0;
    StepStats { ms, iterations: cg_total as u32, max_velocity: vmax as f32, contacts: s.contacts }
}

pub fn compare_positions(a: &Solver, b: &Solver) -> (f32, f32) {
    let n = a.x.len().min(b.x.len());
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut sum = 0.0;
    let mut worst: f64 = 0.0;
    for i in 0..n {
        let d = (a.x[i] - b.x[i]).length();
        sum += d;
        worst = worst.max(d);
    }
    ((sum / n as f64) as f32, worst as f32)
}
