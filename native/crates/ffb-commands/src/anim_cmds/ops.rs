use ffb_anim::cachefmt;
use ffb_anim::clip::Clip;
use ffb_anim::retarget::{Options as RetargetOptions, Retargeter};
use ffb_anim::{bvh, gltf};
use ffb_body::import;
use ffb_pattern::anim::{Interp, Key, Motion, Track};
use serde_json::{json, Map, Value};

use super::rig::{motion_entry, MEASUREMENTS};
use super::{bad, edit_garment, entry, put_entry};
use crate::{args, no_args, Command, CommandError, Group, Session, DESTROY, READ, SET, WRITE};

const MAX_FRAME: u64 = 100_000;
const MAX_FILE: u64 = 64 << 20;
const WIND_FIELDS: [&str; 7] = ["enabled", "strength", "decay", "turbulence", "direction", "origin", "kind"];
const PANEL_FIELDS: [&str; 6] = ["frozen", "disabled", "stiffen", "hold_shape", "shrink_weft", "shrink_warp"];

fn uint(a: &Value, key: &str, max: u64) -> Result<Option<u32>, CommandError> {
    match a.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_u64().filter(|x| *x <= max).map(|x| Some(x as u32)).ok_or_else(|| bad(format!("{key} must be a whole number from 0 to {max}"))),
    }
}

fn cache_json(s: &Session, c: &ffb_pattern::anim::CacheLayer) -> Value {
    let mut out = serde_json::to_value(c).unwrap_or(Value::Null);
    if let Some(map) = out.as_object_mut() {
        match entry(s, &c.entry).map(cachefmt::read_header) {
            Some(Ok(h)) => {
                let max = |f: fn(&cachefmt::FrameStat) -> f32| h.stats.iter().map(f).fold(0.0f32, f32::max);
                map.insert("fps".into(), json!(h.fps));
                map.insert("quantized".into(), json!(h.quantized));
                map.insert("bytes".into(), json!(entry(s, &c.entry).map_or(0, <[u8]>::len)));
                map.insert("panels".into(), json!(h.panels.len()));
                map.insert("max_penetration_mm".into(), json!(max(|x| x.penetration_mm)));
                map.insert("max_self_intersections".into(), json!(h.stats.iter().map(|x| x.self_intersections).max().unwrap_or(0)));
                map.insert("max_displacement_mm".into(), json!(max(|x| x.max_displacement_mm)));
                map.insert("max_seam_gap_mm".into(), json!(max(|x| x.seam_gap_mm)));
            }
            Some(Err(e)) => {
                map.insert("error".into(), json!(e));
            }
            None => {
                map.insert("error".into(), json!("the cache data is missing"));
            }
        }
    }
    out
}

fn get(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    let anim = &s.garment.anim;
    let caches: Vec<Value> = anim.caches.iter().map(|c| cache_json(s, c)).collect();
    Ok(json!({
        "fps": anim.fps,
        "start": anim.start,
        "end": anim.end,
        "current": anim.current,
        "motions": anim.motions,
        "tracks": anim.tracks,
        "caches": caches,
        "active_cache": anim.active_cache,
        "auto_key": anim.auto_key,
        "playing": s.anim.playing.load(std::sync::atomic::Ordering::Acquire),
        "job": super::play::job_json(s, false),
    }))
}

fn set_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "fps": {"type": "number", "minimum": 1, "maximum": 240, "description": "Timeline frame rate."},
            "start": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME, "description": "First frame of the play range."},
            "end": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME, "description": "Last frame of the play range."},
            "current": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME, "description": "Current frame. Changing only this is not an undo step."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let fps = match a.get("fps").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(v.as_f64().filter(|x| x.is_finite() && (1.0..=240.0).contains(x)).ok_or_else(|| bad("fps must be from 1 to 240"))? as f32),
    };
    let (start, end, current) = (uint(a, "start", MAX_FRAME)?, uint(a, "end", MAX_FRAME)?, uint(a, "current", MAX_FRAME)?);
    let old = &s.garment.anim;
    let (ns, ne) = (start.unwrap_or(old.start), end.unwrap_or(old.end));
    if ns > ne {
        return Err(bad("start must not be after end"));
    }
    if fps.is_none() && start.is_none() && end.is_none() {
        if let Some(c) = current {
            s.garment.anim.current = c;
        }
        return Ok(json!({"fps": s.garment.anim.fps, "start": s.garment.anim.start, "end": s.garment.anim.end, "current": s.garment.anim.current, "changed": false}));
    }
    let changed = edit_garment(s, "Animation settings", |g| {
        if let Some(f) = fps {
            g.anim.fps = f;
        }
        g.anim.start = ns;
        g.anim.end = ne;
        if let Some(c) = current {
            g.anim.current = c;
        }
        Ok(())
    })?;
    let anim = &s.garment.anim;
    Ok(json!({"fps": anim.fps, "start": anim.start, "end": anim.end, "current": anim.current, "changed": changed}))
}

