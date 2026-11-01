use ffb_pattern::geom::*;

pub type Cubic = [V2; 4];

const MAX_DEPTH: u32 = 48;
const REFINE_PASSES: usize = 6;

fn b0(t: f64) -> f64 {
    (1.0 - t).powi(3)
}

fn b1(t: f64) -> f64 {
    3.0 * t * (1.0 - t).powi(2)
}

fn b2(t: f64) -> f64 {
    3.0 * t * t * (1.0 - t)
}

fn b3(t: f64) -> f64 {
    t.powi(3)
}

fn eval(c: &Cubic, t: f64) -> V2 {
    let (a, b, d, e) = (b0(t), b1(t), b2(t), b3(t));
    [a * c[0][0] + b * c[1][0] + d * c[2][0] + e * c[3][0], a * c[0][1] + b * c[1][1] + d * c[2][1] + e * c[3][1]]
}

fn first_derivative(c: &Cubic, t: f64) -> V2 {
    let u = 1.0 - t;
    let (d0, d1, d2) = (sub(c[1], c[0]), sub(c[2], c[1]), sub(c[3], c[2]));
    add(add(mul(d0, 3.0 * u * u), mul(d1, 6.0 * u * t)), mul(d2, 3.0 * t * t))
}

fn second_derivative(c: &Cubic, t: f64) -> V2 {
    let u = 1.0 - t;
    let e0 = add(sub(c[2], mul(c[1], 2.0)), c[0]);
    let e1 = add(sub(c[3], mul(c[2], 2.0)), c[1]);
    add(mul(e0, 6.0 * u), mul(e1, 6.0 * t))
}

fn chord_params(pts: &[V2]) -> Vec<f64> {
    let mut u = vec![0.0];
    for w in pts.windows(2) {
        let last = *u.last().unwrap_or(&0.0);
        u.push(last + dist(w[0], w[1]));
    }
    let total = *u.last().unwrap_or(&1.0);
    if total < EPS {
        return (0..pts.len()).map(|i| i as f64 / (pts.len() - 1).max(1) as f64).collect();
    }
    u.iter().map(|x| x / total).collect()
}

fn generate(pts: &[V2], u: &[f64], t1: V2, t2: V2) -> Cubic {
    let n = pts.len();
    let (p0, p3) = (pts[0], pts[n - 1]);
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        let a1 = mul(t1, b1(u[i]));
        let a2 = mul(t2, b2(u[i]));
        c00 += dot(a1, a1);
        c01 += dot(a1, a2);
        c11 += dot(a2, a2);
        let base = add(mul(p0, b0(u[i]) + b1(u[i])), mul(p3, b2(u[i]) + b3(u[i])));
        let tmp = sub(pts[i], base);
        x0 += dot(a1, tmp);
        x1 += dot(a2, tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let seg = dist(p0, p3);
    let (mut al, mut ar) = if det.abs() > 1e-12 { ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det) } else { (0.0, 0.0) };
    if al < 1e-6 * seg || ar < 1e-6 * seg || al > seg * 3.0 || ar > seg * 3.0 {
        al = seg / 3.0;
        ar = seg / 3.0;
    }
    [p0, add(p0, mul(t1, al)), add(p3, mul(t2, ar)), p3]
}

fn max_error(pts: &[V2], c: &Cubic, u: &[f64]) -> (f64, usize) {
    let mut worst = (0.0, pts.len() / 2);
    for i in 1..pts.len() - 1 {
        let d = dist(eval(c, u[i]), pts[i]);
        if d >= worst.0 {
            worst = (d, i);
        }
    }
    worst
}

fn reparameterize(pts: &[V2], u: &[f64], c: &Cubic) -> Vec<f64> {
    pts.iter()
        .zip(u)
        .map(|(p, &t)| {
            let d = sub(eval(c, t), *p);
            let d1 = first_derivative(c, t);
            let d2 = second_derivative(c, t);
            let num = dot(d, d1);
            let den = dot(d1, d1) + dot(d, d2);
            if den.abs() < 1e-12 {
                t
            } else {
                (t - num / den).clamp(0.0, 1.0)
            }
        })
        .collect()
}

fn fit_into(pts: &[V2], t1: V2, t2: V2, tol: f64, out: &mut Vec<Cubic>, depth: u32) {
    let n = pts.len();
    if n == 2 {
        let d = dist(pts[0], pts[1]) / 3.0;
        out.push([pts[0], add(pts[0], mul(t1, d)), add(pts[1], mul(t2, d)), pts[1]]);
        return;
    }
    let mut u = chord_params(pts);
    let mut cubic = generate(pts, &u, t1, t2);
    let (mut err, mut split) = max_error(pts, &cubic, &u);
    if err <= tol {
        out.push(cubic);
        return;
    }
    if err <= tol * 8.0 {
        for _ in 0..REFINE_PASSES {
            u = reparameterize(pts, &u, &cubic);
            cubic = generate(pts, &u, t1, t2);
            (err, split) = max_error(pts, &cubic, &u);
            if err <= tol {
                out.push(cubic);
                return;
            }
        }
    }
    if depth >= MAX_DEPTH {
        out.push(cubic);
        return;
    }
    let split = split.clamp(1, n - 2);
    let centre = unit(sub(pts[split - 1], pts[split + 1])).unwrap_or(t1);
    fit_into(&pts[..=split], t1, centre, tol, out, depth + 1);
    fit_into(&pts[split..], mul(centre, -1.0), t2, tol, out, depth + 1);
}

pub fn fit(pts: &[V2], t1: V2, t2: V2, tol: f64) -> Vec<Cubic> {
    let mut out = Vec::new();
    if pts.len() >= 2 {
        fit_into(pts, t1, t2, tol, &mut out, 0);
    }
    out
}

pub fn start_tangent(pts: &[V2]) -> V2 {
    let chord = unit(sub(pts[1], pts[0])).unwrap_or([1.0, 0.0]);
    if pts.len() < 3 {
        return chord;
    }
    let est = unit(sub(sub(mul(pts[1], 4.0), mul(pts[0], 3.0)), pts[2]));
    match est {
        Some(e) if dot(e, chord) > 0.7 => e,
        _ => chord,
    }
}

pub fn end_tangent(pts: &[V2]) -> V2 {
    let rev: Vec<V2> = pts.iter().rev().copied().collect();
    start_tangent(&rev)
}
