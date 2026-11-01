use crate::geom::*;
use crate::path;
use crate::polygon;
use crate::{Inner, InnerKind, InnerRole, Panel, Point, PointKind};

const TOL: f64 = 0.05;

#[derive(Debug, Clone, Copy)]
pub struct DartSpec {
    pub segment: usize,
    pub t: f64,
    pub width: f64,
    pub length: Option<f64>,
    pub apex: Option<V2>,
}

#[derive(Debug, Clone, Copy)]
pub struct DartReport {
    pub opening: [V2; 2],
    pub apex: V2,
    pub legs: [f64; 2],
    pub chord: f64,
    pub area_removed: f64,
}

pub fn rescale(t: f64, lo: f64, hi: f64) -> f64 {
    if hi - lo < EPS {
        0.0
    } else {
        ((t - lo) / (hi - lo)).clamp(0.0, 1.0)
    }
}

pub fn remap(splits: &[(usize, f64)], seg: usize, t: f64) -> (usize, f64) {
    let before = splits.iter().filter(|s| s.0 < seg).count();
    let mut lo = 0.0;
    let mut j = 0;
    for s in splits.iter().filter(|s| s.0 == seg) {
        if t < s.1 {
            return (seg + before + j, rescale(t, lo, s.1));
        }
        lo = s.1;
        j += 1;
    }
    (seg + before + j, rescale(t, lo, 1.0))
}

pub fn expand<T: Clone>(per: &[T], splits: &[(usize, f64)]) -> Vec<T> {
    let mut out = Vec::with_capacity(per.len() + splits.len());
    for (s, v) in per.iter().enumerate() {
        for _ in 0..=splits.iter().filter(|x| x.0 == s).count() {
            out.push(v.clone());
        }
    }
    out
}

fn corner(p: V2) -> Point {
    Point { p, kind: PointKind::Corner, h_in: None, h_out: None }
}

pub fn validate_outline(points: &[Point]) -> Result<f64, String> {
    let segs = path::flatten_segments(points, true, TOL);
    if let Some((i, j)) = polygon::self_intersections(&segs, true).first() {
        return Err(format!("the outline would cross itself between segments {i} and {j}"));
    }
    let area = polygon::signed_area(&path::flatten(points, true, TOL));
    if area <= EPS {
        return Err("the outline would enclose no area".into());
    }
    Ok(area)
}

pub fn insert(panel: &mut Panel, spec: &DartSpec) -> Result<DartReport, String> {
    let n = panel.outline.len();
    if n < 3 || spec.segment >= n {
        return Err(format!("segment {} is out of range, the outline has {n}", spec.segment));
    }
    let s = path::seg(&panel.outline, spec.segment);
    let total = s.length();
    let centre = s.length_to(spec.t.clamp(0.0, 1.0));
    let (a_len, b_len) = (centre - spec.width / 2.0, centre + spec.width / 2.0);
    if a_len < 1e-6 || b_len > total - 1e-6 {
        return Err(format!("a dart opening of {:.1} mm centred at t {:.2} does not fit on segment {}, which is {:.1} mm long", spec.width, spec.t, spec.segment, total));
    }
    let (ta, tb) = (s.t_at_length(a_len), s.t_at_length(b_len));
    let (a, b) = (s.at(ta), s.at(tb));
    let chord = dist(a, b);
    let dir = unit(sub(b, a)).ok_or("the dart opening has no width")?;
    let mid = lerp(a, b, 0.5);
    let apex = match (spec.apex, spec.length) {
        (Some(p), _) => p,
        (None, Some(l)) => add(mid, mul([-dir[1], dir[0]], l)),
        (None, None) => return Err("give a length or an apex point".into()),
    };
    let before = polygon::signed_area(&path::flatten(&panel.outline, true, TOL));
    if !polygon::contains(&path::flatten(&panel.outline, true, TOL), apex) {
        return Err("the dart apex lies outside the panel".into());
    }
    let mut pts = panel.outline.clone();
    path::split_many(&mut pts, spec.segment, &[ta, tb]);
    let (ia, ib) = (spec.segment + 1, spec.segment + 2);
    pts[ia].h_out = None;
    pts[ia].kind = PointKind::Corner;
    pts[ib].h_in = None;
    pts[ib].kind = PointKind::Corner;
    pts.insert(ib, corner(apex));
    let after = validate_outline(&pts).map_err(|e| format!("{e}; shorten the dart or move its apex"))?;
    let splits = [(spec.segment, ta), (spec.segment, tb)];
    let mid_seg = spec.segment + 1;
    panel.notches.retain_mut(|nt| {
        let (sg, t) = remap(&splits, nt.segment, nt.t);
        if sg == mid_seg {
            return false;
        }
        nt.segment = if sg > mid_seg { sg + 1 } else { sg };
        nt.t = t;
        true
    });
    if let Some(sa) = &mut panel.seam_allowance {
        if sa.per_segment.len() == n {
            let mut v = expand(&sa.per_segment, &splits);
            let leg = v[mid_seg];
            v.insert(mid_seg, leg);
            sa.per_segment = v;
        }
    }
    panel.outline = pts;
    Ok(DartReport { opening: [a, b], apex, legs: [dist(a, apex), dist(b, apex)], chord, area_removed: before - after })
}

pub fn internal(panel: &mut Panel, id: u32, base: V2, apex: V2, width: f64) -> Result<(), String> {
    let dir = unit(sub(apex, base)).ok_or("the dart has no length")?;
    let perp = [-dir[1], dir[0]];
    let corners = [add(base, mul(perp, width / 2.0)), sub(base, mul(perp, width / 2.0)), apex];
    let outline = path::flatten(&panel.outline, true, TOL);
    if corners.iter().any(|&c| !polygon::contains(&outline, c)) {
        return Err("the internal dart reaches outside the panel".into());
    }
    let n = outline.len();
    for i in 0..3 {
        let (p, q) = (corners[i], corners[(i + 1) % 3]);
        if (0..n).any(|k| polygon::segments_intersect(p, q, outline[k], outline[(k + 1) % n])) {
            return Err("the internal dart touches the outline; move it inwards".into());
        }
    }
    let points: Vec<Point> = corners.iter().map(|&c| corner(c)).collect();
    panel.inner.push(Inner { id, kind: InnerKind::Polygon, points, closed: true, role: InnerRole::Dart, ..Inner::default() });
    Ok(())
}
