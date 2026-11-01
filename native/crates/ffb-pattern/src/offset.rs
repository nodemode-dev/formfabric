use crate::bezier::Seg;
use crate::geom::*;
use crate::{path, polygon, CornerStyle, Panel, Point, PointKind, SeamAllowance};

pub const FLATTEN_TOL: f64 = 0.02;
pub const FIT_TOL: f64 = 0.03;
pub const LINE_TOL: f64 = 0.01;
pub const MITER_LIMIT: f64 = 4.0;
const ARC_STEP: f64 = 6.0 * std::f64::consts::PI / 180.0;
const MERGE: f64 = 1e-7;
const SMOOTH: f64 = 1e-5;
const MAX_LOOPS: usize = 1024;

#[derive(Debug, Clone)]
pub struct Spec {
    pub widths: Vec<f64>,
    pub corners: CornerStyle,
    pub miter_limit: f64,
}

#[derive(Debug, Clone, Copy)]
struct Vtx {
    p: V2,
    knot: bool,
    t_in: Option<V2>,
    t_out: Option<V2>,
}

fn knot(p: V2, t_in: Option<V2>, t_out: Option<V2>) -> Vtx {
    Vtx { p, knot: true, t_in, t_out }
}

fn plain(p: V2) -> Vtx {
    Vtx { p, knot: false, t_in: None, t_out: None }
}

fn start_dir(s: &Seg) -> Option<V2> {
    let [p0, p1, p2, p3] = s.p;
    unit(sub(p1, p0)).or_else(|| unit(sub(p2, p0))).or_else(|| unit(sub(p3, p0)))
}

fn end_dir(s: &Seg) -> Option<V2> {
    let [p0, p1, p2, p3] = s.p;
    unit(sub(p3, p2)).or_else(|| unit(sub(p3, p1))).or_else(|| unit(sub(p3, p0)))
}

fn right(dir: V2, side: f64) -> V2 {
    mul([dir[1], -dir[0]], side)
}

struct SegOffset {
    verts: Vec<Vtx>,
    a: V2,
    b: V2,
}

fn offset_segment(s: &Seg, d: f64, side: f64) -> Option<SegOffset> {
    let mut poly = s.polyline(FLATTEN_TOL);
    poly.dedup_by(|x, y| dist(*x, *y) < MERGE);
    if poly.len() < 2 {
        return None;
    }
    let (a, b) = (start_dir(s)?, end_dir(s)?);
    let m = poly.len();
    let edges: Vec<V2> = (0..m - 1).map(|k| right(unit(sub(poly[k + 1], poly[k])).unwrap_or(a), side)).collect();
    let mut verts = Vec::with_capacity(m);
    for (k, q) in poly.iter().enumerate() {
        let n = if k == 0 {
            right(a, side)
        } else if k == m - 1 {
            right(b, side)
        } else {
            let avg = unit(add(edges[k - 1], edges[k])).unwrap_or(edges[k]);
            mul(avg, 1.0 / dot(avg, edges[k]).max(0.25))
        };
        let p = add(*q, mul(n, d));
        verts.push(if k == 0 {
            knot(p, None, Some(a))
        } else if k == m - 1 {
            knot(p, Some(b), None)
        } else {
            plain(p)
        });
    }
    Some(SegOffset { verts, a, b })
}

struct Corner {
    c: V2,
    a: V2,
    b: V2,
    d_in: f64,
    d_out: f64,
    side: f64,
}

fn miter_point(pa: V2, a: V2, pb: V2, b: V2) -> Option<(V2, f64, f64)> {
    let cr = cross(a, b);
    if cr.abs() < 1e-9 {
        return None;
    }
    let delta = sub(pb, pa);
    let s = cross(delta, b) / cr;
    let u = cross(delta, a) / cr;
    Some((add(pa, mul(a, s)), s, -u))
}

