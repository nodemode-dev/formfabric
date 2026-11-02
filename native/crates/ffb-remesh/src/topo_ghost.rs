use ffb_pattern::{path, sew, Garment, SeamEnd};

use crate::geom::dist;
use crate::topology::Topology;
use crate::types::V2;

const SPAN_EPS: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq)]
pub struct Mirror {
    pub seam: u32,
    pub panel: u32,
    pub pos: V2,
    pub fraction: f64,
    pub partner_fraction: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ghost {
    pub panel: u32,
    pub pos: V2,
    pub from_panel: u32,
    pub from_point: u32,
    pub seam: u32,
    pub fraction: f64,
    pub partner_fraction: f64,
    pub occupied: bool,
}

fn on_end(garment: &Garment, end: &SeamEnd, pos: V2, tol: f64) -> Option<(f64, V2, f64)> {
    if end.inner.is_some() || !sew::end_valid(garment, end) {
        return None;
    }
    let (pts, _) = sew::end_path(garment, end)?;
    let seg = path::seg(pts, end.segment);
    let (t, p, d) = seg.nearest(pos);
    let (lo, hi) = (end.t0.min(end.t1), end.t0.max(end.t1));
    (d <= tol && t >= lo - SPAN_EPS && t <= hi + SPAN_EPS).then_some((t, p, d))
}

fn cumulative(garment: &Garment, ends: &[SeamEnd]) -> Option<Vec<f64>> {
    let mut acc = vec![0.0];
    for e in ends {
        if e.inner.is_some() {
            return None;
        }
        let l = sew::end_length(garment, e)?;
        acc.push(acc[acc.len() - 1] + l);
    }
    Some(acc)
}

fn point_at(garment: &Garment, ends: &[SeamEnd], cum: &[f64], target: f64) -> Option<(V2, u32)> {
    let last = ends.len() - 1;
    let k = (0..ends.len()).find(|k| target <= cum[k + 1] || *k == last)?;
    let end = &ends[k];
    let (pts, _) = sew::end_path(garment, end)?;
    let seg = path::seg(pts, end.segment);
    let l = (target - cum[k]).clamp(0.0, cum[k + 1] - cum[k]);
    let t0 = end.t0.clamp(0.0, 1.0);
    let base = seg.length_to(t0);
    let abs = if end.t0 <= end.t1 { base + l } else { base - l };
    Some((seg.at(seg.t_at_length(abs.max(0.0))), end.panel))
}

pub fn mirror(garment: &Garment, panel: u32, pos: V2, tol: f64) -> Vec<Mirror> {
    let mut out = Vec::new();
    for seam in &garment.seams {
        for side in 0..2 {
            let (mine, other) = if side == 0 { (&seam.a, &seam.b) } else { (&seam.b, &seam.a) };
            if mine.is_empty() || other.is_empty() {
                continue;
            }
            let (Some(cm), Some(co)) = (cumulative(garment, mine), cumulative(garment, other)) else { continue };
            let (tm, to) = (cm[cm.len() - 1], co[co.len() - 1]);
            if tm < SPAN_EPS || to < SPAN_EPS {
                continue;
            }
            for (k, end) in mine.iter().enumerate() {
                if end.panel != panel {
                    continue;
                }
                let Some((t, _, _)) = on_end(garment, end, pos, tol) else { continue };
                let Some((pts, _)) = sew::end_path(garment, end) else { continue };
                let seg = path::seg(pts, end.segment);
                let l = (seg.length_to(t) - seg.length_to(end.t0.clamp(0.0, 1.0))).abs();
                let f = ((cm[k] + l) / tm).clamp(0.0, 1.0);
                let f2 = if seam.reversed { 1.0 - f } else { f };
                let Some(p) = point_at(garment, other, &co, f2 * to) else { continue };
                out.push(Mirror { seam: seam.id, panel: p.1, pos: p.0, fraction: f, partner_fraction: f2 });
            }
        }
    }
    out
}

pub fn project_sewn(garment: &Garment, panel: u32, pos: V2, radius: f64) -> Option<V2> {
    let mut best: Option<(f64, V2)> = None;
    for seam in &garment.seams {
        for end in seam.a.iter().chain(seam.b.iter()) {
            if end.panel != panel {
                continue;
            }
            if let Some((_, p, d)) = on_end(garment, end, pos, radius) {
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, p));
                }
            }
        }
    }
    best.map(|b| b.1)
}

pub fn ghosts(garment: &Garment, topo: &Topology, tol: f64) -> Vec<Ghost> {
    let mut out = Vec::new();
    for (panel, t) in &topo.panels {
        if garment.index_of(*panel).is_none() {
            continue;
        }
        for (i, p) in t.points.iter().enumerate() {
            for m in mirror(garment, *panel, *p, tol) {
                if m.panel == *panel && dist(m.pos, *p) < SPAN_EPS {
                    continue;
                }
                let occupied = topo.panels.get(&m.panel).is_some_and(|o| o.points.iter().any(|q| dist(*q, m.pos) <= tol));
                out.push(Ghost { panel: m.panel, pos: m.pos, from_panel: *panel, from_point: i as u32, seam: m.seam, fraction: m.fraction, partner_fraction: m.partner_fraction, occupied });
            }
        }
    }
    out
}
