use ffb_anim::cachefmt::{self, Cache, FrameStat, Header, PanelSpec, Writer};
use ffb_pattern::anim::{Anim, CacheKind, CacheLayer};
use serde_json::{json, Value};

use super::ops::cache_index;
use super::rig::cache_entry;
use super::{bad, edit_garment, entry, put_entry};
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

pub(crate) fn resolve(anim: &Anim, frame: u32, kind: CacheKind) -> Option<usize> {
    anim.caches.iter().rposition(|c| c.kind == kind && c.visible && frame >= c.start && frame - c.start < c.frames)
}

fn refuse_locked(layer: &CacheLayer) -> Result<(), CommandError> {
    if layer.locked {
        Err(CommandError::new("locked", format!("cache layer {} is locked; cache.lock unlocks it", layer.id)))
    } else {
        Ok(())
    }
}

pub(crate) fn guard(s: &Session, k: usize) -> Result<(), CommandError> {
    refuse_locked(&s.garment.anim.caches[k])
}

const MAX_FRAME: u32 = 100_000;

fn span(layer: &CacheLayer) -> Result<u32, CommandError> {
    if layer.frames == 0 || layer.frames > MAX_FRAME || layer.start > MAX_FRAME {
        return Err(CommandError::new("invalid_cache", format!("cache layer {} has an invalid range (start {}, {} frames)", layer.id, layer.start, layer.frames)));
    }
    Ok(layer.start + layer.frames - 1)
}

fn bytes_of(s: &Session, layer: &CacheLayer) -> Result<Vec<u8>, CommandError> {
    span(layer)?;
    entry(s, &layer.entry).map(<[u8]>::to_vec).ok_or_else(|| CommandError::new("not_found", "the cache data is missing"))
}

fn frame_arg(a: &Value, key: &str) -> Result<u32, CommandError> {
    a.get(key).and_then(Value::as_u64).filter(|x| *x <= 100_000).map(|x| x as u32).ok_or_else(|| bad(format!("{key} must be a frame number")))
}

fn cut_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "description": "Cache layer id."},
            "frame": {"type": "integer", "minimum": 1, "description": "Timeline frame inside the layer. The layer keeps the frames before it; this frame and the ones after become a new layer above it."}
        },
        "required": ["id", "frame"],
        "additionalProperties": false
    })
}

fn cut(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = cache_index(s, a)?;
    guard(s, k)?;
    let layer = s.garment.anim.caches[k].clone();
    let frame = frame_arg(a, "frame")?;
    let last = span(&layer)?;
    if frame <= layer.start || frame > last {
        return Err(bad(format!("frame must be inside the layer after its first frame: {} to {}", layer.start + 1, last)));
    }
    let bytes = bytes_of(s, &layer)?;
    super::reserve_trash(s, std::slice::from_ref(&layer.entry))?;
    let head = frame - layer.start;
    let tail = layer.frames - head;
    let first = cachefmt::slice(&bytes, 0, head, layer.start).map_err(|e| CommandError::new("invalid_cache", e))?;
    let second = cachefmt::slice(&bytes, head, tail, frame).map_err(|e| CommandError::new("invalid_cache", e))?;
    let first_id = super::alloc_cache_id(s);
    put_entry(s, cache_entry(first_id), first);
    let new_id = super::alloc_cache_id(s);
    put_entry(s, cache_entry(new_id), second);
    let upper = CacheLayer { id: new_id, name: format!("{} cut", layer.name), start: frame, frames: tail, visible: layer.visible, entry: cache_entry(new_id), kind: layer.kind, locked: false, offset: 0, source: layer.source };
    edit_garment(s, "Cut cache", |g| {
        let low = &mut g.anim.caches[k];
        low.frames = head;
        low.entry = cache_entry(first_id);
        low.offset = 0;
        g.anim.caches.insert(k + 1, upper.clone());
        Ok(())
    })?;
    Ok(json!({"id": layer.id, "frames": head, "new": new_id, "new_start": frame, "new_frames": tail}))
}

fn merge_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"ids": {"type": "array", "items": {"type": "integer"}, "minItems": 2, "description": "Cache layer ids of one kind. They merge into the top selected layer, which wins where layers overlap; gaps between layers are filled by linear interpolation."}},
        "required": ["ids"],
        "additionalProperties": false
    })
}