fn join(k: &Corner, style: CornerStyle, limit: f64, out: &mut Vec<Vtx>) -> Option<V2> {
    let (na, nb) = (right(k.a, k.side), right(k.b, k.side));
    let (pa, pb) = (add(k.c, mul(na, k.d_in)), add(k.c, mul(nb, k.d_out)));
    let cr = cross(k.a, k.b);
    let wide = k.d_in.abs().max(k.d_out.abs());
    let gap = cr * (k.d_in + k.d_out).signum() * k.side > SMOOTH || (cr.abs() <= SMOOTH && dot(k.a, k.b) < 0.0);
    if !gap || wide < MERGE {
        return None;
    }
    let m = miter_point(pa, k.a, pb, k.b).filter(|(_, s, back)| *s >= 0.0 && *back >= 0.0);
    match style {
        CornerStyle::Miter => {
            if let Some((p, _, _)) = m.filter(|(p, _, _)| dist(k.c, *p) <= limit * wide) {
                out.push(knot(p, Some(k.a), Some(k.b)));
                return None;
            }
            square(k, pa, pb, wide, m, out);
            None
        }
        CornerStyle::Square => {
            square(k, pa, pb, wide, m, out);
            None
        }
        CornerStyle::Mirror => {
            mirror(k, pa, pb, m, out);
            None
        }
        CornerStyle::Round => round(k, na, nb, out),
    }
}

fn square(k: &Corner, pa: V2, pb: V2, wide: f64, m: Option<(V2, f64, f64)>, out: &mut Vec<Vtx>) {
    let (ea, eb) = m.map_or((wide, wide), |(_, s, back)| (s.min(wide), back.min(wide)));
    out.push(knot(add(pa, mul(k.a, ea)), Some(k.a), None));
    out.push(knot(sub(pb, mul(k.b, eb)), None, Some(k.b)));
}

fn mirror(k: &Corner, pa: V2, pb: V2, m: Option<(V2, f64, f64)>, out: &mut Vec<Vtx>) {
    let Some((tip, s, back)) = m else {
        return;
    };
    let Some(u) = unit(sub(tip, k.c)) else {
        return;
    };
    let h = (0.5 * (k.d_in.abs() + k.d_out.abs())).min(dist(k.c, tip));
    let along = |p: V2, dir: V2| {
        let slope = dot(dir, u);
        (slope.abs() > 1e-9).then(|| (h - dot(sub(p, k.c), u)) / slope)
    };
    let (Some(sa), Some(sb)) = (along(pa, k.a), along(pb, k.b)) else {
        return;
    };
    let (sa, sb) = (sa.clamp(0.0, s), (-sb).clamp(0.0, back));
    out.push(knot(add(pa, mul(k.a, sa)), Some(k.a), None));
    out.push(knot(sub(pb, mul(k.b, sb)), None, Some(k.b)));
}

fn round(k: &Corner, na: V2, nb: V2, out: &mut Vec<Vtx>) -> Option<V2> {
    let delta = {
        let raw = nb[1].atan2(nb[0]) - na[1].atan2(na[0]);
        (raw + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI
    };
    let steps = (delta.abs() / ARC_STEP).ceil().max(1.0) as usize;
    for i in 1..steps {
        let f = i as f64 / steps as f64;
        let r = k.d_in + (k.d_out - k.d_in) * f;
        out.push(plain(add(k.c, mul(rotate(na, (delta * f).to_degrees()), r))));
    }
    Some(k.b)
}

fn dedupe(v: &mut Vec<Vtx>) {
    let mut out: Vec<Vtx> = Vec::with_capacity(v.len());
    for x in v.drain(..) {
        match out.last_mut() {
            Some(l) if dist(l.p, x.p) < MERGE => {
                l.knot |= x.knot;
                l.t_out = x.t_out.or(l.t_out);
                l.t_in = l.t_in.or(x.t_in);
            }
            _ => out.push(x),
        }
    }
    while out.len() > 1 && dist(out[0].p, out[out.len() - 1].p) < MERGE {
        if let Some(l) = out.pop() {
            out[0].knot |= l.knot;
            out[0].t_in = l.t_in.or(out[0].t_in);
        }
    }
    *v = out;
}

fn cross_point(a: V2, b: V2, c: V2, d: V2) -> Option<V2> {
    let (r, s) = (sub(b, a), sub(d, c));
    let den = cross(r, s);
    if den.abs() < 1e-12 {
        return None;
    }
    let qp = sub(c, a);
    let t = cross(qp, s) / den;
    let u = cross(qp, r) / den;
    (t > 1e-9 && t < 1.0 - 1e-9 && u > 1e-9 && u < 1.0 - 1e-9).then(|| add(a, mul(r, t)))
}

fn find_crossing(v: &[Vtx]) -> Option<(usize, usize, V2)> {
    let n = v.len();
    for i in 0..n {
        let (a, b) = (v[i].p, v[(i + 1) % n].p);
        let (lo_x, hi_x, lo_y, hi_y) = (a[0].min(b[0]), a[0].max(b[0]), a[1].min(b[1]), a[1].max(b[1]));
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (c, d) = (v[j].p, v[(j + 1) % n].p);
            if c[0].max(d[0]) < lo_x || c[0].min(d[0]) > hi_x || c[1].max(d[1]) < lo_y || c[1].min(d[1]) > hi_y {
                continue;
            }
            if let Some(x) = cross_point(a, b, c, d) {
                return Some((i, j, x));
            }
        }
    }
    None
}

