use crate::geom::*;
use crate::path::reverse;
use crate::Panel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirror {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
pub struct Xform {
    pub mirror: Option<Mirror>,
    pub scale: Option<V2>,
    pub rotate: f64,
    pub translate: V2,
    pub pivot: V2,
}

fn wrap(deg: f64) -> f64 {
    let r = deg.rem_euclid(360.0);
    if r > 180.0 {
        r - 360.0
    } else {
        r
    }
}

fn bake(panel: &mut Panel, l: V2, pivot: V2) {
    let place = panel.placement();
    let f = |w: V2| [pivot[0] + l[0] * (w[0] - pivot[0]), pivot[1] + l[1] * (w[1] - pivot[1])];
    let fv = |v: V2| [l[0] * v[0], l[1] * v[1]];
    let new_pos = f(panel.position);
    let world_grain = panel.rotation + if panel.mirror { 180.0 - panel.grain } else { panel.grain };
    let g = fv(rotate([1.0, 0.0], world_grain));
    let point = |p: V2| sub(f(place.to_world(p)), new_pos);
    let handle = |h: V2| fv(place.vec_to_world(h));
    let flips = l[0] * l[1] < 0.0;
    let paths = std::iter::once(&mut panel.outline).chain(panel.inner.iter_mut().map(|i| &mut i.points));
    for pts in paths {
        for p in pts.iter_mut() {
            p.p = point(p.p);
            p.h_in = p.h_in.map(handle);
            p.h_out = p.h_out.map(handle);
        }
        if flips {
            reverse(pts);
        }
    }
    panel.grain_line = panel.grain_line.map(|[a, b]| [point(a), point(b)]);
    if flips {
        crate::notch::on_reverse(panel);
        crate::sew::note_reverse(panel);
    }
    panel.position = new_pos;
    panel.rotation = 0.0;
    panel.mirror = false;
    panel.grain = wrap(g[1].atan2(g[0]).to_degrees());
}

fn uniform(panel: &mut Panel, s: f64, pivot: V2) {
    panel.position = add(pivot, mul(sub(panel.position, pivot), s));
    let paths = std::iter::once(&mut panel.outline).chain(panel.inner.iter_mut().map(|i| &mut i.points));
    for pts in paths {
        for p in pts.iter_mut() {
            p.p = mul(p.p, s);
            p.h_in = p.h_in.map(|h| mul(h, s));
            p.h_out = p.h_out.map(|h| mul(h, s));
        }
    }
    panel.grain_line = panel.grain_line.map(|[a, b]| [mul(a, s), mul(b, s)]);
}

pub fn apply(panel: &mut Panel, x: &Xform) {
    if let Some(m) = x.mirror {
        bake(panel, if m == Mirror::X { [-1.0, 1.0] } else { [1.0, -1.0] }, x.pivot);
    }
    if let Some(s) = x.scale {
        if (s[0] - s[1]).abs() < 1e-12 {
            uniform(panel, s[0], x.pivot);
        } else {
            bake(panel, s, x.pivot);
        }
    }
    if x.rotate != 0.0 {
        panel.position = add(x.pivot, rotate(sub(panel.position, x.pivot), x.rotate));
        panel.rotation = wrap(panel.rotation + x.rotate);
    }
    panel.position = add(panel.position, x.translate);
}
