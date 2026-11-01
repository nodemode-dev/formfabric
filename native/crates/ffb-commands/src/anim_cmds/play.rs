use std::sync::atomic::Ordering;

use ffb_anim::cachefmt::{BodyFrame, BodySpec, Cache};
use ffb_body::math::IDENTITY;
use ffb_body::{skin, Body};
use ffb_pattern::anim::CacheLayer;
use serde_json::{json, Value};

use super::ops::cache_index;
use super::record::{flat, plan_for, spawn, summary, MAX_RECORD_FRAMES};
use super::rig::cache_entry;
use super::{bad, edit_garment, entry, lock_job, put_entry, OBJECT_BASE};
use crate::{no_args, Command, CommandError, Group, Session, READ, SET, WRITE};

pub(crate) fn job_json(s: &Session, with_stats: bool) -> Value {
    let j = lock_job(&s.anim.job);
    let mut out = json!({
        "phase": j.phase,
        "frame": j.frame,
        "total": j.total,
        "start": j.start,
        "end": j.end,
        "name": j.name,
        "message": j.message,
        "elapsed_ms": j.ms,
        "ready": j.finished.is_some(),
        "warnings": j.warnings,
        "summary": summary(&j),
        "play_frame": s.anim.play_frame.load(Ordering::Acquire),
        "playing": s.anim.playing.load(Ordering::Acquire),
    });
    if with_stats {
        if let Some(map) = out.as_object_mut() {
            map.insert("stats".into(), json!(j.stats));
        }
    }
    out
}

fn harvest(s: &mut Session) -> Result<Option<u32>, CommandError> {
    let finished = lock_job(&s.anim.job).finished.take();
    let Some(done) = finished else { return Ok(None) };
    let id = s.garment.anim.caches.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    let name = cache_entry(id);
    put_entry(s, name.clone(), done.bytes);
    edit_garment(s, "Record animation", |g| {
        g.anim.caches.push(CacheLayer { id, name: done.name.clone(), start: done.start, frames: done.frames, visible: true, entry: name.clone() });
        g.anim.active_cache = Some(id);
        Ok(())
    })?;
    Ok(Some(id))
}

fn job_state(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    let cache = harvest(s)?;
    let mut out = job_json(s, true);
    if let Some(map) = out.as_object_mut() {
        map.insert("cache".into(), json!(cache));
    }
    Ok(out)
}

