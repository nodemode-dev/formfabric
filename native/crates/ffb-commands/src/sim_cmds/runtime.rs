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
    pub closing_done: u32,
    pub closing_over: bool,
    pub close_total: u32,
    pub want_gpu: bool,
    pub device_note: String,
    pub on_gpu: bool,
}

fn closing_params(base: &Params, u: f32) -> Params {
    let ramp = ((u - 0.6) / 0.4).clamp(0.0, 1.0);
    let gravity = 0.1 + 0.9 * ramp;
    let damping = base.air_damping.max(6.0) * (1.0 - ramp) + base.air_damping * ramp;
    Params { gravity: base.gravity.map(|g| g * gravity), air_damping: damping, ..*base }
}

fn closing_links(u: f32) -> f64 {
    let up = (u / 0.35).clamp(0.0, 1.0);
    let down = ((u - 0.7) / 0.3).clamp(0.0, 1.0);
    f64::from(1.0 + 24.0 * up * (1.0 - down))
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
                if adapter::accepts(st, &wanted) {
                    st.set_params(wanted);
                } else {
                    self.reset();
                }
            }
        }
    }

    pub fn make_stepper(&mut self, cloth: Cloth) -> Stepper {
        let (stepper, note) = adapter::new_stepper(cloth, self.params, self.want_gpu);
        self.on_gpu = adapter::is_gpu(&stepper);
        self.device_note = note;
        stepper
    }

    pub fn install(&mut self, assembled: Assembled) {
        self.install_inner(assembled, false);
    }

    pub fn install_keep(&mut self, assembled: Assembled) {
        self.install_inner(assembled, true);
    }

    fn install_inner(&mut self, assembled: Assembled, keep: bool) {
        self.sync_params();
        let Assembled { mut cloth, slices, colliders, extras } = assembled;
        let colliders = if keep { self.colliders.clone() } else { colliders };
        let mut start = cloth.clone();
        if let Some(old) = &self.stepper {
            let x = old.positions();
            for s in &slices {
                let kept = self.slices.iter().find(|o| o.id == s.id && (keep || o.sig == s.sig) && o.count == s.count);
                if let Some(o) = kept.filter(|o| o.start + o.count <= x.len()) {
                    start.positions[s.start..s.start + s.count].copy_from_slice(&x[o.start..o.start + o.count]);
                }
            }
        }
        self.solid = states::finish(&mut start, &extras);
        states::finish(&mut cloth, &extras);
        self.pins.clone_from(&start.pins);
        let mut stepper = self.make_stepper(start);
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
        self.closing_done = 0;
        self.close_total = 0;
        self.closing_over = keep;
    }

    pub fn reset(&mut self) {
        self.sync_params();
        if self.base.positions.is_empty() {
            return;
        }
        let mut stepper = self.make_stepper(self.base.clone());
        stepper.set_colliders(self.colliders.clone());
        self.stepper = Some(stepper);
        self.pins.clone_from(&self.base.pins);
        self.frame = 0;
        self.stats = Stats::default();
        self.grabbed = None;
        self.pinch = None;
        self.wind_on = false;
        self.closing_done = 0;
        self.close_total = 0;
        self.closing_over = false;
    }

    pub fn step_frames(&mut self, n: u32) -> Stats {
        self.sync_params();
        let Some(stepper) = self.stepper.as_mut() else { return self.stats };
        let mut last = self.stats;
        let base = self.params;
        for _ in 0..n {
            if !self.closing_over && self.closing_done == 0 {
                if base.close_frames == 0 || stepper.link_count() == 0 {
                    self.closing_over = true;
                } else {
                    stepper.push_out(0.0);
                }
            }
            let total = if self.close_total == 0 { base.close_frames } else { self.close_total }.max(1) as f32;
            if !self.closing_over {
                let u = self.closing_done as f32 / total;
                stepper.set_params(closing_params(&base, u));
                stepper.scale_links(closing_links(u));
            }
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
            if !self.closing_over {
                let before = self.closing_done as f32 / total;
                stepper.damp_velocities(f64::from(0.5 + 0.5 * ((before - 0.6) / 0.4).clamp(0.0, 1.0)));
                self.closing_done += 1;
                let u = self.closing_done as f32 / total;
                if u >= 1.0 {
                    self.closing_over = true;
                    stepper.set_params(base);
                    stepper.scale_links(1.0);
                } else if self.close_total == 0 && (0.5..=0.6).contains(&u) && stepper.max_link_gap() < base.close_gap {
                    self.close_total = ((self.closing_done as f32 / 0.6).ceil() as u32).max(self.closing_done + 1);
                }
            }
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
