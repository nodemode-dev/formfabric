use serde_json::{json, Value};

use super::info::{arc_to_x, Info, RING_N};

pub type P2 = [f64; 2];

pub fn d2(a: P2, b: P2) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

#[derive(Clone)]
pub struct DPt {
    pub p: P2,
    pub h_in: Option<P2>,
    pub h_out: Option<P2>,
    pub smooth: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bodice,
    Sleeve,
    Band,
    Skirt,
}

#[derive(Clone)]
pub struct Sleeve {
    pub sign: f64,
    pub bw: f64,
    pub hw: f64,
    pub ls: f64,
    pub cap: Vec<P2>,
    pub front: (usize, String),
    pub back: (usize, String),
}

#[derive(Clone)]
pub struct PanelDraft {
    pub name: String,
    pub kind: Kind,
    pub back: bool,
    pub pts: Vec<DPt>,
    pub runs: Vec<(String, usize, usize)>,
    pub y0: f64,
    pub side_sum: Vec<(f64, f64)>,
    pub hip_clamp: Option<f64>,
    pub ad: f64,
    pub d_top: f64,
    pub sleeve: Option<Sleeve>,
    pub pd: f64,
    pub notches: Vec<(String, f64)>,
    pub march: bool,
}

#[derive(Clone)]
pub struct SpanRef {
    pub panel: usize,
    pub run: String,
    pub t: Option<(f64, f64)>,
}

#[derive(Clone)]
pub struct SeamDraft {
    pub name: String,
    pub a: Vec<SpanRef>,
    pub b: Vec<SpanRef>,
    pub fix: bool,
}

pub struct Draft {
    pub title: String,
    pub panels: Vec<PanelDraft>,
    pub seams: Vec<SeamDraft>,
    pub report: Value,
    pub circles: bool,
}

fn span(panel: usize, run: &str) -> SpanRef {
    SpanRef { panel, run: run.to_string(), t: None }
}

fn seam(name: &str, a: Vec<SpanRef>, b: Vec<SpanRef>, fix: bool) -> SeamDraft {
    SeamDraft { name: name.to_string(), a, b, fix }
}

pub fn bez(p: [P2; 4], t: f64) -> P2 {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [a * p[0][0] + b * p[1][0] + c * p[2][0] + d * p[3][0], a * p[0][1] + b * p[1][1] + c * p[2][1] + d * p[3][1]]
}

pub fn bez_poly(p: [P2; 4], n: usize) -> Vec<P2> {
    (0..=n).map(|k| bez(p, k as f64 / n as f64)).collect()
}

pub fn poly_len(pts: &[P2]) -> f64 {
    pts.windows(2).map(|w| d2(w[0], w[1])).sum()
}

#[derive(Default)]
struct Builder {
    pts: Vec<DPt>,
    runs: Vec<(String, usize, usize)>,
}

impl Builder {
    fn start(p: P2) -> Builder {
        Builder { pts: vec![DPt { p, h_in: None, h_out: None, smooth: false }], runs: Vec::new() }
    }

    fn line_to(&mut self, name: &str, p: P2) {
        let first = self.pts.len() - 1;
        self.pts.push(DPt { p, h_in: None, h_out: None, smooth: false });
        self.runs.push((name.to_string(), first, 1));
    }

    fn close_line(&mut self, name: &str) {
        self.runs.push((name.to_string(), self.pts.len() - 1, 1));
    }

    fn curve(&mut self, name: &str, samples: &[P2], closing: bool) {
        let n = samples.len() - 1;
        let first = self.pts.len() - 1;
        let tan = |i: usize| -> P2 {
            if i == 0 {
                [samples[1][0] - samples[0][0], samples[1][1] - samples[0][1]]
            } else if i == n {
                [samples[n][0] - samples[n - 1][0], samples[n][1] - samples[n - 1][1]]
            } else {
                [(samples[i + 1][0] - samples[i - 1][0]) * 0.5, (samples[i + 1][1] - samples[i - 1][1]) * 0.5]
            }
        };
        let t0 = tan(0);
        let last = self.pts.len() - 1;
        self.pts[last].h_out = Some([t0[0] / 3.0, t0[1] / 3.0]);
        #[allow(clippy::needless_range_loop)]
        for i in 1..=n {
            let t = tan(i);
            let h_in = Some([-t[0] / 3.0, -t[1] / 3.0]);
            if i == n && closing {
                self.pts[0].h_in = h_in;
            } else {
                let interior = i < n;
                self.pts.push(DPt { p: samples[i], h_in, h_out: if interior { Some([t[0] / 3.0, t[1] / 3.0]) } else { None }, smooth: interior });
            }
        }
        self.runs.push((name.to_string(), first, n));
    }

