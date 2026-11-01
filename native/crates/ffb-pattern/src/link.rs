use crate::geom::*;
use crate::{Garment, LinkKind, Panel, PanelLink};

pub fn wrap(deg: f64) -> f64 {
    let r = deg.rem_euclid(360.0);
    if r > 180.0 {
        r - 360.0
    } else {
        r
    }
}

pub fn reflect(p: V2, a: V2, b: V2) -> V2 {
    let Some(u) = unit(sub(b, a)) else { return p };
    let v = sub(p, a);
    add(a, sub(mul(u, 2.0 * dot(v, u)), v))
}

pub fn reflect_vec(v: V2, a: V2, b: V2) -> V2 {
    let Some(u) = unit(sub(b, a)) else { return v };
    sub(mul(u, 2.0 * dot(v, u)), v)
}

pub fn instance_axis(interval: V2, ordinal: usize) -> [V2; 2] {
    [interval, [ordinal as f64, 0.0]]
}

pub fn is_derived(panel: &Panel) -> bool {
    panel.link.is_some()
}

pub fn has_links(garment: &Garment) -> bool {
    garment.panels.iter().any(is_derived)
}

pub fn derived_of(garment: &Garment, source: u32) -> Vec<usize> {
    garment.panels.iter().enumerate().filter(|(_, p)| p.link.as_ref().is_some_and(|l| l.source == source)).map(|(i, _)| i).collect()
}

pub fn sync(source: &Panel, target: &mut Panel, next_id: &mut u32) -> bool {
    let Some(PanelLink { kind, axis: Some(axis), .. }) = target.link.clone() else { return false };
    match kind {
        LinkKind::Mirror => {
            let phi = (axis[1][1] - axis[0][1]).atan2(axis[1][0] - axis[0][0]).to_degrees();
            target.position = reflect(source.position, axis[0], axis[1]);
            target.rotation = wrap(2.0 * phi + 180.0 - source.rotation);
            target.mirror = !source.mirror;
        }
        LinkKind::Instance => {
            target.position = add(source.position, mul(axis[0], axis[1][0]));
            target.rotation = source.rotation;
            target.mirror = source.mirror;
        }
    }
    target.outline = source.outline.clone();
    target.grain = source.grain;
    target.grain_line = source.grain_line;
    target.seam_allowance = source.seam_allowance.clone();
    target.notches = source.notches.clone();
    let old: Vec<u32> = target.inner.iter().map(|i| i.id).collect();
    target.inner = source.inner.clone();
    for (k, inner) in target.inner.iter_mut().enumerate() {
        inner.id = old.get(k).copied().unwrap_or_else(|| {
            let id = *next_id;
            *next_id += 1;
            id
        });
    }
    true
}
