use std::sync::atomic::Ordering;

use ffb_sim::Params;
use serde_json::{json, Value};

use super::adapter;
use super::build::{ensure, rebuild};
use super::runtime::{lock, publish, spawn, MAX_TIME_STEP, MIN_TIME_STEP};
use crate::{no_args, Command, CommandError, Group, Session, READ, SET, WRITE};

const MAX_STEP_FRAMES: u64 = 2000;
const MAX_COMPARE_FRAMES: u64 = 300;
const MAX_SYNC_VERTICES: usize = 60_000;
const STEP_BUDGET_SECONDS: f32 = 20.0;
const COMPARE_BUDGET_SECONDS: f32 = 30.0;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn round(x: f64, places: i32) -> f64 {
    let k = 10f64.powi(places);
    (x * k).round() / k
}

const MAX_POSITIONS: usize = 5000;

fn positions_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id as listed by pattern.get."},
            "decimate": {"type": "integer", "minimum": 1, "maximum": 1000, "default": 1, "description": "Keep every n-th vertex of the panel mesh."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn positions(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = a.get("panel").and_then(Value::as_u64).ok_or_else(|| bad("panel must be a panel id"))? as u32;
    let every = a.get("decimate").and_then(Value::as_u64).unwrap_or(1).clamp(1, 1000) as usize;
    let c = lock(&s.sim.core);
    let found = c.slices.iter().find(|x| x.id == id);
    let (Some(st), Some(sl)) = (c.stepper.as_ref(), found) else {
        return Err(CommandError::new("not_found", format!("panel {id} is not in the built cloth; run sim.reset first")));
    };
    let step = every.max(sl.count.div_ceil(MAX_POSITIONS)).max(1);
    let all = st.positions();
    let mut indices = Vec::new();
    let mut points = Vec::new();
    for local in (0..sl.count).step_by(step) {
        if let Some(p) = all.get(sl.start + local) {
            indices.push(local);
            points.push([round(f64::from(p[0]), 2), round(f64::from(p[1]), 2), round(f64::from(p[2]), 2)]);
        }
    }
    Ok(json!({"panel": id, "frame": c.frame, "vertices": sl.count, "step": step, "returned": points.len(), "indices": indices, "positions": points}))
}

pub(super) fn preset_params(name: &str, base: Params) -> Option<Params> {
    adapter::preset(name, base)
}

pub(super) fn state_json(session: &Session) -> Value {
    let described = match session.sim.core.try_lock() {
        Ok(c) => describe(&c, session),
        Err(std::sync::TryLockError::Poisoned(e)) => describe(&e.into_inner(), session),
        Err(std::sync::TryLockError::WouldBlock) => {
            let mut cached = session.sim.live.lock().unwrap_or_else(|e| e.into_inner()).clone();
            if let Some(map) = cached.as_object_mut() {
                map.insert("running".into(), json!(session.sim.running.load(Ordering::Acquire)));
                map.insert("busy".into(), json!(true));
                return cached;
            }
            json!({"running": session.sim.running.load(Ordering::Acquire), "busy": true})
        }
    };
    *session.sim.live.lock().unwrap_or_else(|e| e.into_inner()) = described.clone();
    described
}

fn describe(c: &super::runtime::Core, session: &Session) -> Value {
    let p = c.params;
    json!({
        "busy": false,
        "running": session.sim.running.load(Ordering::Acquire),
        "built": c.stepper.is_some(),
        "stale": c.stale,
        "frame": c.frame,
        "panels": c.slices.len(),
        "vertices": c.solid,
        "triangles": c.base.triangles.len(),
        "seams": c.base.seams.len(),
        "pins": session.garment.pins.len(),
        "pins_built": c.pins.len(),
        "folds": c.base.folds.len(),
        "placed_poses": session.garment.panels.iter().filter(|p| p.place3d.as_ref().is_some_and(|q| q.pose.is_some())).count(),
        "grabbed": c.grabbed,
        "pinched": c.pinch.as_ref().map(|p| p.members.len()),
        "wind": c.wind.enabled,
        "frozen_vertices": c.extras.frozen.iter().map(|f| f.1).sum::<usize>(),
        "held_vertices": c.extras.hold.iter().map(|f| f.1).sum::<usize>(),
        "preset": session.sim.preset,
        "method": adapter::method_name(&p),
        "device": session.sim.device,
        "device_active": if c.on_gpu { "gpu" } else { "cpu" },
        "device_note": c.device_note,
        "fps": round(1.0 / f64::from(p.time_step), 2),
        "kinetic_energy": c.stepper.as_ref().map(|s| round(f64::from(s.kinetic_energy()), 3)),
        "seam_gap": c.stepper.as_ref().map(|s| round(f64::from(s.max_link_gap()), 2)),
        "closing": !c.closing_over && c.stepper.is_some(),
        "closing_frames_done": c.closing_done,
        "last": {
            "ms": round(f64::from(c.stats.ms), 3),
            "iterations": c.stats.iterations,
            "max_velocity": round(f64::from(c.stats.max_velocity), 3),
            "contacts": c.stats.contacts
        }
    })
}

fn start(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    if s.sim.running.load(Ordering::Acquire) {
        return Ok(state_json(s));
    }
    ensure(s)?;
    let mine = s.sim.generation.fetch_add(1, Ordering::AcqRel) + 1;
    s.sim.running.store(true, Ordering::Release);
    if !spawn(s.sim.core.clone(), s.sim.running.clone(), s.sim.generation.clone(), mine) {
        s.sim.running.store(false, Ordering::Release);
        return Err(CommandError::new("unsupported", "this build cannot run a background simulation; use sim.step"));
    }
    Ok(state_json(s))
}

fn stop(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    s.sim.running.store(false, Ordering::Release);
    s.sim.generation.fetch_add(1, Ordering::AcqRel);
    Ok(state_json(s))
}

fn toggle(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    if s.sim.running.load(Ordering::Acquire) {
        stop(s, a)
    } else {
        start(s, a)
    }
}

fn step_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"frames": {"type": "integer", "minimum": 1, "maximum": MAX_STEP_FRAMES, "default": 1, "description": "How many frames to step. Each frame runs the configured substeps."}},
        "required": [],
        "additionalProperties": false
    })
}

