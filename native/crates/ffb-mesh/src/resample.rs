use ffb_pattern::bezier::Seg;
use ffb_pattern::geom::{add, dist, dot, lerp, mul, sub, V2};

const MAX_DEPTH: u32 = 16;
const MIN_DEPTH: u32 = 2;

pub fn flatten_tolerance(particle_distance: f64) -> f64 {
    (particle_distance * 0.01).clamp(0.005, 0.1)
}

fn off_chord(p: V2, a: V2, b: V2) -> f64 {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    if l2 < 1e-18 {
        return dist(p, a);
    }
    let t = (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0);
    dist(p, add(a, mul(ab, t)))
}

fn split(seg: &Seg, range: (f64, V2, f64, V2), tol: f64, depth: u32, out: &mut Vec<(f64, V2)>) {
    let (t0, p0, t1, p1) = range;
    let tm = 0.5 * (t0 + t1);
    let pm = seg.at(tm);
    if depth >= MIN_DEPTH {
        let flat = off_chord(pm, p0, p1) <= tol
            && off_chord(seg.at(0.5 * (t0 + tm)), p0, p1) <= tol
            && off_chord(seg.at(0.5 * (tm + t1)), p0, p1) <= tol;
        if flat || depth >= MAX_DEPTH {
            return;
        }
    }
    split(seg, (t0, p0, tm, pm), tol, depth + 1, out);
    out.push((tm, pm));
    split(seg, (tm, pm, t1, p1), tol, depth + 1, out);
}

pub fn flatten_t(seg: &Seg, tol: f64) -> Vec<(f64, V2)> {
    let mut out = vec![(0.0, seg.p[0])];
    if seg.curved {
        split(seg, (0.0, seg.p[0], 1.0, seg.p[3]), tol, 0, &mut out);
    }
    out.push((1.0, seg.p[3]));
    out
}

pub fn arc_length(seg: &Seg, tol: f64) -> f64 {
    if !seg.curved {
        return dist(seg.p[0], seg.p[3]);
    }
    flatten_t(seg, tol).windows(2).map(|w| dist(w[0].1, w[1].1)).sum()
}

pub fn natural_count(length: f64, particle_distance: f64) -> usize {
    ((length / particle_distance).round() as usize).max(1)
}

pub fn resample(seg: &Seg, count: usize, tol: f64) -> Vec<(f64, V2)> {
    let count = count.max(1);
    if !seg.curved {
        return (0..count)
            .map(|k| {
                let t = k as f64 / count as f64;
                (t, lerp(seg.p[0], seg.p[3], t))
            })
            .collect();
    }
    let poly = flatten_t(seg, tol);
    let mut cum = Vec::with_capacity(poly.len());
    let mut total = 0.0;
    cum.push(0.0);
    for w in poly.windows(2) {
        total += dist(w[0].1, w[1].1);
        cum.push(total);
    }
    let mut out = Vec::with_capacity(count);
    let mut cell = 0;
    for k in 0..count {
        let s = total * k as f64 / count as f64;
        while cell + 2 < poly.len() && cum[cell + 1] < s {
            cell += 1;
        }
        let span = cum[cell + 1] - cum[cell];
        let f = if span > 1e-12 { ((s - cum[cell]) / span).clamp(0.0, 1.0) } else { 0.0 };
        let t = poly[cell].0 + (poly[cell + 1].0 - poly[cell].0) * f;
        out.push((t, if k == 0 { seg.p[0] } else { seg.at(t) }));
    }
    out
}