    fn finish(self) -> (Vec<DPt>, Vec<(String, usize, usize)>) {
        (self.pts, self.runs)
    }
}

pub fn seg_bez(pts: &[DPt], i: usize) -> [P2; 4] {
    let a = &pts[i];
    let b = &pts[(i + 1) % pts.len()];
    let h0 = a.h_out.unwrap_or([0.0, 0.0]);
    let h1 = b.h_in.unwrap_or([0.0, 0.0]);
    [a.p, [a.p[0] + h0[0], a.p[1] + h0[1]], [b.p[0] + h1[0], b.p[1] + h1[1]], b.p]
}

impl PanelDraft {
    pub fn run(&self, name: &str) -> Option<(usize, usize)> {
        self.runs.iter().find(|r| r.0 == name).map(|r| (r.1, r.2))
    }

    pub fn run_poly(&self, name: &str, t: Option<(f64, f64)>) -> Vec<P2> {
        let Some((first, count)) = self.run(name) else { return Vec::new() };
        let mut out: Vec<P2> = Vec::new();
        for s in first..first + count {
            let b = seg_bez(&self.pts, s);
            let (t0, t1) = match t {
                Some(x) if count == 1 => x,
                _ => (0.0, 1.0),
            };
            let n = 10;
            for k in 0..=n {
                if k == 0 && !out.is_empty() {
                    continue;
                }
                out.push(bez(b, t0 + (t1 - t0) * k as f64 / n as f64));
            }
        }
        out
    }

    pub fn json_points(&self) -> Value {
        Value::Array(
            self.pts
                .iter()
                .map(|q| {
                    let mut v = json!({"p": q.p, "kind": if q.smooth { "smooth" } else { "corner" }});
                    if let Some(h) = q.h_in {
                        v["h_in"] = json!(h);
                    }
                    if let Some(h) = q.h_out {
                        v["h_out"] = json!(h);
                    }
                    v
                })
                .collect(),
        )
    }

    fn mirror_x(&mut self, width: f64) {
        let n = self.pts.len();
        let mut m: Vec<DPt> = self
            .pts
            .iter()
            .map(|q| DPt { p: [width - q.p[0], q.p[1]], h_in: q.h_in.map(|h| [-h[0], h[1]]), h_out: q.h_out.map(|h| [-h[0], h[1]]), smooth: q.smooth })
            .collect();
        m.reverse();
        for q in m.iter_mut() {
            std::mem::swap(&mut q.h_in, &mut q.h_out);
        }
        self.pts = m;
        for r in self.runs.iter_mut() {
            r.1 = (2 * n - 1 - r.1 - r.2) % n;
        }
    }
}

fn arm_ring_arcs(info: &Info, y: f64, d: f64, x: f64) -> (f64, f64) {
    let pts = info.ring_pts(y, d);
    let xm = pts.iter().map(|p| p[0]).fold(f64::MIN, f64::max) - 1.0;
    let x = x.min(xm);
    (arc_to_x(&pts, 0, 1, x), arc_to_x(&pts, RING_N / 2, -1, x))
}

struct Bod {
    side: Vec<P2>,
    ad_sp: f64,
    ad_np: f64,
    xsp: f64,
    xnp: f64,
    drop: f64,
    arm_in: f64,
    arm_lo: f64,
    neck_steps: usize,
}

