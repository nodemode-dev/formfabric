use std::collections::BTreeMap;

use super::locate::TrimMesh;
use super::style::param;
use crate::geom::{closest_on_segment, dist, V2};
use crate::sew::{end_path, span_curve};
use crate::{path, EdgeKind, Garment, SeamEnd, StyleKind};

const TOL: f64 = 0.2;

struct Band {
    panel: u32,
    lines: Vec<Vec<V2>>,
    half: f64,
    factor: f64,
}

fn polyline(garment: &Garment, end: &SeamEnd) -> Vec<V2> {
    let Some((pts, closed)) = end_path(garment, end) else { return Vec::new() };
    if end.segment >= path::seg_count(pts.len(), closed) {
        return Vec::new();
    }
    let seg = path::seg(pts, end.segment);
    span_curve(&seg, end.t0.clamp(0.0, 1.0), end.t1.clamp(0.0, 1.0)).polyline(TOL)
}

fn gather(garment: &Garment, ends: &[SeamEnd], half: f64, factor: f64, out: &mut Vec<Band>) {
    let mut panels: Vec<u32> = ends.iter().map(|e| e.panel).collect();
    panels.sort_unstable();
    panels.dedup();
    for panel in panels {
        let lines: Vec<Vec<V2>> = ends.iter().filter(|e| e.panel == panel).map(|e| polyline(garment, e)).filter(|l| l.len() >= 2).collect();
        if !lines.is_empty() && factor > 1.0 {
            out.push(Band { panel, lines, half, factor });
        }
    }
}

fn bands_of(garment: &Garment) -> Vec<Band> {
    let mut out = Vec::new();
    for f in &garment.finishes {
        let (default_strength, default_width) = match f.kind {
            EdgeKind::Tape => (3.0, 10.0),
            EdgeKind::Bond => (5.0, 10.0),
            EdgeKind::Press => (1.5, 10.0),
            _ => continue,
        };
        let strength = if f.strength > 0.0 { f.strength } else { default_strength };
        let width = if f.width > 0.0 { f.width } else { default_width };
        gather(garment, &f.spans, width, strength, &mut out);
    }
    for s in &garment.stitches {
        let Some(style) = garment.styles.iter().find(|x| x.id == s.style) else { continue };
        let factor = match style.kind {
            StyleKind::Topstitch => param(style, "stiffen", 1.15),
            StyleKind::Puckering => param(style, "stiffen", 1.0),
            _ => continue,
        };
        gather(garment, &s.spans, 2.0 + s.offset.abs(), factor, &mut out);
    }
    out
}

fn near(lines: &[Vec<V2>], p: V2) -> f64 {
    let mut best = f64::MAX;
    for line in lines {
        for w in line.windows(2) {
            best = best.min(dist(p, closest_on_segment(p, w[0], w[1]).1));
        }
    }
    best
}

pub fn stiffen_bands(garment: &Garment, meshes: &BTreeMap<u32, TrimMesh>) -> BTreeMap<u32, Vec<f32>> {
    let mut out: BTreeMap<u32, Vec<f32>> = BTreeMap::new();
    for band in bands_of(garment) {
        let Some(mesh) = meshes.get(&band.panel) else { continue };
        if garment.panels.iter().any(|p| p.id == band.panel && p.state.disabled) {
            continue;
        }
        let slot = out.entry(band.panel).or_insert_with(|| vec![1.0; mesh.triangles.len()]);
        if slot.len() != mesh.triangles.len() {
            continue;
        }
        for (k, tri) in mesh.triangles.iter().enumerate() {
            let corners: Vec<V2> = tri.iter().filter_map(|i| mesh.positions.get(*i as usize).copied()).collect();
            if corners.len() != 3 {
                continue;
            }
            let centre = [(corners[0][0] + corners[1][0] + corners[2][0]) / 3.0, (corners[0][1] + corners[1][1] + corners[2][1]) / 3.0];
            if corners.iter().chain(std::iter::once(&centre)).any(|p| near(&band.lines, *p) <= band.half) {
                slot[k] = slot[k].max(band.factor as f32);
            }
        }
    }
    out.retain(|_, v| v.iter().any(|x| *x > 1.0));
    out
}
