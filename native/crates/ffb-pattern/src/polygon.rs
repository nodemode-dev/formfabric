use crate::geom::*;

pub fn signed_area(poly: &[V2]) -> f64 {
    let n = poly.len();
    (0..n).map(|i| cross(poly[i], poly[(i + 1) % n])).sum::<f64>() * 0.5
}

pub fn orientation(area: f64) -> &'static str {
    if area > EPS {
        "ccw"
    } else if area < -EPS {
        "cw"
    } else {
        "degenerate"
    }
}

pub fn centroid(poly: &[V2]) -> V2 {
    let a = signed_area(poly);
    if a.abs() < EPS {
        return bounds_of(poly.iter().copied()).map_or([0.0, 0.0], center);
    }
    let n = poly.len();
    let (mut x, mut y) = (0.0, 0.0);
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let c = cross(p, q);
        x += (p[0] + q[0]) * c;
        y += (p[1] + q[1]) * c;
    }
    [x / (6.0 * a), y / (6.0 * a)]
}

pub fn contains(poly: &[V2], p: V2) -> bool {
    let n = poly.len();
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            inside = !inside;
        }
    }
    inside
}

fn on_segment(a: V2, b: V2, p: V2) -> bool {
    p[0] >= a[0].min(b[0]) - EPS && p[0] <= a[0].max(b[0]) + EPS && p[1] >= a[1].min(b[1]) - EPS && p[1] <= a[1].max(b[1]) + EPS
}

pub fn segments_intersect(a: V2, b: V2, c: V2, d: V2) -> bool {
    let d1 = cross(sub(d, c), sub(a, c));
    let d2 = cross(sub(d, c), sub(b, c));
    let d3 = cross(sub(b, a), sub(c, a));
    let d4 = cross(sub(b, a), sub(d, a));
    if d1 * d2 < 0.0 && d3 * d4 < 0.0 {
        return true;
    }
    (d1.abs() < EPS && on_segment(c, d, a)) || (d2.abs() < EPS && on_segment(c, d, b)) || (d3.abs() < EPS && on_segment(a, b, c)) || (d4.abs() < EPS && on_segment(a, b, d))
}

fn polylines_cross(a: &[V2], b: &[V2]) -> bool {
    let (ba, bb) = (bounds_of(a.iter().copied()), bounds_of(b.iter().copied()));
    if let (Some(x), Some(y)) = (ba, bb) {
        if x[1][0] < y[0][0] || y[1][0] < x[0][0] || x[1][1] < y[0][1] || y[1][1] < x[0][1] {
            return false;
        }
    }
    a.windows(2).any(|s| b.windows(2).any(|t| segments_intersect(s[0], s[1], t[0], t[1])))
}

pub fn self_intersections(segs: &[Vec<V2>], closed: bool) -> Vec<(usize, usize)> {
    let n = segs.len();
    let mut out = Vec::new();
    for i in 0..n {
        for j in i + 2..n {
            if closed && i == 0 && j == n - 1 {
                continue;
            }
            if polylines_cross(&segs[i], &segs[j]) {
                out.push((i, j));
            }
        }
    }
    out
}
