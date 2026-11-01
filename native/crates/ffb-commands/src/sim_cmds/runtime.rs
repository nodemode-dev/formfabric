use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex, MutexGuard};

use ffb_pattern::Wind;
use ffb_sim::{Cloth, Collider, Params, PinPoint};

use super::adapter::{self, Stats, Stepper};
use super::assemble::{Assembled, Slice};
use super::pinch::Pinch;
use super::states::{self, Extras};
use super::wind;
use crate::FrameSink;

pub(crate) type Snapshot = Vec<(u32, Vec<f32>)>;

#[derive(Default)]
pub(crate) struct Core {
    pub stepper: Option<Stepper>,
    pub base: Cloth,
    pub slices: Vec<Slice>,
    pub colliders: Vec<Collider>,
    pub params: Params,
    pub frame: u64,
    pub stats: Stats,
    pub sink: Option<Arc<dyn FrameSink>>,
    pub grabbed: Option<u32>,
    pub stale: bool,
    pub wind: Wind,
    pub wind_on: bool,
    pub forces: Vec<[f32; 3]>,
    pub solid: usize,
    pub extras: Extras,
    pub pins: Vec<PinPoint>,
    pub pinch: Option<Pinch>,
    pub shared: Arc<Mutex<Params>>,
}

pub(crate) const MIN_TIME_STEP: f32 = 1.0 / 240.0;
pub(crate) const MAX_TIME_STEP: f32 = 1.0 / 15.0;

pub(crate) fn lock(core: &Arc<Mutex<Core>>) -> MutexGuard<'_, Core> {
    core.lock().unwrap_or_else(|e| e.into_inner())
}

impl Core {
    pub fn sync_params(&mut self) {
        let wanted = match self.shared.try_lock() {
            Ok(p) => *p,
            Err(std::sync::TryLockError::Poisoned(e)) => *e.into_inner(),
            Err(_) => return,
        };
        if wanted != self.params {
            self.params = wanted;
            if let Some(st) = self.stepper.as_mut() {
                st.set_params(wanted);
            }
        }
    }

    pub fn install(&mut self, assembled: Assembled) {
        self.sync_params();
        let Assembled { mut cloth, slices, colliders, extras } = assembled;
        let mut start = cloth.clone();
        if let Some(old) = &self.stepper {
            let x = old.positions();
            for s in &slices {
                let kept = self.slices.iter().find(|o| o.id == s.id && o.sig == s.sig && o.count == s.count);
                if let Some(o) = kept.filter(|o| o.start + o.count <= x.len()) {
                    start.positions[s.start..s.start + s.count].copy_from_slice(&x[o.start..o.start + o.count]);
                }
            }
        }
        self.solid = states::finish(&mut start, &extras);
        states::finish(&mut cloth, &extras);
        self.pins.clone_from(&start.pins);
        let mut stepper = Stepper::new(start, self.params);
        stepper.set_colliders(colliders.clone());
        self.stepper = Some(stepper);
        self.base = cloth;
        self.slices = slices;
        self.colliders = colliders;
        self.extras = extras;
        self.grabbed = None;
        self.pinch = None;
        self.wind_on = false;
        self.stale = false;
    }

    pub fn reset(&mut self) {
        self.sync_params();
        if self.base.positions.is_empty() {
            return;
        }
        let mut stepper = Stepper::new(self.base.clone(), self.params);
        stepper.set_colliders(self.colliders.clone());
        self.stepper = Some(stepper);
        self.pins.clone_from(&self.base.pins);
        self.frame = 0;
        self.stats = Stats::default();
        self.grabbed = None;
        self.pinch = None;
        self.wind_on = false;
    }

    pub fn step_frames(&mut self, n: u32) -> Stats {
        self.sync_params();
        let Some(stepper) = self.stepper.as_mut() else { return self.stats };
        let mut last = self.stats;
        for _ in 0..n {
            if self.wind.enabled {
                let time = self.frame as f64 * f64::from(self.params.time_step);
                wind::forces(&self.wind, stepper.positions(), &self.base.triangles, time, &mut self.forces);
                adapter::set_external(stepper, &self.forces);
                self.wind_on = true;
            } else if self.wind_on {
                adapter::set_external(stepper, &[]);
                self.wind_on = false;
            }
            last = stepper.step();
            self.frame += 1;
        }
        self.stats = last;
        last
    }

    pub fn snapshot(&self) -> Snapshot {
        let Some(stepper) = &self.stepper else { return Vec::new() };
        let x = stepper.positions();
        self.slices
            .iter()
            .filter(|s| s.start + s.count <= x.len())
            .map(|s| (s.id, x[s.start..s.start + s.count].iter().flat_map(|p| p.iter().copied()).collect()))
            .collect()
    }

    pub fn placed_snapshot(&self) -> Snapshot {
        let x = &self.base.positions;
        self.slices
            .iter()
            .filter(|s| s.start + s.count <= x.len())
            .map(|s| (s.id, x[s.start..s.start + s.count].iter().flat_map(|p| p.iter().copied()).collect()))
            .collect()
    }
}

pub(crate) fn publish(sink: &Option<Arc<dyn FrameSink>>, snapshot: &Snapshot) {
    if let Some(sink) = sink {
        for (id, positions) in snapshot {
            sink.positions(*id, positions);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn spawn(core: Arc<Mutex<Core>>, running: Arc<AtomicBool>, generation: Arc<AtomicU64>, mine: u64) -> bool {
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};
    let body = move || loop {
        if !running.load(Ordering::Acquire) || generation.load(Ordering::Acquire) != mine {
            break;
        }
        let began = Instant::now();
        let (snapshot, sink, period) = {
            let mut c = lock(&core);
            if c.stepper.is_none() {
                running.store(false, Ordering::Release);
                break;
            }
            c.step_frames(1);
            (c.snapshot(), c.sink.clone(), c.params.time_step)
        };
        publish(&sink, &snapshot);
        let period = Duration::from_secs_f32(period.clamp(MIN_TIME_STEP, MAX_TIME_STEP));
        std::thread::sleep(period.saturating_sub(began.elapsed()).max(Duration::from_millis(1)));
    };
    std::thread::Builder::new().name("ffb-sim".into()).spawn(body).is_ok()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn spawn(_core: Arc<Mutex<Core>>, _running: Arc<AtomicBool>, _generation: Arc<AtomicU64>, _mine: u64) -> bool {
    false
}