fn step(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let frames = a.get("frames").and_then(Value::as_u64).unwrap_or(1);
    if !(1..=MAX_STEP_FRAMES).contains(&frames) {
        return Err(bad(format!("frames must be from 1 to {MAX_STEP_FRAMES}")));
    }
    ensure(s)?;
    let clock = Clock::start();
    let mut done = 0u64;
    {
        let mut c = lock(&s.sim.core);
        c.sink = s.sink.clone();
        if c.solid > MAX_SYNC_VERTICES {
            return Err(bad(format!("sim.step refuses {} vertices (limit {MAX_SYNC_VERTICES}); use sim.start for large garments", c.solid)));
        }
    }
    while done < frames && !clock.over(STEP_BUDGET_SECONDS) {
        lock(&s.sim.core).step_frames(1);
        done += 1;
    }
    let (snapshot, sink) = {
        let c = lock(&s.sim.core);
        (c.snapshot(), c.sink.clone())
    };
    publish(&sink, &snapshot);
    let mut out = state_json(s);
    if let Some(map) = out.as_object_mut() {
        map.insert("frames_done".into(), json!(done));
        map.insert("budget_reached".into(), json!(done < frames));
    }
    Ok(out)
}

pub(super) struct Clock(#[cfg(not(target_arch = "wasm32"))] std::time::Instant);

impl Clock {
    pub(super) fn start() -> Clock {
        #[cfg(not(target_arch = "wasm32"))]
        return Clock(std::time::Instant::now());
        #[cfg(target_arch = "wasm32")]
        return Clock();
    }

    pub(super) fn over(&self, seconds: f32) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return self.0.elapsed().as_secs_f32() > seconds;
        #[cfg(target_arch = "wasm32")]
        return {
            let _ = seconds;
            false
        };
    }
}

