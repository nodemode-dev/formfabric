use glam::DVec3;

use super::draft::{d2, Draft, Kind, PanelDraft, P2};
use super::info::{add, at_arc, len, lerp, mul, sub, Info, V, RING_N};

const SUPPORT: f64 = 90.0;
const MIN_D: f64 = 7.0;
const ROOT_AXIAL: f64 = 6.0;

struct Loft {
    sign: f64,
    table: Vec<V>,
    centre: V,
    a: V,
    b: V,
    radius: f64,
    cap_x: Vec<f64>,
    cap_y: Vec<f64>,
    cap_f: Vec<f64>,
    bw: f64,
    hw: f64,
    ls: f64,
    pub cuff: f64,
}

pub struct Fitter<'a> {
    info: &'a Info,
    draft: &'a Draft,
    corr: Vec<Vec<(P2, V)>>,
    lofts: Vec<Option<Loft>>,
}

fn at_frac(poly: &[P2], tau: f64) -> P2 {
    let total: f64 = poly.windows(2).map(|w| d2(w[0], w[1])).sum();
    let mut left = tau.clamp(0.0, 1.0) * total;
    for w in poly.windows(2) {
        let l = d2(w[0], w[1]);
        if left <= l || l < 1e-12 {
            let t = if l < 1e-12 { 0.0 } else { left / l };
            return [w[0][0] + (w[1][0] - w[0][0]) * t, w[0][1] + (w[1][1] - w[0][1]) * t];
        }
        left -= l;
    }
    poly[poly.len() - 1]
}

fn interp_table(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    for i in 1..xs.len() {
        if x <= xs[i] {
            let t = (x - xs[i - 1]) / (xs[i] - xs[i - 1]).max(1e-12);
            return ys[i - 1] + (ys[i] - ys[i - 1]) * t;
        }
    }
    ys[ys.len() - 1]
}