fn armhole_curve(b: &Bod, ua: P2, sp: P2) -> Vec<P2> {
    let ad = b.ad_sp;
    let dx = ua[0] - sp[0];
    let p1 = [sp[0] - b.arm_in, ad * 0.6];
    let p2 = [sp[0] + 0.50 * dx, ua[1] + b.arm_lo * ad];
    bez_poly([sp, p1, p2, ua], 6)
}

#[allow(clippy::type_complexity)]
fn bodice(b: &Bod) -> (Vec<DPt>, Vec<(String, usize, usize)>, Vec<P2>) {
    let last = *b.side.last().unwrap_or(&[200.0, -300.0]);
    let ua = b.side[0];
    let sp = [b.xsp, b.ad_sp];
    let np = [b.xnp, b.ad_np];
    let mut bd = Builder::start([-last[0], last[1]]);
    bd.line_to("hem", [last[0], last[1]]);
    let right: Vec<P2> = b.side.iter().rev().copied().collect();
    if right.len() == 2 {
        bd.line_to("side_r", right[1]);
    } else {
        bd.curve("side_r", &right, false);
    }
    let ah = armhole_curve(b, ua, sp);
    let ah_r: Vec<P2> = ah.iter().rev().copied().collect();
    bd.curve("armhole_r", &ah_r, false);
    bd.line_to("shoulder_r", np);
    let mut neck = Vec::new();
    for k in 0..=b.neck_steps {
        let a = std::f64::consts::PI * k as f64 / b.neck_steps as f64;
        neck.push([np[0] * a.cos(), b.ad_np - b.drop * a.sin()]);
    }
    bd.curve("neck", &neck, false);
    bd.line_to("shoulder_l", [-sp[0], sp[1]]);
    let ah_l: Vec<P2> = ah.iter().map(|p| [-p[0], p[1]]).collect();
    bd.curve("armhole_l", &ah_l, false);
    let left: Vec<P2> = b.side.iter().map(|p| [-p[0], p[1]]).collect();
    if left.len() == 2 {
        bd.close_line("side_l");
    } else {
        bd.curve("side_l", &left, true);
    }
    let (pts, runs) = bd.finish();
    (pts, runs, ah)
}

fn side_sum(front: &[P2], back: &[P2]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for p in front.iter().rev() {
        let xb = interp(back, p[1]);
        out.push((p[1], p[0] + xb));
    }
    out
}

fn interp(poly: &[P2], y: f64) -> f64 {
    let mut pts: Vec<P2> = poly.to_vec();
    pts.sort_by(|a, b| a[1].total_cmp(&b[1]));
    if y <= pts[0][1] {
        return pts[0][0];
    }
    for w in pts.windows(2) {
        if y <= w[1][1] {
            let t = (y - w[0][1]) / (w[1][1] - w[0][1]).max(1e-9);
            return w[0][0] + (w[1][0] - w[0][0]) * t;
        }
    }
    pts[pts.len() - 1][0]
}

fn shoulder_dims(info: &Info, y0: f64, strap: Option<f64>, d_top: f64) -> (f64, f64, f64, f64, f64, f64) {
    let sw = info.measure("shoulder_width");
    let x_np = info.neck_r + 14.0;
    let y_np = info.y_neck + 6.0;
    let (x_sp, y_sp) = match strap {
        None => (sw * 0.5 + 8.0, info.y_sp - 3.0),
        Some(w) => (x_np + w, info.y_neck - 12.0 - w * 0.12),
    };
    let (nf, nb) = arm_ring_arcs(info, y_np, d_top, x_np);
    let (sf, sb) = arm_ring_arcs(info, y_sp, d_top, x_sp);
    let ad_sp = 0.5 * (info.march_len(y0, y_sp, sf, false, d_top) + info.march_len(y0, y_sp, sb, true, d_top));
    let ad_np = 0.5 * (info.march_len(y0, y_np, nf, false, d_top) + info.march_len(y0, y_np, nb, true, d_top));
    (ad_sp, ad_np, nf, nb, sf, sb)
}

fn armhole_target(info: &Info) -> f64 {
    std::f64::consts::PI * info.arms[0].root_radius() * 1.02
}

