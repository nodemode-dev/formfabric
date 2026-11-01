use ffb_pattern::geom::*;
use ffb_pattern::{Point, PointKind};

use super::fit::{self, Cubic};
use crate::dxf::{Poly, Vert};

const SAME: f64 = 1e-4;

#[derive(Clone, Copy)]
struct Piece {
    c: Cubic,
    line: bool,
}

pub fn arc_cubics(p0: V2, p1: V2, bulge: f64) -> Vec<Cubic> {
    let theta = 4.0 * bulge.atan();
    let chord = dist(p0, p1);
    if chord < EPS || theta.abs() < 1e-9 {
        return vec![straight(p0, p1)];
    }
    let d = mul(sub(p1, p0), 1.0 / chord);
    let left = [-d[1], d[0]];
    let mid = lerp(p0, p1, 0.5);
    let radius = chord / (2.0 * (theta / 2.0).sin().abs());
    let centre = add(mid, mul(left, (chord / 2.0) / (theta / 2.0).tan()));
    let a0 = (p0[1] - centre[1]).atan2(p0[0] - centre[0]);
    let pieces = ((theta.abs() / std::f64::consts::FRAC_PI_2).ceil() as usize).max(1);
    let phi = theta / pieces as f64;
    let k = 4.0 / 3.0 * (phi / 4.0).tan();
    (0..pieces)
        .map(|m| {
            let s = a0 + phi * m as f64;
            let e = s + phi;
            let ps = if m == 0 { p0 } else { add(centre, [radius * s.cos(), radius * s.sin()]) };
            let pe = if m + 1 == pieces { p1 } else { add(centre, [radius * e.cos(), radius * e.sin()]) };
            let ts = [-s.sin(), s.cos()];
            let te = [-e.sin(), e.cos()];
            [ps, add(ps, mul(ts, k * radius)), sub(pe, mul(te, k * radius)), pe]
        })
        .collect()
}

fn straight(a: V2, b: V2) -> Cubic {
    [a, lerp(a, b, 1.0 / 3.0), lerp(a, b, 2.0 / 3.0), b]
}

fn merged(poly: &Poly) -> (Vec<Vert>, bool) {
    let mut verts: Vec<Vert> = Vec::new();
    for v in &poly.verts {
        if let Some(last) = verts.last_mut() {
            if dist(last.p, v.p) < SAME {
                if v.bulge.abs() > 1e-9 {
                    last.bulge = v.bulge;
                }
                continue;
            }
        }
        verts.push(v.clone());
    }
    let mut closed = poly.closed;
    if verts.len() > 2 && dist(verts[0].p, verts[verts.len() - 1].p) < SAME {
        verts.pop();
        closed = true;
    }
    (verts, closed)
}

pub fn poly_points(poly: &Poly, tol: f64) -> Option<(Vec<Point>, bool)> {
    let (verts, closed) = merged(poly);
    let n = verts.len();
    if n < 2 || (closed && n == 2 && verts.iter().all(|v| v.bulge.abs() < 1e-9)) {
        return None;
    }
    let bulged = |i: usize| verts[i % n].bulge.abs() > 1e-9;
    let mut breaks: Vec<bool> = (0..n)
        .map(|i| {
            let open_end = !closed && (i == 0 || i + 1 == n);
            let before = (closed || i > 0) && bulged((i + n - 1) % n);
            let after = (closed || i + 1 < n) && bulged(i);
            open_end || !verts[i].curve || before || after
        })
        .collect();
    let mut smooth = vec![false; n];
    if closed && !breaks.iter().any(|b| *b) {
        breaks[0] = true;
        smooth[0] = true;
        if n >= 6 {
            breaks[n / 2] = true;
            smooth[n / 2] = true;
        }
    }
    let stops: Vec<usize> = (0..n).filter(|i| breaks[*i]).collect();
    let runs = if closed { stops.len() } else { stops.len().saturating_sub(1) };
    let central = |i: usize| unit(sub(verts[(i + 1) % n].p, verts[(i + n - 1) % n].p));
    let mut pieces: Vec<Piece> = Vec::new();
    for k in 0..runs {
        let (i, j) = (stops[k], stops[(k + 1) % stops.len()]);
        let steps = if closed {
            match (j + n - i) % n {
                0 => n,
                s => s,
            }
        } else {
            j - i
        };
        if steps == 1 {
            let p0 = verts[i].p;
            let p1 = verts[j % n].p;
            if bulged(i) {
                pieces.extend(arc_cubics(p0, p1, verts[i].bulge).into_iter().map(|c| Piece { c, line: false }));
            } else {
                pieces.push(Piece { c: straight(p0, p1), line: true });
            }
            continue;
        }
        let pts: Vec<V2> = (0..=steps).map(|m| verts[(i + m) % n].p).collect();
        let t1 = if smooth[i] { central(i).unwrap_or_else(|| fit::start_tangent(&pts)) } else { fit::start_tangent(&pts) };
        let t2 = if smooth[j % n] { central(j % n).map(|c| mul(c, -1.0)).unwrap_or_else(|| fit::end_tangent(&pts)) } else { fit::end_tangent(&pts) };
        pieces.extend(fit::fit(&pts, t1, t2, tol).into_iter().map(|c| Piece { c, line: false }));
    }
    if pieces.is_empty() {
        return None;
    }
    let m = pieces.len();
    let mut points: Vec<Point> = pieces.iter().map(|pc| Point { p: pc.c[0], ..Point::default() }).collect();
    if !closed {
        points.push(Point { p: pieces[m - 1].c[3], ..Point::default() });
    }
    for (k, pc) in pieces.iter().enumerate() {
        if pc.line {
            continue;
        }
        let end = if closed { (k + 1) % m } else { k + 1 };
        let out = sub(pc.c[1], pc.c[0]);
        let inn = sub(pc.c[2], pc.c[3]);
        points[k].h_out = (len(out) > SAME).then_some(out);
        points[end].h_in = (len(inn) > SAME).then_some(inn);
    }
    for p in points.iter_mut() {
        if let (Some(a), Some(b)) = (p.h_in, p.h_out) {
            if let (Some(ua), Some(ub)) = (unit(a), unit(b)) {
                if cross(ua, ub).abs() < 0.03 && dot(ua, ub) < 0.0 {
                    p.kind = PointKind::Smooth;
                }
            }
        }
    }
    Some((points, closed))
}