impl<'a> Fitter<'a> {
    pub fn new(info: &'a Info, draft: &'a Draft) -> Fitter<'a> {
        let n = draft.panels.len();
        let mut f = Fitter { info, draft, corr: vec![Vec::new(); n], lofts: (0..n).map(|_| None).collect() };
        f.build_corrections();
        for i in 0..n {
            if draft.panels[i].kind == Kind::Sleeve {
                f.lofts[i] = f.build_loft(i);
            }
        }
        f
    }

    pub fn cuff_axial(&self, pi: usize) -> Option<f64> {
        self.lofts[pi].as_ref().map(|l| l.cuff)
    }

    fn d_at(&self, p: &PanelDraft, y: f64) -> f64 {
        let table = &p.side_sum;
        if table.is_empty() {
            return MIN_D;
        }
        let ys: Vec<f64> = table.iter().map(|t| t.0).collect();
        let at = |yy: f64| -> f64 {
            let sum = interp_table(&ys, &table.iter().map(|t| t.1).collect::<Vec<_>>(), yy);
            let y_w = p.y0 + yy;
            let ring_y = match p.hip_clamp {
                Some(c) if y_w < c => c,
                _ => y_w,
            };
            let per = self.info.ring_perimeter(ring_y);
            ((2.0 * sum - per) / std::f64::consts::TAU).clamp(MIN_D, 260.0)
        };
        if y <= 0.0 {
            at(y)
        } else {
            let d0 = at(0.0);
            let k = (y / p.ad.max(1.0)).clamp(0.0, 1.0);
            d0 + (p.d_top - d0) * k
        }
    }

    fn base(&self, pi: usize, q: P2) -> V {
        let p = &self.draft.panels[pi];
        let y_w = if q[1] > 0.0 && p.march { self.info.march_up(p.y0, q[1], q[0], p.back, &|v| self.d_at(p, v)) } else { p.y0 + q[1] };
        let ring_y = match p.hip_clamp {
            Some(c) if y_w < c => c,
            _ => y_w,
        };
        let d = self.d_at(p, q[1]);
        let pts = self.info.ring_pts(ring_y, d);
        let start = if p.back { RING_N / 2 } else { 0 };
        let a = at_arc(&pts, start, q[0]);
        [a[0], y_w, a[1]]
    }

    #[allow(clippy::type_complexity)]
    fn build_circles(&mut self) {
        let d = self.draft;
        let sets: [(usize, bool, [(&str, f64); 2]); 2] = [(0, true, [("armhole_r", 1.0), ("armhole_l", -1.0)]), (1, false, [("armhole_l", 1.0), ("armhole_r", -1.0)])];
        for (pi, front, runs) in sets {
            for (run, sign) in runs {
                let Some(arm) = self.info.arms.iter().find(|a| a.sign == sign) else { continue };
                let mut poly = d.panels[pi].run_poly(run, None);
                let rises = poly.first().map(|p| p[1]) < poly.last().map(|p| p[1]);
                if rises != front {
                    poly.reverse();
                }
                for k in 0..=20 {
                    let t = k as f64 / 20.0;
                    let q = at_frac(&poly, t);
                    let g = if front { 0.5 * t } else { 0.5 + 0.5 * t };
                    let target = arm.circle(g, arm.root_radius(), ROOT_AXIAL);
                    let base = self.base(pi, q);
                    self.corr[pi].push((q, sub(target, base)));
                }
            }
        }
        let shoulders: [(&str, &str, f64); 2] = [("shoulder_r", "shoulder_l", 1.0), ("shoulder_l", "shoulder_r", -1.0)];
        for (fr, br, sign) in shoulders {
            let Some(arm) = self.info.arms.iter().find(|a| a.sign == sign) else { continue };
            let orient = |mut poly: Vec<P2>| -> Vec<P2> {
                if poly.first().map(|p| p[0].abs()) < poly.last().map(|p| p[0].abs()) {
                    poly.reverse();
                }
                poly
            };
            let pf = orient(d.panels[0].run_poly(fr, None));
            let pb = orient(d.panels[1].run_poly(br, None));
            if pf.len() < 2 || pb.len() < 2 {
                continue;
            }
            let np_f = pf[pf.len() - 1];
            let np_b = pb[pb.len() - 1];
            let np_mid = lerp(self.base(0, np_f), self.base(1, np_b), 0.5);
            let sp = arm.circle(0.5, arm.root_radius(), ROOT_AXIAL);
            for (pi, poly) in [(0usize, &pf), (1usize, &pb)] {
                for k in 0..=12 {
                    let t = k as f64 / 12.0;
                    let q = at_frac(poly, t);
                    let target = lerp(sp, np_mid, t);
                    let base = self.base(pi, q);
                    self.corr[pi].push((q, sub(target, base)));
                }
            }
        }
    }

    fn build_corrections(&mut self) {
        if self.draft.circles {
            self.build_circles();
        }
        for s in &self.draft.seams {
            if !s.fix {
                continue;
            }
            let poly = |refs: &[super::draft::SpanRef]| -> Vec<P2> {
                let mut out: Vec<P2> = Vec::new();
                for r in refs {
                    let pl = self.draft.panels[r.panel].run_poly(&r.run, r.t);
                    if out.is_empty() {
                        out = pl;
                    } else {
                        out.extend(pl.into_iter().skip(1));
                    }
                }
                out
            };
            let (pa, pb) = (poly(&s.a), poly(&s.b));
            if pa.len() < 2 || pb.len() < 2 {
                continue;
            }
            let (ia, ib) = (s.a[0].panel, s.b[0].panel);
            let n = 16;
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let qa = at_frac(&pa, t);
                let qb = at_frac(&pb, 1.0 - t);
                let (ma, mb) = (self.base(ia, qa), self.base(ib, qb));
                let target = lerp(ma, mb, 0.5);
                self.corr[ia].push((qa, sub(target, ma)));
                self.corr[ib].push((qb, sub(target, mb)));
            }
        }
    }

    fn displacement(&self, pi: usize, q: P2) -> V {
        let list = &self.corr[pi];
        let mut num = [0.0; 3];
        let mut den = 0.0;
        let mut dmin = f64::MAX;
        for (c, dv) in list {
            let d = d2(*c, q);
            dmin = dmin.min(d);
            if d >= SUPPORT {
                continue;
            }
            let w = ((SUPPORT - d) / (SUPPORT * d.max(1.0))).powi(2);
            num = add(num, mul(*dv, w));
            den += w;
        }
        if den <= 0.0 || dmin >= SUPPORT {
            return [0.0; 3];
        }
        let fall = (1.0 - dmin / SUPPORT).powi(2);
        mul(num, fall / den)
    }

    fn push_out(&self, v: V, extra: f64) -> V {
        let (p, _) = ffb_sim::collide::resolve(std::slice::from_ref(&self.info.prep), DVec3::new(v[0], v[1], v[2]), self.info.skin + extra);
        [p.x, p.y, p.z]
    }

    fn torso(&self, pi: usize, q: P2) -> V {
        let m = add(self.base(pi, q), self.displacement(pi, q));
        self.push_out(m, 1.5)
    }

    fn build_loft(&self, pi: usize) -> Option<Loft> {
        let p = &self.draft.panels[pi];
        let sl = p.sleeve.as_ref()?;
        let arm = self.info.arms.iter().find(|a| a.sign == sl.sign)?;
        let mut cap_x = Vec::new();
        let mut cap_y = Vec::new();
        let mut cap_f = Vec::new();
        let total: f64 = sl.cap.windows(2).map(|w| d2(w[0], w[1])).sum();
        let mut acc = 0.0;
        for (i, c) in sl.cap.iter().enumerate() {
            if i > 0 {
                acc += d2(sl.cap[i - 1], *c);
            }
            cap_x.push(c[0]);
            cap_y.push(c[1]);
            cap_f.push(acc / total);
        }
        let n = 160;
        let mut table = Vec::with_capacity(n + 1);
        let (fi, fr) = &sl.front;
        let (bi, br) = &sl.back;
        let mut front = self.draft.panels[*fi].run_poly(fr, None);
        if front.first().map(|p| p[1]) > front.last().map(|p| p[1]) {
            front.reverse();
        }
        let mut back = self.draft.panels[*bi].run_poly(br, None);
        if back.first().map(|p| p[1]) < back.last().map(|p| p[1]) {
            back.reverse();
        }
        for k in 0..=n {
            let f = k as f64 / n as f64;
            if self.draft.circles {
                table.push(arm.circle(f, arm.root_radius(), ROOT_AXIAL));
            } else if f <= 0.5 {
                table.push(self.torso(*fi, at_frac(&front, f / 0.5)));
            } else {
                table.push(self.torso(*bi, at_frac(&back, (f - 0.5) / 0.5)));
            }
        }
        let (down, front_dir) = (arm.down, arm.front);
        let radius = sl.hw / std::f64::consts::TAU;
        let mut best = (f64::MAX, 100.0);
        let mut t = 60.0;
        while t < arm.length + 120.0 {
            let c = add(arm.shoulder, mul(arm.axis, t));
            let mut err = 0.0;
            for (k, l) in table.iter().enumerate().step_by(4) {
                let f = k as f64 / n as f64;
                let th = std::f64::consts::TAU * f;
                let r = add(c, add(mul(down, radius * th.cos()), mul(front_dir, radius * th.sin())));
                let xn = interp_table(&cap_f, &cap_x, f);
                let pat = interp_table(&cap_x, &cap_y, xn) + sl.ls;
                err += (len(sub(*l, r)) - pat).powi(2);
            }
            if err < best.0 {
                best = (err, t);
            }
            t += 2.0;
        }
        let centre = add(arm.shoulder, mul(arm.axis, best.1));
        Some(Loft { sign: sl.sign, table, centre, a: down, b: front_dir, radius, cap_x, cap_y, cap_f, bw: sl.bw, hw: sl.hw, ls: sl.ls, cuff: best.1 })
    }

    fn sleeve(&self, pi: usize, q: P2) -> V {
        let Some(l) = self.lofts[pi].as_ref() else { return [0.0; 3] };
        let x = if l.sign < 0.0 { l.bw - q[0] } else { q[0] };
        let y = q[1];
        let (xl, xr) = if y >= 0.0 {
            (0.0, l.bw)
        } else {
            let k = (-y / l.ls).clamp(0.0, 1.0);
            let xl = (l.bw - l.hw) * 0.5 * k;
            (xl, l.bw - xl)
        };
        let fx = ((x - xl) / (xr - xl).max(1e-6)).clamp(0.0, 1.0);
        let xn = fx * l.bw;
        let f = interp_table(&l.cap_x, &l.cap_f, xn);
        let ycap = interp_table(&l.cap_x, &l.cap_y, xn);
        let t = ((ycap - y) / (ycap + l.ls).max(1e-6)).clamp(0.0, 1.0);
        let n = l.table.len() - 1;
        let fi = f.clamp(0.0, 1.0) * n as f64;
        let i = (fi.floor() as usize).min(n - 1);
        let w = fi - i as f64;
        let top = lerp(l.table[i], l.table[i + 1], w);
        let th = std::f64::consts::TAU * f;
        let ring = add(l.centre, add(mul(l.a, l.radius * th.cos()), mul(l.b, l.radius * th.sin())));
        self.push_out(lerp(top, ring, t), 1.5)
    }

    pub fn point(&self, pi: usize, q: P2) -> V {
        match self.draft.panels[pi].kind {
            Kind::Sleeve => self.sleeve(pi, q),
            _ => self.torso(pi, q),
        }
    }
}
