use crate::geom::*;
use crate::{path, polygon, Panel};

pub const MIN_LENGTH: f64 = 20.0;

pub fn anchor(panel: &Panel) -> Option<(V2, Bounds)> {
    let bounds = path::bounds(&panel.outline, true)?;
    let mid = center(bounds);
    let poly = path::flatten(&panel.outline, true, 0.1);
    if polygon::contains(&poly, mid) {
        Some((mid, bounds))
    } else {
        Some((polygon::centroid(&poly), bounds))
    }
}

pub fn default_length(bounds: Bounds) -> f64 {
    (0.5 * (bounds[1][1] - bounds[0][1])).max(MIN_LENGTH)
}

pub fn through(centre: V2, direction: V2, length: f64) -> [V2; 2] {
    let half = mul(direction, 0.5 * length);
    [sub(centre, half), add(centre, half)]
}

pub fn default_line(panel: &Panel) -> Option<[V2; 2]> {
    let (mid, bounds) = anchor(panel)?;
    Some(through(mid, [0.0, 1.0], default_length(bounds)))
}

pub fn effective(panel: &Panel) -> Option<([V2; 2], bool)> {
    match panel.grain_line {
        Some(line) => Some((line, false)),
        None => default_line(panel).map(|l| (l, true)),
    }
}

pub fn angle_of(line: [V2; 2]) -> f64 {
    let d = sub(line[1], line[0]);
    d[1].atan2(d[0]).to_degrees()
}
