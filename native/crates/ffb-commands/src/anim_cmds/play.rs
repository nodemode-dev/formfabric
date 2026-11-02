use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ffb_anim::cachefmt::{BodyFrame, BodySpec, Cache};
use ffb_body::math::IDENTITY;
use ffb_body::{skin, Body};
use ffb_pattern::anim::{CacheKind, CacheLayer};
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
    let size = match lock_job(&s.anim.job).finished.as_ref() {
        Some(f) => f.bytes.len(),
        None => return Ok(None),
    };
    let stored: usize = s.unknown.iter().map(|(_, d)| d.len()).sum();
    if stored + size > PROJECT_BUDGET {
        lock_job(&s.anim.job).finished = None;
        return Err(CommandError::new("too_large", format!("the recording ({} MiB) does not fit the project size limit of 1 GiB next to the {} MiB already stored; it was discarded. Remove a cache layer or record with storage quantized", size >> 20, stored >> 20)));
    }
    let replace = lock_job(&s.anim.job).replace;
    if let Some(target) = replace {
        if s.garment.anim.caches.iter().any(|c| c.id == target && c.locked) {
            return Err(CommandError::new("locked", format!("cache layer {target} is locked; the recording is kept until it is unlocked or stored as a new layer")));
        }
    }
    let finished = lock_job(&s.anim.job).finished.take();
    lock_job(&s.anim.job).replace = None;
    let Some(done) = finished else { return Ok(None) };
    let id = super::alloc_cache_id(s);
    let name = cache_entry(id);
    let trims = super::layers::trim_cache(s, &done.bytes, &done.name);
    put_entry(s, name.clone(), done.bytes);
    let trim_id = trims.map(|bytes| {
        let tid = super::alloc_cache_id(s);
        put_entry(s, cache_entry(tid), bytes);
        tid
    });
    let target = replace.filter(|t| s.garment.anim.caches.iter().any(|c| c.id == *t && c.kind == CacheKind::Pattern));
    edit_garment(s, "Record animation", |g| {
        let mut stored = id;
        match target.and_then(|t| g.anim.caches.iter().position(|c| c.id == t)) {
            Some(at) => {
                let c = &mut g.anim.caches[at];
                c.start = done.start;
                c.frames = done.frames;
                c.entry = name.clone();
                c.offset = 0;
                stored = c.id;
            }
            None => g.anim.caches.push(CacheLayer { id, name: done.name.clone(), start: done.start, frames: done.frames, visible: true, entry: name.clone(), kind: CacheKind::Pattern, locked: false, offset: 0 }),
        }
        if let Some(tid) = trim_id {
            g.anim.caches.retain(|c| !(c.kind == CacheKind::Trim && c.name == format!("{} trims", done.name)));
            g.anim.caches.push(CacheLayer { id: tid, name: format!("{} trims", done.name), start: done.start, frames: done.frames, visible: true, entry: cache_entry(tid), kind: CacheKind::Trim, locked: false, offset: 0 });
        }
        g.anim.active_cache = Some(stored);
        Ok(())
    })?;
    Ok(Some(target.unwrap_or(id)))
}

const PROJECT_BUDGET: usize = 900 << 20;

fn job_state(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    Ok(job_json(s, true))
}

