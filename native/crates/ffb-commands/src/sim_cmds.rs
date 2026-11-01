mod adapter;
mod apply;
mod assemble;
mod build;
mod drape;
mod ops;
mod pins;
mod runtime;
mod seams;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::{Command, Session};

pub struct SimState {
    pub(crate) core: Arc<Mutex<runtime::Core>>,
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) generation: Arc<AtomicU64>,
    pub(crate) drape: drape::DrapeCache,
    pub(crate) seen: HashMap<u32, u64>,
    pub(crate) preset: String,
}

impl Default for SimState {
    fn default() -> Self {
        SimState {
            core: Arc::new(Mutex::new(runtime::Core::default())),
            running: Arc::new(AtomicBool::new(false)),
            generation: Arc::new(AtomicU64::new(0)),
            drape: drape::DrapeCache::default(),
            seen: HashMap::new(),
            preset: "normal".into(),
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
        if sim.seen.insert(panel.id, sig) == Some(sig) {
            continue;
        }
        let Ok(built) = mesh.ensure(panel, false).built else { continue };
        if let Some(cached) = sim.drape.get(panel.id, sig, built.mesh.positions.len()) {
            let flat: Vec<f32> = cached.iter().flat_map(|p| p.iter().copied()).collect();
            sink.positions(panel.id, &flat);
        }
    }
}

pub(crate) fn after_change(session: &mut Session, _command: &str) {
    overlay(session);
    let core = session.sim.core.clone();
    let built = {
        let mut c = runtime::lock(&core);
        c.sink = session.sink.clone();
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
    all
}
