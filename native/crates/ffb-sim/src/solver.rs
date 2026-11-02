use glam::DVec3;

use crate::collide::{contact_force, deepest, max_penetration, nearest, pair_friction, resolve, Contact, Prepared};
use crate::energy::{Acc, HingeEval, NonLinear};
use crate::model::{Model, KIND_HINGE, KIND_LINK, KIND_TRI};
use crate::{Cloth, Collider, Params, PinPoint, StepStats};

pub const GRAB_STIFFNESS: f64 = 100.0;
pub const AIR_RATE: f64 = 10.0;
pub const APPROACH_CAP: f64 = 0.5;
pub const APPROACH_ZONE: f64 = 8.0;
#[derive(Debug, Clone, Copy)]
pub(crate) struct PinState {
    pub vertex: u32,
    pub prev: DVec3,
    pub target: DVec3,
}

#[derive(Debug, Clone)]
pub struct Solver {
    pub(crate) params: Params,
    pub(crate) model: Model,
    pub(crate) x: Vec<DVec3>,
    pub(crate) xn: Vec<DVec3>,
    pub(crate) y: Vec<DVec3>,
    pub(crate) v: Vec<DVec3>,
    pub(crate) acc: Vec<DVec3>,
    pub(crate) ext: Vec<DVec3>,
    pub(crate) outside: Vec<DVec3>,
    pub(crate) vp: Vec<DVec3>,
    pub(crate) pins: Vec<PinState>,
    pub(crate) pinned: Vec<bool>,
    pub(crate) pin_pos: Vec<DVec3>,
    pub(crate) grab: Option<(u32, DVec3)>,
    pub(crate) colliders: Vec<Prepared>,
    pub(crate) contact: Vec<Option<Contact>>,
    pub(crate) inv_mass: Vec<f64>,
    out: Vec<[f32; 3]>,
    pub(crate) contacts: u32,
    hev: Vec<HingeEval>,
    hst: Vec<(f64, f64)>,
    tst: Vec<TriState>,
    pub(crate) imp: Option<Box<crate::implicit::State>>,
    link_base: Vec<f64>,
}

pub(crate) fn v3(p: [f32; 3]) -> DVec3 {
    DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64)
}

struct Ctx<'a> {
    model: &'a Model,
    xn: &'a [DVec3],
    y: &'a [DVec3],
    grab: Option<(u32, DVec3)>,
    h: f64,
    nl: NonLinear,
    colliders: &'a [Prepared],
    offset: f64,
    mu: f64,
    hev: &'a [HingeEval],
}

#[derive(Debug, Clone, Copy, Default)]
struct TriState {
    fu: DVec3,
    fv: DVec3,
    du: DVec3,
    dv: DVec3,
}

fn moved(model: &Model, tst: &mut [TriState], hev: &[HingeEval], hst: &mut [(f64, f64)], i: usize, delta: DVec3) {
    let s = model.adj_start[i] as usize;
    let e = model.adj_start[i + 1] as usize;
    for a in &model.adj[s..e] {
        let slot = a.slot as usize;
        match a.kind {
            KIND_TRI => {
                let el = &model.tris[a.idx as usize];
                let st = &mut tst[a.idx as usize];
                let du = delta * el.alpha[slot];
                let dv = delta * el.beta[slot];
                st.fu += du;
                st.du += du;
                st.fv += dv;
                st.dv += dv;
            }
            KIND_HINGE => {
                let dd = hev[a.idx as usize].dot_g(slot, delta);
                let h = &mut hst[a.idx as usize];
                h.0 += dd;
                h.1 += dd;
            }
            _ => {}
        }
    }
}