#[allow(clippy::type_complexity)]
fn fit_armhole(info: &Info, cw: f64, strap: Option<f64>, d_top: f64) -> (f64, f64, f64, f64, f64, f64, f64) {
    let target = armhole_target(info);
    let mut delta = 12.0;
    loop {
        let y0 = info.y_armpit - delta;
        let (ad_sp, ad_np, nf, _nb, sf, _sb) = shoulder_dims(info, y0, strap, d_top);
        let probe = Bod { side: vec![[cw, 0.0], [cw, -100.0]], ad_sp, ad_np, xsp: sf, xnp: nf, drop: 60.0, arm_in: if strap.is_some() { 16.0 } else { 4.0 }, arm_lo: if strap.is_some() { 0.16 } else { 0.10 }, neck_steps: 4 };
        let len = poly_len(&armhole_curve(&probe, [cw, 0.0], [sf, ad_sp]));
        if len >= target || delta > 90.0 {
            let (a, b, c, d, e, f) = shoulder_dims(info, y0, strap, d_top);
            return (y0, a, b, c, d, e, f);
        }
        delta += 3.0;
    }
}

fn finish_panel(name: &str, kind: Kind, back: bool, built: (Vec<DPt>, Vec<(String, usize, usize)>), y0: f64, side_sum: Vec<(f64, f64)>) -> PanelDraft {
    PanelDraft { name: name.to_string(), kind, back, pts: built.0, runs: built.1, y0, side_sum, hip_clamp: None, ad: 200.0, d_top: 10.0, sleeve: None, pd: 22.0, notches: Vec::new(), march: kind == Kind::Bodice }
}

fn sleeve_panel(info: &Info, arm_index: usize, ah_f: f64, ah_b: f64, name: &str) -> (PanelDraft, f64) {
    let arm = &info.arms[arm_index];
    let ls = (info.measure("arm_length") * 0.32).clamp(100.0, 260.0);
    let a_c0 = ls * 0.9 + 30.0;
    let tau = std::f64::consts::TAU;
    let hw = tau * (arm.radius(a_c0) + 15.0);
    let target = (ah_f + ah_b) * 0.5 * 1.02;
    let bw = (tau * (arm.radius(60.0) + 14.0)).min(0.80 * (ah_f + ah_b)).max(hw * 1.05);
    let h = bw * 0.5;
    let half = |ch: f64| -> Vec<P2> { bez_poly([[0.0, 0.0], [0.30 * h, 0.10 * ch], [0.55 * h, 0.98 * ch], [h, ch]], 24) };
    let (mut lo, mut hi) = (5.0, 400.0);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if poly_len(&half(mid)) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let ch = 0.5 * (lo + hi);
    let left = half(ch);
    let mut cap: Vec<P2> = left.clone();
    for p in left.iter().rev().skip(1) {
        cap.push([bw - p[0], p[1]]);
    }
    let hl = (bw - hw) * 0.5;
    let mut bd = Builder::start([hl, -ls]);
    bd.line_to("hem", [hl + hw, -ls]);
    bd.line_to("side_r", [bw, 0.0]);
    let sb: Vec<P2> = left.iter().step_by(4).map(|p| [bw - p[0], p[1]]).collect();
    let sf: Vec<P2> = left.iter().rev().step_by(4).copied().collect();
    bd.curve("cap_b", &sb, false);
    bd.curve("cap_f", &sf, false);
    bd.close_line("side_l");
    let mut p = finish_panel(name, Kind::Sleeve, false, bd.finish(), 0.0, Vec::new());
    p.sleeve = Some(Sleeve { sign: 1.0, bw, hw, ls, cap, front: (0, "armhole_r".into()), back: (1, "armhole_l".into()) });
    (p, ch)
}

pub fn mirrored_sleeve(mut p: PanelDraft, name: &str) -> PanelDraft {
    let bw = p.sleeve.as_ref().map(|s| s.bw).unwrap_or(0.0);
    p.mirror_x(bw);
    if let Some(s) = p.sleeve.as_mut() {
        s.sign = -1.0;
        s.front.1 = "armhole_l".into();
        s.back.1 = "armhole_r".into();
    }
    p.name = name.to_string();
    p
}

