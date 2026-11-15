use super::lengths::end_valid;
use super::pending::Op;
use crate::geom::EPS;
use crate::{path, Garment, Point, Seam, SeamEnd};

const SLIVER: f64 = 1e-9;

pub fn split_end(end: &SeamEnd, cuts: &[f64]) -> Vec<SeamEnd> {
    let reversed = end.t0 > end.t1;
    let (lo, hi) = if reversed { (end.t1, end.t0) } else { (end.t0, end.t1) };
    let mut edges = vec![0.0];
    let mut sorted = cuts.to_vec();
    sorted.sort_by(f64::total_cmp);
    edges.extend(sorted);
    edges.push(1.0);
    let mut out = Vec::new();
    for (j, pair) in edges.windows(2).enumerate() {
        let (a, b) = (pair[0], pair[1]);
        let (s, e) = (lo.max(a), hi.min(b));
        if e - s > SLIVER && b - a > EPS {
            out.push(SeamEnd { segment: end.segment + j, t0: (s - a) / (b - a), t1: (e - a) / (b - a), ..end.clone() });
        }
    }
    if out.is_empty() {
        let j = edges.windows(2).position(|p| lo < p[1]).unwrap_or(edges.len() - 2);
        let (a, b) = (edges[j], edges[j + 1]);
        let t = if b - a > EPS { ((lo - a) / (b - a)).clamp(0.0, 1.0) } else { 0.0 };
        out.push(SeamEnd { segment: end.segment + j, t0: t, t1: t, ..end.clone() });
    }
    if reversed {
        out.reverse();
        for e in out.iter_mut() {
            std::mem::swap(&mut e.t0, &mut e.t1);
        }
    }
    out
}

fn on_split(seams: &mut [Seam], panel: u32, inner: Option<u32>, segment: usize, cuts: &[f64]) {
    let extra = cuts.len();
    for seam in seams.iter_mut() {
        for side in [&mut seam.a, &mut seam.b] {
            let mut next = Vec::with_capacity(side.len());
            for end in side.drain(..) {
                if end.panel != panel || end.inner != inner || end.segment < segment {
                    next.push(end);
                } else if end.segment > segment {
                    next.push(SeamEnd { segment: end.segment + extra, ..end });
                } else {
                    next.extend(split_end(&end, cuts));
                }
            }
            *side = next;
        }
    }
}

fn locate(points: &[Point], at: [f64; 2], start: bool) -> Option<(usize, f64)> {
    let near = path::nearest(points, true, at)?;
    let n = path::seg_count(points.len(), true);
    Some(if start && near.t > 1.0 - 1e-6 {
        ((near.seg + 1) % n, 0.0)
    } else if !start && near.t < 1e-6 {
        ((near.seg + n - 1) % n, 1.0)
    } else {
        (near.seg, near.t)
    })
}

fn reproject_end(end: &SeamEnd, old: &[Point], new: &[Point]) -> Vec<SeamEnd> {
    let old_count = path::seg_count(old.len(), true);
    let new_count = path::seg_count(new.len(), true);
    if end.segment >= old_count || new_count == 0 {
        return vec![end.clone()];
    }
    let reversed = end.t0 > end.t1;
    let (lo, hi) = if reversed { (end.t1, end.t0) } else { (end.t0, end.t1) };
    let seg = path::seg(old, end.segment);
    let (Some((s0, t0)), Some((s1, t1))) = (locate(new, seg.at(lo), true), locate(new, seg.at(hi), false)) else {
        return vec![end.clone()];
    };
    let mut pieces = Vec::new();
    let mut s = s0;
    let mut guard = 0;
    loop {
        let from = if s == s0 { t0 } else { 0.0 };
        let to = if s == s1 { t1 } else { 1.0 };
        pieces.push(SeamEnd { segment: s, t0: from, t1: to.max(from), ..end.clone() });
        guard += 1;
        if s == s1 || guard > new_count {
            break;
        }
        s = (s + 1) % new_count;
    }
    if pieces.len() > 1 && s0 == s1 && t1 < t0 {
        pieces.truncate(1);
        pieces[0].t1 = pieces[0].t0;
    }
    if reversed {
        pieces.reverse();
        for e in pieces.iter_mut() {
            std::mem::swap(&mut e.t0, &mut e.t1);
        }
    }
    pieces
}

fn on_reproject(seams: &mut [Seam], panel: u32, old: &[Point], new: &[Point]) {
    for seam in seams.iter_mut() {
        for side in [&mut seam.a, &mut seam.b] {
            let mut next = Vec::with_capacity(side.len());
            for end in side.drain(..) {
                if end.panel == panel && end.inner.is_none() {
                    next.extend(reproject_end(&end, old, new));
                } else {
                    next.push(end);
                }
            }
            *side = next;
        }
    }
}

fn on_reverse(seams: &mut [Seam], panel: u32, paths: &[(Option<u32>, usize, bool)]) {
    for seam in seams.iter_mut() {
        for end in seam.a.iter_mut().chain(seam.b.iter_mut()) {
            let Some(&(_, n, closed)) = paths.iter().find(|p| end.panel == panel && p.0 == end.inner) else { continue };
            let count = path::seg_count(n, closed);
            if end.segment < count {
                end.segment = if closed { (2 * n - 2 - end.segment) % n } else { count - 1 - end.segment };
                end.t0 = 1.0 - end.t0;
                end.t1 = 1.0 - end.t1;
            }
        }
    }
}

pub fn apply_pending(garment: &mut Garment, ops: Vec<Op>) {
    for op in ops {
        match op {
            Op::Split { panel, inner, segment, cuts, points } => {
                let fits = garment.panels.iter().find(|p| p.id == panel).and_then(|p| p.path(inner)).is_some_and(|(pts, _)| pts.len() == points);
                if fits {
                    on_split(&mut garment.seams, panel, inner, segment, &cuts);
                }
            }
            Op::Reproject { panel, old, points } => {
                if let Some(p) = garment.panels.iter().find(|p| p.id == panel).filter(|p| p.outline.len() == points) {
                    let new = p.outline.clone();
                    on_reproject(&mut garment.seams, panel, &old, &new);
                }
            }
            Op::Reverse { panel, points } => {
                if let Some(p) = garment.panels.iter().find(|p| p.id == panel && p.outline.len() == points) {
                    let paths: Vec<(Option<u32>, usize, bool)> = std::iter::once((None, points, true)).chain(p.inner.iter().map(|i| (Some(i.id), i.points.len(), i.closed))).collect();
                    on_reverse(&mut garment.seams, panel, &paths);
                }
            }
        }
    }
}

pub fn prune(garment: &mut Garment) {
    let mut seams = std::mem::take(&mut garment.seams);
    for seam in seams.iter_mut() {
        for end in seam.a.iter_mut().chain(seam.b.iter_mut()) {
            end.t0 = end.t0.clamp(0.0, 1.0);
            end.t1 = end.t1.clamp(0.0, 1.0);
        }
        seam.a.retain(|e| end_valid(garment, e));
        seam.b.retain(|e| end_valid(garment, e));
    }
    seams.retain(|s| !s.a.is_empty() && !s.b.is_empty());
    garment.seams = seams;
    let live: Vec<u32> = garment.panels.iter().map(|p| p.id).collect();
    garment.tacks.retain(|t| [&t.a, &t.b].iter().all(|e| e.panel.is_none_or(|p| live.contains(&p))));
}