fn import_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Absolute path of a .bvh, .glb, .gltf or (desktop) .fbx motion file."},
            "body": {"type": "integer", "description": "Body id the motion drives. Defaults to the first body."},
            "name": {"type": "string"},
            "units": {"type": "string", "enum": ["auto", "mm", "cm", "m", "in", "ft"], "description": "Units of the motion file. BVH defaults to a guess from the skeleton height, glTF to metres. FBX is converted by the reader."},
            "up": {"type": "string", "description": "BVH up axis: y, -y, z, -z, x or -x. Default y."},
            "fps": {"type": "number", "minimum": 1, "maximum": 240, "description": "Sampling rate for glTF and FBX motions. Default 30."},
            "animation": {"type": "string", "description": "glTF animation name or index. Default the first."},
            "start": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME, "description": "Timeline frame where the motion begins. Default the play range start."},
            "lead_in": {"type": "integer", "minimum": 0, "maximum": 1000, "description": "Frames spent blending from the current pose to the first motion pose."},
            "retarget": {"type": "object", "description": "{map: {motion joint: body joint}, root: keep|inplace|off, scale, loop, align}."}
        },
        "required": ["path"],
        "additionalProperties": false
    })
}

fn unit_factor(name: &str) -> Result<Option<f32>, CommandError> {
    Ok(match name.trim().to_lowercase().as_str() {
        "auto" => None,
        "mm" => Some(1.0),
        "cm" => Some(10.0),
        "m" => Some(1000.0),
        "in" | "inch" => Some(25.4),
        "ft" | "foot" => Some(304.8),
        _ => return Err(bad("units must be auto, mm, cm, m, in or ft")),
    })
}