fn record_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "start": {"type": "integer", "minimum": 0, "description": "First frame to record. Default the play range start."},
            "end": {"type": "integer", "minimum": 0, "description": "Last frame to record. Default the play range end."},
            "name": {"type": "string", "description": "Cache layer name."},
            "steps": {"type": "integer", "minimum": 1, "maximum": 8, "description": "Solver frames per timeline frame; the body moves between them. Default 2."},
            "storage": {"type": "string", "enum": ["f32", "quantized"], "description": "f32 keeps exact positions; quantized stores 16 bit positions per panel and frame (half the size, error under extent / 131070)."},
            "validate": {"type": "boolean", "description": "Measure body penetration, self intersections and vertex displacement every frame. Default true."},
            "reset": {"type": "boolean", "description": "Rebuild the cloth from its placed state first. Default false: recording continues from the current draped state."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn record(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    if lock_job(&s.anim.job).phase == "running" {
        return Err(CommandError::new("busy", "a recording is already running; anim.stop ends it"));
    }
    let anim = &s.garment.anim;
    let get = |k: &str, d: u32| -> Result<u32, CommandError> {
        match a.get(k).filter(|v| !v.is_null()) {
            None => Ok(d),
            Some(v) => v.as_u64().filter(|x| *x <= 100_000).map(|x| x as u32).ok_or_else(|| bad(format!("{k} must be a frame number"))),
        }
    };
    let (start, end) = (get("start", anim.start)?, get("end", anim.end)?);
    if start > end {
        return Err(bad("start must not be after end"));
    }
    if end - start + 1 > MAX_RECORD_FRAMES {
        return Err(bad(format!("a recording is limited to {MAX_RECORD_FRAMES} frames")));
    }
    let steps = a.get("steps").and_then(Value::as_u64).map_or(2, |x| x.clamp(1, 8) as u32);
    let validate = a.get("validate").and_then(Value::as_bool).unwrap_or(true);
    let quantized = a.get("storage").and_then(Value::as_str) == Some("quantized");
    let id_next = s.garment.anim.caches.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    let name = match a.get("name").and_then(Value::as_str) {
        Some(n) => {
            crate::args::clean_name("name", n)?;
            n.to_string()
        }
        None => format!("Cache {id_next}"),
    };
    s.sim.running.store(false, Ordering::Release);
    s.sim.generation.fetch_add(1, Ordering::AcqRel);
    let state = crate::dispatch(s, "sim.state", &json!({}))?;
    let built = state.get("built").and_then(Value::as_bool).unwrap_or(false);
    let stale = state.get("stale").and_then(Value::as_bool).unwrap_or(false);
    if a.get("reset").and_then(Value::as_bool).unwrap_or(false) || !built {
        crate::dispatch(s, "sim.reset", &json!({}))?;
    } else if stale {
        crate::dispatch(s, "sim.step", &json!({"frames": 1}))?;
    }
    s.anim.cancel.store(false, Ordering::Release);
    let plan = plan_for(s, start, end, name.clone(), steps, validate, quantized)?;
    {
        let mut j = lock_job(&s.anim.job);
        let warnings = std::mem::take(&mut j.warnings);
        *j = super::Job { phase: "running", total: end - start + 1, start, end, name, warnings, ..super::Job::default() };
    }
    if !spawn(plan) {
        lock_job(&s.anim.job).phase = "failed";
        return Err(CommandError::new("unsupported", "this build cannot record in the background"));
    }
    Ok(job_json(s, false))
}

fn stop(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    s.anim.cancel.store(true, Ordering::Release);
    Ok(job_json(s, false))
}

pub(crate) fn pose_body(bodies: &[Body], spec: &BodySpec, bf: &BodyFrame, scratch: &mut Body) -> Vec<[f32; 3]> {
    scratch.pose.clear();
    for (name, q) in spec.joints.iter().zip(&bf.quats) {
        if *q != IDENTITY {
            scratch.pose.insert(name.clone(), *q);
        }
    }
    let mut pos = skin::posed_positions(scratch);
    if let Some(to) = spec.morph_to.and_then(|id| bodies.iter().find(|b| b.id == id)) {
        let target = skin::posed_positions(to);
        if target.len() == pos.len() && bf.morph > 0.0 {
            pos = super::rig::blend_positions(&pos, &target, bf.morph);
        }
    }
    if bf.offset != [0.0; 3] {
        for v in pos.iter_mut() {
            *v = [v[0] + bf.offset[0], v[1] + bf.offset[1], v[2] + bf.offset[2]];
        }
    }
    pos
}

fn frame_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "description": "Cache layer id. Default the active cache."},
            "frame": {"type": "integer", "minimum": 0, "description": "Timeline frame, inside the cache range."}
        },
        "required": ["frame"],
        "additionalProperties": false
    })
}

fn cache_pick(s: &Session, a: &Value) -> Result<usize, CommandError> {
    if a.get("id").is_some_and(|v| !v.is_null()) {
        return cache_index(s, a);
    }
    let active = s.garment.anim.active_cache.and_then(|id| s.garment.anim.caches.iter().position(|c| c.id == id));
    active.or_else(|| s.garment.anim.caches.len().checked_sub(1)).ok_or_else(|| CommandError::new("not_found", "there is no cache; record one with anim.record"))
}

