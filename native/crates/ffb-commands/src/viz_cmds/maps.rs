use ffb_sim::energy::COMPRESSION;
use ffb_sim::material::FORCE_PER_N_PER_M;
use glam::DVec3;
use serde_json::{json, Value};

use super::scene::{PanelScene, Scene};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Strain,
    Stress,
    Pressure,
    Fit,
    Contact,
    Thickness,
    Compare,
}

impl Kind {
    pub fn parse(name: &str) -> Option<Kind> {
        Some(match name {
            "strain" => Kind::Strain,
            "stress" => Kind::Stress,
            "pressure" => Kind::Pressure,
            "fit" => Kind::Fit,
            "contact" => Kind::Contact,
            "thickness" => Kind::Thickness,
            _ => return None,
        })
    }

    pub fn id(self) -> u32 {
        match self {
            Kind::Strain => 1,
            Kind::Stress => 2,
            Kind::Pressure => 3,
            Kind::Fit => 4,
            Kind::Contact => 5,
            Kind::Thickness => 6,
            Kind::Compare => 7,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Strain => "strain",
            Kind::Stress => "stress",
            Kind::Pressure => "pressure",
            Kind::Fit => "fit",
            Kind::Contact => "contact",
            Kind::Thickness => "thickness",
            Kind::Compare => "compare",
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            Kind::Strain => "%",
            Kind::Stress | Kind::Pressure => "kPa",
            Kind::Fit | Kind::Thickness | Kind::Compare => "mm",
            Kind::Contact => "flag",
        }
    }

    pub fn fixed_range(self) -> Option<[f32; 2]> {
        (self == Kind::Contact).then_some([0.0, 1.0])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Component {
    Max,
    Min,
    Weft,
    Warp,
    Shear,
}

impl Component {
    pub fn parse(name: &str) -> Option<Component> {
        Some(match name {
            "max" => Component::Max,
            "min" => Component::Min,
            "weft" => Component::Weft,
            "warp" => Component::Warp,
            "shear" => Component::Shear,
            _ => return None,
        })
    }

    pub fn id(self) -> u32 {
        match self {
            Component::Max => 0,
            Component::Min => 1,
            Component::Weft => 2,
            Component::Warp => 3,
            Component::Shear => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Component::Max => "max",
            Component::Min => "min",
            Component::Weft => "weft",
            Component::Warp => "warp",
            Component::Shear => "shear",
        }
    }
}

struct Grad {
    fu: DVec3,
    fv: DVec3,
    area: f64,
}

fn v3(p: [f32; 3]) -> DVec3 {
    DVec3::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
}

fn gradient(rest: &[[f64; 2]], t: [u32; 3], x: &[[f32; 3]]) -> Option<Grad> {
    let (a, b, c) = (rest[t[0] as usize], rest[t[1] as usize], rest[t[2] as usize]);
    let (p, q, r, s) = (b[0] - a[0], c[0] - a[0], b[1] - a[1], c[1] - a[1]);
    let det = p * s - q * r;
    if det.abs() < 1.0e-9 {
        return None;
    }
    let (ia, ib, ic, id) = (s / det, -q / det, -r / det, p / det);
    let (x0, x1, x2) = (v3(x[t[0] as usize]), v3(x[t[1] as usize]), v3(x[t[2] as usize]));
    let (d1, d2) = (x1 - x0, x2 - x0);
    Some(Grad { fu: d1 * ia + d2 * ic, fv: d1 * ib + d2 * id, area: 0.5 * det.abs() })
}

fn principal(g: &Grad) -> (f64, f64) {
    let (a, b, d) = (g.fu.length_squared(), g.fu.dot(g.fv), g.fv.length_squared());
    let mean = 0.5 * (a + d);
    let spread = (0.25 * (a - d) * (a - d) + b * b).sqrt();
    ((mean + spread).max(0.0).sqrt(), (mean - spread).max(0.0).sqrt())
}

fn strain_value(g: &Grad, component: Component) -> f64 {
    let (hi, lo) = principal(g);
    match component {
        Component::Max => (hi - 1.0) * 100.0,
        Component::Min => (lo - 1.0) * 100.0,
        Component::Weft => (g.fu.length() - 1.0) * 100.0,
        Component::Warp => (g.fv.length() - 1.0) * 100.0,
        Component::Shear => {
            let n = g.fu.length() * g.fv.length();
            if n > 1.0e-12 {
                g.fu.dot(g.fv) / n * 100.0
            } else {
                0.0
            }
        }
    }
}

fn tensions(panel: &PanelScene, g: &Grad) -> (f64, f64, f64) {
    let m = &panel.material;
    let scale = 1.0 / FORCE_PER_N_PER_M;
    let one = |len: f64, k: f32| {
        let e = len - 1.0;
        f64::from(k) * scale * if e < 0.0 { e * COMPRESSION } else { e }
    };
    (one(g.fu.length(), m.stretch_weft), one(g.fv.length(), m.stretch_warp), f64::from(m.shear) * scale * g.fu.dot(g.fv))
}

fn stress_value(panel: &PanelScene, g: &Grad, component: Component) -> f64 {
    let (u, v, s) = tensions(panel, g);
    let n = match component {
        Component::Weft => u,
        Component::Warp => v,
        Component::Shear => s,
        Component::Min => u.min(v),
        Component::Max => {
            let top = u.max(v);
            if s.abs() > top {
                s.abs()
            } else {
                top
            }
        }
    };
    n / f64::from(panel.thickness.max(0.05))
}

fn scatter(panel: &PanelScene, x: &[[f32; 3]], value: &dyn Fn(&PanelScene, &Grad) -> f64) -> Vec<f32> {
    let n = panel.rest.len();
    let mut sum = vec![0.0f64; n];
    let mut weight = vec![0.0f64; n];
    for t in &panel.triangles {
        if t.iter().any(|&i| i as usize >= n) {
            continue;
        }
        let Some(g) = gradient(&panel.rest, *t, x) else { continue };
        let val = value(panel, &g);
        if !val.is_finite() {
            continue;
        }
        for &i in t {
            sum[i as usize] += val * g.area;
            weight[i as usize] += g.area;
        }
    }
    (0..n).map(|i| if weight[i] > 0.0 { (sum[i] / weight[i]) as f32 } else { f32::NAN }).collect()
}

fn nearest_body(scene: &Scene, p: DVec3) -> Option<(f64, DVec3)> {
    let mut best: Option<(f64, DVec3)> = None;
    for b in scene.bodies() {
        if let Some((d, n)) = b.signed_distance(p, 1000.0) {
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, n));
            }
        }
    }
    best
}