fn local_solve(c: &Ctx, tst: &[TriState], hst: &[(f64, f64)], x: &[DVec3], i: usize) -> DVec3 {
    let m = c.model.mass[i];
    let ih2 = 1.0 / (c.h * c.h);
    let xi = x[i];
    let mut acc = Acc::new(m * ih2);
    acc.g = (xi - c.y[i]) * (m * ih2);
    let s = c.model.adj_start[i] as usize;
    let e = c.model.adj_start[i + 1] as usize;
    let ih = 1.0 / c.h;
    for a in &c.model.adj[s..e] {
        let slot = a.slot as usize;
        match a.kind {
            KIND_TRI => {
                let el = &c.model.tris[a.idx as usize];
                let st = &tst[a.idx as usize];
                el.accumulate(st.fu, st.fv, st.du, st.dv, slot, c.nl, el.kd * ih, &mut acc);
            }
            KIND_HINGE => {
                let el = &c.model.hinges[a.idx as usize];
                let (tl, ds) = hst[a.idx as usize];
                c.hev[a.idx as usize].accumulate(slot, tl, ds, el.kd * ih, &mut acc);
            }
            KIND_LINK => {
                let el = &c.model.links[a.idx as usize];
                let ev = el.eval([x[el.v[0] as usize], x[el.v[1] as usize], x[el.v[2] as usize]]);
                acc.g += ev.grad(slot);
                acc.add_mat(ev.hess(slot, slot));
            }
            _ => {}
        }
    }
    if let Some((gv, target)) = c.grab {
        if gv as usize == i {
            let k = m * ih2 * GRAB_STIFFNESS;
            acc.g += (xi - target) * k;
            acc.add_diag(k);
        }
    }
    if !c.colliders.is_empty() {
        if let Some(ct) = deepest(c.colliders, xi, c.offset) {
            let cf = contact_force(ct, m * ih2, pair_friction(c.mu, c.model.friction[i]), xi - c.xn[i]);
            acc.g += cf.grad;
            acc.add_mat(cf.hess);
        }
    }
    xi + acc.newton()
}

impl Solver {
    pub fn new(cloth: Cloth, params: Params) -> Solver {
        let model = Model::build(&cloth);
        let n = cloth.positions.len();
        let x: Vec<DVec3> = cloth.positions.iter().map(|p| v3(*p)).collect();
        let inv_mass = model.mass.iter().map(|m| 1.0 / m).collect();
        let mut s = Solver {
            params,
            model,
            xn: x.clone(),
            y: x.clone(),
            x,
            v: vec![DVec3::ZERO; n],
            acc: vec![DVec3::ZERO; n],
            ext: vec![DVec3::ZERO; n],
            outside: Vec::new(),
            vp: vec![DVec3::ZERO; n],
            pins: Vec::new(),
            pinned: vec![false; n],
            pin_pos: vec![DVec3::ZERO; n],
            grab: None,
            colliders: Vec::new(),
            contact: vec![None; n],
            inv_mass,
            out: Vec::new(),
            contacts: 0,
            hev: Vec::new(),
            hst: Vec::new(),
            tst: Vec::new(),
            imp: None,
            link_base: Vec::new(),
        };
        s.link_base = s.model.links.iter().map(|l| l.k).collect();
        s.set_pins(cloth.pins);
        s.sync_out();
        s
    }

    pub fn set_colliders(&mut self, colliders: Vec<Collider>) {
        self.colliders = colliders.iter().map(Prepared::from_collider).collect();
    }

    pub fn set_pins(&mut self, pins: Vec<PinPoint>) {
        let old = std::mem::take(&mut self.pins);
        for p in self.pinned.iter_mut() {
            *p = false;
        }
        for p in pins {
            let i = p.vertex as usize;
            if i >= self.x.len() {
                continue;
            }
            let prev = old.iter().find(|o| o.vertex == p.vertex).map_or(self.x[i], |o| o.target);
            self.pinned[i] = true;
            self.pins.push(PinState { vertex: p.vertex, prev, target: v3(p.target) });
        }
        for i in 0..self.x.len() {
            self.inv_mass[i] = if self.pinned[i] { 0.0 } else { 1.0 / self.model.mass[i] };
        }
    }

    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    pub fn params(&self) -> Params {
        self.params
    }

    pub fn positions(&self) -> &[[f32; 3]] {
        &self.out
    }

    pub fn vertex_count(&self) -> usize {
        self.x.len()
    }

    pub fn set_positions(&mut self, p: &[[f32; 3]]) {
        for (i, q) in p.iter().enumerate().take(self.x.len()) {
            self.x[i] = v3(*q);
        }
        self.xn.clone_from(&self.x);
        self.y.clone_from(&self.x);
        for i in 0..self.x.len() {
            self.v[i] = DVec3::ZERO;
            self.acc[i] = DVec3::ZERO;
            self.contact[i] = None;
        }
        for p in self.pins.iter_mut() {
            p.prev = p.target;
        }
        if let Some(st) = self.imp.as_mut() {
            st.reset();
        }
        self.sync_out();
    }

