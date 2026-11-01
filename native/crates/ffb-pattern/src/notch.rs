use crate::geom::*;
use crate::{offset, path, polygon, Notch, Panel, Point};

pub fn next_id(panel: &Panel) -> u32 {
    panel.notches.iter().map(|n| n.id).max().unwrap_or(0) + 1
}

pub fn outward_side(outline: &[Point]) -> f64 {
    if polygon::signed_area(&path::flatten(outline, true, 0.1)) < 0.0 {
        -1.0
    } else {
        1.0
    }
}

pub fn locate(panel: &Panel, n: &Notch) -> Option<(V2, V2)> {
    if n.segment >= path::seg_count(panel.outline.len(), true) {
        return None;
    }
    let s = path::seg(&panel.outline, n.segment);
    let t = n.t.clamp(0.0, 1.0);
    let tangent = unit(s.tangent(t)).or_else(|| unit(sub(s.p[3], s.p[0])))?;
    let side = outward_side(&panel.outline);
    Some((s.at(t), mul([tangent[1], -tangent[0]], side)))
}

pub fn on_split(panel: &mut Panel, segment: usize, ts: &[f64]) {
    let mut cuts: Vec<f64> = ts.to_vec();
    cuts.sort_by(f64::total_cmp);
    let extra = cuts.len();
    for n in panel.notches.iter_mut() {
        if n.segment > segment {
            n.segment += extra;
        } else if n.segment == segment {
            let piece = cuts.iter().filter(|c| **c < n.t).count();
            let lo = if piece == 0 { 0.0 } else { cuts[piece - 1] };
            let hi = cuts.get(piece).copied().unwrap_or(1.0);
            n.t = if hi - lo > EPS { ((n.t - lo) / (hi - lo)).clamp(0.0, 1.0) } else { 0.0 };
            n.segment += piece;
        }
    }
    if let Some(sa) = panel.seam_allowance.as_mut() {
        offset::split_widths(sa, segment, extra + 1);
    }
}

pub fn reproject(panel: &mut Panel, old: &[Point]) {
    let places: Vec<Option<V2>> = panel
        .notches
        .iter()
        .map(|n| (n.segment < path::seg_count(old.len(), true)).then(|| path::seg(old, n.segment).at(n.t.clamp(0.0, 1.0))))
        .collect();
    for (n, at) in panel.notches.iter_mut().zip(places) {
        if let Some(near) = at.and_then(|q| path::nearest(&panel.outline, true, q)) {
            n.segment = near.seg;
            n.t = near.t;
        }
    }
}

pub fn on_delete(panel: &mut Panel, old: &[Point], index: usize) {
    reproject(panel, old);
    if let Some(sa) = panel.seam_allowance.as_mut() {
        offset::remove_width(sa, index);
    }
}

pub fn on_reverse(panel: &mut Panel) {
    let n = panel.outline.len();
    if n < 2 {
        return;
    }
    for note in panel.notches.iter_mut() {
        note.segment = (2 * n - 2 - note.segment) % n;
        note.t = 1.0 - note.t;
    }
    if let Some(sa) = panel.seam_allowance.as_mut() {
        offset::reverse_widths(sa, n);
    }
}