fn lens(pts: &[DPt], runs: &[(String, usize, usize)], name: &str) -> f64 {
    let Some(r) = runs.iter().find(|r| r.0 == name) else { return 0.0 };
    (r.1..r.1 + r.2).map(|i| poly_len(&bez_poly(seg_bez(pts, i), 16))).sum()
}

#[allow(clippy::type_complexity)]
pub fn tshirt(info: &Info) -> Draft {
    let b = info.measure("bust").max(600.0);
    let h = info.measure("hip").max(700.0);
    let ease = 100.0;
    let hem_y = info.y_hip - 20.0;
    let chest = b + ease;
    let hem = chest.max(h + 50.0);
    let d_top = 10.0;
    let (y0, ad_sp, ad_np, nf, nb, sf, sb) = fit_armhole(info, chest / 4.0 + 5.0, None, d_top);
    let lh = y0 - hem_y;
    let mk = |back: bool| -> (Vec<DPt>, Vec<(String, usize, usize)>, Vec<P2>) {
        let sgn = if back { -1.0 } else { 1.0 };
        let cw = chest / 4.0 + 5.0 * sgn;
        let hwd = hem / 4.0 + 5.0 * sgn;
        bodice(&Bod {
            side: vec![[cw, 0.0], [hwd, -lh]],
            ad_sp,
            ad_np,
            xsp: if back { sb } else { sf },
            xnp: if back { nb } else { nf },
            drop: if back { 22.0 } else { info.neck_r * 1.35 },
            arm_in: 4.0,
            arm_lo: 0.10,
            neck_steps: if back { 6 } else { 8 },
        })
    };
    let (fp, fr, fah) = mk(false);
    let (bp, br, bah) = mk(true);
    let sumt = side_sum(
        &[[chest / 4.0 + 5.0, 0.0], [hem / 4.0 + 5.0, -lh]],
        &[[chest / 4.0 - 5.0, 0.0], [hem / 4.0 - 5.0, -lh]],
    );
    let mut front = finish_panel("T-Shirt Front", Kind::Bodice, false, (fp, fr), y0, sumt.clone());
    let mut back = finish_panel("T-Shirt Back", Kind::Bodice, true, (bp, br), y0, sumt);
    front.ad = ad_sp;
    back.ad = ad_sp;
    front.d_top = d_top;
    back.d_top = d_top;
    let ah_f = poly_len(&fah);
    let ah_b = poly_len(&bah);
    let (sleeve_l, ch) = sleeve_panel(info, 0, ah_f, ah_b, "T-Shirt Sleeve L");
    let cap_len = lens(&sleeve_l.pts, &sleeve_l.runs, "cap_f") + lens(&sleeve_l.pts, &sleeve_l.runs, "cap_b");
    let sleeve_r = mirrored_sleeve(sleeve_l.clone(), "T-Shirt Sleeve R");
    front.notches = vec![("shoulder_r".into(), 0.0), ("armhole_r".into(), 0.0)];
    back.notches = vec![("shoulder_l".into(), 1.0), ("armhole_l".into(), 1.0)];
    let mut sleeve_l = sleeve_l;
    let mut sleeve_r = sleeve_r;
    sleeve_l.notches = vec![("cap_b".into(), 1.0)];
    sleeve_r.notches = vec![("cap_b".into(), 1.0)];
    let panels = vec![front, back, sleeve_l, sleeve_r];
    let seams = vec![
        seam("side +x", vec![span(0, "side_r")], vec![span(1, "side_l")], true),
        seam("side -x", vec![span(0, "side_l")], vec![span(1, "side_r")], true),
        seam("shoulder +x", vec![span(0, "shoulder_r")], vec![span(1, "shoulder_l")], true),
        seam("shoulder -x", vec![span(0, "shoulder_l")], vec![span(1, "shoulder_r")], true),
        seam("sleeve tube +x", vec![span(2, "side_l")], vec![span(2, "side_r")], false),
        seam("sleeve tube -x", vec![span(3, "side_l")], vec![span(3, "side_r")], false),
        seam("cap front +x", vec![span(2, "cap_f")], vec![span(0, "armhole_r")], false),
        seam("cap back +x", vec![span(2, "cap_b")], vec![span(1, "armhole_l")], false),
        seam("cap front -x", vec![span(3, "cap_f")], vec![span(0, "armhole_l")], false),
        seam("cap back -x", vec![span(3, "cap_b")], vec![span(1, "armhole_r")], false),
    ];
    let report = json!({
        "bust": b, "hip": h, "chest_ease": ease, "armhole_front": ah_f, "armhole_back": ah_b, "cap_length": cap_len,
        "cap_ease_percent": (cap_len / (ah_f + ah_b) - 1.0) * 100.0, "cap_height": ch, "hem_y": hem_y, "chest_line_y": y0,
        "shoulder_arcs": [sf, sb], "neck_arcs": [nf, nb]
    });
    Draft { title: "Basic T-Shirt".into(), panels, seams, report, circles: false }
}

