use crate::geom::*;
use crate::path;
use crate::polygon;
use crate::{Panel, Point};

const TOL: f64 = 0.05;
const MIN_AREA: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Traced {
    pub outline: Vec<Point>,
    pub position: V2,
    pub area: f64,
}

pub fn world_points(source: &Panel, points: &[Point]) -> Vec<Point> {
    let place = source.placement();
    points
        .iter()
        .map(|p| Point { p: place.to_world(p.p), kind: p.kind, h_in: p.h_in.map(|h| place.vec_to_world(h)), h_out: p.h_out.map(|h| place.vec_to_world(h)) })
        .collect()
}

pub fn world_grain(panel: &Panel) -> f64 {
    let g = panel.rotation + if panel.mirror { 180.0 - panel.grain } else { panel.grain };
    crate::link::wrap(g)
}

pub fn localize(mut world: Vec<Point>) -> Result<Traced, String> {
    if world.len() < 3 {
        return Err("a panel needs at least 3 points".into());
    }
    let area = polygon::signed_area(&path::flatten(&world, true, TOL));
    if area.abs() < MIN_AREA {
        return Err("the region encloses no area".into());
    }
    if let Some((i, j)) = polygon::self_intersections(&path::flatten_segments(&world, true, TOL), true).first() {
        return Err(format!("the region crosses itself between segments {i} and {j}"));
    }
    if area < 0.0 {
        path::reverse(&mut world);
    }
    let bounds = path::bounds(&world, true).ok_or("the region is empty")?;
    let origin = bounds[0];
    for p in world.iter_mut() {
        p.p = sub(p.p, origin);
    }
    Ok(Traced { outline: world, position: origin, area: area.abs() })
}

pub fn from_inner(source: &Panel, inner: u32) -> Result<Traced, String> {
    let shape = source.inner.iter().find(|i| i.id == inner).ok_or_else(|| format!("panel {} has no inner shape {inner}", source.id))?;
    if !shape.closed || shape.points.len() < 3 {
        return Err(format!("inner shape {inner} is not a closed shape"));
    }
    localize(world_points(source, &shape.points))
}

pub fn region_at(source: &Panel, world: V2) -> Option<u32> {
    let local = source.placement().to_local(world);
    source
        .inner
        .iter()
        .filter(|i| i.closed && i.points.len() >= 3)
        .filter_map(|i| {
            let poly = path::flatten(&i.points, true, TOL);
            polygon::contains(&poly, local).then(|| (i.id, polygon::signed_area(&poly).abs()))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|x| x.0)
}