fn merge(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids: Vec<u32> = a.get("ids").and_then(Value::as_array).ok_or_else(|| bad("ids must be a list of cache ids"))?.iter().filter_map(|v| v.as_u64().map(|x| x as u32)).collect();
    let mut picked: Vec<usize> = Vec::new();
    for id in &ids {
        let k = s.garment.anim.caches.iter().position(|c| c.id == *id).ok_or_else(|| CommandError::new("not_found", format!("no cache with id {id}")))?;
        if !picked.contains(&k) {
            picked.push(k);
        }
    }
    if picked.len() < 2 {
        return Err(bad("merge needs at least two different layers"));
    }
    picked.sort_unstable();
    let kind = s.garment.anim.caches[picked[0]].kind;
    for k in &picked {
        guard(s, *k)?;
        if s.garment.anim.caches[*k].kind != kind {
            return Err(bad("layers merge only within their own kind"));
        }
    }
    let mut stored = Vec::new();
    for k in &picked {
        let layer = &s.garment.anim.caches[*k];
        stored.push((layer.start, bytes_of(s, layer)?));
    }
    let entries: Vec<String> = picked.iter().map(|k| s.garment.anim.caches[*k].entry.clone()).collect();
    super::reserve_trash(s, &entries)?;
    let parts: Vec<(u32, &[u8])> = stored.iter().map(|(st, b)| (*st, b.as_slice())).collect();
    let done = cachefmt::compose(&parts).map_err(|e| CommandError::new("invalid_cache", e))?;
    let top = *picked.last().unwrap_or(&0);
    let top_id = s.garment.anim.caches[top].id;
    let fresh = super::alloc_cache_id(s);
    put_entry(s, cache_entry(fresh), done.bytes);
    let removed: Vec<u32> = picked.iter().filter(|k| **k != top).map(|k| s.garment.anim.caches[*k].id).collect();
    edit_garment(s, "Merge cache", |g| {
        let at = g.anim.caches.iter().position(|c| c.id == top_id).unwrap_or(0);
        g.anim.caches[at].start = done.start;
        g.anim.caches[at].frames = done.frames;
        g.anim.caches[at].entry = cache_entry(fresh);
        g.anim.caches[at].offset = 0;
        g.anim.caches.retain(|c| !removed.contains(&c.id));
        if g.anim.active_cache.is_some_and(|id| removed.contains(&id)) {
            g.anim.active_cache = Some(top_id);
        }
        Ok(())
    })?;
    Ok(json!({"id": top_id, "merged": removed, "start": done.start, "frames": done.frames}))
}

fn move_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "description": "Cache layer id."},
            "frames": {"type": "integer", "minimum": -100000, "maximum": 100000, "description": "Frames to shift the layer start by; negative moves it earlier."}
        },
        "required": ["id", "frames"],
        "additionalProperties": false
    })
}

fn shift(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = cache_index(s, a)?;
    guard(s, k)?;
    let by = a.get("frames").and_then(Value::as_i64).filter(|x| x.abs() <= 100_000).ok_or_else(|| bad("frames must be a whole number from -100000 to 100000"))?;
    let layer = &s.garment.anim.caches[k];
    let start = i64::from(layer.start) + by;
    if !(0..=100_000).contains(&start) {
        return Err(bad(format!("the layer would start at frame {start}; frames run from 0 to 100000")));
    }
    let id = layer.id;
    edit_garment(s, "Move cache", |g| {
        g.anim.caches[k].start = start as u32;
        g.anim.caches[k].offset += by as i32;
        Ok(())
    })?;
    let layer = &s.garment.anim.caches[k];
    Ok(json!({"id": id, "start": layer.start, "offset": layer.offset, "end": layer.start + layer.frames.saturating_sub(1)}))
}

fn lock_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"id": {"type": "integer"}, "locked": {"type": "boolean", "description": "A locked layer refuses cut, merge, move, rename, removal and re-recording."}},
        "required": ["id", "locked"],
        "additionalProperties": false
    })
}

fn lock(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let k = cache_index(s, a)?;
    let on = a.get("locked").and_then(Value::as_bool).ok_or_else(|| bad("locked must be true or false"))?;
    let id = s.garment.anim.caches[k].id;
    let changed = edit_garment(s, "Lock cache", |g| {
        g.anim.caches[k].locked = on;
        Ok(())
    })?;
    Ok(json!({"id": id, "locked": on, "changed": changed}))
}

