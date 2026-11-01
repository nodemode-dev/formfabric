use crate::bezier::Seg;
use crate::geom::EPS;
use crate::{path, Garment, Point, SeamEnd};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeamLengths {
    pub a: f64,
    pub b: f64,
    pub mismatch: f64,
}

pub fn end_path<'a>(garment: &'a Garment, end: &SeamEnd) -> Option<(&'a [Point], bool)> {
    let panel = garment.panels.iter().find(|p| p.id == end.panel)?;
    panel.path(end.inner).map(|(pts, closed)| (pts.as_slice(), closed))
}

pub fn end_valid(garment: &Garment, end: &SeamEnd) -> bool {
    end_path(garment, end).is_some_and(|(pts, closed)| end.segment < path::seg_count(pts.len(), closed))
}

pub fn is_full(end: &SeamEnd) -> bool {
    end.inner.is_none() && end.t0.abs() < EPS && (end.t1 - 1.0).abs() < EPS
}

pub fn span_curve(seg: &Seg, t0: f64, t1: f64) -> Seg {
    let (lo, hi) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
    let rest = if lo > EPS { seg.split(lo).1 } else { *seg };
    let cut = (hi - lo) / (1.0 - lo).max(EPS);
    if cut >= 1.0 - EPS {
        rest
    } else {
        rest.split(cut).0
    }
}

pub fn end_length(garment: &Garment, end: &SeamEnd) -> Option<f64> {
    let (pts, closed) = end_path(garment, end)?;
    if end.segment >= path::seg_count(pts.len(), closed) {
        return None;
    }
    let seg = path::seg(pts, end.segment);
    Some(span_curve(&seg, end.t0.clamp(0.0, 1.0), end.t1.clamp(0.0, 1.0)).length())
}

pub fn side_length(garment: &Garment, ends: &[SeamEnd]) -> Option<f64> {
    ends.iter().try_fold(0.0, |sum, e| end_length(garment, e).map(|l| sum + l))
}

pub fn t_after_length(garment: &Garment, end: &SeamEnd, from_t: f64, length: f64) -> Option<f64> {
    let (pts, closed) = end_path(garment, end)?;
    if end.segment >= path::seg_count(pts.len(), closed) {
        return None;
    }
    let seg = path::seg(pts, end.segment);
    let tail = if from_t > EPS { seg.split(from_t).1 } else { seg };
    let local = tail.t_at_length(length.max(0.0));
    Some(from_t + local * (1.0 - from_t))
}
