use std::f64::consts::TAU;

use crate::geom::*;
use crate::{Point, PointKind};

pub fn rectangle(x: f64, y: f64, w: f64, h: f64) -> Vec<Point> {
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]].into_iter().map(|p| Point { p, ..Point::default() }).collect()
}

pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, segments: usize) -> Vec<Point> {
    let n = segments.max(4);
    let step = TAU / n as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    (0..n)
        .map(|i| {
            let a = step * i as f64;
            let (s, c) = a.sin_cos();
            let tangent = mul([-rx * s, ry * c], k);
            Point { p: [cx + rx * c, cy + ry * s], kind: PointKind::Smooth, h_in: Some(mul(tangent, -1.0)), h_out: Some(tangent) }
        })
        .collect()
}