fn body_index(s: &Session, a: &Value, key: &str) -> Result<usize, CommandError> {
    match a.get(key).filter(|v| !v.is_null()) {
        Some(v) => {
            let id = v.as_u64().ok_or_else(|| bad(format!("{key} must be a body id")))?;
            s.bodies.iter().position(|b| u64::from(b.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no body with id {id}")))
        }
        None if s.bodies.is_empty() => Err(CommandError::new("not_found", "there are no bodies; use body.load_default or body.import")),
        None => Ok(0),
    }
}

fn read_clip(path: &str, a: &Value, name: &str) -> Result<Clip, CommandError> {
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let meta = std::fs::metadata(path).map_err(|_| CommandError::new("not_found", format!("no file at {path}")))?;
    if !meta.is_file() || meta.len() > MAX_FILE {
        return Err(bad(format!("{path} is not a regular file under {} MB", MAX_FILE >> 20)));
    }
    let units = match a.get("units").and_then(Value::as_str) {
        Some(u) => unit_factor(u)?,
        None => None,
    };
    let fps = a.get("fps").and_then(Value::as_f64).map_or(30.0, |x| x as f32);
    let fail = |e: String| CommandError::new("import_failed", e);
    match ext.as_str() {
        "bvh" => {
            let up = match a.get("up").and_then(Value::as_str) {
                Some(u) => import::up_rotation(u).ok_or_else(|| bad("up must be y, -y, z, -z, x or -x"))?,
                None => ffb_body::math::IDENTITY,
            };
            let bytes = std::fs::read(path).map_err(|e| fail(format!("could not read {path}: {e}")))?;
            bvh::parse(&bytes, &bvh::Options { units_mm: units, up, name: name.to_string() }).map_err(fail)
        }
        "glb" | "gltf" => {
            let bytes = std::fs::read(path).map_err(|e| fail(format!("could not read {path}: {e}")))?;
            let options = gltf::Options { fps, animation: a.get("animation").and_then(Value::as_str).map(str::to_string), units_mm: units.unwrap_or(1000.0), name: name.to_string() };
            gltf::parse(&bytes, std::path::Path::new(path).parent(), &options).map_err(fail)
        }
        #[cfg(not(target_arch = "wasm32"))]
        "fbx" => ffb_anim::fbx::parse(path, fps, name.to_string()).map_err(fail),
        #[cfg(target_arch = "wasm32")]
        "fbx" => Err(CommandError::new("unsupported", "FBX motion import is not available in the browser build")),
        other => Err(bad(format!("unsupported motion format .{other}; motion.import reads .bvh, .glb, .gltf and .fbx"))),
    }
}

fn next_motion_id(s: &mut Session) -> u32 {
    super::alloc_motion_id(s)
}

fn report(clip: &Clip, rt: &Retargeter) -> Value {
    let pairs: Vec<Value> = rt.mapping(clip).into_iter().map(|(a, b)| json!([a, b])).collect();
    let mapped: Vec<&str> = rt.mapping(clip).iter().map(|p| p.0).collect();
    let unmapped: Vec<&str> = clip.joints.iter().map(|j| j.name.as_str()).filter(|n| !mapped.contains(n)).take(40).collect();
    json!({"mapped": pairs, "unmapped": unmapped, "root_scale": rt.ratio()})
}

fn import_motion(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::file_path(a, "path", None)?;
    let bi = body_index(s, a, "body")?;
    let stem = std::path::Path::new(&path).file_stem().and_then(|x| x.to_str()).unwrap_or("Motion").to_string();
    let name = match a.get("name").and_then(Value::as_str) {
        Some(n) => {
            args::clean_name("name", n)?;
            n.to_string()
        }
        None => stem,
    };
    let clip = read_clip(&path, a, &name)?;
    let retarget = a.get("retarget").cloned().unwrap_or(Value::Null);
    let options = RetargetOptions::from_json(&retarget).map_err(bad)?;
    let rt = Retargeter::new(&clip, &s.bodies[bi].joints, &options).map_err(|e| CommandError::new("import_failed", e))?;
    let detail = report(&clip, &rt);
    let id = next_motion_id(s);
    let fps = s.garment.anim.fps;
    let start = uint(a, "start", MAX_FRAME)?.unwrap_or(s.garment.anim.start);
    let lead_in = uint(a, "lead_in", 1000)?.unwrap_or(0);
    let frames = ((clip.duration() * fps).ceil() as u32 + 1).min(MAX_FRAME as u32);
    let source = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let body_id = s.bodies[bi].id;
    let first = s.garment.anim.motions.is_empty();
    put_entry(s, motion_entry(id), clip.encode());
    edit_garment(s, "Import motion", |g| {
        g.anim.motions.push(Motion { id, name: name.clone(), source, body: body_id, start, frames, fps: clip.fps, lead_in, retarget: options.to_json() });
        if first {
            g.anim.start = start;
            g.anim.end = (start + lead_in + frames.saturating_sub(1)).min(MAX_FRAME as u32);
        }
        Ok(())
    })?;
    Ok(json!({"id": id, "name": name, "body": body_id, "joints": clip.joints.len(), "clip_frames": clip.frames, "clip_fps": clip.fps, "duration": clip.duration(), "frames": frames, "start": start, "lead_in": lead_in, "retarget": detail}))
}

fn motion_set_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "description": "Motion id."},
            "name": {"type": "string"},
            "body": {"type": "integer"},
            "start": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME},
            "lead_in": {"type": "integer", "minimum": 0, "maximum": 1000},
            "retarget": {"type": "object", "description": "Options merged over the current ones: map, root, scale, loop, align."}
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn motion_id(s: &Session, a: &Value) -> Result<usize, CommandError> {
    let id = a.get("id").and_then(Value::as_u64).ok_or_else(|| bad("id must be a motion id"))?;
    s.garment.anim.motions.iter().position(|m| u64::from(m.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no motion with id {id}")))
}

fn motion_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = motion_id(s, a)?;
    let current = s.garment.anim.motions[k].clone();
    let mut next = current.clone();
    if let Some(n) = a.get("name").and_then(Value::as_str) {
        args::clean_name("name", n)?;
        next.name = n.to_string();
    }
    if a.get("body").is_some() {
        let bi = body_index(s, a, "body")?;
        next.body = s.bodies[bi].id;
    }
    if let Some(v) = uint(a, "start", MAX_FRAME)? {
        next.start = v;
    }
    if let Some(v) = uint(a, "lead_in", 1000)? {
        next.lead_in = v;
    }
    let mut detail = Value::Null;
    if let Some(r) = a.get("retarget") {
        let mut merged: Map<String, Value> = current.retarget.as_object().cloned().unwrap_or_default();
        for (key, v) in r.as_object().ok_or_else(|| bad("retarget must be an object"))? {
            merged.insert(key.clone(), v.clone());
        }
        let options = RetargetOptions::from_json(&Value::Object(merged)).map_err(bad)?;
        next.retarget = options.to_json();
    }
    if next.source.starts_with("morph:") {
        if a.get("retarget").is_some() {
            return Err(bad("a morph has no retarget"));
        }
    } else if next.retarget != current.retarget || next.body != current.body {
        let data = entry(s, &motion_entry(next.id)).ok_or_else(|| CommandError::new("not_found", "the motion data is missing"))?;
        let clip = Clip::decode(data).map_err(|e| CommandError::new("invalid_geometry", e))?;
        let bi = s.bodies.iter().position(|b| b.id == next.body).ok_or_else(|| CommandError::new("not_found", "the motion body no longer exists"))?;
        let options = RetargetOptions::from_json(&next.retarget).map_err(bad)?;
        let rt = Retargeter::new(&clip, &s.bodies[bi].joints, &options).map_err(|e| CommandError::new("invalid_geometry", e))?;
        detail = report(&clip, &rt);
    }
    let changed = edit_garment(s, "Edit motion", |g| {
        g.anim.motions[k] = next.clone();
        Ok(())
    })?;
    Ok(json!({"id": next.id, "changed": changed, "motion": next, "retarget": detail}))
}

fn id_schema() -> Value {
    json!({"type": "object", "properties": {"id": {"type": "integer"}}, "required": ["id"], "additionalProperties": false})
}

fn motion_remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = motion_id(s, a)?;
    let id = s.garment.anim.motions[k].id;
    edit_garment(s, "Remove motion", |g| {
        g.anim.motions.remove(k);
        Ok(())
    })?;
    Ok(json!({"removed": id}))
}

