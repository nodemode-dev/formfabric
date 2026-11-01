use crate::geom::{dist, unit, V2};
use crate::sew::{end_path, side_length, span_curve};
use crate::{path, shapes, Garment, Inner, InnerRole, Panel, PanelState, Place3D, SeamEnd};

const TOL: f64 = 0.1;
const WRAP_ANGLE: f64 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripKind {
    Piping,
    Binding,
    Elastic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Strip {
    pub panel: Panel,
    pub inner: Option<u32>,
    pub edge: Vec<SeamEnd>,
    pub length: f64,
}

pub fn span_path(garment: &Garment, ends: &[SeamEnd]) -> Option<Vec<V2>> {
    let mut out: Vec<V2> = Vec::new();
    for end in ends {
        let (pts, closed) = end_path(garment, end)?;
        if end.segment >= path::seg_count(pts.len(), closed) {
            return None;
        }
        let seg = path::seg(pts, end.segment);
        let mut line = span_curve(&seg, end.t0.clamp(0.0, 1.0), end.t1.clamp(0.0, 1.0)).polyline(TOL);
        if end.t0 > end.t1 {
            line.reverse();
        }
        let skip = usize::from(out.last().is_some_and(|q| line.first().is_some_and(|p| dist(*q, *p) < 1e-6)));
        out.extend(line.into_iter().skip(skip));
    }
    (out.len() >= 2).then_some(out)
}

pub fn crease_points(garment: &Garment, ends: &[SeamEnd]) -> Result<(u32, [V2; 2]), String> {
    let panel = ends.first().map(|e| e.panel).ok_or("a crease needs a span")?;
    if ends.iter().any(|e| e.panel != panel) {
        return Err("the spans of a crease must lie on one panel".into());
    }
    let line = span_path(garment, ends).ok_or("the spans do not lie on the panel")?;
    let (a, b) = (line[0], line[line.len() - 1]);
    if dist(a, b) < 1.0 {
        return Err("the crease would be shorter than 1 mm".into());
    }
    Ok((panel, [a, b]))
}

fn normalized(q: [f64; 4]) -> [f64; 4] {
    let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
    if n < 1e-12 || !n.is_finite() {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        q.map(|x| x / n)
    }
}

fn multiply(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [aw * bx + ax * bw + ay * bz - az * by, aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw, aw * bw - ax * bx - ay * by - az * bz]
}

fn rotate3(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let [x, y, z, w] = q;
    let t = [2.0 * (y * v[2] - z * v[1]), 2.0 * (z * v[0] - x * v[2]), 2.0 * (x * v[1] - y * v[0])];
    [v[0] + w * t[0] + (y * t[2] - z * t[1]), v[1] + w * t[1] + (z * t[0] - x * t[2]), v[2] + w * t[2] + (x * t[1] - y * t[0])]
}

pub fn build_strip(garment: &Garment, kind: StripKind, finish: u32, spans: &[SeamEnd], width: f64, ratio: f64, fabric: Option<u32>) -> Result<Strip, String> {
    let line = span_path(garment, spans).ok_or("the spans do not lie on their panels")?;
    let host = garment.panels.iter().find(|p| Some(p.id) == spans.first().map(|s| s.panel)).ok_or("the spans have no panel")?;
    let length = side_length(garment, spans).ok_or("the spans do not lie on their panels")?;
    if length < 2.0 {
        return Err("the spans are shorter than 2 mm".into());
    }
    let (first, last) = (line[0], line[line.len() - 1]);
    let d = unit([last[0] - first[0], last[1] - first[1]]).or_else(|| unit([line[1][0] - line[0][0], line[1][1] - line[0][1]])).ok_or("the spans have no direction")?;
    let total = if kind == StripKind::Binding { width * 2.0 } else { width };
    let outward = [d[1], -d[0]];
    let origin = if kind == StripKind::Binding { [last[0] - outward[0] * width, last[1] - outward[1] * width] } else { last };
    let angle = (-d[1]).atan2(-d[0]).to_degrees();
    let place = host.placement();
    let world_x = place.vec_to_world([-d[0], -d[1]]);
    let id = garment.next_id();
    let inner = (kind == StripKind::Binding).then_some(id + 1);
    let mut strip = Panel {
        id,
        name: format!("{} {finish}", match kind {
            StripKind::Piping => "Piping",
            StripKind::Binding => "Binding",
            StripKind::Elastic => "Elastic",
        }),
        outline: strip_outline(length, total, inner.is_none()),
        position: place.to_world(origin),
        rotation: world_x[1].atan2(world_x[0]).to_degrees(),
        particle_distance: (total / 3.0).clamp(1.0, host.particle_distance.max(1.0)),
        fabric: fabric.or(host.fabric),
        layer: host.layer,
        tags: vec!["generated".into(), format!("finish:{finish}")],
        ..Panel::default()
    };
    if let Some(inner_id) = inner {
        let points = [[0.0, width], [length, width]].into_iter().map(|p| crate::Point { p, ..crate::Point::default() }).collect();
        strip.inner.push(Inner { id: inner_id, points, role: InnerRole::Fold, fold_angle: WRAP_ANGLE, fold_strength: 1.0, ..Inner::default() });
    }
    if kind == StripKind::Elastic {
        strip.state = PanelState { shrink_weft: ratio.clamp(0.05, 4.0), ..PanelState::default() };
    }
    if let Some(p3) = &host.place3d {
        let q = normalized(p3.rotation);
        let r = multiply(q, [0.0, 0.0, (angle.to_radians() * 0.5).sin(), (angle.to_radians() * 0.5).cos()]);
        let shift = rotate3(q, [origin[0], origin[1], 0.0]);
        strip.place3d = Some(Place3D { origin: [p3.origin[0] + shift[0], p3.origin[1] + shift[1], p3.origin[2] + shift[2]], rotation: r, bend: 0.0, pose: None });
    }
    let edge = match inner {
        Some(i) => vec![SeamEnd { panel: id, inner: Some(i), segment: 0, t0: 0.0, t1: 1.0 }],
        None => (0..2).map(|segment| SeamEnd { panel: id, inner: None, segment, t0: 0.0, t1: 1.0 }).collect(),
    };
    Ok(Strip { panel: strip, inner, edge, length })
}

fn strip_outline(length: f64, total: f64, split: bool) -> Vec<crate::Point> {
    let mut outline = shapes::rectangle(0.0, 0.0, length, total);
    if split {
        outline.insert(1, crate::Point { p: [length * 0.5, 0.0], ..crate::Point::default() });
    }
    outline
}