fn cache_frame(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = cache_pick(s, a)?;
    let layer = s.garment.anim.caches[k].clone();
    let frame = a.get("frame").and_then(Value::as_u64).filter(|f| *f <= 100_000).ok_or_else(|| bad("frame must be a frame number"))? as u32;
    let data = entry(s, &layer.entry).ok_or_else(|| CommandError::new("not_found", "the cache data is missing"))?;
    let cache = Cache::parse(data).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let rel = frame.checked_sub(cache.header.start).filter(|r| *r < cache.header.frames).ok_or_else(|| bad(format!("frame {frame} is outside the cache range {} to {}", cache.header.start, cache.header.start + cache.header.frames.saturating_sub(1))))?;
    let mut count = 0usize;
    let mut sent = Vec::new();
    for (i, p) in cache.header.panels.iter().enumerate() {
        if let Some(v) = cache.panel_frame(rel, i) {
            count += v.len() / 3;
            sent.push((p.id, v));
        }
    }
    let mut bodies_out = Vec::new();
    for (i, spec) in cache.header.bodies.iter().enumerate() {
        let (Some(bf), Some(body)) = (cache.body_frame(rel, i), s.bodies.iter().find(|b| b.id == spec.id)) else { continue };
        let mut scratch = body.clone();
        let pos = pose_body(&s.bodies, spec, &bf, &mut scratch);
        count += pos.len();
        bodies_out.push((OBJECT_BASE + spec.id, flat(&pos)));
    }
    let streamed = match &s.sink {
        Some(sink) => {
            for (id, v) in &sent {
                sink.positions(*id, v);
            }
            for (id, v) in &bodies_out {
                sink.positions(*id, v);
            }
            true
        }
        None => false,
    };
    let stat = cache.header.stats.get(rel as usize).cloned();
    s.garment.anim.current = frame;
    Ok(json!({"id": layer.id, "frame": frame, "count": count, "panels": sent.len(), "bodies": bodies_out.len(), "streamed": streamed, "stats": stat}))
}

fn play_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "description": "Cache layer id. Default the active cache."},
            "from": {"type": "integer", "minimum": 0, "description": "Frame to start from. Default the current frame, or the cache start when it is outside the cache."},
            "loop": {"type": "boolean", "description": "Start over at the end. Default false."},
            "speed": {"type": "number", "minimum": 0.1, "maximum": 8, "description": "Playback speed, 1 is the cache frame rate."}
        },
        "required": [],
        "additionalProperties": false
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn play(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    use std::time::{Duration, Instant};
    let k = cache_pick(s, a)?;
    let layer = s.garment.anim.caches[k].clone();
    let data = entry(s, &layer.entry).ok_or_else(|| CommandError::new("not_found", "the cache data is missing"))?.to_vec();
    let header = Cache::parse(&data).map_err(|e| CommandError::new("invalid_geometry", e))?.header;
    let sink = s.sink.clone().ok_or_else(|| CommandError::new("unsupported", "playback needs a connected viewer"))?;
    let last = header.start + header.frames.saturating_sub(1);
    let from = a.get("from").and_then(Value::as_u64).map_or(s.garment.anim.current, |x| x.min(100_000) as u32);
    let from = if from < header.start || from > last { header.start } else { from };
    let looped = a.get("loop").and_then(Value::as_bool).unwrap_or(false);
    let speed = a.get("speed").and_then(Value::as_f64).map_or(1.0, |x| x.clamp(0.1, 8.0));
    let bodies: Vec<Body> = s.bodies.clone();
    s.anim.playing.store(false, Ordering::Release);
    let mine = s.anim.play_generation.fetch_add(1, Ordering::AcqRel) + 1;
    s.anim.playing.store(true, Ordering::Release);
    s.anim.play_frame.store(from, Ordering::Release);
    let (playing, generation, position) = (s.anim.playing.clone(), s.anim.play_generation.clone(), s.anim.play_frame.clone());
    let work = move || {
        let Ok(cache) = Cache::parse(&data) else { return };
        let mut scratch: Vec<Option<Body>> = cache.header.bodies.iter().map(|spec| bodies.iter().find(|b| b.id == spec.id).cloned()).collect();
        let began = Instant::now();
        let mut frame = from;
        let mut shown = 0u64;
        let period = 1.0 / (f64::from(cache.header.fps.max(1.0)) * speed);
        loop {
            if !playing.load(Ordering::Acquire) || generation.load(Ordering::Acquire) != mine {
                break;
            }
            let rel = frame - cache.header.start;
            for (i, p) in cache.header.panels.iter().enumerate() {
                if let Some(v) = cache.panel_frame(rel, i) {
                    sink.positions(p.id, &v);
                }
            }
            for (i, spec) in cache.header.bodies.iter().enumerate() {
                if let (Some(bf), Some(Some(body))) = (cache.body_frame(rel, i), scratch.get_mut(i)) {
                    let pos = pose_body(&bodies, spec, &bf, body);
                    sink.positions(OBJECT_BASE + spec.id, &flat(&pos));
                }
            }
            position.store(frame, Ordering::Release);
            shown += 1;
            if frame >= last {
                if looped {
                    frame = cache.header.start;
                } else {
                    break;
                }
            } else {
                frame += 1;
            }
            let due = Duration::from_secs_f64(shown as f64 * period);
            let spent = began.elapsed();
            if due > spent {
                std::thread::sleep(due - spent);
            }
        }
        if generation.load(Ordering::Acquire) == mine {
            playing.store(false, Ordering::Release);
        }
    };
    if std::thread::Builder::new().name("ffb-anim-play".into()).spawn(work).is_err() {
        s.anim.playing.store(false, Ordering::Release);
        return Err(CommandError::new("unsupported", "could not start the playback thread"));
    }
    Ok(json!({"playing": true, "id": layer.id, "from": from, "to": last, "fps": header.fps}))
}