fn parse_target(s: &Session, target: &str) -> Result<(), CommandError> {
    if let Some(id) = super::autokey::pin_target(target) {
        return if s.garment.pins.iter().any(|p| p.id == id) { Ok(()) } else { Err(CommandError::new("not_found", format!("no pin with id {id}"))) };
    }
    let parts: Vec<&str> = target.split('.').collect();
    let unknown = || bad(format!("unknown key target {target}; use body.ID.pose, body.ID.MEASUREMENT, wind.FIELD, panel.ID.state.FIELD, fabric.ID.FIELD or pin:ID"));
    let number = |t: &str| t.parse::<u32>().map_err(|_| unknown());
    match parts.as_slice() {
        ["wind", field] if WIND_FIELDS.contains(field) => Ok(()),
        ["body", id, field] => {
            let id = number(id)?;
            if !s.bodies.iter().any(|b| b.id == id) {
                return Err(CommandError::new("not_found", format!("no body with id {id}")));
            }
            if *field == "pose" || MEASUREMENTS.contains(field) {
                Ok(())
            } else {
                Err(unknown())
            }
        }
        ["panel", id, "state", field] if PANEL_FIELDS.contains(field) => {
            let id = number(id)?;
            if s.garment.index_of(id).is_none() {
                return Err(CommandError::new("not_found", format!("no panel with id {id}")));
            }
            Ok(())
        }
        ["fabric", id, field] => {
            let id = number(id)?;
            let fabric = s.garment.fabrics.iter().find(|f| f.id == id).ok_or_else(|| CommandError::new("not_found", format!("no fabric with id {id}")))?;
            let physical = serde_json::to_value(&fabric.physical).unwrap_or(Value::Null);
            if physical.get(*field).is_some() {
                Ok(())
            } else {
                Err(unknown())
            }
        }
        _ => Err(unknown()),
    }
}