fn signed(w: &[Vtx], sign: f64) -> f64 {
    let pts: Vec<V2> = w.iter().map(|q| q.p).collect();
    polygon::signed_area(&pts) * sign
}

fn clean(mut v: Vec<Vtx>, sign: f64) -> Vec<Vtx> {
    for _ in 0..MAX_LOOPS {
        let Some((i, j, x)) = find_crossing(&v) else {
            break;
        };
        let cut = knot(x, None, None);
        let mut inner = vec![cut];
        inner.extend_from_slice(&v[i + 1..=j]);
        let mut rest: Vec<Vtx> = v[..=i].to_vec();
        rest.push(cut);
        rest.extend_from_slice(&v[j + 1..]);
        let (ai, ar) = (signed(&inner, sign), signed(&rest, sign));
        v = if ai < 0.0 || (ar >= 0.0 && ai < ar) { rest } else { inner };
        dedupe(&mut v);
        if v.len() < 3 {
            break;
        }
    }
    v
}

fn bez_at(b: &[V2; 4], t: f64) -> V2 {
    Seg { p: *b, curved: true }.at(t)
}

fn chord_params(d: &[V2]) -> Vec<f64> {
    let mut u = vec![0.0; d.len()];
    for i in 1..d.len() {
        u[i] = u[i - 1] + dist(d[i], d[i - 1]);
    }
    let total = u[d.len() - 1];
    if total > EPS {
        u.iter_mut().for_each(|x| *x /= total);
    }
    u
}

fn generate(d: &[V2], u: &[f64], t1: V2, t2: V2) -> [V2; 4] {
    let (first, last) = (d[0], d[d.len() - 1]);
    let mut c = [[0.0; 2]; 2];
    let mut x = [0.0; 2];
    for (q, &t) in d.iter().zip(u) {
        let m = 1.0 - t;
        let b = [m * m * m, 3.0 * t * m * m, 3.0 * t * t * m, t * t * t];
        let (a1, a2) = (mul(t1, b[1]), mul(t2, b[2]));
        c[0][0] += dot(a1, a1);
        c[0][1] += dot(a1, a2);
        c[1][1] += dot(a2, a2);
        let tmp = sub(*q, add(mul(first, b[0] + b[1]), mul(last, b[2] + b[3])));
        x[0] += dot(a1, tmp);
        x[1] += dot(a2, tmp);
    }
    c[1][0] = c[0][1];
    let det = c[0][0] * c[1][1] - c[1][0] * c[0][1];
    let span = dist(first, last);
    let (mut al, mut ar) = (0.0, 0.0);
    if det.abs() > 1e-12 {
        al = (x[0] * c[1][1] - x[1] * c[0][1]) / det;
        ar = (c[0][0] * x[1] - c[1][0] * x[0]) / det;
    }
    if al < 1e-6 * span || ar < 1e-6 * span {
        al = span / 3.0;
        ar = span / 3.0;
    }
    [first, add(first, mul(t1, al)), add(last, mul(t2, ar)), last]
}

fn newton(b: &[V2; 4], p: V2, u: f64) -> f64 {
    let s = Seg { p: *b, curved: true };
    let (q, q1) = (s.at(u), s.tangent(u));
    let m = 1.0 - u;
    let q2 = mul(add(mul(add(sub(b[2], mul(b[1], 2.0)), b[0]), m), mul(add(sub(b[3], mul(b[2], 2.0)), b[1]), u)), 6.0);
    let diff = sub(q, p);
    let den = dot(q1, q1) + dot(diff, q2);
    if den.abs() < 1e-12 {
        u
    } else {
        (u - dot(diff, q1) / den).clamp(0.0, 1.0)
    }
}

