use ffb_pattern::geom::*;
use ffb_pattern::{path, polygon};
use ffb_pattern::{CornerStyle, Inner, InnerKind, InnerRole, Notch, NotchKind, Panel, Point, SeamAllowance};

use super::conv::poly_points;
use crate::dxf::Poly;

const MAX_NOTCH_DISTANCE: f64 = 60.0;
const SEW_SAMPLES: usize = 8;
const OVERRIDE_STEP: f64 = 0.1;

#[derive(Debug, Clone)]
pub struct RawNotch {
    pub p: V2,
    pub kind: NotchKind,
    pub angle: Option<f64>,
    pub depth: f64,
    pub width: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct Piece {
    pub name: String,
    pub cut: Option<Poly>,
    pub sew: Option<Poly>,
    pub inner: Vec<(Poly, Option<String>)>,
    pub grain: Option<[V2; 2]>,
    pub notches: Vec<RawNotch>,
}

pub fn shift(p: V2, origin: V2) -> V2 {
    sub(p, origin)
}

fn label_value<'a>(label: &'a str, key: &str) -> Option<&'a str> {
    label.trim_start_matches("FF:").split(';').find_map(|kv| kv.split_once('=').filter(|(k, _)| k.trim() == key).map(|(_, v)| v.trim()))
}

fn role_of(label: &Option<String>) -> InnerRole {
    match label.as_deref().and_then(|l| label_value(l, "role")) {
        Some("shape") => InnerRole::Shape,
        Some("fold") => InnerRole::Fold,
        Some("dart") => InnerRole::Dart,
        _ => InnerRole::Reference,
    }
}

pub fn classify(count: usize, closed: bool) -> InnerKind {
    if closed {
        InnerKind::Polygon
    } else if count == 2 {
        InnerKind::Line
    } else {
        InnerKind::Polyline
    }
}

fn kind_of(label: &Option<String>, count: usize, closed: bool) -> InnerKind {
    match label.as_deref().and_then(|l| label_value(l, "kind")) {
        Some("rectangle") if closed => InnerKind::Rectangle,
        Some("ellipse") if closed => InnerKind::Ellipse,
        Some("polygon") if closed => InnerKind::Polygon,
        Some("polyline") => InnerKind::Polyline,
        Some("line") if count == 2 && !closed => InnerKind::Line,
        _ => classify(count, closed),
    }
}

fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    if n % 2 == 1 {
        values[n / 2]
    } else {
        0.5 * (values[n / 2 - 1] + values[n / 2])
    }
}

fn distance_to_loop(q: V2, poly: &[V2]) -> f64 {
    let n = poly.len();
    (0..n).map(|i| closest_on_segment(q, poly[i], poly[(i + 1) % n]).1).map(|c| dist(q, c)).fold(f64::MAX, f64::min)
}

fn poly_area(poly: &Poly) -> f64 {
    let pts: Vec<V2> = poly.verts.iter().map(|v| v.p).collect();
    if pts.len() < 3 {
        return 0.0;
    }
    polygon::signed_area(&pts).abs()
}

fn enclosed(inner: &[V2], outer: &[V2]) -> bool {
    let tol = 0.05;
    let hits = inner.iter().filter(|p| polygon::contains(outer, **p) || distance_to_loop(**p, outer) <= tol).count();
    hits * 10 >= inner.len() * 9
}

fn translate(points: &mut [Point], origin: V2) {
    for p in points.iter_mut() {
        p.p = shift(p.p, origin);
    }
}

pub fn build(piece: Piece, tol: f64, use_sew: bool, notes: &mut Vec<String>) -> Result<Panel, String> {
    let name = piece.name.clone();
    let mut cut = piece.cut.clone().ok_or_else(|| format!("{name}: no closed boundary on layer 1"))?;
    let mut inner_polys = piece.inner.clone();
    let mut sew = None;
    if let Some(s) = piece.sew.clone() {
        if !use_sew {
            inner_polys.push((s, None));
        } else if poly_area(&s) > poly_area(&cut) {
            notes.push(format!("{name}: the sew line was larger than the cut line, the two were swapped"));
            sew = Some(std::mem::replace(&mut cut, s));
        } else {
            sew = Some(s);
        }
    }
    let cut_points = poly_points(&cut, tol).filter(|(_, closed)| *closed).map(|(p, _)| p).ok_or_else(|| format!("{name}: the cut line is not a closed shape"))?;
    if let Some(s) = sew {
        let net = poly_points(&s, tol).filter(|(_, closed)| *closed).map(|(p, _)| p);
        if let Some(net) = net {
            let cut_flat = path::flatten(&cut_points, true, 0.005);
            let net_flat = path::flatten(&net, true, 0.05);
            if net_flat.len() > 2 && enclosed(&net_flat, &cut_flat) {
                return finish(piece, name, net, Some(cut_flat), inner_polys, tol, notes);
            }
            notes.push(format!("{name}: the sew line does not lie inside the cut line and was kept as a reference line"));
        }
        inner_polys.push((s, None));
    }
    finish(piece, name, cut_points, None, inner_polys, tol, notes)
}