#[cfg(target_arch = "wasm32")]
fn play(_s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    Err(CommandError::new("unsupported", "this build cannot play in the background; use cache.frame"))
}

fn pause(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    s.anim.playing.store(false, Ordering::Release);
    s.anim.play_generation.fetch_add(1, Ordering::AcqRel);
    Ok(json!({"playing": false, "frame": s.anim.play_frame.load(Ordering::Acquire)}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command { id: "anim.record", mcp: "anim_record", group: Group::Anim, label: "Record animation", description: "Record a cache layer on a background thread: for each frame pose the bodies (motions, pose, measurement and morph tracks, skinned through ffb-body), move the colliders, apply wind tracks, step the cloth with solver substeps, and store every panel's positions plus the body pose. Per frame it measures body penetration, self intersections and vertex displacement. It continues from the current draped state, so drape first. Poll job.state.", keys: &[], hints: WRITE, schema: record_schema, run: record },
        Command { id: "anim.stop", mcp: "anim_stop", group: Group::Anim, label: "Stop recording", description: "Stop a running recording; the frames recorded so far are kept.", keys: &[], hints: SET, schema: no_args, run: stop },
        Command { id: "job.state", mcp: "job_state", group: Group::Anim, label: "Recording state", description: "Read the recording job: phase, frame, total, per-frame validation statistics and warnings. When a recording has finished, the call stores it as a cache layer (one undo step) and returns its id.", keys: &[], hints: READ, schema: no_args, run: job_state },
        Command { id: "cache.frame", mcp: "cache_frame", group: Group::Anim, label: "Show cache frame", description: "Stream one cached frame's positions for every panel and the posed bodies to the viewer and return the vertex count. Frames are timeline frames inside the cache range.", keys: &[], hints: SET, schema: frame_schema, run: cache_frame },
        Command { id: "anim.play", mcp: "anim_play", group: Group::Anim, label: "Play cache", description: "Play a cache layer to the viewer from a frame at the cache frame rate on a background thread.", keys: &[], hints: SET, schema: play_schema, run: play },
        Command { id: "anim.pause", mcp: "anim_pause", group: Group::Anim, label: "Pause playback", description: "Pause cache playback and return the frame it stopped on.", keys: &[], hints: SET, schema: no_args, run: pause },
    ]
}