fn reset(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    rebuild(s)?;
    let (snapshot, sink) = {
        let mut c = lock(&s.sim.core);
        c.reset();
        (c.placed_snapshot(), c.sink.clone())
    };
    publish(&sink, &snapshot);
    Ok(state_json(s))
}

fn settings_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "preset": {"type": "string", "enum": ["fast", "normal", "quality", "final"], "description": "Implicit method: fast 2 substeps, normal 3, quality 4 with self contact, final 6 with 3 Newton iterations and self contact. Block descent: fast 3x6, normal 5x10, quality 8x15 and final 12x25 (substeps x iterations), the last two with self contact. Leave out every field to read the settings."},
            "time_step": {"type": "number", "minimum": 0.004, "maximum": 0.0667, "unit": "s", "description": "Seconds per frame, from 1/240 to 1/15. The run thread steps at 1 / time_step frames per second."},
            "substeps": {"type": "integer", "minimum": 1, "maximum": 40},
            "iterations": {"type": "integer", "minimum": 1, "maximum": 200, "description": "Solver iterations per substep."},
            "gravity": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm/s2"},
            "air_damping": {"type": "number", "minimum": 0, "maximum": 1},
            "skin_offset": {"type": "number", "minimum": 0, "maximum": 100, "unit": "mm"},
            "collision_thickness": {"type": "number", "minimum": 0, "maximum": 100, "unit": "mm"},
            "friction": {"type": "number", "minimum": 0, "maximum": 5},
            "self_collision": {"type": "boolean"},
            "method": {"type": "string", "enum": ["vbd", "implicit"], "description": "Solver method: vbd is vertex block descent, implicit is implicit Euler with conjugate gradient."},
            "newton": {"type": "integer", "minimum": 1, "maximum": 10, "description": "Newton iterations per substep (implicit method)."},
            "cg_iterations": {"type": "integer", "minimum": 1, "maximum": 2000, "description": "Conjugate gradient iterations per Newton iteration (implicit method)."},
            "cg_tolerance": {"type": "number", "minimum": 0.000001, "maximum": 0.1, "description": "Conjugate gradient relative tolerance (implicit method)."},
            "close_frames": {"type": "integer", "minimum": 0, "maximum": 600, "description": "Seam closing phase at the start of a drape: up to this many frames with stiffened seams, reduced gravity, strong damping and collisions on, then the normal simulation. 0 turns it off. Only applies when the garment has seams or links."},
            "close_gap": {"type": "number", "minimum": 0.1, "maximum": 50, "unit": "mm", "description": "The closing phase may end early once the largest seam gap is under this."},
            "device": {"type": "string", "enum": ["cpu", "gpu"], "description": "Where the solver runs. gpu needs an engine built with the gpu feature and an adapter; otherwise it is reported unavailable and the simulation stays on the cpu. The gpu path supports the vbd method only."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn number(a: &Value, key: &str, min: f64, max: f64) -> Result<Option<f32>, CommandError> {
    match a.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_f64().filter(|x| x.is_finite() && *x >= min && *x <= max).map(|x| Some(x as f32)).ok_or_else(|| bad(format!("{key} must be a number from {min} to {max}"))),
    }
}

fn params_json(p: &Params) -> Value {
    json!({
        "time_step": p.time_step, "substeps": p.substeps, "iterations": p.iterations, "gravity": p.gravity,
        "air_damping": p.air_damping, "skin_offset": p.skin_offset, "collision_thickness": p.collision_thickness,
        "friction": p.friction, "self_collision": p.self_collision, "method": adapter::method_name(p),
        "newton": p.newton, "cg_iterations": p.cg_iterations, "cg_tolerance": p.cg_tolerance,
        "close_frames": p.close_frames, "close_gap": p.close_gap
    })
}

