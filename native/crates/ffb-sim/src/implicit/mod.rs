mod assemble;
pub mod coarse;
pub mod matrix;
pub mod pcg;

use glam::DVec3;

use crate::collide::deepest;
use crate::model::Model;
use crate::solver::Solver;
use crate::StepStats;

use matrix::{row_apply, Pattern};
use pcg::{Filter, Work, FREE, PINNED, PLANE};

const MAX_ROUNDS: usize = 3;
const RELEASE_FRACTION: f64 = 1.0e-2;
const ABS_FORCE: f64 = 1.0e-3;
const WARM_START: f64 = 0.5;
const FIRST_LOOSE: f64 = 5.0;
const COARSE_LEVELS: usize = 2;
const COARSE_MAX_CONSTRAINED: usize = 12;

#[derive(Debug, Clone)]
pub struct State {
    pub(crate) pat: Pattern,
    pub(crate) a: Vec<f64>,
    pub(crate) b: Vec<f64>,
    pub(crate) dx: Vec<f64>,
    pub(crate) kind: Vec<u8>,
    pub(crate) nrm: Vec<DVec3>,
    pub(crate) lam: Vec<f64>,
    pub(crate) fw: Vec<f64>,
    pub(crate) fg: Vec<DVec3>,
    pub(crate) was: Vec<bool>,
    pub(crate) cons: Vec<u32>,
    mass_norm: f64,
    work: Work,
    coarse: Option<coarse::Coarse>,
    pub prof: [f64; 4],
}

#[derive(Clone, Copy)]
struct Clock {
    #[cfg(not(target_arch = "wasm32"))]
    at: std::time::Instant,
}

impl Clock {
    fn now() -> Clock {
        Clock {
            #[cfg(not(target_arch = "wasm32"))]
            at: std::time::Instant::now(),
        }
    }

    fn lap(&mut self) -> f64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let t = std::time::Instant::now();
            let d = t.duration_since(self.at).as_secs_f64() * 1000.0;
            self.at = t;
            d
        }
        #[cfg(target_arch = "wasm32")]
        0.0
    }
}

impl State {
    pub fn new(model: &Model) -> State {
        let n = model.n;
        let pat = Pattern::build(model);
        let nnz = pat.nnz();
        let coarse = if n >= 64 { Some(coarse::Coarse::build(&pat, COARSE_LEVELS)) } else { None };
        State {
            pat,
            a: vec![0.0; 9 * nnz],
            b: vec![0.0; 3 * n],
            dx: vec![0.0; 3 * n],
            kind: vec![FREE; n],
            nrm: vec![DVec3::Y; n],
            lam: vec![0.0; n],
            fw: vec![0.0; n],
            fg: vec![DVec3::ZERO; n],
            was: vec![false; n],
            cons: Vec::with_capacity(n),
            mass_norm: model.mass.iter().map(|m| m * m).sum::<f64>().sqrt(),
            work: Work::new(n),
            coarse,
            prof: [0.0; 4],
        }
    }

    pub fn reset(&mut self) {
        self.was.fill(false);
        self.lam.fill(0.0);
    }

    pub fn nonzero_blocks(&self) -> usize {
        self.pat.nnz()
    }

    fn release(&mut self, i: usize) {
        self.kind[i] = FREE;
        self.was[i] = false;
        if self.fw[i] != 0.0 {
            let n = self.nrm[i];
            let nv = [n.x, n.y, n.z];
            let d = self.pat.diag[i] as usize * 9;
            for r in 0..3 {
                for c in 0..3 {
                    self.a[d + 3 * r + c] -= self.fw[i] * ((if r == c { 1.0 } else { 0.0 }) - nv[r] * nv[c]);
                }
            }
            self.b[3 * i] += self.fg[i].x;
            self.b[3 * i + 1] += self.fg[i].y;
            self.b[3 * i + 2] += self.fg[i].z;
            self.fw[i] = 0.0;
            self.fg[i] = DVec3::ZERO;
        }
    }
}

impl Solver {
    pub(crate) fn step_implicit(&mut self) -> StepStats {
        #[cfg(not(target_arch = "wasm32"))]
        let t0 = std::time::Instant::now();
        let mut st = match self.imp.take() {
            Some(s) if s.pat.n == self.x.len() => s,
            _ => Box::new(State::new(&self.model)),
        };
        let substeps = self.params.substeps.max(1);
        let h = self.substep_time();
        let mut its = 0usize;
        let mut vmax = 0.0;
        for sub in 0..substeps {
            let mut clock = Clock::now();
            self.begin_substep(h, (sub + 1) as f64 / substeps as f64, false);
            st.prof[2] += clock.lap();
            its += self.implicit_substep(&mut st, h);
            clock.lap();
            vmax = self.end_substep(h);
            st.prof[3] += clock.lap();
            for i in 0..self.x.len() {
                st.was[i] = st.kind[i] == PLANE;
            }
        }
        self.finish_frame();
        self.imp = Some(st);
        #[cfg(not(target_arch = "wasm32"))]
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        #[cfg(target_arch = "wasm32")]
        let ms = 0.0;
        StepStats { ms, iterations: its as u32, max_velocity: vmax as f32, contacts: self.contacts }
    }