fn auto_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"on": {"type": "boolean", "description": "When on, changing a keyed property at the current frame adds or updates a key there. The first key of a property stays manual (key.set)."}},
        "required": ["on"],
        "additionalProperties": false
    })
}

fn auto_key(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let on = a.get("on").and_then(Value::as_bool).ok_or_else(|| bad("on must be true or false"))?;
    let changed = edit_garment(s, "Auto key", |g| {
        g.anim.auto_key = on;
        Ok(())
    })?;
    Ok(json!({"on": on, "changed": changed}))
}

pub(crate) fn trim_cache(s: &mut Session, pattern: &[u8], name: &str) -> Option<Vec<u8>> {
    let cache = Cache::parse(pattern).ok()?;
    let bindings = crate::trim_cmds::session_trim_bindings(s);
    let used: Vec<_> = bindings.hardware.iter().filter(|h| cache.header.panels.iter().any(|p| p.id == h.at.panel)).collect();
    if used.is_empty() {
        return None;
    }
    let header = Header { name: format!("{name} trims"), fps: cache.header.fps, start: cache.header.start, frames: 0, quantized: false, panels: used.iter().map(|h| PanelSpec { id: h.hardware, vertices: 1 }).collect(), bodies: Vec::new(), stats: Vec::new() };
    let mut writer = Writer::new(header);
    let slots: Vec<usize> = used.iter().filter_map(|h| cache.header.panels.iter().position(|p| p.id == h.at.panel)).collect();
    for rel in 0..cache.header.frames {
        let mut points = Vec::with_capacity(used.len());
        for (h, slot) in used.iter().zip(&slots) {
            let frame = cache.panel_frame(rel, *slot)?;
            let mut p = [0.0f32; 3];
            for (v, w) in h.at.vertices.iter().zip(h.at.weights) {
                let at = *v as usize * 3;
                for (c, slot) in p.iter_mut().enumerate() {
                    *slot += frame.get(at + c).copied().unwrap_or(0.0) * w as f32;
                }
            }
            points.push(p.to_vec());
        }
        let stat = FrameStat { frame: cache.header.start + rel, ..FrameStat::default() };
        writer.push(&points, &[], stat).ok()?;
    }
    writer.finish().ok()
}

pub(crate) fn trim_points(bytes: &[u8], rel: u32) -> Option<Vec<(u32, [f32; 3])>> {
    let cache = Cache::parse(bytes).ok()?;
    let mut out = Vec::new();
    for (i, p) in cache.header.panels.iter().enumerate() {
        let v = cache.panel_frame(rel, i)?;
        out.push((p.id, [*v.first()?, *v.get(1)?, *v.get(2)?]));
    }
    Some(out)
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command { id: "cache.cut", mcp: "cache_cut", group: Group::Anim, label: "Cut cache layer", description: "Split a recorded cache layer at a frame: the layer keeps the frames before it and the rest becomes a new layer above it. New stored entries are made; one undo step. A locked layer refuses.", keys: &[], hints: WRITE, schema: cut_schema, run: cut },
        Command { id: "cache.merge", mcp: "cache_merge", group: Group::Anim, label: "Merge cache layers", description: "Merge two or more cache layers of one kind into the top selected layer. Where layers overlap the top one wins; a gap between layers is filled by linear interpolation. The other layers are removed. One undo step.", keys: &[], hints: DESTROY, schema: merge_schema, run: merge },
        Command { id: "cache.move", mcp: "cache_move", group: Group::Anim, label: "Move cache layer", description: "Shift a cache layer in time by a number of frames (negative is earlier). The stored frames are not changed; the layer start and offset are.", keys: &[], hints: SET, schema: move_schema, run: shift },
        Command { id: "cache.lock", mcp: "cache_lock", group: Group::Anim, label: "Lock cache layer", description: "Lock or unlock a cache layer. A locked layer refuses cut, merge, move, rename, removal and re-recording.", keys: &[], hints: SET, schema: lock_schema, run: lock },
        Command { id: "anim.auto_key", mcp: "anim_auto_key", group: Group::Anim, label: "Auto key", description: "Turn auto keying on or off. When on, a change to a property that already has a key (wind, panel state, fabric physical, or a pin position), made at the current frame, writes a key at that frame in the same undo step. The first key stays manual.", keys: &[], hints: SET, schema: auto_schema, run: auto_key },
    ]
}