fn settings(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mut p = *s.sim.params.lock().unwrap_or_else(|e| e.into_inner());
    let mut preset = s.sim.preset.clone();
    let mut touched = false;
    if let Some(name) = a.get("preset").and_then(Value::as_str) {
        p = preset_params(name, p).ok_or_else(|| bad("preset must be fast, normal, quality or final"))?;
        preset = name.into();
        touched = true;
    }
    let mut manual = false;
    if let Some(x) = number(a, "time_step", f64::from(MIN_TIME_STEP) * 0.96, f64::from(MAX_TIME_STEP) * 1.001)? {
        p.time_step = x.clamp(MIN_TIME_STEP, MAX_TIME_STEP);
        manual = true;
    }
    for (key, max) in [("substeps", 40.0), ("iterations", 200.0)] {
        if let Some(x) = number(a, key, 1.0, max)? {
            if key == "substeps" {
                p.substeps = x as u32;
            } else {
                p.iterations = x as u32;
            }
            manual = true;
        }
    }
    if let Some(list) = a.get("gravity").filter(|v| !v.is_null()) {
        let list = list.as_array().filter(|l| l.len() == 3).ok_or_else(|| bad("gravity must hold 3 numbers"))?;
        for (slot, v) in p.gravity.iter_mut().zip(list) {
            *slot = v.as_f64().filter(|x| x.is_finite() && x.abs() <= 1e6).ok_or_else(|| bad("gravity must hold 3 finite numbers"))? as f32;
        }
        manual = true;
    }
    for (key, max) in [("air_damping", 1.0), ("skin_offset", 100.0), ("collision_thickness", 100.0), ("friction", 5.0)] {
        if let Some(x) = number(a, key, 0.0, max)? {
            match key {
                "air_damping" => p.air_damping = x,
                "skin_offset" => p.skin_offset = x,
                "collision_thickness" => p.collision_thickness = x,
                _ => p.friction = x,
            }
            manual = true;
        }
    }
    if let Some(on) = a.get("self_collision").and_then(Value::as_bool) {
        p.self_collision = on;
        manual = true;
    }
    if let Some(name) = a.get("method").and_then(Value::as_str) {
        if !adapter::set_method(&mut p, name) {
            return Err(bad("method must be vbd or implicit"));
        }
        if !manual && preset != "custom" {
            if let Some(q) = preset_params(&preset, p) {
                p = q;
            }
        }
        touched = true;
    }
    if let Some(x) = number(a, "newton", 1.0, 10.0)? {
        p.newton = x as u32;
        touched = true;
    }
    if let Some(x) = number(a, "cg_iterations", 1.0, 2000.0)? {
        p.cg_iterations = x as u32;
        touched = true;
    }
    if let Some(x) = number(a, "cg_tolerance", 1.0e-6, 0.1)? {
        p.cg_tolerance = x;
        touched = true;
    }
    if let Some(x) = number(a, "close_frames", 0.0, 600.0)? {
        p.close_frames = x as u32;
        touched = true;
    }
    if let Some(x) = number(a, "close_gap", 0.1, 50.0)? {
        p.close_gap = x;
        touched = true;
    }
    let mut requested = s.sim.device.clone();
    if let Some(name) = a.get("device").and_then(Value::as_str) {
        if !["cpu", "gpu"].contains(&name) {
            return Err(bad("device must be cpu or gpu"));
        }
        requested = name.into();
    }
    if manual {
        preset = "custom".into();
        touched = true;
    }
    if touched {
        *s.sim.params.lock().unwrap_or_else(|e| e.into_inner()) = p;
        s.sim.preset = preset.clone();
    }
    let gpu = adapter::gpu_available();
    let switched = s.sim.device != requested;
    s.sim.device = requested.clone();
    let want = requested == "gpu";
    let (active, note) = {
        let mut c = super::runtime::lock(&s.sim.core);
        c.want_gpu = want;
        if switched && c.stepper.is_some() && c.on_gpu != (want && gpu) {
            c.reset();
        }
        (if c.on_gpu { "gpu" } else { "cpu" }, c.device_note.clone())
    };
    let message = if want && !gpu {
        "gpu solver is not available (no adapter or not built in); running on the cpu".to_string()
    } else {
        note
    };
    Ok(json!({
        "preset": s.sim.preset,
        "params": params_json(&p),
        "changed": touched,
        "presets": ["fast", "normal", "quality", "final"],
        "device": {"requested": requested, "active": active, "gpu_available": gpu, "message": message}
    }))
}

