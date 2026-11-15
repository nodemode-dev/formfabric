use crate::bezier::Seg;
use crate::geom::*;
use crate::{Point, PointKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Near {
    pub seg: usize,
    pub t: f64,
    pub point: V2,
    pub dist: f64,
}

pub fn seg_count(n: usize, closed: bool) -> usize {
    match n {
        0 | 1 => 0,
        _ if closed => n,
        _ => n - 1,
    }
}

pub fn seg(points: &[Point], i: usize) -> Seg {
    let a = &points[i];
    let b = &points[(i + 1) % points.len()];
    if a.h_out.is_none() && b.h_in.is_none() {
        return Seg::line(a.p, b.p);
    }
    Seg { p: [a.p, add(a.p, a.h_out.unwrap_or_default()), add(b.p, b.h_in.unwrap_or_default()), b.p], curved: true }
}

pub fn segments(points: &[Point], closed: bool) -> Vec<Seg> {
    (0..seg_count(points.len(), closed)).map(|i| seg(points, i)).collect()
}

pub fn flatten_segments(points: &[Point], closed: bool, tol: f64) -> Vec<Vec<V2>> {
    segments(points, closed).iter().map(|s| s.polyline(tol)).collect()
}

pub fn flatten(points: &[Point], closed: bool, tol: f64) -> Vec<V2> {
    let mut out = Vec::new();
    for (i, s) in segments(points, closed).iter().enumerate() {
        if i == 0 {
            out.push(s.p[0]);
        }
        s.flatten_into(tol, &mut out);
    }
    if closed && out.len() > 1 {
        out.pop();
    }
    if out.is_empty() {
        out.extend(points.iter().map(|p| p.p));
    }
    out
}

pub fn length(points: &[Point], closed: bool) -> f64 {
    segments(points, closed).iter().map(Seg::length).sum()
}

pub fn bounds(points: &[Point], closed: bool) -> Option<Bounds> {
    segments(points, closed)
        .iter()
        .map(Seg::bounds)
        .reduce(union)
        .or_else(|| bounds_of(points.iter().map(|p| p.p)))
}

pub fn nearest(points: &[Point], closed: bool, q: V2) -> Option<Near> {
    segments(points, closed)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let (t, point, dist) = s.nearest(q);
            Near { seg: i, t, point, dist }
        })
        .min_by(|a, b| a.dist.total_cmp(&b.dist))
}

pub fn split_at(points: &mut Vec<Point>, i: usize, t: f64) -> usize {
    let j = (i + 1) % points.len();
    let s = seg(points, i);
    let (l, r) = s.split(t);
    let mut m = Point { p: l.p[3], kind: PointKind::Corner, h_in: None, h_out: None };
    if s.curved {
        m.kind = PointKind::Smooth;
        m.h_in = Some(sub(l.p[2], m.p));
        m.h_out = Some(sub(r.p[1], m.p));
        points[i].h_out = Some(sub(l.p[1], points[i].p));
        points[j].h_in = Some(sub(r.p[2], points[j].p));
    }
    points.insert(i + 1, m);
    i + 1
}

pub fn split_many(points: &mut Vec<Point>, i: usize, ts: &[f64]) -> Vec<usize> {
    let mut sorted: Vec<f64> = ts.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut upper = 1.0;
    for t in sorted.iter().rev() {
        let part = t / upper;
        split_at(points, i, if part.is_finite() { part.clamp(0.0, 1.0) } else { 0.0 });
        upper = *t;
    }
    (1..=sorted.len()).map(|k| i + k).collect()
}

pub fn reverse(points: &mut [Point]) {
    points.reverse();
    for p in points.iter_mut() {
        std::mem::swap(&mut p.h_in, &mut p.h_out);
    }
}

pub fn auto_handles(points: &mut [Point], closed: bool) {
    let n = points.len();
    for i in 0..n {
        if points[i].kind != PointKind::Curve {
            continue;
        }
        let ends = !closed && (i == 0 || i + 1 == n);
        let (prev, next) = (points[(i + n - 1) % n].p, points[(i + 1) % n].p);
        let dir = if ends { None } else { unit(sub(next, prev)) };
        let here = points[i].p;
        points[i].h_out = dir.map(|d| mul(d, dist(here, next) / 3.0));
        points[i].h_in = dir.map(|d| mul(d, -dist(here, prev) / 3.0));
    }
}