    pub fn implicit_profile(&self) -> [f64; 4] {
        self.imp.as_ref().map_or([0.0; 4], |s| s.prof)
    }

    fn implicit_substep(&mut self, st: &mut State, h: f64) -> usize {
        let n = self.x.len();
        let ih2 = 1.0 / (h * h);
        let offset = self.params.skin_offset.max(0.0) as f64;
        let nl = self.nl();
        let tol = self.params.cg_tolerance.max(1.0e-9) as f64;
        let max_it = self.params.cg_iterations.max(1) as usize;
        let glen = DVec3::new(self.params.gravity[0] as f64, self.params.gravity[1] as f64, self.params.gravity[2] as f64).length();
        let abs = ABS_FORCE * glen * st.mass_norm;
        st.cons.clear();
        for i in 0..n {
            st.kind[i] = FREE;
            if self.pinned[i] {
                st.kind[i] = PINNED;
                st.cons.push(i as u32);
                continue;
            }
            self.x[i] = self.xn[i] + self.vp[i] * h;
            if self.colliders.is_empty() {
                continue;
            }
            if let Some(c) = deepest(&self.colliders, self.y[i], offset) {
                st.kind[i] = PLANE;
                st.nrm[i] = c.normal;
                if !st.was[i] {
                    st.lam[i] = self.model.mass[i] * c.depth * ih2;
                }
                let q = self.y[i] + c.normal * c.depth;
                let xi = self.x[i];
                self.x[i] = xi - c.normal * c.normal.dot(xi - q);
                st.cons.push(i as u32);
            }
        }
        let mut its = 0usize;
        let newton = self.params.newton.max(1);
        let loose = FIRST_LOOSE;
        let mut floor = abs;
        let warm = WARM_START;
        for k in 0..self.params.newton.max(1) {
            let tk = if k + 1 < newton { tol * loose } else { tol };
            let mut clock = Clock::now();
            assemble::assemble(self, st, h, nl, true);
            st.prof[0] += clock.lap();
            st.dx.fill(0.0);
            if k == 0 && warm > 0.0 {
                for i in 0..n {
                    if st.kind[i] != PINNED {
                        let g = self.acc[i] * (warm * h * h);
                        st.dx[3 * i] = g.x;
                        st.dx[3 * i + 1] = g.y;
                        st.dx[3 * i + 2] = g.z;
                    }
                }
            }
            let first = Self::run_pcg(st, tk, floor, max_it);
            its += first;
            if k == 0 {
                floor = floor.max(tol * st.work.bnorm);
            }
            for _ in 0..MAX_ROUNDS {
                let mut freed = false;
                for k in 0..st.cons.len() {
                    let i = st.cons[k] as usize;
                    if st.kind[i] != PLANE {
                        continue;
                    }
                    let row = row_apply(&st.pat, &st.a, &st.dx, i);
                    let nr = st.nrm[i];
                    let lam = nr.x * (row[0] - st.b[3 * i]) + nr.y * (row[1] - st.b[3 * i + 1]) + nr.z * (row[2] - st.b[3 * i + 2]);
                    if lam < -RELEASE_FRACTION * self.model.mass[i] * glen {
                        st.release(i);
                        freed = true;
                    } else {
                        st.lam[i] = lam.max(0.0);
                    }
                }
                if !freed {
                    break;
                }
                let kind = &st.kind;
                st.cons.retain(|&i| kind[i as usize] != FREE);
                its += Self::run_pcg(st, tk, floor, max_it);
            }
            for i in 0..n {
                self.x[i] += DVec3::new(st.dx[3 * i], st.dx[3 * i + 1], st.dx[3 * i + 2]);
            }
            if first == 0 {
                break;
            }
        }
        if self.x.iter().any(|p| !p.is_finite()) {
            self.x.clone_from(&self.xn);
        }
        its
    }

    fn run_pcg(st: &mut State, tol: f64, abs: f64, max_it: usize) -> usize {
        let mut clock = Clock::now();
        let State { pat, a, b, dx, kind, nrm, cons, work, coarse, .. } = &mut *st;
        let f = Filter { kind, nrm, cons };
        let use_coarse = cons.len() * COARSE_MAX_CONSTRAINED < pat.n;
        let used = pcg::solve(pat, a, &f, b, dx, work, tol, abs, max_it, coarse.as_mut().filter(|_| use_coarse));
        st.prof[1] += clock.lap();
        used
    }
}
