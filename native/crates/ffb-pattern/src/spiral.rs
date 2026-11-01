use std::f64::consts::TAU;

use crate::geom::*;
use crate::{Point, PointKind};

#[derive(Debug, Clone, Copy)]
pub struct SpiralSpec {
    pub center: V2,
    pub inner: f64,
    pub outer: f64,
    pub turns: f64,
    pub width: f64,
    pub ccw: bool,
    pub start: f64,
    pub per_turn: usize,
}

fn centre_at(s: &SpiralSpec, u: f64) -> V2 {
    let r = s.inner + (s.outer - s.inner) * u;
    let phi = s.start.to_radians() + if s.ccw { 1.0 } else { -1.0 } * TAU * s.turns * u;
    add(s.center, [r * phi.cos(), r * phi.sin()])
}

fn edge_at(s: &SpiralSpec, u: f64, side: f64) -> V2 {
    let h = 1e-6;
    let (a, b) = (centre_at(s, (u - h).max(0.0)), centre_at(s, (u + h).min(1.0)));
    let tangent = unit(sub(b, a)).unwrap_or([1.0, 0.0]);
    add(centre_at(s, u), mul([-tangent[1], tangent[0]], side * s.width / 2.0))
}

fn edge_slope(s: &SpiralSpec, u: f64, side: f64) -> V2 {
    let h = 1e-5;
    let (lo, hi) = ((u - h).max(0.0), (u + h).min(1.0));
    mul(sub(edge_at(s, hi, side), edge_at(s, lo, side)), 1.0 / (hi - lo))
}

pub fn outline(s: &SpiralSpec) -> Result<Vec<Point>, String> {
    if s.inner - s.width / 2.0 < 0.1 {
        return Err("the inner radius must be larger than half the width".into());
    }
    if s.outer <= s.inner {
        return Err("the outer radius must be larger than the inner radius".into());
    }
    let pitch = (s.outer - s.inner) / s.turns;
    if pitch <= s.width + 0.01 {
        return Err(format!("adjacent turns would overlap: the spacing between turns is {pitch:.1} mm and the width is {:.1} mm", s.width));
    }
    let n = ((s.turns * s.per_turn as f64).ceil() as usize).max(4);
    let step = 1.0 / n as f64;
    let mut pts = Vec::with_capacity(2 * (n + 1));
    for i in 0..=n {
        let u = i as f64 * step;
        let d = mul(edge_slope(s, u, 1.0), step / 3.0);
        pts.push(Point {
            p: edge_at(s, u, 1.0),
            kind: if i == 0 || i == n { PointKind::Corner } else { PointKind::Smooth },
            h_in: (i > 0).then(|| mul(d, -1.0)),
            h_out: (i < n).then_some(d),
        });
    }
    for i in (0..=n).rev() {
        let u = i as f64 * step;
        let d = mul(edge_slope(s, u, -1.0), step / 3.0);
        pts.push(Point {
            p: edge_at(s, u, -1.0),
            kind: if i == 0 || i == n { PointKind::Corner } else { PointKind::Smooth },
            h_in: (i < n).then_some(d),
            h_out: (i > 0).then(|| mul(d, -1.0)),
        });
    }
    Ok(pts)
}