fn skirt_outline(side: &[P2]) -> (Vec<DPt>, Vec<(String, usize, usize)>) {
    let top = side[0];
    let last = side[side.len() - 1];
    let mut bd = Builder::start([-last[0], last[1]]);
    bd.line_to("hem", [last[0], last[1]]);
    let right: Vec<P2> = side.iter().rev().copied().collect();
    if right.len() == 2 {
        bd.line_to("side_r", right[1]);
    } else {
        bd.curve("side_r", &right, false);
    }
    bd.line_to("top", [-top[0], top[1]]);
    let left: Vec<P2> = side.iter().map(|p| [-p[0], p[1]]).collect();
    if left.len() == 2 {
        bd.close_line("side_l");
    } else {
        bd.curve("side_l", &left, true);
    }
    bd.finish()
}

pub fn skirt(info: &Info) -> Draft {
    let w = info.measure("waist").max(450.0);
    let h = info.measure("hip").max(700.0);
    let band_h = 36.0;
    let wl = w + 25.0;
    let y_band = info.y_waist + 10.0;
    let y0 = y_band - band_h;
    let hem_y = info.y_knee + 45.0;
    let hip_drop = (info.y_hip - y0).min(-60.0);
    let hem_drop = hem_y - y0;
    let hip_half = (h + 50.0) / 4.0;
    let hem_half = hip_half + 60.0;
    let mk = |sgn: f64| -> Vec<P2> { vec![[wl / 4.0 + 5.0 * sgn, 0.0], [hip_half + 5.0 * sgn, hip_drop], [hem_half + 5.0 * sgn, hem_drop]] };
    let (fs, bs) = (mk(1.0), mk(-1.0));
    let sums: Vec<(f64, f64)> = (0..3).map(|i| (fs[i][1], fs[i][0] + bs[i][0])).collect();
    let mut table = sums.clone();
    table.reverse();
    let mut front = finish_panel("Skirt Front", Kind::Skirt, false, skirt_outline(&fs), y0, table.clone());
    let mut back = finish_panel("Skirt Back", Kind::Skirt, true, skirt_outline(&bs), y0, table);
    front.hip_clamp = Some(info.y_hip);
    back.hip_clamp = Some(info.y_hip);
    let mut bd = Builder::start([-wl / 2.0, -band_h]);
    bd.line_to("bottom", [wl / 2.0, -band_h]);
    bd.line_to("end_r", [wl / 2.0, 0.0]);
    bd.line_to("top", [-wl / 2.0, 0.0]);
    bd.close_line("end_l");
    let band = finish_panel("Waistband", Kind::Band, false, bd.finish(), y_band, vec![(-band_h, wl / 2.0), (0.0, wl / 2.0)]);
    let wf = wl / 4.0 + 5.0;
    let t0 = (wl / 2.0 - wf) / wl;
    let t1 = (wl / 2.0 + wf) / wl;
    let sp = |panel: usize, run: &str, a: f64, b: f64| SpanRef { panel, run: run.to_string(), t: Some((a, b)) };
    let seams = vec![
        seam("side +x", vec![span(0, "side_r")], vec![span(1, "side_l")], true),
        seam("side -x", vec![span(0, "side_l")], vec![span(1, "side_r")], true),
        seam("waistband front", vec![span(0, "top")], vec![sp(2, "bottom", t0, t1)], true),
        seam("waistband back -x", vec![sp(1, "top", 0.0, 0.5)], vec![sp(2, "bottom", 0.0, t0)], true),
        seam("waistband back +x", vec![sp(1, "top", 0.5, 1.0)], vec![sp(2, "bottom", t1, 1.0)], true),
        seam("waistband closure", vec![span(2, "end_l")], vec![span(2, "end_r")], false),
    ];
    let mut panels = vec![front, back, band];
    for p in panels.iter_mut() {
        p.d_top = MIN_D_TOP;
    }
    let report = json!({"waist": w, "hip": h, "waistband_length": wl, "hem_y": hem_y, "waist_y": info.y_waist, "hip_y": info.y_hip, "length_waist_to_hem": info.y_waist - hem_y});
    Draft { title: "A-Line Skirt".into(), panels, seams, report, circles: false }
}

