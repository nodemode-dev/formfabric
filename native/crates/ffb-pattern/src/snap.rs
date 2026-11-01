use crate::geom::*;
use crate::path;
use crate::{Garment, Panel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapKind {
    Point,
    Midpoint,
    Inner,
    Segment,
    Grid,
}

impl SnapKind {
    pub const ORDER: [SnapKind; 5] = [SnapKind::Point, SnapKind::Midpoint, SnapKind::Inner, SnapKind::Segment, SnapKind::Grid];

    pub fn name(self) -> &'static str {
        match self {
            SnapKind::Point => "point",
            SnapKind::Midpoint => "midpoint",
            SnapKind::Inner => "inner",
            SnapKind::Segment => "segment",
            SnapKind::Grid => "grid",
        }
    }

    pub fn parse(text: &str) -> Option<SnapKind> {
        SnapKind::ORDER.into_iter().find(|k| k.name() == text)
    }
}

#[derive(Debug, Clone)]
pub struct SnapOptions {
    pub kinds: Vec<SnapKind>,
    pub tolerance: f64,
    pub grid: f64,
    pub exclude: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Snapped {
    pub point: V2,
    pub kind: SnapKind,
    pub distance: f64,
    pub panel: Option<u32>,
    pub inner: Option<u32>,
    pub index: Option<usize>,
}

fn candidate(kind: SnapKind, q: V2, point: V2, panel: Option<&Panel>, inner: Option<u32>, index: Option<usize>) -> Snapped {
    Snapped { point, kind, distance: dist(q, point), panel: panel.map(|p| p.id), inner, index }
}

fn path_candidates(kind: SnapKind, panel: &Panel, inner: Option<u32>, points: &[crate::Point], closed: bool, q: V2, out: &mut Vec<Snapped>) {
    let place = panel.placement();
    let ql = place.to_local(q);
    let segs = path::segments(points, closed);
    match kind {
        SnapKind::Point => {
            for (i, p) in points.iter().enumerate() {
                out.push(candidate(kind, q, place.to_world(p.p), Some(panel), inner, Some(i)));
            }
        }
        SnapKind::Midpoint => {
            for (i, s) in segs.iter().enumerate() {
                let mid = s.at(s.t_at_length(s.length() * 0.5));
                out.push(candidate(kind, q, place.to_world(mid), Some(panel), inner, Some(i)));
            }
        }
        SnapKind::Segment => {
            for (i, s) in segs.iter().enumerate() {
                out.push(candidate(kind, q, place.to_world(s.nearest(ql).1), Some(panel), inner, Some(i)));
            }
        }
        _ => {}
    }
}

fn candidates(g: &Garment, q: V2, o: &SnapOptions, kind: SnapKind) -> Vec<Snapped> {
    let mut out = Vec::new();
    if kind == SnapKind::Grid {
        if o.grid > 0.0 {
            let p = [(q[0] / o.grid).round() * o.grid, (q[1] / o.grid).round() * o.grid];
            out.push(candidate(kind, q, p, None, None, None));
        }
        return out;
    }
    for panel in g.panels.iter().filter(|p| !o.exclude.contains(&p.id)) {
        if kind == SnapKind::Inner {
            for inner in &panel.inner {
                for k in [SnapKind::Point, SnapKind::Midpoint, SnapKind::Segment] {
                    let start = out.len();
                    path_candidates(k, panel, Some(inner.id), &inner.points, inner.closed, q, &mut out);
                    out[start..].iter_mut().for_each(|c| c.kind = SnapKind::Inner);
                }
            }
        } else {
            path_candidates(kind, panel, None, &panel.outline, true, q, &mut out);
        }
    }
    out
}

pub fn snap(g: &Garment, q: V2, o: &SnapOptions) -> Option<Snapped> {
    SnapKind::ORDER
        .into_iter()
        .filter(|k| o.kinds.contains(k))
        .find_map(|k| candidates(g, q, o, k).into_iter().filter(|c| c.distance <= o.tolerance).min_by(|a, b| a.distance.total_cmp(&b.distance)))
}
