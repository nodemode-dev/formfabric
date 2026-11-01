mod adapter;
mod apply;
mod assemble;
mod build;
mod drape;
mod ops;
mod pinch;
mod pins;
mod refit;
mod runtime;
mod seams;
mod states;
mod wind;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::{Command, Session};

pub(crate) use pinch::current_positions;
pub(crate) use drape::rigid_fit;
pub(crate) use pins::MAX_PINS;

pub struct SimState {
    pub(crate) core: Arc<Mutex<runtime::Core>>,
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) generation: Arc<AtomicU64>,
    pub(crate) seen: HashMap<u32, u64>,
    pub(crate) preset: String,
    pub(crate) device: String,
    pub(crate) params: Arc<Mutex<ffb_sim::Params>>,
    pub(crate) live: Mutex<serde_json::Value>,
}

impl Default for SimState {
    fn default() -> Self {
        let core = runtime::Core::default();
        SimState {
            params: core.shared.clone(),
            live: Mutex::new(serde_json::Value::Null),
            core: Arc::new(Mutex::new(core)),
            running: Arc::new(AtomicBool::new(false)),
            generation: Arc::new(AtomicU64::new(0)),
            seen: HashMap::new(),
            preset: "normal".into(),
            device: "cpu".into(),
        }
    }
}

impl Drop for SimState {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}

fn overlay(session: &mut Session) {
    let Session { garment, mesh, sim, sink, .. } = session;
    let Some(sink) = sink.clone() else {
        sim.seen.clear();
        return;
    };
    for panel in &garment.panels {
        let sig = drape::placement_signature(panel);
        let previous = sim.seen.insert(panel.id, sig);
        if previous == Some(sig) {
            continue;
        }
        let Ok(built) = mesh.ensure(panel, false).built else { continue };
        let posed = panel.place3d.as_ref().and_then(|p| p.pose.as_ref()).filter(|p| p.len() == built.mesh.positions.len());
        match posed {
            Some(pose) => {
                let flat: Vec<f32> = pose.iter().flat_map(|p| p.iter().copied()).collect();
                sink.positions(panel.id, &flat);
            }
            None if previous.is_some() => sink.positions(panel.id, &ffb_mesh::layout::positions_3d(&built.mesh, panel)),
            None => {}
        }
    }
}

pub(crate) fn after_change(session: &mut Session, command: &str) {
    if command.starts_with("file.") {
        session.sim.seen.clear();
    }
    overlay(session);
    let core = session.sim.core.clone();
    let built = {
        let mut c = runtime::lock(&core);
        c.sink = session.sink.clone();
        c.wind = session.garment.wind.clone();
        if c.stepper.is_some() {
            c.stale = true;
        }
        c.stepper.is_some()
    };
    if built && session.sim.running.load(Ordering::Acquire) && build::ensure(session).is_err() {
        session.sim.running.store(false, Ordering::Release);
        session.sim.generation.fetch_add(1, Ordering::AcqRel);
        runtime::lock(&core).stepper = None;
    }
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = ops::commands();
    all.extend(pins::commands());
    all.extend(apply::commands());
    all.extend(pinch::commands());
    all.extend(refit::commands());
    all
}