fn worst(d: &[V2], u: &[f64], b: &[V2; 4]) -> (f64, usize) {
    let mut split = d.len() / 2;
    let mut max = 0.0;
    for i in 1..d.len() - 1 {
        let e = {
            let r = sub(bez_at(b, u[i]), d[i]);
            dot(r, r)
        };
        if e >= max {
            max = e;
            split = i;
        }
    }
    (max, split)
}

fn fit_cubic(d: &[V2], t1: V2, t2: V2, tol: f64, out: &mut Vec<[V2; 4]>) {
    let n = d.len();
    if n == 2 {
        let span = dist(d[0], d[1]) / 3.0;
        out.push([d[0], add(d[0], mul(t1, span)), add(d[1], mul(t2, span)), d[1]]);
        return;
    }
    let mut u = chord_params(d);
    let mut b = generate(d, &u, t1, t2);
    let (mut err, mut split) = worst(d, &u, &b);
    if err < tol * tol {
        out.push(b);
        return;
    }
    if err < 16.0 * tol * tol {
        for _ in 0..4 {
            u = d.iter().zip(&u).map(|(q, &t)| newton(&b, *q, t)).collect();
            b = generate(d, &u, t1, t2);
            (err, split) = worst(d, &u, &b);
            if err < tol * tol {
                out.push(b);
                return;
            }
        }
    }
    let centre = unit(sub(d[split - 1], d[split + 1])).or_else(|| unit(sub(d[split - 1], d[split]))).unwrap_or(t1);
    fit_cubic(&d[..=split], t1, centre, tol, out);
    fit_cubic(&d[split..], mul(centre, -1.0), t2, tol, out);
}

fn straight(pts: &[V2]) -> bool {
    let (a, b) = (pts[0], pts[pts.len() - 1]);
    dist(a, b) > EPS && pts.iter().all(|q| dist_to_line(*q, a, b) <= LINE_TOL)
}

fn fit_run(pts: &[V2], t_in: Option<V2>, t_out: Option<V2>, out: &mut Vec<([V2; 4], bool)>) {
    let (first, last) = (pts[0], pts[pts.len() - 1]);
    if pts.len() == 2 || straight(pts) {
        out.push(([first, lerp(first, last, 1.0 / 3.0), lerp(first, last, 2.0 / 3.0), last], false));
        return;
    }
    let n = pts.len();
    let t1 = t_out.or_else(|| unit(sub(pts[1], pts[0]))).unwrap_or([1.0, 0.0]);
    let t2 = t_in.map(|t| mul(t, -1.0)).or_else(|| unit(sub(pts[n - 2], pts[n - 1]))).unwrap_or([-1.0, 0.0]);
    let mut fitted = Vec::new();
    fit_cubic(pts, t1, t2, FIT_TOL, &mut fitted);
    out.extend(fitted.into_iter().map(|b| (b, true)));
}

fn build_points(v: &[Vtx]) -> Vec<Point> {
    let first = v.iter().position(|q| q.knot).unwrap_or(0);
    let mut w: Vec<Vtx> = v[first..].iter().chain(v[..first].iter()).copied().collect();
    w[0].knot = true;
    let knots: Vec<usize> = (0..w.len()).filter(|&i| w[i].knot).collect();
    let mut beziers: Vec<([V2; 4], bool)> = Vec::new();
    for (k, &from) in knots.iter().enumerate() {
        let to = knots.get(k + 1).copied().unwrap_or(w.len());
        let mut pts: Vec<V2> = w[from..to.min(w.len())].iter().map(|q| q.p).collect();
        pts.push(if to >= w.len() { w[0].p } else { w[to].p });
        let end = if to >= w.len() { &w[0] } else { &w[to] };
        fit_run(&pts, end.t_in, w[from].t_out, &mut beziers);
    }
    let m = beziers.len();
    let mut points = Vec::with_capacity(m);
    for k in 0..m {
        let (b, cubic) = &beziers[k];
        let (pb, pcubic) = &beziers[(k + m - 1) % m];
        let mut p = Point { p: b[0], kind: PointKind::Corner, h_in: None, h_out: None };
        if *cubic {
            p.h_out = Some(sub(b[1], b[0]));
        }
        if *pcubic {
            p.h_in = Some(sub(pb[2], pb[3]));
        }
        if let (Some(i), Some(o)) = (p.h_in, p.h_out) {
            if len(i) > EPS && len(o) > EPS && cross(unit(i).unwrap_or(i), unit(o).unwrap_or(o)).abs() < 1e-3 && dot(i, o) < 0.0 {
                p.kind = PointKind::Smooth;
            }
        }
        points.push(p);
    }
    points
}