fn check_value(target: &str, v: &Value) -> Result<(), CommandError> {
    let numeric = |v: &Value| v.as_f64().is_some_and(f64::is_finite);
    let flag = |v: &Value| v.is_boolean() || numeric(v);
    let ok = if target.starts_with("pin:") {
        v.as_array().is_some_and(|a| a.len() == 3 && a.iter().all(numeric))
    } else if target.ends_with(".pose") {
        v.is_string() || v.as_object().is_some_and(|m| m.values().all(|q| q.as_array().is_some_and(|a| (a.len() == 4) && a.iter().all(numeric))))
    } else if target == "wind.kind" {
        v.is_string()
    } else if target == "wind.direction" || target == "wind.origin" {
        v.as_array().is_some_and(|a| a.len() == 3 && a.iter().all(numeric))
    } else if target == "wind.enabled" || target.contains(".state.frozen") || target.contains(".state.disabled") || target.contains(".state.hold_shape") {
        flag(v)
    } else {
        numeric(v)
    };
    if ok {
        Ok(())
    } else {
        Err(bad(format!("the value does not fit {target}")))
    }
}

fn key_set_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "target": {"type": "string", "description": "body.ID.pose (a pose name or {joint: [x, y, z, w]}), body.ID.MEASUREMENT, wind.FIELD, panel.ID.state.FIELD, fabric.ID.FIELD, where ID is a number and FIELD or MEASUREMENT a name; or pin:ID for a pin, with a value [x, y, z] in mm that is where recording holds the pin."},
            "frame": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME},
            "value": {"description": "A number, a boolean, a vector, a pose name or a joint rotation object, by target."},
            "interp": {"type": "string", "enum": ["step", "linear", "smooth"], "description": "Interpolation from this key to the next. smooth is a cubic Hermite spline through Catmull-Rom tangents. Default linear."}
        },
        "required": ["target", "frame", "value"],
        "additionalProperties": false
    })
}

fn key_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let target = a.get("target").and_then(Value::as_str).ok_or_else(|| bad("target must be text"))?.to_string();
    let frame = uint(a, "frame", MAX_FRAME)?.ok_or_else(|| bad("frame is required"))?;
    let value = a.get("value").cloned().ok_or_else(|| bad("value is required"))?;
    let interp = match a.get("interp").and_then(Value::as_str) {
        None | Some("linear") => Interp::Linear,
        Some("step") => Interp::Step,
        Some("smooth") => Interp::Smooth,
        Some(_) => return Err(bad("interp must be step, linear or smooth")),
    };
    parse_target(s, &target)?;
    check_value(&target, &value)?;
    let next_id = s.garment.anim.tracks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let mut track_id = 0;
    edit_garment(s, "Set key", |g| {
        let at = match g.anim.tracks.iter().position(|t| t.target == target) {
            Some(i) => i,
            None => {
                g.anim.tracks.push(Track { id: next_id, target: target.clone(), keys: Vec::new() });
                g.anim.tracks.len() - 1
            }
        };
        let track = &mut g.anim.tracks[at];
        track_id = track.id;
        track.keys.retain(|k| k.frame != frame);
        track.keys.push(Key { frame, value: value.clone(), interp });
        track.keys.sort_by_key(|k| k.frame);
        Ok(())
    })?;
    let keys = s.garment.anim.tracks.iter().find(|t| t.target == target).map_or(0, |t| t.keys.len());
    Ok(json!({"track": track_id, "target": target, "frame": frame, "keys": keys}))
}

fn key_remove_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"target": {"type": "string"}, "frame": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME}},
        "required": ["target", "frame"],
        "additionalProperties": false
    })
}