const MIN_D_TOP: f64 = 8.0;

pub fn dress(info: &Info) -> Draft {
    let b = info.measure("bust").max(600.0);
    let w = info.measure("waist").max(450.0);
    let h = info.measure("hip").max(700.0);
    let hem_y = info.y_knee + 45.0;
    let d_top = 10.0;
    let chest_half = (b + 60.0) / 4.0;
    let (y0, ad_sp, ad_np, nf, nb, sf, sb) = fit_armhole(info, chest_half - 5.0, Some(48.0), d_top);
    let waist_half = (w + 80.0) / 4.0;
    let hip_half = (h + 60.0) / 4.0;
    let hem_half = hip_half + 70.0;
    let waist_drop = (info.y_waist - y0).min(-120.0);
    let hip_drop = (info.y_hip - y0).min(waist_drop - 60.0);
    let hem_drop = hem_y - y0;
    let side = |sgn: f64| -> Vec<P2> {
        vec![[chest_half - 10.0 + 5.0 * sgn, 0.0], [waist_half + 4.0 * sgn, waist_drop], [hip_half + 5.0 * sgn, hip_drop], [hem_half + 5.0 * sgn, hem_drop]]
    };
    let mk = |back: bool| {
        let sgn = if back { -1.0 } else { 1.0 };
        bodice(&Bod {
            side: side(sgn),
            ad_sp,
            ad_np,
            xsp: if back { sb } else { sf },
            xnp: if back { nb } else { nf },
            drop: if back { 40.0 } else { info.neck_r * 1.9 },
            arm_in: 16.0,
            arm_lo: 0.16,
            neck_steps: 8,
        })
    };
    let (fp, fr, _) = mk(false);
    let (bp, br, _) = mk(true);
    let (fs, bs) = (side(1.0), side(-1.0));
    let table: Vec<(f64, f64)> = (0..4).rev().map(|i| (fs[i][1], fs[i][0] + bs[i][0])).collect();
    let mut front = finish_panel("Dress Front", Kind::Bodice, false, (fp, fr), y0, table.clone());
    let mut back = finish_panel("Dress Back", Kind::Bodice, true, (bp, br), y0, table);
    for p in [&mut front, &mut back] {
        p.ad = ad_sp;
        p.d_top = d_top;
        p.hip_clamp = Some(info.y_hip);
    }
    let seams = vec![
        seam("side +x", vec![span(0, "side_r")], vec![span(1, "side_l")], true),
        seam("side -x", vec![span(0, "side_l")], vec![span(1, "side_r")], true),
        seam("shoulder +x", vec![span(0, "shoulder_r")], vec![span(1, "shoulder_l")], true),
        seam("shoulder -x", vec![span(0, "shoulder_l")], vec![span(1, "shoulder_r")], true),
    ];
    let report = json!({"bust": b, "waist": w, "hip": h, "hem_y": hem_y, "chest_line_y": y0});
    Draft { title: "Sleeveless Dress".into(), panels: vec![front, back], seams, report, circles: false }
}
