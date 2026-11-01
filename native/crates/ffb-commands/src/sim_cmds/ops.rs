use std::sync::atomic::Ordering;

use ffb_sim::Params;
use serde_json::{json, Value};

use super::build::{ensure, rebuild};
use super::runtime::{lock, publish, spawn};
use crate::{no_args, Command, CommandError, Group, Session, READ, SET, WRITE};

const MAX_STEP_FRAMES: u64 = 2000;
const MAX_COMPARE_FRAMES: u64 = 300;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn round(x: f64, places: i32) -> f64 {
    let k = 10f64.powi(places);
    (x * k).round() / k
}

pub(super) fn preset_params(name: &str, base: Params) -> Option<Params> {
    let (substeps, iterations, self_collision, collision_thickness) = match name {
        "normal" => (5, 10, false, 1.5),
        "fast" => (3, 6, false, 2.0),
        "fine" => (10, 20, true, 1.0),
        _ => return None,
    };
    Some(Params { substeps, iterations, self_collision, collision_thickness, ..base })
}

pub(super) fn state_json(session: &Session) -> Value {
    let c = lock(&session.sim.core);
    let p = c.params;
    json!({
        "running": session.sim.running.load(Ordering::Acquire),
        "built": c.stepper.is_some(),
        "stale": c.stale,
        "frame": c.frame,
        "panels": c.slices.len(),
        "vertices": c.base.positions.len(),
        "triangles": c.base.triangles.len(),
        "seams": c.base.seams.len(),
        "pins": session.garment.pins.len(),
        "pins_built": c.base.pins.len(),
        "folds": c.base.folds.len(),
        "grabbed": c.grabbed,
        "preset": session.sim.preset,
        "fps": round(1.0 / f64::from(p.time_step), 2),
        "kinetic_energy": c.stepper.as_ref().map(|s| round(f64::from(s.kinetic_energy()), 3)),
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
    let (snapshot, sink) = {
        let mut c = lock(&s.sim.core);
        c.sink = s.sink.clone();
        c.step_frames(frames as u32);
        (c.snapshot(), c.sink.clone())
    };
    publish(&sink, &snapshot);
    Ok(state_json(s))
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
            "preset": {"type": "string", "enum": ["normal", "fast", "fine"], "description": "normal: 5 substeps, 10 iterations. fast: 3 substeps, 6 iterations, thicker contact. fine: 10 substeps, 20 iterations, self collision, thin contact. Leave out every field to read the settings."},
            "time_step": {"type": "number", "minimum": 0.002, "maximum": 0.1, "unit": "s", "description": "Seconds per frame. The run thread steps at 1 / time_step frames per second."},
            "substeps": {"type": "integer", "minimum": 1, "maximum": 40},
            "iterations": {"type": "integer", "minimum": 1, "maximum": 200, "description": "Solver iterations per substep."},
            "gravity": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm/s2"},
            "air_damping": {"type": "number", "minimum": 0, "maximum": 1},
            "skin_offset": {"type": "number", "minimum": 0, "maximum": 100, "unit": "mm"},
            "collision_thickness": {"type": "number", "minimum": 0, "maximum": 100, "unit": "mm"},
            "friction": {"type": "number", "minimum": 0, "maximum": 5},
            "self_collision": {"type": "boolean"}
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
        "friction": p.friction, "self_collision": p.self_collision
    })
}

fn settings(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mut p = lock(&s.sim.core).params;
    let mut preset = s.sim.preset.clone();
    let mut touched = false;
    if let Some(name) = a.get("preset").and_then(Value::as_str) {
        p = preset_params(name, p).ok_or_else(|| bad("preset must be normal, fast or fine"))?;
        preset = name.into();
        touched = true;
    }
    let mut manual = false;
    if let Some(x) = number(a, "time_step", 0.002, 0.1)? {
        p.time_step = x;
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
    if manual {
        preset = "custom".into();
        touched = true;
    }
    if touched {
        let mut c = lock(&s.sim.core);
        c.params = p;
        if let Some(stepper) = c.stepper.as_mut() {
            stepper.set_params(p);
        }
        drop(c);
        s.sim.preset = preset.clone();
    }
    Ok(json!({"preset": s.sim.preset, "params": params_json(&p), "changed": touched, "presets": ["normal", "fast", "fine"]}))
}

fn state(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    lock(&s.sim.core).sink = s.sink.clone();
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
    let r = super::adapter::compare(&base, &colliders, params, frames as u32);
    Ok(json!({
        "frames": frames,
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
        Command { id: "sim.settings", mcp: "sim_settings", group: Group::Sim, label: "Simulation settings", description: "Get or set the solver parameters and the quality preset (normal, fast, fine). Settings are not saved in the document.", keys: &[], hints: SET, schema: settings_schema, run: settings },
        Command { id: "sim.state", mcp: "sim_state", group: Group::Sim, label: "Simulation state", description: "Read whether the simulation runs, the frame, the vertex count, the last step statistics and the preset.", keys: &[], hints: READ, schema: no_args, run: state },
        Command { id: "sim.compare", mcp: "sim_compare", group: Group::Sim, label: "Compare solvers", description: "Run the same start for n frames with the interactive solver and with the reference solver and report the mean and maximum vertex distance in mm. Does not touch the running simulation.", keys: &[], hints: READ, schema: compare_schema, run: compare },
    ]
}