fn finish(piece: Piece, name: String, mut points: Vec<Point>, cut_flat: Option<Vec<V2>>, inner_polys: Vec<(Poly, Option<String>)>, tol: f64, notes: &mut Vec<String>) -> Result<Panel, String> {
    let area = polygon::signed_area(&path::flatten(&points, true, 0.05));
    if area.abs() < 1.0 {
        return Err(format!("{name}: the boundary encloses no area"));
    }
    if area < 0.0 {
        path::reverse(&mut points);
    }
    let bounds = path::bounds(&points, true).ok_or_else(|| format!("{name}: empty boundary"))?;
    let origin = bounds[0];
    translate(&mut points, origin);

    let mut panel = Panel { name: name.chars().filter(|c| !c.is_control()).take(64).collect(), position: origin, ..Panel::default() };

    if let Some(flat) = cut_flat {
        let cut: Vec<V2> = flat.iter().map(|p| shift(*p, origin)).collect();
        let segs = path::segments(&points, true);
        let mut all = Vec::new();
        let mut per = Vec::new();
        for s in &segs {
            let mut d: Vec<f64> = (0..SEW_SAMPLES).map(|k| distance_to_loop(s.at((k as f64 + 0.5) / SEW_SAMPLES as f64), &cut)).collect();
            all.extend(d.iter().copied());
            per.push(median(&mut d));
        }
        let width = (median(&mut all) * 100.0).round() / 100.0;
        let overrides: Vec<Option<f64>> = per.iter().map(|m| ((m - width).abs() > OVERRIDE_STEP).then(|| (m * 100.0).round() / 100.0)).collect();
        let per_segment = if overrides.iter().any(Option::is_some) { overrides } else { Vec::new() };
        panel.seam_allowance = Some(SeamAllowance { width, corners: CornerStyle::Miter, per_segment });
    }

    for (k, (poly, label)) in inner_polys.iter().enumerate() {
        let Some((mut pts, closed)) = poly_points(poly, tol) else {
            notes.push(format!("{name}: internal line {k} has too few points and was skipped"));
            continue;
        };
        translate(&mut pts, origin);
        let kind = kind_of(label, pts.len(), closed);
        let fold = label.as_deref().and_then(|l| label_value(l, "angle")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(180.0);
        let strength = label.as_deref().and_then(|l| label_value(l, "strength")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        panel.inner.push(Inner { id: 0, kind, points: pts, closed, fold_angle: fold, fold_strength: strength, role: role_of(label) });
    }

    panel.grain_line = piece.grain.map(|g| [shift(g[0], origin), shift(g[1], origin)]);

    let reach = panel.seam_allowance.as_ref().map_or(0.0, |s| s.per_segment.iter().flatten().fold(s.width, |a, b| a.max(*b)));
    let limit = MAX_NOTCH_DISTANCE.max(reach * 3.0);
    let mut next = 1;
    for raw in &piece.notches {
        let q = shift(raw.p, origin);
        let Some(near) = path::nearest(&points, true, q) else { continue };
        if near.dist > limit {
            notes.push(format!("{name}: a notch lies {:.1} mm from the boundary and was skipped", near.dist));
            continue;
        }
        let tangent = path::seg(&points, near.seg).tangent(near.t);
        let base = tangent[1].atan2(tangent[0]).to_degrees();
        let angle = raw.angle.map_or(90.0, |a| (a - base).rem_euclid(360.0).clamp(10.0, 170.0));
        let duplicate = panel.notches.iter().any(|n| n.kind == raw.kind && n.segment == near.seg && (n.t - near.t).abs() < 1e-3);
        if duplicate {
            continue;
        }
        let default = Notch::default();
        let width = match raw.kind {
            NotchKind::Slit => 0.0,
            _ => raw.width.filter(|w| *w > 0.0).unwrap_or(default.width),
        };
        let depth = if raw.depth > 1e-6 { raw.depth } else { default.depth };
        panel.notches.push(Notch { id: next, segment: near.seg, t: near.t, kind: raw.kind, depth, width, angle });
        next += 1;
    }

    panel.outline = points;
    Ok(panel)
}