fn mean_curvature(panel: &PanelScene, x: &[[f32; 3]]) -> Vec<DVec3> {
    let n = panel.rest.len();
    let mut acc = vec![DVec3::ZERO; n];
    let mut area = vec![0.0f64; n];
    let cot = |a: DVec3, b: DVec3, c: DVec3| {
        let (u, w) = (b - a, c - a);
        let cross = u.cross(w).length();
        if cross < 1.0e-12 {
            0.0
        } else {
            u.dot(w) / cross
        }
    };
    for t in &panel.triangles {
        if t.iter().any(|&i| i as usize >= n) {
            continue;
        }
        let p = [v3(x[t[0] as usize]), v3(x[t[1] as usize]), v3(x[t[2] as usize])];
        let tri_area = 0.5 * (p[1] - p[0]).cross(p[2] - p[0]).length();
        for k in 0..3 {
            let (a, b, c) = (k, (k + 1) % 3, (k + 2) % 3);
            let cot_b = cot(p[b], p[c], p[a]).clamp(-20.0, 20.0);
            let cot_c = cot(p[c], p[a], p[b]).clamp(-20.0, 20.0);
            acc[t[a] as usize] += (p[b] - p[a]) * cot_c + (p[c] - p[a]) * cot_b;
            area[t[a] as usize] += tri_area / 3.0;
        }
    }
    acc.iter().zip(&area).map(|(a, w)| if *w > 1.0e-9 { *a / (2.0 * w) } else { DVec3::ZERO }).collect()
}

fn mean_tension(panel: &PanelScene, x: &[[f32; 3]]) -> Vec<f32> {
    scatter(panel, x, &|p, g| {
        let (u, v, _) = tensions(p, g);
        0.5 * (u.max(0.0) + v.max(0.0))
    })
}

