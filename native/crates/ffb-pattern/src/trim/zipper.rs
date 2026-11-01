use std::collections::BTreeMap;

use super::style::param;
use crate::sew::{self, end_length, side_length, t_after_length, Binding, MeshView};
use crate::{Garment, Seam, SeamEnd};

pub fn closed_ends(garment: &Garment, ends: &[SeamEnd], fraction: f64) -> Vec<SeamEnd> {
    let total = side_length(garment, ends).unwrap_or(0.0);
    let target = total * fraction.clamp(0.0, 1.0);
    let mut out = Vec::new();
    let mut spent = 0.0;
    for end in ends {
        let Some(len) = end_length(garment, end) else { continue };
        if spent + len <= target + 1e-6 {
            out.push(end.clone());
            spent += len;
            continue;
        }
        let rest = target - spent;
        if rest > 1e-3 {
            let mut cut = end.clone();
            if end.t0 <= end.t1 {
                if let Some(t) = t_after_length(garment, end, end.t0, rest) {
                    cut.t1 = t.min(end.t1);
                }
            } else if let Some(t) = t_after_length(garment, end, end.t1, len - rest) {
                cut.t1 = t.max(end.t1);
            }
            if (cut.t1 - cut.t0).abs() > 1e-6 {
                out.push(cut);
            }
        }
        break;
    }
    out
}

pub fn zipper_bindings(garment: &Garment, meshes: &BTreeMap<u32, MeshView>) -> Vec<Binding> {
    let mut seams = Vec::new();
    for z in &garment.zippers {
        if z.closed <= 1e-6 {
            continue;
        }
        let (a, b) = (closed_ends(garment, &z.a, z.closed), closed_ends(garment, &z.b, z.closed));
        if a.is_empty() || b.is_empty() {
            continue;
        }
        let stiffness = garment.styles.iter().find(|s| s.id == z.style).map_or(1.0, |s| param(s, "stiffness", 1.0));
        seams.push(Seam { id: z.id, a, b, reversed: false, tension: stiffness, ..Seam::default() });
    }
    if seams.is_empty() {
        return Vec::new();
    }
    let temp = Garment { panels: garment.panels.clone(), seams, ..Garment::default() };
    sew::seam_bindings(&temp, meshes)
}