fn thin(points: &mut Vec<Point>) {
    loop {
        let n = points.len();
        if n <= 3 {
            return;
        }
        let drop = (0..n).find(|&i| {
            let (prev, p, next) = (&points[(i + n - 1) % n], &points[i], &points[(i + 1) % n]);
            p.h_in.is_none()
                && p.h_out.is_none()
                && prev.h_out.is_none()
                && next.h_in.is_none()
                && dist_to_line(p.p, prev.p, next.p) < 1e-4
                && dot(sub(p.p, prev.p), sub(next.p, p.p)) > 0.0
        });
        match drop {
            Some(i) => {
                points.remove(i);
            }
            None => return,
        }
    }
}

pub fn offset_closed(points: &[Point], spec: &Spec) -> Result<Vec<Point>, String> {
    let n = points.len();
    if n < 3 {
        return Err("an outline needs at least 3 points".into());
    }
    if spec.widths.len() != n {
        return Err("there must be one width per segment".into());
    }
    let segs = path::segments(points, true);
    let area = polygon::signed_area(&path::flatten(points, true, FLATTEN_TOL));
    if area.abs() < EPS {
        return Err("the outline encloses no area".into());
    }
    let side = area.signum();
    let offsets: Vec<SegOffset> = segs
        .iter()
        .zip(&spec.widths)
        .enumerate()
        .map(|(i, (s, w))| offset_segment(s, *w, side).ok_or_else(|| format!("segment {i} has no length")))
        .collect::<Result<_, _>>()?;
    let mut verts: Vec<Vtx> = Vec::new();
    for i in 0..n {
        let j = (i + 1) % n;
        verts.extend(offsets[i].verts.iter().copied());
        let corner = Corner { c: points[j].p, a: offsets[i].b, b: offsets[j].a, d_in: spec.widths[i], d_out: spec.widths[j], side };
        let mut extra = Vec::new();
        if let Some(t_in) = join(&corner, spec.corners, spec.miter_limit, &mut extra) {
            if let Some(last) = verts.last_mut() {
                last.t_out = Some(corner.a);
            }
            verts.extend(extra);
            let next = &offsets[j].verts[0];
            let mut patched = *next;
            patched.t_in = Some(t_in);
            verts.push(Vtx { p: patched.p, knot: true, t_in: patched.t_in, t_out: patched.t_out });
            continue;
        }
        verts.extend(extra);
    }
    dedupe(&mut verts);
    let mut verts = clean(verts, area.signum());
    dedupe(&mut verts);
    if verts.len() < 3 || find_crossing(&verts).is_some() || signed(&verts, area.signum()) < 1.0 {
        return Err("the offset collapses the outline or cannot be resolved; use a smaller distance".into());
    }
    let mut out = build_points(&verts);
    thin(&mut out);
    Ok(out)
}

pub fn widths(panel: &Panel, sa: &SeamAllowance) -> Vec<f64> {
    (0..panel.outline.len()).map(|i| sa.per_segment.get(i).copied().flatten().unwrap_or(sa.width)).collect()
}

pub fn seam_outline(panel: &Panel) -> Option<Result<Vec<Point>, String>> {
    let sa = panel.seam_allowance.as_ref()?;
    let spec = Spec { widths: widths(panel, sa), corners: sa.corners, miter_limit: MITER_LIMIT };
    Some(offset_closed(&panel.outline, &spec))
}

pub fn split_widths(sa: &mut SeamAllowance, segment: usize, pieces: usize) {
    if let Some(&w) = sa.per_segment.get(segment) {
        for _ in 1..pieces {
            sa.per_segment.insert(segment + 1, w);
        }
    }
}

pub fn remove_width(sa: &mut SeamAllowance, index: usize) {
    if index < sa.per_segment.len() {
        sa.per_segment.remove(index);
    }
}

pub fn reverse_widths(sa: &mut SeamAllowance, n: usize) {
    if sa.per_segment.is_empty() || n < 2 {
        return;
    }
    let full: Vec<Option<f64>> = (0..n).map(|i| sa.per_segment.get(i).copied().flatten()).collect();
    sa.per_segment = (0..n).map(|k| full[(2 * n - 2 - k) % n]).collect();
}
