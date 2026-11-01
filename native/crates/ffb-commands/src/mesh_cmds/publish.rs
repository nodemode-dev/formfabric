use std::collections::HashSet;

use ffb_mesh::hash::Fnv;
use ffb_mesh::layout;
use ffb_pattern::{Garment, Panel};

use super::state::{MeshState, Sent};
use crate::{FrameSink, Session};

const EMPTY: u64 = 1;

fn placement_signature(panel: &Panel) -> u64 {
    let mut h = Fnv::default();
    match &panel.place3d {
        Some(p) => {
            h.u64(1);
            for v in p.origin.iter().chain(p.rotation.iter()) {
                h.f64(*v);
            }
        }
        None => {
            h.u64(0);
            h.pair(panel.position);
            h.f64(panel.rotation);
            h.u64(u64::from(panel.mirror));
        }
    }
    h.finish()
}

pub(super) fn sync_panel(panel: &Panel, mesh: &mut MeshState, sink: &dyn FrameSink, full: bool) {
    let ensured = mesh.ensure(panel, false);
    let previous = mesh.sent.get(&panel.id).copied();
    match ensured.built {
        Err(_) => {
            if full || previous.map(|s| s.topology) != Some(EMPTY) {
                sink.topology(panel.id, &[], &[]);
            }
            mesh.sent.insert(panel.id, Sent { topology: EMPTY, positions: 0 });
        }
        Ok(built) => {
            let flip = layout::mirrored_flat(panel);
            let mut h = Fnv::default();
            h.u64(ensured.key);
            h.u64(u64::from(flip));
            let topology = h.finish().max(2);
            let positions = placement_signature(panel);
            let new_topology = full || previous.map(|s| s.topology) != Some(topology);
            if new_topology {
                sink.topology(panel.id, &layout::indices(&built.mesh, flip), &layout::panel_ids(&built.mesh));
            }
            if new_topology || previous.map(|s| s.positions) != Some(positions) {
                sink.positions(panel.id, &layout::positions_3d(&built.mesh, panel));
            }
            mesh.sent.insert(panel.id, Sent { topology, positions });
        }
    }
}

fn sync(garment: &Garment, mesh: &mut MeshState, sink: &dyn FrameSink, full: bool) {
    let live: HashSet<u32> = garment.panels.iter().map(|p| p.id).collect();
    let gone: Vec<u32> = mesh.sent.keys().filter(|id| !live.contains(id)).copied().collect();
    for id in gone {
        sink.topology(id, &[], &[]);
        mesh.sent.remove(&id);
    }
    for panel in &garment.panels {
        sync_panel(panel, mesh, sink, full);
    }
}

pub(super) fn on_change(session: &mut Session) {
    let Session { garment, mesh, sink, .. } = session;
    let live: HashSet<u32> = garment.panels.iter().map(|p| p.id).collect();
    mesh.forget_missing(&live);
    match sink.clone() {
        Some(sink) => sync(garment, mesh, sink.as_ref(), false),
        None => mesh.sent.clear(),
    }
}

pub(super) fn full(session: &mut Session) {
    let Session { garment, mesh, sink, .. } = session;
    let live: HashSet<u32> = garment.panels.iter().map(|p| p.id).collect();
    mesh.forget_missing(&live);
    if let Some(sink) = sink.clone() {
        sync(garment, mesh, sink.as_ref(), true);
    }
}