fn key_remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let target = a.get("target").and_then(Value::as_str).ok_or_else(|| bad("target must be text"))?.to_string();
    let frame = uint(a, "frame", MAX_FRAME)?.ok_or_else(|| bad("frame is required"))?;
    let at = s.garment.anim.tracks.iter().position(|t| t.target == target).ok_or_else(|| CommandError::new("not_found", format!("no track for {target}")))?;
    if !s.garment.anim.tracks[at].keys.iter().any(|k| k.frame == frame) {
        return Err(CommandError::new("not_found", format!("{target} has no key at frame {frame}")));
    }
    edit_garment(s, "Remove key", |g| {
        g.anim.tracks[at].keys.retain(|k| k.frame != frame);
        if g.anim.tracks[at].keys.is_empty() {
            g.anim.tracks.remove(at);
        }
        Ok(())
    })?;
    let keys = s.garment.anim.tracks.iter().find(|t| t.target == target).map_or(0, |t| t.keys.len());
    Ok(json!({"target": target, "frame": frame, "keys": keys}))
}

fn cache_set_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "description": "Cache layer id."},
            "name": {"type": "string"},
            "visible": {"type": "boolean"},
            "active": {"type": "boolean", "description": "Make this the cache that anim.play and the timeline use."}
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

pub(crate) fn cache_index(s: &Session, a: &Value) -> Result<usize, CommandError> {
    let id = a.get("id").and_then(Value::as_u64).ok_or_else(|| bad("id must be a cache id"))?;
    s.garment.anim.caches.iter().position(|c| u64::from(c.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no cache with id {id}")))
}

fn cache_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = cache_index(s, a)?;
    if let Some(n) = a.get("name").and_then(Value::as_str) {
        args::clean_name("name", n)?;
        if n != s.garment.anim.caches[k].name {
            super::layers::guard(s, k)?;
        }
    }
    let id = s.garment.anim.caches[k].id;
    let changed = edit_garment(s, "Edit cache", |g| {
        if let Some(n) = a.get("name").and_then(Value::as_str) {
            g.anim.caches[k].name = n.to_string();
        }
        if let Some(v) = a.get("visible").and_then(Value::as_bool) {
            g.anim.caches[k].visible = v;
        }
        match a.get("active").and_then(Value::as_bool) {
            Some(true) => g.anim.active_cache = Some(id),
            Some(false) if g.anim.active_cache == Some(id) => g.anim.active_cache = None,
            _ => {}
        }
        Ok(())
    })?;
    Ok(json!({"id": id, "changed": changed, "cache": cache_json(s, &s.garment.anim.caches[k])}))
}

fn cache_remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = cache_index(s, a)?;
    super::layers::guard(s, k)?;
    let id = s.garment.anim.caches[k].id;
    edit_garment(s, "Remove cache", |g| {
        g.anim.caches.remove(k);
        if g.anim.active_cache == Some(id) {
            g.anim.active_cache = g.anim.caches.last().map(|c| c.id);
        }
        Ok(())
    })?;
    Ok(json!({"removed": id}))
}

fn morph_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "from_body": {"type": "integer", "description": "Body id at the start. It stays the body that collides."},
            "to_body": {"type": "integer", "description": "Body id at the end. It needs the same vertex count and is left out of collisions while the morph is recorded."},
            "frames": {"type": "integer", "minimum": 1, "maximum": MAX_FRAME, "description": "Frames the morph takes."},
            "start": {"type": "integer", "minimum": 0, "maximum": MAX_FRAME, "description": "Frame where the morph begins. Default the play range start."}
        },
        "required": ["from_body", "to_body", "frames"],
        "additionalProperties": false
    })
}

