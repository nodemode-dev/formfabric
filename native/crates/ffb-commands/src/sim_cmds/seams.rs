use std::collections::HashMap;

use ffb_mesh::PanelMesh;
use ffb_pattern::{InnerRole, Panel};
use ffb_sim::Fold;

pub(crate) fn inner_folds(panel: &Panel, mesh: &PanelMesh, start: u32) -> Vec<Fold> {
    let wanted: Vec<_> = panel.inner.iter().filter(|i| i.role == InnerRole::Fold).collect();
    if wanted.is_empty() {
        return Vec::new();
    }
    let mut edges: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for t in &mesh.triangles {
        for (a, b, o) in [(t[0], t[1], t[2]), (t[1], t[2], t[0]), (t[2], t[0], t[1])] {
            edges.entry((a.min(b), a.max(b))).or_default().push(o);
        }
    }
    let mut out = Vec::new();
    for inner in wanted {
        for (id, chain) in &mesh.inner_edges {
            if *id != inner.id {
                continue;
            }
            for w in chain.windows(2) {
                if let Some(o) = edges.get(&(w[0].min(w[1]), w[0].max(w[1]))).filter(|o| o.len() == 2) {
                    out.push(Fold {
                        edge: [start + w[0], start + w[1]],
                        opposite: [start + o[0], start + o[1]],
                        angle: inner.fold_angle.to_radians() as f32,
                        strength: inner.fold_strength as f32,
                    });
                }
            }
        }
    }
    out
}
