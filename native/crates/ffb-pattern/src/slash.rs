use crate::dart::{expand, remap, validate_outline};
use crate::geom::*;
use crate::path;
use crate::polygon;
use crate::{Panel, Point, PointKind};

const TOL: f64 = 0.05;
const ON_LINE: f64 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub struct SlashSpec {
    pub from: V2,
    pub to: V2,
    pub hinge_from: bool,
    pub angle: Option<f64>,
    pub spread: Option<f64>,
    pub side: Side,
    pub smooth: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SlashReport {
    pub hinge: V2,
    pub far: V2,
    pub angle: f64,
    pub length: f64,
    pub opening: f64,
    pub area_added: f64,
}

struct Crossing {
    seg: usize,
    t: f64,
    p: V2,
    s: f64,
}

enum Loc {
    Vertex(usize),
    Split(usize, f64),
}

fn crossings(points: &[Point], origin: V2, dir: V2) -> Vec<Crossing> {
    const STEPS: usize = 48;
    let mut out = Vec::new();
    for i in 0..points.len() {
        let seg = path::seg(points, i);
        let f = |t: f64| cross(dir, sub(seg.at(t), origin));
        let mut prev = 0.0;
        let mut fp = f(0.0);
        for k in 1..=STEPS {
            let t = k as f64 / STEPS as f64;
            let fc = f(t);
            if (fp >= 0.0) != (fc >= 0.0) {
                let (mut lo, mut hi) = (prev, t);
                for _ in 0..60 {
                    let mid = 0.5 * (lo + hi);
                    if (f(mid) >= 0.0) == (fp >= 0.0) {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let tt = 0.5 * (lo + hi);
                let p = seg.at(tt);
                out.push(Crossing { seg: i, t: tt, p, s: dot(dir, sub(p, origin)) });
            }
            prev = t;
            fp = fc;
        }
    }
    out
}

fn locate(n: usize, c: &Crossing) -> Loc {
    if c.t < 1e-9 {
        Loc::Vertex(c.seg)
    } else if c.t > 1.0 - 1e-9 {
        Loc::Vertex((c.seg + 1) % n)
    } else {
        Loc::Split(c.seg, c.t)
    }
}

fn spin(p: V2, pivot: V2, degrees: f64) -> V2 {
    add(pivot, rotate(sub(p, pivot), degrees))
}

fn turned(pt: &Point, pivot: V2, degrees: f64) -> Point {
    Point { p: spin(pt.p, pivot, degrees), kind: pt.kind, h_in: pt.h_in.map(|h| rotate(h, degrees)), h_out: pt.h_out.map(|h| rotate(h, degrees)) }
}

fn chain_side(ring: &[Point], start: usize, count: usize, hinge: V2, dir: V2) -> f64 {
    let m = ring.len();
    let mut best = 0.0f64;
    for k in 0..count {
        for p in path::seg(ring, (start + k) % m).polyline(0.5) {
            let c = cross(dir, sub(p, hinge));
            if c.abs() > best.abs() {
                best = c;
            }
        }
    }
    best
}

fn unit_or(v: V2, fallback: V2) -> V2 {
    unit(v).or_else(|| unit(fallback)).unwrap_or([1.0, 0.0])
}

pub fn spread(panel: &mut Panel, spec: &SlashSpec) -> Result<SlashReport, String> {
    let n = panel.outline.len();
    if n < 3 {
        return Err("the panel has no outline".into());
    }
    let dir0 = unit(sub(spec.to, spec.from)).ok_or("the slash line has no length")?;
    let found = crossings(&panel.outline, spec.from, dir0);
    if found.len() < 2 {
        return Err("the slash line does not cross the outline twice; it must run from edge to edge".into());
    }
    let nearest = |q: V2, skip: Option<usize>| {
        found.iter().enumerate().filter(|(i, _)| Some(*i) != skip).min_by(|a, b| dist(a.1.p, q).total_cmp(&dist(b.1.p, q))).map(|(i, _)| i)
    };
    let ia = nearest(spec.from, None).ok_or("no crossing found")?;
    let ib = nearest(spec.to, Some(ia)).ok_or("no crossing found")?;
    let (xa, xb) = (&found[ia], &found[ib]);
    if dist(xa.p, xb.p) < 1e-3 {
        return Err("both ends of the slash line meet the outline at the same place".into());
    }
    let (lo, hi) = (xa.s.min(xb.s), xa.s.max(xb.s));
    if found.iter().enumerate().any(|(i, c)| i != ia && i != ib && c.s > lo + 1e-6 && c.s < hi - 1e-6) {
        return Err("the slash line leaves the panel before it reaches the second edge".into());
    }
    let locs = [locate(n, xa), locate(n, xb)];
    let mut order: Vec<(usize, f64)> = locs.iter().filter_map(|l| if let Loc::Split(s, t) = l { Some((*s, *t)) } else { None }).collect();
    order.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut ring = panel.outline.clone();
    if order.len() == 2 && order[0].0 == order[1].0 {
        path::split_many(&mut ring, order[0].0, &[order[0].1, order[1].1]);
    } else {
        for (s, t) in order.iter().rev() {
            path::split_at(&mut ring, *s, *t);
        }
    }
    let fin = |loc: &Loc| match loc {
        Loc::Vertex(i) => i + order.iter().filter(|s| s.0 < *i).count(),
        Loc::Split(sg, t) => sg + 1 + order.iter().filter(|s| s.0 < *sg).count() + order.iter().filter(|s| s.0 == *sg && s.1 < *t).count(),
    };
    let (fa, fb) = (fin(&locs[0]), fin(&locs[1]));
    let m = ring.len();
    let (hi_i, fi) = if spec.hinge_from { (fa, fb) } else { (fb, fa) };
    let (hinge, far) = (ring[hi_i].p, ring[fi].p);
    let length = dist(hinge, far);
    let dir = unit(sub(far, hinge)).ok_or("the slash line has no length")?;
    let theta = match (spec.angle, spec.spread) {
        (Some(a), _) => a,
        (None, Some(sp)) => 2.0 * (sp / (2.0 * length)).clamp(-1.0, 1.0).asin().to_degrees(),
        (None, None) => return Err("give a spread distance or an angle".into()),
    };
    if theta.abs() < 1e-6 {
        return Err("the spread is zero".into());
    }
    let out_n = (fi + m - hi_i) % m;
    let back_n = m - out_n;
    let out_side = chain_side(&ring, hi_i, out_n, hinge, dir);
    let back_side = chain_side(&ring, fi, back_n, hinge, dir);
    if out_side * back_side >= 0.0 {
        return Err("the slash line does not divide the outline into two parts".into());
    }
    let (left, right) = match spec.side {
        Side::Left => (theta, 0.0),
        Side::Right => (0.0, -theta),
        Side::Both => (theta / 2.0, -theta / 2.0),
    };
    let a_out = if out_side > 0.0 { left } else { right };
    let a_back = if back_side > 0.0 { left } else { right };

    let mut inner_turn = Vec::with_capacity(panel.inner.len());
    for inner in &panel.inner {
        let sides: Vec<f64> = inner.points.iter().map(|p| cross(dir, sub(p.p, hinge))).collect();
        let a = if sides.iter().all(|c| c.abs() <= ON_LINE) {
            0.0
        } else if sides.iter().all(|c| *c >= -ON_LINE) {
            left
        } else if sides.iter().all(|c| *c <= ON_LINE) {
            right
        } else {
            return Err(format!("inner shape {} crosses the slash line", inner.id));
        };
        inner_turn.push(a);
    }

    let mut new: Vec<Point> = Vec::with_capacity(m + 1);
    for k in 0..=out_n {
        new.push(turned(&ring[(hi_i + k) % m], hinge, a_out));
    }
    for k in 0..back_n {
        new.push(turned(&ring[(fi + k) % m], hinge, a_back));
    }
    new[0].h_in = ring[hi_i].h_in.map(|h| rotate(h, a_back));
    if (a_out - a_back).abs() > 1e-9 {
        new[0].kind = PointKind::Corner;
    }
    let gap = dist(new[out_n].p, new[out_n + 1].p);
    if gap < 1e-6 {
        return Err("the spread is too small to open the cut".into());
    }
    new[out_n].kind = PointKind::Corner;
    new[out_n + 1].kind = PointKind::Corner;
    new[out_n].h_out = None;
    new[out_n + 1].h_in = None;
    if spec.smooth {
        let t_out = unit_or(path::seg(&new, out_n - 1).tangent(1.0), sub(new[out_n].p, new[out_n - 1].p));
        let t_back = unit_or(path::seg(&new, out_n + 1).tangent(0.0), sub(new[(out_n + 2) % new.len()].p, new[out_n + 1].p));
        new[out_n].h_out = Some(mul(t_out, gap / 3.0));
        new[out_n + 1].h_in = Some(mul(t_back, -gap / 3.0));
        if new[out_n].h_in.is_some() {
            new[out_n].kind = PointKind::Smooth;
        }
        if new[out_n + 1].h_out.is_some() {
            new[out_n + 1].kind = PointKind::Smooth;
        }
    }
    let before = polygon::signed_area(&path::flatten(&panel.outline, true, TOL));
    let after = validate_outline(&new).map_err(|e| format!("{e}; use a smaller spread or another slash line"))?;

    let mut map = vec![0usize; m];
    for k in 0..out_n {
        map[(hi_i + k) % m] = k;
    }
    for k in 0..back_n {
        map[(fi + k) % m] = out_n + 1 + k;
    }
    for nt in panel.notches.iter_mut() {
        let (sg, t) = remap(&order, nt.segment, nt.t);
        nt.segment = map[sg.min(m - 1)];
        nt.t = t;
    }
    if let Some(sa) = &mut panel.seam_allowance {
        if sa.per_segment.len() == n {
            let v = expand(&sa.per_segment, &order);
            let mut out = vec![None; m + 1];
            for (s, x) in v.into_iter().enumerate() {
                out[map[s]] = x;
            }
            sa.per_segment = out;
        }
    }
    for (inner, a) in panel.inner.iter_mut().zip(inner_turn) {
        if a != 0.0 {
            for p in inner.points.iter_mut() {
                *p = turned(p, hinge, a);
            }
        }
    }
    if let Some(g) = &mut panel.grain_line {
        let mid = lerp(g[0], g[1], 0.5);
        let c = cross(dir, sub(mid, hinge));
        let a = if c > ON_LINE {
            left
        } else if c < -ON_LINE {
            right
        } else {
            0.0
        };
        if a != 0.0 {
            *g = [spin(g[0], hinge, a), spin(g[1], hinge, a)];
        }
    }
    panel.outline = new;
    Ok(SlashReport { hinge, far, angle: theta, length, opening: 2.0 * length * (theta.abs().to_radians() / 2.0).sin(), area_added: after - before })
}
