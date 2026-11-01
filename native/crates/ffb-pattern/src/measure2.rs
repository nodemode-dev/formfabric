use crate::geom::*;
use crate::path;
use crate::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    Scale,
    MoveEnd,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub before: f64,
    pub after: f64,
    pub factor: f64,
    pub shift: f64,
}

const ITERATIONS: usize = 60;

pub fn run_length(points: &[Point], closed: bool, first: usize, count: usize) -> f64 {
    let n = path::seg_count(points.len(), closed);
    (0..count).map(|k| path::seg(points, (first + k) % n).length()).sum()
}

pub fn contiguous(indices: &[usize], total: usize, closed: bool) -> Option<(usize, usize)> {
    let mut sorted: Vec<usize> = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.is_empty() || sorted.iter().any(|&i| i >= total) {
        return None;
    }
    let count = sorted.len();
    if count == total {
        return Some((0, count));
    }
    sorted.iter().copied().find(|&first| (0..count).all(|j| sorted.contains(&if closed { (first + j) % total } else { first + j }))).map(|first| (first, count))
}

fn scaled(points: &[Point], closed: bool, first: usize, count: usize, k: f64) -> Vec<Point> {
    let n = points.len();
    let mut out = points.to_vec();
    let origin = points[first].p;
    for step in 0..=count {
        let i = (first + step) % n;
        out[i].p = add(origin, mul(sub(points[i].p, origin), k));
        if step > 0 || count == n {
            out[i].h_in = points[i].h_in.map(|h| mul(h, k));
        }
        if step < count {
            out[i].h_out = points[i].h_out.map(|h| mul(h, k));
        }
    }
    path::auto_handles(&mut out, closed);
    out
}

fn moved(points: &[Point], closed: bool, end: usize, shift: V2) -> Vec<Point> {
    let mut out = points.to_vec();
    out[end].p = add(points[end].p, shift);
    path::auto_handles(&mut out, closed);
    out
}

pub fn fit(points: &mut Vec<Point>, closed: bool, first: usize, count: usize, target: f64, how: How) -> Result<Fit, String> {
    let n = points.len();
    let total = path::seg_count(n, closed);
    if count == 0 || count > total || (closed && count == total) {
        return Err("choose a run of segments that leaves the outline open at one end".into());
    }
    if target <= EPS {
        return Err("the target length must be positive".into());
    }
    let before = run_length(points, closed, first, count);
    if before <= EPS {
        return Err("the segments have no length to scale".into());
    }
    let end = (first + count) % n;
    let result = match how {
        How::Scale => {
            let mut k = target / before;
            let mut best = scaled(points, closed, first, count, k);
            for _ in 0..8 {
                let now = run_length(&best, closed, first, count);
                if (now - target).abs() < 1e-4 {
                    break;
                }
                k *= target / now;
                best = scaled(points, closed, first, count, k);
            }
            (best, k, 0.0)
        }
        How::MoveEnd => {
            let dir = unit(sub(points[end].p, points[first].p)).ok_or_else(|| "the run starts and ends at the same spot".to_string())?;
            let chord = dist(points[end].p, points[first].p);
            let length_at = |x: f64| run_length(&moved(points, closed, end, mul(dir, x)), closed, first, count);
            let (mut lo, mut hi) = (-chord * 0.999, target.max(chord) * 4.0 + 1.0);
            if length_at(lo) > target {
                return Err("the run cannot be shortened that far by moving its end".into());
            }
            for _ in 0..ITERATIONS {
                let mid = 0.5 * (lo + hi);
                if length_at(mid) < target {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let shift = 0.5 * (lo + hi);
            (moved(points, closed, end, mul(dir, shift)), 1.0, shift)
        }
    };
    let after = run_length(&result.0, closed, first, count);
    *points = result.0;
    Ok(Fit { before, after, factor: result.1, shift: result.2 })
}