    pub fn push_out(&mut self, margin: f32) -> usize {
        let offset = f64::from(self.params.skin_offset.max(0.0) + margin);
        let mut moved = Vec::with_capacity(self.x.len());
        let mut count = 0;
        for (i, x) in self.x.iter().enumerate() {
            let (p, hit) = resolve(&self.colliders, *x, offset);
            if hit.is_some() && !self.pinned[i] {
                count += 1;
                moved.push([p.x as f32, p.y as f32, p.z as f32]);
            } else {
                moved.push([x.x as f32, x.y as f32, x.z as f32]);
            }
        }
        if count > 0 {
            self.set_positions(&moved);
        }
        count
    }

    pub fn scale_links(&mut self, factor: f64) {
        for (l, k) in self.model.links.iter_mut().zip(&self.link_base) {
            l.k = k * factor;
        }
    }

    pub fn link_count(&self) -> usize {
        self.model.links.len()
    }

    pub fn max_link_gap(&self) -> f32 {
        let mut worst: f64 = 0.0;
        for l in &self.model.links {
            let mut d = DVec3::ZERO;
            for s in 0..l.n {
                d += self.x[l.v[s] as usize] * l.c[s];
            }
            worst = worst.max(d.length() - l.rest);
        }
        worst as f32
    }

    pub fn damp_velocities(&mut self, keep: f64) {
        for v in self.v.iter_mut() {
            *v *= keep;
        }
    }

    pub fn grab(&mut self, vertex: u32, target: Option<[f32; 3]>) {
        self.grab = match target {
            Some(t) if (vertex as usize) < self.x.len() => Some((vertex, v3(t))),
            _ => None,
        };
    }

    pub fn kinetic_energy(&self) -> f32 {
        let mut e = 0.0;
        for i in 0..self.x.len() {
            e += 0.5 * self.model.mass[i] * self.v[i].length_squared();
        }
        e as f32
    }

    pub fn max_penetration(&self) -> f32 {
        max_penetration(&self.colliders, &self.x) as f32
    }

    pub fn max_stretch(&self) -> f32 {
        let mut worst: f64 = 0.0;
        for t in &self.model.tris {
            let p = [self.x[t.v[0] as usize], self.x[t.v[1] as usize], self.x[t.v[2] as usize]];
            let fu = p[0] * t.alpha[0] + p[1] * t.alpha[1] + p[2] * t.alpha[2];
            let fv = p[0] * t.beta[0] + p[1] * t.beta[1] + p[2] * t.beta[2];
            worst = worst.max(fu.length() - 1.0).max(fv.length() - 1.0);
        }
        worst as f32
    }

    pub fn max_compression(&self) -> f32 {
        let mut worst: f64 = 0.0;
        for t in &self.model.tris {
            let p = [self.x[t.v[0] as usize], self.x[t.v[1] as usize], self.x[t.v[2] as usize]];
            let fu = p[0] * t.alpha[0] + p[1] * t.alpha[1] + p[2] * t.alpha[2];
            let fv = p[0] * t.beta[0] + p[1] * t.beta[1] + p[2] * t.beta[2];
            worst = worst.max(1.0 - fu.length()).max(1.0 - fv.length());
        }
        worst as f32
    }

    pub(crate) fn sync_out(&mut self) {
        self.out.clear();
        self.out.extend(self.x.iter().map(|p| [p.x as f32, p.y as f32, p.z as f32]));
    }

    pub(crate) fn nl(&self) -> NonLinear {
        NonLinear { on: self.params.nonlinear, gain: self.params.stiffening as f64 }
    }

    pub(crate) fn substep_time(&self) -> f64 {
        (self.params.time_step.max(1.0e-6) / self.params.substeps.max(1) as f32) as f64
    }