fn state(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    if let Ok(mut c) = s.sim.core.try_lock() {
        c.sink = s.sink.clone();
    }
    Ok(state_json(s))
}

fn compare_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"frames": {"type": "integer", "minimum": 1, "maximum": MAX_COMPARE_FRAMES, "default": 30, "description": "Frames to run with the block descent solver and with the reference solver from the same start."}},
        "required": [],
        "additionalProperties": false
    })
}

fn compare(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let frames = a.get("frames").and_then(Value::as_u64).unwrap_or(30);
    if !(1..=MAX_COMPARE_FRAMES).contains(&frames) {
        return Err(bad(format!("frames must be from 1 to {MAX_COMPARE_FRAMES}")));
    }
    ensure(s)?;
    let (base, colliders, params) = {
        let c = lock(&s.sim.core);
        (c.base.clone(), c.colliders.clone(), c.params)
    };
    if base.positions.len() > MAX_SYNC_VERTICES {
        return Err(bad(format!("sim.compare refuses {} vertices (limit {MAX_SYNC_VERTICES})", base.positions.len())));
    }
    let clock = Clock::start();
    let r = super::adapter::compare(&base, &colliders, params, frames as u32, &|| clock.over(COMPARE_BUDGET_SECONDS));
    Ok(json!({
        "frames": r.frames,
        "requested_frames": frames,
        "vertices": base.positions.len(),
        "mean_distance": round(f64::from(r.mean), 4),
        "max_distance": round(f64::from(r.max), 4),
        "unit": "mm",
        "solver_ms": round(f64::from(r.main_ms), 2),
        "reference_ms": round(f64::from(r.reference_ms), 2)
    }))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command { id: "sim.start", mcp: "sim_start", group: Group::Sim, label: "Start simulation", description: "Start the background simulation at the frame rate. Positions stream to the viewer. Native builds only; use sim.step elsewhere.", keys: &[], hints: SET, schema: no_args, run: start },
        Command { id: "sim.stop", mcp: "sim_stop", group: Group::Sim, label: "Stop simulation", description: "Stop the background simulation. The draped state stays where it is.", keys: &[], hints: SET, schema: no_args, run: stop },
        Command { id: "sim.toggle", mcp: "sim_toggle", group: Group::Sim, label: "Simulate", description: "Start the simulation if it is stopped, stop it if it runs.", keys: &["Space"], hints: WRITE, schema: no_args, run: toggle },
        Command { id: "sim.step", mcp: "sim_step", group: Group::Sim, label: "Step simulation", description: "Advance the simulation by a number of frames on the calling thread and publish the result. Works without a background thread.", keys: &[], hints: WRITE, schema: step_schema, run: step },
        Command { id: "sim.reset", mcp: "sim_reset", group: Group::Sim, label: "Reset simulation", description: "Rebuild the cloth from the garment and return every panel to its placed state, frame zero.", keys: &[], hints: WRITE, schema: no_args, run: reset },
        Command { id: "sim.settings", mcp: "sim_settings", group: Group::Sim, label: "Simulation settings", description: "Get or set the solver parameters and the quality preset (fast, normal, quality, final). Settings are not saved in the document.", keys: &[], hints: SET, schema: settings_schema, run: settings },
        Command { id: "sim.state", mcp: "sim_state", group: Group::Sim, label: "Simulation state", description: "Read whether the simulation runs, the frame, the vertex count, the last step statistics and the preset.", keys: &[], hints: READ, schema: no_args, run: state },
        Command { id: "sim.positions", mcp: "sim_positions", group: Group::Sim, label: "Cloth positions", description: "Read the current simulated 3D positions of one panel's vertices. decimate keeps every n-th vertex; the answer never holds more than 5000 vertices, so large panels are decimated further. Read only.", keys: &[], hints: READ, schema: positions_schema, run: positions },
        Command { id: "sim.compare", mcp: "sim_compare", group: Group::Sim, label: "Compare solvers", description: "Run the same start for n frames with the interactive solver and with the reference solver and report the mean and maximum vertex distance in mm. Does not touch the running simulation.", keys: &[], hints: READ, schema: compare_schema, run: compare },
    ]
}
