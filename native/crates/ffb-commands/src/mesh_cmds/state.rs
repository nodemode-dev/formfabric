use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use ffb_mesh::hash::panel_key;
use ffb_mesh::quality::measure;
use ffb_mesh::{build_panel, MeshOptions, PanelMesh, Quality};
use ffb_pattern::Panel;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now_ms() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() * 1000.0
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn now_ms() -> f64 {
    0.0
}

pub(crate) const MAX_TRIANGLES: f64 = 2_000_000.0;

fn too_many_triangles(panel: &Panel) -> Option<String> {
    let d = panel.particle_distance;
    if !d.is_finite() || d <= 0.0 || panel.outline.len() < 3 {
        return None;
    }
    let flat = ffb_pattern::path::flatten(&panel.outline, true, (d * 0.05).clamp(0.05, 2.0));
    let estimate = ffb_pattern::polygon::signed_area(&flat).abs() / (0.433_012_701_892_219_3 * d * d);
    (estimate.is_nan() || estimate > MAX_TRIANGLES).then(|| format!("panel {} would need about {:.0} triangles at a particle distance of {d} mm; the limit is {MAX_TRIANGLES:.0}, so raise the particle distance", panel.name, estimate))
}

#[derive(Debug)]
pub(crate) struct Built {
    pub mesh: PanelMesh,
    pub quality: Quality,
    pub ms: f64,
}

#[derive(Debug)]
pub(crate) struct Entry {
    pub key: u64,
    pub built: Result<Arc<Built>, String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Sent {
    pub topology: u64,
    pub positions: u64,
}

pub(crate) struct Ensured {
    pub key: u64,
    pub built: Result<Arc<Built>, String>,
    pub rebuilt: bool,
}

#[derive(Debug, Default)]
pub struct MeshState {
    pub(crate) cache: HashMap<u32, Entry>,
    pub(crate) forced: HashMap<u32, BTreeMap<usize, usize>>,
    pub(crate) sent: HashMap<u32, Sent>,
    pub(crate) builds: u64,
    pub(crate) hits: u64,
}

impl MeshState {
    pub(crate) fn options(&self, panel: u32) -> MeshOptions {
        MeshOptions { segment_counts: self.forced.get(&panel).cloned().unwrap_or_default(), ..MeshOptions::default() }
    }

    pub(crate) fn set_segment_counts(&mut self, panel: u32, counts: BTreeMap<usize, usize>) {
        if counts.is_empty() {
            self.forced.remove(&panel);
        } else {
            self.forced.insert(panel, counts);
        }
    }

    pub(crate) fn ensure(&mut self, panel: &Panel, force: bool) -> Ensured {
        let options = self.options(panel.id);
        let key = panel_key(panel, &options);
        if !force {
            if let Some(entry) = self.cache.get(&panel.id).filter(|e| e.key == key) {
                self.hits += 1;
                return Ensured { key, built: entry.built.clone(), rebuilt: false };
            }
        }
        let start = now_ms();
        let built = match too_many_triangles(panel) {
            Some(message) => Err(message),
            None => build_panel(panel, &options)
                .map(|mesh| {
                    let quality = measure(&mesh);
                    Arc::new(Built { mesh, quality, ms: now_ms() - start })
                })
                .map_err(|e| e.to_string()),
        };
        self.builds += 1;
        self.cache.insert(panel.id, Entry { key, built: built.clone() });
        Ensured { key, built, rebuilt: true }
    }

    pub(crate) fn forget_missing(&mut self, live: &HashSet<u32>) {
        self.cache.retain(|id, _| live.contains(id));
        self.forced.retain(|id, _| live.contains(id));
    }
}