    pub(crate) fn begin_substep(&mut self, h: f64, frac: f64, adaptive: bool) {
        let g = v3(self.params.gravity);
        let n = self.x.len();
        self.xn.clone_from(&self.x);
        for e in self.ext.iter_mut() {
            *e = DVec3::ZERO;
        }
        for c in self.contact.iter_mut() {
            *c = None;
        }
        for &(ti, p) in &self.model.tri_pressure {
            let t = self.model.triangles[ti as usize];
            let area_vec = (self.x[t[1] as usize] - self.x[t[0] as usize]).cross(self.x[t[2] as usize] - self.x[t[0] as usize]);
            let f = area_vec * (p / 6.0);
            for &i in &t {
                self.ext[i as usize] += f;
            }
        }
        for (e, w) in self.ext.iter_mut().zip(&self.outside) {
            *e += *w;
        }
        for p in &self.pins {
            self.pin_pos[p.vertex as usize] = p.prev + (p.target - p.prev) * frac;
        }
        let damp = 1.0 / (1.0 + self.params.air_damping.max(0.0) as f64 * AIR_RATE * h);
        let gl = g.length();
        let gdir = if gl > 0.0 { g / gl } else { DVec3::ZERO };
        for i in 0..n {
            if self.pinned[i] {
                self.x[i] = self.pin_pos[i];
                self.y[i] = self.pin_pos[i];
                self.vp[i] = DVec3::ZERO;
                continue;
            }
            let vp = self.v[i] * damp;
            self.vp[i] = vp;
            let a = g + self.ext[i] / self.model.mass[i];
            self.y[i] = self.xn[i] + vp * h + a * (h * h);
            let init = if adaptive {
                let along = self.acc[i].dot(gdir).clamp(0.0, gl);
                gdir * along
            } else {
                a
            };
            self.x[i] = self.xn[i] + vp * h + init * (h * h);
        }
        if !adaptive {
            self.x.clone_from(&self.y);
        }
    }

    pub(crate) fn project_all(&mut self) {
        let offset = self.params.skin_offset.max(0.0) as f64;
        if self.colliders.is_empty() {
            return;
        }
        for i in 0..self.x.len() {
            if self.pinned[i] {
                continue;
            }
            self.x[i] = resolve(&self.colliders, self.x[i], offset).0;
        }
    }

    pub(crate) fn end_substep(&mut self, h: f64) -> f64 {
        let n = self.x.len();
        let offset = self.params.skin_offset.max(0.0) as f64;
        if self.params.self_collision && n > 0 {
            self.resolve_self_contact();
        }
        let mut pre = vec![DVec3::ZERO; if self.colliders.is_empty() { 0 } else { n }];
        if !self.colliders.is_empty() {
            for i in 0..n {
                if self.pinned[i] {
                    continue;
                }
                let cap = APPROACH_CAP * offset;
                if cap > 0.0 && (self.x[i] - self.xn[i]).length_squared() > cap * cap {
                    let zone = (APPROACH_ZONE * offset).max(20.0);
                    if let Some((sd, n)) = nearest(&self.colliders, self.xn[i], zone) {
                        let inward = (self.xn[i] - self.x[i]).dot(n);
                        if sd < zone && inward > cap {
                            self.x[i] += n * (inward - cap);
                        }
                    }
                }
                if let Some(p) = pre.get_mut(i) {
                    *p = self.x[i];
                }
                let moved = (self.x[i] - self.xn[i]).length();
                if moved > 0.5 * offset.max(0.5) {
                    for col in &self.colliders {
                        if let Some(ct) = col.sweep(self.xn[i], self.x[i], offset) {
                            self.x[i] += ct.normal * ct.depth.max(0.0);
                        }
                    }
                }
                let (p, c) = resolve(&self.colliders, self.x[i], offset);
                self.x[i] = p;
                self.contact[i] = c;
            }
        }
        let mut vmax: f64 = 0.0;
        let mut contacts = 0;
        for i in 0..n {
            let mut nv = (self.x[i] - self.xn[i]) / h;
            if let (Some(c), Some(p)) = (self.contact[i], pre.get(i)) {
                nv = (*p - self.xn[i]) / h;
                let vn = nv.dot(c.normal);
                if vn < 0.0 {
                    nv -= c.normal * vn;
                }
            }
            self.acc[i] = (nv - self.v[i]) / h;
            self.v[i] = nv;
            vmax = vmax.max(nv.length());
            if self.contact[i].is_some() {
                contacts += 1;
            }
        }
        self.contacts = contacts;
        vmax
    }

    pub(crate) fn finish_frame(&mut self) {
        for p in self.pins.iter_mut() {
            p.prev = p.target;
        }
        self.sync_out();
    }

    pub fn step(&mut self) -> StepStats {
        if self.params.method == crate::Method::Implicit {
            return self.step_implicit();
        }
        #[cfg(not(target_arch = "wasm32"))]
        let t0 = std::time::Instant::now();
        let substeps = self.params.substeps.max(1);
        let iters = self.params.iterations.max(1);
        let h = self.substep_time();
        let mut vmax: f64 = 0.0;
        for s in 0..substeps {
            self.begin_substep(h, (s + 1) as f64 / substeps as f64, true);
            self.vbd_iterate(h, iters);
            vmax = self.end_substep(h);
        }
        self.finish_frame();
        #[cfg(not(target_arch = "wasm32"))]
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        #[cfg(target_arch = "wasm32")]
        let ms = 0.0;
        StepStats { ms, iterations: substeps * iters, max_velocity: vmax as f32, contacts: self.contacts }
    }

