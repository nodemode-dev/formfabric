use crate::geom::*;
use crate::path::auto_handles;
use crate::{Point, PointKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    In,
    Out,
}

pub fn put_handle(points: &mut [Point], i: usize, which: Which, v: Option<V2>, independent: bool) {
    let p = &mut points[i];
    let (this, other) = match which {
        Which::In => (&mut p.h_in, &mut p.h_out),
        Which::Out => (&mut p.h_out, &mut p.h_in),
    };
    *this = v;
    let Some(v) = v else {
        if p.kind == PointKind::Curve {
            p.kind = PointKind::Corner;
        }
        return;
    };
    if independent || p.kind == PointKind::Corner {
        p.kind = PointKind::Corner;
        return;
    }
    p.kind = PointKind::Smooth;
    let reach = other.map_or_else(|| len(v), len);
    *other = unit(v).map(|d| mul(d, -reach));
}

pub fn set_kind(points: &mut [Point], i: usize, kind: PointKind, closed: bool) {
    match kind {
        PointKind::Corner => {
            points[i].kind = kind;
            points[i].h_in = None;
            points[i].h_out = None;
        }
        PointKind::Curve => {
            points[i].kind = kind;
            auto_handles(points, closed);
        }
        PointKind::Smooth => {
            if points[i].h_in.is_none() && points[i].h_out.is_none() {
                points[i].kind = PointKind::Curve;
                auto_handles(points, closed);
            }
            let (which, v) = match (points[i].h_out, points[i].h_in) {
                (Some(v), _) => (Which::Out, v),
                (None, Some(v)) => (Which::In, v),
                _ => (Which::Out, [0.0, 0.0]),
            };
            points[i].kind = PointKind::Smooth;
            if len(v) > EPS {
                put_handle(points, i, which, Some(v), false);
            }
        }
    }
}

pub fn curvature(points: &mut [Point], i: usize, amount: f64) {
    let j = (i + 1) % points.len();
    let (a, b) = (points[i].p, points[j].p);
    if amount.abs() < EPS {
        put_handle(points, i, Which::Out, None, false);
        put_handle(points, j, Which::In, None, false);
        return;
    }
    let Some(dir) = unit(sub(b, a)) else { return };
    let right = [dir[1], -dir[0]];
    let lift = mul(right, amount / 0.75);
    let third = mul(sub(b, a), 1.0 / 3.0);
    put_handle(points, i, Which::Out, Some(add(third, lift)), false);
    put_handle(points, j, Which::In, Some(add(mul(third, -1.0), lift)), false);
}

fn is_line(points: &[Point], a: usize, b: usize) -> bool {
    points[a].h_out.is_none() && points[b].h_in.is_none()
}

pub fn round_corners(points: &mut Vec<Point>, closed: bool, indices: &[usize], length: f64, curvature: f64) -> usize {
    let mut order: Vec<usize> = indices.to_vec();
    order.sort_unstable_by(|a, b| b.cmp(a));
    order.dedup();
    let mut done = 0;
    for i in order {
        let n = points.len();
        if i >= n || (!closed && (i == 0 || i + 1 == n)) {
            continue;
        }
        let (a, b) = ((i + n - 1) % n, (i + 1) % n);
        if !is_line(points, a, i) || !is_line(points, i, b) {
            continue;
        }
        let p = points[i].p;
        let (Some(da), Some(db)) = (unit(sub(points[a].p, p)), unit(sub(points[b].p, p))) else { continue };
        let alpha = dot(da, db).clamp(-1.0, 1.0).acos();
        if !(1e-3..=std::f64::consts::PI - 1e-3).contains(&alpha) {
            continue;
        }
        let d = length.min(0.5 * dist(p, points[a].p)).min(0.5 * dist(p, points[b].p));
        if d < 1e-6 {
            continue;
        }
        let radius = d * (alpha / 2.0).tan();
        let sweep = std::f64::consts::PI - alpha;
        let hl = 4.0 / 3.0 * (sweep / 4.0).tan() * radius * curvature;
        let handle = |dir: V2| (hl > EPS).then(|| mul(dir, -hl));
        let first = Point { p: add(p, mul(da, d)), kind: PointKind::Corner, h_in: None, h_out: handle(da) };
        let second = Point { p: add(p, mul(db, d)), kind: PointKind::Corner, h_in: handle(db), h_out: None };
        points[i] = first;
        points.insert(i + 1, second);
        done += 1;
    }
    done
}