fn morph_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let from = body_index(s, a, "from_body")?;
    let to = body_index(s, a, "to_body")?;
    if from == to {
        return Err(bad("from_body and to_body must be different bodies"));
    }
    let (fb, tb) = (&s.bodies[from], &s.bodies[to]);
    if fb.positions.len() != tb.positions.len() || fb.triangles.len() != tb.triangles.len() {
        return Err(CommandError::new("invalid_geometry", format!("body {} and body {} do not share a mesh topology ({} and {} vertices); a morph needs bodies made from the same mesh", fb.id, tb.id, fb.positions.len(), tb.positions.len())));
    }
    let frames = uint(a, "frames", MAX_FRAME)?.filter(|f| *f >= 1).ok_or_else(|| bad("frames must be at least 1"))?;
    let start = uint(a, "start", MAX_FRAME)?.unwrap_or(s.garment.anim.start);
    let (from_id, to_id) = (fb.id, tb.id);
    let name = format!("Morph {} to {}", fb.name, tb.name);
    let id = next_motion_id(s);
    let fps = s.garment.anim.fps;
    edit_garment(s, "Set morph", |g| {
        g.anim.motions.retain(|m| !(m.body == from_id && m.source.starts_with("morph:")));
        g.anim.motions.push(Motion { id, name: name.clone(), source: format!("morph:{to_id}"), body: from_id, start, frames, fps, lead_in: 0, retarget: Value::Null });
        let last = start.saturating_add(frames);
        if g.anim.end < last {
            g.anim.end = last.min(MAX_FRAME as u32);
        }
        Ok(())
    })?;
    Ok(json!({"id": id, "from_body": from_id, "to_body": to_id, "start": start, "frames": frames}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command { id: "anim.get", mcp: "anim_get", group: Group::Anim, label: "Animation state", description: "Read the animation timeline: frame rate, play range, current frame, motions, key tracks, cache layers with their validation numbers (worst body penetration, self intersections, vertex displacement, seam gap) and the recording job.", keys: &[], hints: READ, schema: no_args, run: get },
        Command { id: "anim.set", mcp: "anim_set", group: Group::Anim, label: "Set animation range", description: "Set the timeline frame rate, play range or current frame. Frame rate and range are one undo step; moving only the current frame is not.", keys: &[], hints: SET, schema: set_schema, run: set },
        Command { id: "motion.import", mcp: "motion_import", group: Group::Anim, label: "Import motion", description: "Read a motion file (.bvh, .glb or .gltf animation with linear, step and cubic spline channels, and .fbx on desktop) and retarget it onto a body by joint-name mapping with rest pose alignment (A pose or T pose source), a root motion option (inplace, keep or off) and a lead-in. The converted motion is stored in the project. Returns which joints mapped.", keys: &[], hints: WRITE, schema: import_schema, run: import_motion },
        Command { id: "motion.set", mcp: "motion_set", group: Group::Anim, label: "Edit motion", description: "Change a motion's start frame, lead-in frames (blend from the current pose to the first motion pose), body, name or retarget options (map, root, scale, loop, align).", keys: &[], hints: SET, schema: motion_set_schema, run: motion_set },
        Command { id: "motion.remove", mcp: "motion_remove", group: Group::Anim, label: "Remove motion", description: "Remove a motion from the timeline.", keys: &[], hints: DESTROY, schema: id_schema, run: motion_remove },
        Command { id: "key.set", mcp: "key_set", group: Group::Anim, label: "Set key", description: "Set a keyframe on a track: body.ID.pose, body.ID.MEASUREMENT, wind.FIELD, panel.ID.state.FIELD or fabric.ID.FIELD. Interpolation step, linear or smooth (cubic Hermite with Catmull-Rom tangents). Recording applies the body, measurement and wind tracks every frame.", keys: &[], hints: SET, schema: key_set_schema, run: key_set },
        Command { id: "key.remove", mcp: "key_remove", group: Group::Anim, label: "Remove key", description: "Remove the key at a frame from a track; an empty track is deleted.", keys: &[], hints: DESTROY, schema: key_remove_schema, run: key_remove },
        Command { id: "cache.set", mcp: "cache_set", group: Group::Anim, label: "Edit cache layer", description: "Rename a cache layer, show or hide it, or make it the active cache.", keys: &[], hints: SET, schema: cache_set_schema, run: cache_set },
        Command { id: "cache.remove", mcp: "cache_remove", group: Group::Anim, label: "Remove cache layer", description: "Delete a cache layer and its stored frames. Undo brings it back.", keys: &[], hints: DESTROY, schema: id_schema, run: cache_remove },
        Command { id: "morph.set", mcp: "morph_set", group: Group::Anim, label: "Set body morph", description: "Morph a body into another body made from the same mesh over a number of frames. Recording moves the collider through the blend like a motion.", keys: &[], hints: SET, schema: morph_schema, run: morph_set },
    ]
}