    fn vbd_iterate(&mut self, h: f64, iters: u32) {
        let offset = self.params.skin_offset.max(0.0) as f64;
        let nl = self.nl();
        let rho = self.params.accel.clamp(0.0, 0.999) as f64;
        let (mut hist1, mut hist2) = if rho > 0.0 { (self.x.clone(), self.x.clone()) } else { (Vec::new(), Vec::new()) };
        let mut omega = 1.0;
        let mu = self.params.friction.max(0.0) as f64;
        let mut hev = std::mem::take(&mut self.hev);
        let mut hst = std::mem::take(&mut self.hst);
        let mut tst = std::mem::take(&mut self.tst);
        hev.resize(self.model.hinges.len(), HingeEval::default());
        hst.resize(self.model.hinges.len(), (0.0, 0.0));
        tst.resize(self.model.tris.len(), TriState::default());
        for it in 0..iters {
            for (k, (e, el)) in hev.iter_mut().zip(&self.model.hinges).enumerate() {
                let q = el.v.map(|v| v as usize);
                *e = el.eval([self.x[q[0]], self.x[q[1]], self.x[q[2]], self.x[q[3]]]);
                hst[k] = (e.dtheta(), e.dot_all([self.x[q[0]] - self.xn[q[0]], self.x[q[1]] - self.xn[q[1]], self.x[q[2]] - self.xn[q[2]], self.x[q[3]] - self.xn[q[3]]]));
            }
            for (st, el) in tst.iter_mut().zip(&self.model.tris) {
                let q = el.v.map(|v| v as usize);
                let p = [self.x[q[0]], self.x[q[1]], self.x[q[2]]];
                let d = [p[0] - self.xn[q[0]], p[1] - self.xn[q[1]], p[2] - self.xn[q[2]]];
                st.fu = p[0] * el.alpha[0] + p[1] * el.alpha[1] + p[2] * el.alpha[2];
                st.fv = p[0] * el.beta[0] + p[1] * el.beta[1] + p[2] * el.beta[2];
                st.du = d[0] * el.alpha[0] + d[1] * el.alpha[1] + d[2] * el.alpha[2];
                st.dv = d[0] * el.beta[0] + d[1] * el.beta[1] + d[2] * el.beta[2];
            }
            let ctx = Ctx { model: &self.model, xn: &self.xn, y: &self.y, grab: self.grab, h, nl, colliders: &self.colliders, offset, mu, hev: &hev };
            for color in &self.model.colors {
                #[cfg(feature = "rayon")]
                {
                    use rayon::prelude::*;
                    let xr = &self.x;
                    let pinned = &self.pinned;
                    let (tr, hr) = (&tst, &hst);
                    let upd: Vec<(u32, DVec3)> = color.par_iter().filter(|&&i| !pinned[i as usize]).map(|&i| (i, local_solve(&ctx, tr, hr, xr, i as usize))).collect();
                    for (i, p) in upd {
                        if p.is_finite() {
                            let iu = i as usize;
                            let delta = p - self.x[iu];
                            self.x[iu] = p;
                            moved(&self.model, &mut tst, &hev, &mut hst, iu, delta);
                        }
                    }
                }
                #[cfg(not(feature = "rayon"))]
                for &iu in color {
                    let i = iu as usize;
                    if self.pinned[i] {
                        continue;
                    }
                    let p = local_solve(&ctx, &tst, &hst, &self.x, i);
                    if p.is_finite() {
                        let delta = p - self.x[i];
                        self.x[i] = p;
                        moved(&self.model, &mut tst, &hev, &mut hst, i, delta);
                    }
                }
            }
            if rho > 0.0 {
                omega = match it {
                    0 => 1.0,
                    1 => 2.0 / (2.0 - rho * rho),
                    _ => 4.0 / (4.0 - rho * rho * omega),
                };
                if it >= 1 {
                    for ((p, h2), pin) in self.x.iter_mut().zip(&hist2).zip(&self.pinned) {
                        if !*pin {
                            *p = (*p - *h2) * omega + *h2;
                        }
                    }
                }
                std::mem::swap(&mut hist1, &mut hist2);
                hist1.clone_from(&self.x);
            }
        }
        self.hev = hev;
        self.hst = hst;
        self.tst = tst;
    }
}