fn job_harvest(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
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
            "reset": {"type": "boolean", "description": "Rebuild the cloth from its placed state first. Default false: recording continues from the current draped state."},
            "layer": {"type": "integer", "description": "Re-record into this cache layer instead of adding a new one. A locked layer refuses."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn record(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    if lock_job(&s.anim.job).phase == "running" {
        return Err(CommandError::new("busy", "a recording is already running; anim.stop ends it"));
    }
    let replace = match a.get("layer").filter(|v| !v.is_null()) {
        None => None,
        Some(_) => {
            let k = cache_index(s, &json!({"id": a.get("layer")}))?;
            super::layers::guard(s, k)?;
            Some(s.garment.anim.caches[k].id)
        }
    };
    let _ = harvest(s);
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
    if s.anim.alive.load(Ordering::Acquire) {
        return Err(CommandError::new("busy", "the previous recording is still stopping; try again in a moment"));
    }
    let id_next = s.anim.next_cache.max(s.garment.anim.caches.iter().map(|c| c.id).max().unwrap_or(0) + 1);
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
    s.anim.cancel = Arc::new(AtomicBool::new(false));
    let plan = plan_for(s, start, end, name.clone(), steps, validate, quantized)?;
    {
        let mut j = lock_job(&s.anim.job);
        let warnings = std::mem::take(&mut j.warnings);
        *j = super::Job { phase: "running", total: end - start + 1, start, end, name, warnings, replace, ..super::Job::default() };
    }
    s.anim.alive.store(true, Ordering::Release);
    if !spawn(plan) {
        s.anim.alive.store(false, Ordering::Release);
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
    active.or_else(|| s.garment.anim.caches.iter().rposition(|c| c.kind == CacheKind::Pattern)).ok_or_else(|| CommandError::new("not_found", "there is no cache; record one with anim.record"))
}

fn cache_frame(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let frame = a.get("frame").and_then(Value::as_u64).filter(|f| *f <= 100_000).ok_or_else(|| bad("frame must be a frame number"))? as u32;
    let k = match super::layers::resolve(&s.garment.anim, frame, CacheKind::Pattern) {
        Some(top) if !a.get("id").is_some_and(|v| !v.is_null()) => top,
        _ => cache_pick(s, a)?,
    };
    let layer = s.garment.anim.caches[k].clone();
    let data = entry(s, &layer.entry).ok_or_else(|| CommandError::new("not_found", "the cache data is missing"))?;
    let cache = Cache::parse(data).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let rel = frame.checked_sub(layer.start).filter(|r| *r < cache.header.frames).ok_or_else(|| bad(format!("frame {frame} is outside the cache range {} to {}", layer.start, layer.start + cache.header.frames.saturating_sub(1))))?;
    let trims: Vec<Value> = super::layers::resolve(&s.garment.anim, frame, CacheKind::Trim)
        .and_then(|t| {
            let tl = &s.garment.anim.caches[t];
            entry(s, &tl.entry).and_then(|b| super::layers::trim_points(b, frame - tl.start))
        })
        .map(|list| list.into_iter().map(|(id, p)| json!({"id": id, "point": p})).collect())
        .unwrap_or_default();
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
    Ok(json!({"id": layer.id, "frame": frame, "count": count, "panels": sent.len(), "bodies": bodies_out.len(), "streamed": streamed, "stats": stat, "trims": trims}))
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
    let picks: Vec<usize> = if a.get("id").is_some_and(|v| !v.is_null()) {
        vec![cache_pick(s, a)?]
    } else {
        let visible: Vec<usize> = (0..s.garment.anim.caches.len()).filter(|i| s.garment.anim.caches[*i].kind == CacheKind::Pattern && s.garment.anim.caches[*i].visible).collect();
        if visible.is_empty() {
            vec![cache_pick(s, a)?]
        } else {
            visible
        }
    };
    let mut layers: Vec<(u32, Arc<Vec<u8>>)> = Vec::new();
    let mut ids = Vec::new();
    for k in &picks {
        let layer = s.garment.anim.caches[*k].clone();
        let data = match s.anim.play_data.as_ref() {
            Some((name, data)) if *name == layer.entry => data.clone(),
            _ => {
                let copy = Arc::new(entry(s, &layer.entry).ok_or_else(|| CommandError::new("not_found", "the cache data is missing"))?.to_vec());
                s.anim.play_data = Some((layer.entry.clone(), copy.clone()));
                copy
            }
        };
        Cache::parse(&data).map_err(|e| CommandError::new("invalid_geometry", e))?;
        layers.push((layer.start, data));
        ids.push(layer.id);
    }
    let first_frame = picks.iter().map(|k| s.garment.anim.caches[*k].start).min().unwrap_or(0);
    let last = picks.iter().map(|k| s.garment.anim.caches[*k].start + s.garment.anim.caches[*k].frames.saturating_sub(1)).max().unwrap_or(0);
    let fps = Cache::parse(&layers[layers.len() - 1].1).map(|c| c.header.fps).unwrap_or(30.0);
    let sink = s.sink.clone().ok_or_else(|| CommandError::new("unsupported", "playback needs a connected viewer"))?;
    let from = a.get("from").and_then(Value::as_u64).map_or(s.garment.anim.current, |x| x.min(100_000) as u32);
    let from = if from < first_frame || from > last { first_frame } else { from };
    let looped = a.get("loop").and_then(Value::as_bool).unwrap_or(false);
    let speed = a.get("speed").and_then(Value::as_f64).map_or(1.0, |x| x.clamp(0.1, 8.0));
    let bodies: Vec<Body> = s.bodies.clone();
    s.anim.playing.store(false, Ordering::Release);
    let mine = s.anim.play_generation.fetch_add(1, Ordering::AcqRel) + 1;
    s.anim.playing.store(true, Ordering::Release);
    s.anim.play_frame.store(from, Ordering::Release);
    let (playing, generation, position) = (s.anim.playing.clone(), s.anim.play_generation.clone(), s.anim.play_frame.clone());
    let work = move || {
        let parsed: Vec<(u32, Cache)> = layers.iter().filter_map(|(st, d)| Cache::parse(d).ok().map(|c| (*st, c))).collect();
        if parsed.is_empty() {
            return;
        }
        let mut scratch: Vec<Vec<Option<Body>>> = parsed.iter().map(|(_, c)| c.header.bodies.iter().map(|spec| bodies.iter().find(|b| b.id == spec.id).cloned()).collect()).collect();
        let began = Instant::now();
        let mut frame = from;
        let mut shown = 0u64;
        let period = 1.0 / (f64::from(fps.max(1.0)) * speed);
        loop {
            if !playing.load(Ordering::Acquire) || generation.load(Ordering::Acquire) != mine {
                break;
            }
            if let Some(li) = parsed.iter().rposition(|(st, c)| frame >= *st && frame - *st < c.header.frames) {
                let (st, cache) = &parsed[li];
                let rel = frame - *st;
                for (i, p) in cache.header.panels.iter().enumerate() {
                    if let Some(v) = cache.panel_frame(rel, i) {
                        sink.positions(p.id, &v);
                    }
                }
                for (i, spec) in cache.header.bodies.iter().enumerate() {
                    if let (Some(bf), Some(Some(body))) = (cache.body_frame(rel, i), scratch[li].get_mut(i)) {
                        let pos = pose_body(&bodies, spec, &bf, body);
                        sink.positions(OBJECT_BASE + spec.id, &flat(&pos));
                    }
                }
            }
            position.store(frame, Ordering::Release);
            shown += 1;
            if frame >= last {
                if looped {
                    frame = first_frame;
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
    Ok(json!({"playing": true, "id": ids.last(), "layers": ids, "from": from, "to": last, "fps": fps}))
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
        Command { id: "job.state", mcp: "job_state", group: Group::Anim, label: "Recording state", description: "Read the recording job: phase, frame, total, per-frame validation statistics and warnings. It changes nothing; when ready is true, call job.harvest to store the recording as a cache layer.", keys: &[], hints: READ, schema: no_args, run: job_state },
        Command { id: "job.harvest", mcp: "job_harvest", group: Group::Anim, label: "Store recording", description: "Store a finished recording as a cache layer (one undo step) and return its id in cache; null when nothing is waiting. Call it when job.state reports ready.", keys: &[], hints: WRITE, schema: no_args, run: job_harvest },
        Command { id: "cache.frame", mcp: "cache_frame", group: Group::Anim, label: "Show cache frame", description: "Stream one cached frame's positions for every panel and the posed bodies to the viewer and return the vertex count. Frames are timeline frames inside the cache range.", keys: &[], hints: SET, schema: frame_schema, run: cache_frame },
        Command { id: "anim.play", mcp: "anim_play", group: Group::Anim, label: "Play cache", description: "Play a cache layer to the viewer from a frame at the cache frame rate on a background thread.", keys: &[], hints: SET, schema: play_schema, run: play },
        Command { id: "anim.pause", mcp: "anim_pause", group: Group::Anim, label: "Pause playback", description: "Pause cache playback and return the frame it stopped on.", keys: &[], hints: SET, schema: no_args, run: pause },
    ]
}
