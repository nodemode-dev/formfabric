use ffb_pattern::geom::*;
use ffb_pattern::path;
use ffb_pattern::{CornerStyle, Point, PointKind};

const SHARP: f64 = 0.06;
const MITER_LIMIT: f64 = 4.0;
const CONCAVE_LIMIT: f64 = 50.0;

#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub pts: Vec<V2>,
    pub turn: Vec<bool>,
    pub edge_seg: Vec<usize>,
}

fn is_turn(points: &[Point], closed: bool, i: usize) -> bool {
    let n = points.len();
    if !closed && (i == 0 || i + 1 == n) {
        return true;
    }
    if points[i].kind == PointKind::Corner {
        return true;
    }
    let before = path::seg(points, (i + n - 1) % n).curved;
    let after = path::seg(points, i).curved;
    !(before && after)
}

pub fn trace(points: &[Point], closed: bool, tol: f64) -> Trace {
    let mut t = Trace::default();
    let count = path::seg_count(points.len(), closed);
    for i in 0..count {
        let poly = path::seg(points, i).polyline(tol);
        for (k, p) in poly.iter().enumerate() {
            if k + 1 == poly.len() {
                break;
            }
            t.pts.push(*p);
            t.turn.push(k == 0 && is_turn(points, closed, i));
            t.edge_seg.push(i);
        }
        if !closed && i + 1 == count {
            if let Some(last) = poly.last() {
                t.pts.push(*last);
                t.turn.push(true);
            }
        }
    }
    t
}

fn intersect(p1: V2, d1: V2, p2: V2, d2: V2) -> Option<V2> {
    let c = cross(d1, d2);
    if c.abs() < 1e-9 {
        return None;
    }
    let t = cross(sub(p2, p1), d2) / c;
    Some(add(p1, mul(d1, t)))
}

fn clean(t: &Trace) -> Trace {
    let mut out = Trace::default();
    for k in 0..t.pts.len() {
        if let Some(last) = out.pts.last() {
            if dist(*last, t.pts[k]) < 1e-9 {
                if let Some(f) = out.turn.last_mut() {
                    *f |= t.turn[k];
                }
                continue;
            }
        }
        out.pts.push(t.pts[k]);
        out.turn.push(t.turn[k]);
        out.edge_seg.push(t.edge_seg[k]);
    }
    if out.pts.len() > 1 && dist(out.pts[0], out.pts[out.pts.len() - 1]) < 1e-9 {
        out.pts.pop();
        out.turn.pop();
        out.edge_seg.pop();
    }
    out
}

pub fn offset(source: &Trace, widths: &[f64], style: CornerStyle, tol: f64) -> Trace {
    let t = clean(source);
    let n = t.pts.len();
    let mut out = Trace::default();
    if n < 3 {
        return out;
    }
    let dirs: Vec<V2> = (0..n).map(|k| unit(sub(t.pts[(k + 1) % n], t.pts[k])).unwrap_or([1.0, 0.0])).collect();
    let width = |k: usize| widths.get(t.edge_seg[k]).copied().unwrap_or(0.0);
    let mut push = |p: V2, turn: bool| {
        out.pts.push(p);
        out.turn.push(turn);
    };
    for k in 0..n {
        let prev = (k + n - 1) % n;
        let (d0, d1) = (dirs[prev], dirs[k]);
        let (w0, w1) = (width(prev), width(k));
        let (n0, n1) = ([d0[1], -d0[0]], [d1[1], -d1[0]]);
        let p = t.pts[k];
        let (a, b) = (add(p, mul(n0, w0)), add(p, mul(n1, w1)));
        let cr = cross(d0, d1);
        let angle = cr.atan2(dot(d0, d1));
        let flag = t.turn[k];
        if angle.abs() < 1e-7 && (w0 - w1).abs() < 1e-9 {
            push(a, flag);
            continue;
        }
        let miter = intersect(a, d0, b, d1);
        let reach = w0.abs().max(w1.abs());
        if cr <= 0.0 {
            match miter {
                Some(x) if dist(x, p) <= CONCAVE_LIMIT * reach.max(1.0) => push(x, flag),
                _ => {
                    push(a, true);
                    push(b, true);
                }
            }
            continue;
        }
        if angle < SHARP {
            match miter {
                Some(x) if dist(x, p) <= MITER_LIMIT * reach.max(1e-6) => push(x, flag),
                _ => {
                    push(a, true);
                    push(b, true);
                }
            }
            continue;
        }
        match style {
            CornerStyle::Miter | CornerStyle::Mirror => match miter {
                Some(x) if dist(x, p) <= MITER_LIMIT * reach => push(x, true),
                _ => {
                    push(a, true);
                    push(b, true);
                }
            },
            CornerStyle::Square => {
                push(a, true);
                push(b, true);
            }
            CornerStyle::Round => {
                push(a, true);
                if (w0 - w1).abs() < 1e-6 && w0 > 1e-6 {
                    let step = (2.0 * (1.0 - (tol / w0).min(0.5)).acos()).max(0.05);
                    let pieces = (angle / step).ceil().max(1.0) as usize;
                    let start = n0[1].atan2(n0[0]);
                    for m in 1..pieces {
                        let th = start + angle * m as f64 / pieces as f64;
                        push(add(p, [w0 * th.cos(), w0 * th.sin()]), false);
                    }
                }
                push(b, true);
            }
        }
    }
    out
}