fn pressure(scene: &Scene, panel: &PanelScene, x: &[[f32; 3]]) -> Vec<f32> {
    let n = panel.rest.len();
    if !scene.has_bodies() {
        return vec![f32::NAN; n];
    }
    let curvature = mean_curvature(panel, x);
    let tension = mean_tension(panel, x);
    let weight = f64::from(panel.density) * 1.0e-3 * 9.81 * 1.0e-3;
    let reach = f64::from(scene.offset) + 2.0;
    (0..n)
        .map(|i| {
            let Some((sd, normal)) = nearest_body(scene, v3(x[i])) else { return 0.0 };
            if sd > reach {
                return 0.0;
            }
            let pull = if tension[i].is_finite() { f64::from(tension[i]) * (-curvature[i].dot(normal)).max(0.0) } else { 0.0 };
            (pull + weight * normal.y.max(0.0)) as f32
        })
        .collect()
}

pub(crate) fn compute(scene: &Scene, positions: &[Vec<[f32; 3]>], kind: Kind, component: Component) -> Vec<Vec<f32>> {
    scene
        .panels
        .iter()
        .zip(positions)
        .map(|(panel, x)| {
            let n = panel.rest.len();
            if x.len() != n {
                return vec![f32::NAN; n];
            }
            match kind {
                Kind::Strain => scatter(panel, x, &|_, g| strain_value(g, component)),
                Kind::Stress => scatter(panel, x, &|p, g| stress_value(p, g, component)),
                Kind::Pressure => pressure(scene, panel, x),
                Kind::Fit => x.iter().map(|p| nearest_body(scene, v3(*p)).map_or(f32::NAN, |(d, _)| d as f32)).collect(),
                Kind::Contact => {
                    let reach = f64::from(scene.offset) + 1.0;
                    x.iter().map(|p| nearest_body(scene, v3(*p)).map_or(f32::NAN, |(d, _)| if d <= reach { 1.0 } else { 0.0 })).collect()
                }
                Kind::Thickness => vec![scene.shell.unwrap_or(panel.thickness); n],
                Kind::Compare => vec![f32::NAN; n],
            }
        })
        .collect()
}

pub(crate) fn distance(a: &[Vec<[f32; 3]>], b: &[Vec<[f32; 3]>]) -> Vec<Vec<f32>> {
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            if x.len() != y.len() {
                return vec![f32::NAN; x.len()];
            }
            x.iter().zip(y).map(|(p, q)| (v3(*p) - v3(*q)).length() as f32).collect()
        })
        .collect()
}

pub(crate) fn auto_range(values: &[Vec<f32>], kind: Kind) -> [f32; 2] {
    if let Some(r) = kind.fixed_range() {
        return r;
    }
    let mut all: Vec<f32> = values.iter().flatten().copied().filter(|v| v.is_finite()).collect();
    if all.is_empty() {
        return [0.0, 1.0];
    }
    all.sort_by(f32::total_cmp);
    let at = |q: f32| all[((all.len() - 1) as f32 * q).round() as usize];
    let (mut lo, mut hi) = (at(0.0), at(1.0));
    if matches!(kind, Kind::Stress | Kind::Pressure | Kind::Thickness | Kind::Compare) {
        lo = lo.min(0.0);
    }
    if hi - lo < 1.0e-6 {
        hi = lo + 1.0;
    }
    [lo, hi]
}

fn round(x: f32) -> f64 {
    if x.is_finite() {
        (f64::from(x) * 1.0e4).round() / 1.0e4
    } else {
        0.0
    }
}

fn summary(values: &[f32]) -> Value {
    let mut v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return json!({"count": 0, "covered": 0});
    }
    v.sort_by(f32::total_cmp);
    let at = |q: f32| round(v[((v.len() - 1) as f32 * q).round() as usize]);
    let mean = v.iter().map(|x| f64::from(*x)).sum::<f64>() / v.len() as f64;
    json!({
        "count": values.len(),
        "covered": v.len(),
        "min": round(v[0]),
        "max": round(v[v.len() - 1]),
        "mean": (mean * 1.0e4).round() / 1.0e4,
        "percentiles": {"p1": at(0.01), "p5": at(0.05), "p25": at(0.25), "p50": at(0.5), "p75": at(0.75), "p95": at(0.95), "p99": at(0.99)}
    })
}

pub(crate) fn stats(scene: &Scene, values: &[Vec<f32>]) -> Value {
    let all: Vec<f32> = values.iter().flatten().copied().collect();
    let mut out = summary(&all);
    let per: Vec<Value> = scene
        .panels
        .iter()
        .zip(values)
        .map(|(p, v)| {
            let mut s = summary(v);
            s["panel"] = json!(p.id);
            s
        })
        .collect();
    out["panels"] = json!(per);
    out
}
