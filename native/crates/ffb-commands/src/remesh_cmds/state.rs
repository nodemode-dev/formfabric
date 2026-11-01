use std::collections::{BTreeMap, HashMap};

use ffb_mesh::hash::{panel_key, Fnv};
use ffb_mesh::MeshOptions;
use ffb_pattern::{grain, Garment, Panel};
use ffb_remesh::{Metrics, QuadMesh};

pub(crate) type Guides = Vec<Vec<[f64; 2]>>;

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub key: u64,
    pub mesh: QuadMesh,
    pub metrics: Metrics,
    pub ms: f64,
}

#[derive(Debug, Default)]
pub struct RemeshState {
    pub(crate) meshes: HashMap<u32, Entry>,
}

pub(crate) fn guides_of(garment: &Garment, panel: u32) -> Guides {
    let Some(list) = garment.output.retopo.get("guides").and_then(|v| v.as_array()) else { return Vec::new() };
    let mut out = Vec::new();
    for g in list {
        if g.get("panel").and_then(|v| v.as_u64()) != Some(u64::from(panel)) {
            continue;
        }
        let Some(points) = g.get("points").and_then(|v| v.as_array()) else { continue };
        let line: Vec<[f64; 2]> = points
            .iter()
            .filter_map(|p| {
                let a = p.as_array()?;
                Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
            })
            .collect();
        if line.len() >= 2 {
            out.push(line);
        }
    }
    out
}

pub(crate) fn key_of(panel: &Panel, density: f64, pure: bool, counts: &BTreeMap<usize, usize>, guides: &Guides) -> u64 {
    let mut h = Fnv::default();
    h.u64(panel_key(panel, &MeshOptions::default()));
    h.f64(density);
    h.u64(u64::from(pure));
    for (s, c) in counts {
        h.u64(*s as u64);
        h.u64(*c as u64);
    }
    for g in guides {
        h.u64(g.len() as u64);
        for p in g {
            h.pair(*p);
        }
    }
    h.f64(grain::effective(panel).map_or(0.0, |(l, _)| grain::angle_of(l)));
    h.finish()
}
