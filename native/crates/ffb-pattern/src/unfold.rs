use crate::dart::validate_outline;
use crate::geom::*;
use crate::link::{reflect, reflect_vec};
use crate::path;
use crate::{Panel, Point, PointKind};

#[derive(Debug, Clone, Copy)]
pub struct UnfoldReport {
    pub points: usize,
    pub inner_mirrored: usize,
    pub notches_mirrored: usize,
    pub axis: [V2; 2],
}

fn collinear_opposite(a: Option<V2>, b: Option<V2>) -> bool {
    match (a.and_then(unit), b.and_then(unit)) {
        (Some(x), Some(y)) => cross(x, y).abs() < 1e-6 && dot(x, y) < 0.0,
        _ => false,
    }
}

fn mirrored_point(q: &Point, a: V2, b: V2) -> Point {
    Point { p: reflect(q.p, a, b), kind: q.kind, h_in: q.h_out.map(|h| reflect_vec(h, a, b)), h_out: q.h_in.map(|h| reflect_vec(h, a, b)) }
}

pub fn unfold(panel: &mut Panel, segment: usize, next_id: &mut u32) -> Result<UnfoldReport, String> {
    let n = panel.outline.len();
    if n < 3 || segment >= n {
        return Err(format!("segment {segment} is out of range, the outline has {n}"));
    }
    let (k, k1) = (segment, (segment + 1) % n);
    let (a, b) = (panel.outline[k].p, panel.outline[k1].p);
    if dist(a, b) < 1e-6 {
        return Err("the axis segment has no length".into());
    }
    if panel.outline[k].h_out.is_some() || panel.outline[k1].h_in.is_some() {
        return Err("the axis segment must be straight".into());
    }
    let src = &panel.outline;
    let mut ring: Vec<Point> = (0..n).map(|j| src[(k1 + j) % n].clone()).collect();
    for i in 1..=n - 2 {
        ring.push(mirrored_point(&src[(k + n - i) % n], a, b));
    }
    let last = n - 1;
    ring[0].h_in = src[k1].h_out.map(|h| reflect_vec(h, a, b));
    ring[0].kind = if collinear_opposite(ring[0].h_in, ring[0].h_out) { PointKind::Smooth } else { PointKind::Corner };
    ring[last].h_out = src[k].h_in.map(|h| reflect_vec(h, a, b));
    ring[last].kind = if collinear_opposite(ring[last].h_in, ring[last].h_out) { PointKind::Smooth } else { PointKind::Corner };
    validate_outline(&ring).map_err(|e| format!("{e}; the half panel must lie on one side of the axis"))?;

    let mut mirrored_notches = 0;
    let mut notches = Vec::with_capacity(panel.notches.len() * 2);
    let mut next_notch = panel.notches.iter().map(|x| x.id).max().unwrap_or(0) + 1;
    for nt in &panel.notches {
        if nt.segment == k || nt.segment >= n {
            continue;
        }
        let mut first = nt.clone();
        first.segment = (nt.segment + n - k1) % n;
        notches.push(first);
        let mut twin = nt.clone();
        twin.id = next_notch;
        next_notch += 1;
        twin.segment = n - 1 + (k + n - 1 - nt.segment) % n;
        twin.t = 1.0 - nt.t;
        notches.push(twin);
        mirrored_notches += 1;
    }
    if let Some(sa) = &mut panel.seam_allowance {
        if sa.per_segment.len() == n {
            let mut out = vec![None; 2 * n - 2];
            for (j, v) in sa.per_segment.iter().enumerate() {
                if j != k {
                    out[(j + n - k1) % n] = *v;
                    out[n - 1 + (k + n - 1 - j) % n] = *v;
                }
            }
            sa.per_segment = out;
        }
    }
    let mut twins = Vec::new();
    for inner in &panel.inner {
        if inner.points.iter().all(|p| dist_to_line(p.p, a, b) < 1e-6) {
            continue;
        }
        let mut twin = inner.clone();
        twin.id = *next_id;
        *next_id += 1;
        twin.points = inner.points.iter().map(|p| Point { p: reflect(p.p, a, b), kind: p.kind, h_in: p.h_in.map(|h| reflect_vec(h, a, b)), h_out: p.h_out.map(|h| reflect_vec(h, a, b)) }).collect();
        if twin.closed {
            path::reverse(&mut twin.points);
        }
        twins.push(twin);
    }
    let inner_mirrored = twins.len();
    panel.inner.extend(twins);
    panel.notches = notches;
    panel.outline = ring;
    Ok(UnfoldReport { points: panel.outline.len(), inner_mirrored, notches_mirrored: mirrored_notches, axis: [a, b] })
}
