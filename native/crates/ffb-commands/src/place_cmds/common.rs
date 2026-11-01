use ffb_body::place::{PointFrame, Rig};
use ffb_body::{light_copy, Body, BodyChange, PlacementPoint, PlacementVolume};
use serde_json::{json, Value};

use crate::{CommandError, Edit, Session};

pub fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub fn body_schema() -> Value {
    json!({"type": "integer", "minimum": 1, "description": "Body id as listed by body.list. Defaults to the first body."})
}

pub fn selector(description: &str) -> Value {
    json!({"type": ["integer", "string"], "description": description})
}

pub fn obj(props: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

pub fn pick(s: &Session, a: &Value) -> Result<usize, CommandError> {
    match a.get("body").filter(|v| !v.is_null()) {
        Some(v) => {
            let id = v.as_u64().ok_or_else(|| bad("body must be a whole body id"))?;
            s.bodies.iter().position(|b| u64::from(b.id) == id).ok_or_else(|| missing(format!("no body with id {id}")))
        }
        None if s.bodies.is_empty() => Err(missing("there are no bodies; use body.load_default or body.import")),
        None => Ok(0),
    }
}

pub fn number(a: &Value, key: &str, lo: f64, hi: f64) -> Result<Option<f64>, CommandError> {
    match a.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi).map(Some).ok_or_else(|| bad(format!("{key} must be a number from {lo} to {hi}"))),
    }
}

pub fn pair(a: &Value, key: &str, lo: f64, hi: f64) -> Result<Option<[f64; 2]>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad(format!("{key} must hold two numbers")))?;
    let mut out = [0.0; 2];
    for (slot, item) in out.iter_mut().zip(list) {
        *slot = item.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi).ok_or_else(|| bad(format!("{key} must hold two numbers from {lo} to {hi}")))?;
    }
    Ok(Some(out))
}

pub fn triple(a: &Value, key: &str) -> Result<Option<[f64; 3]>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() == 3).ok_or_else(|| bad(format!("{key} must hold three numbers")))?;
    let mut out = [0.0; 3];
    for (slot, item) in out.iter_mut().zip(list) {
        *slot = item.as_f64().filter(|x| x.is_finite() && x.abs() <= 1e6).ok_or_else(|| bad(format!("{key} must hold three finite numbers")))?;
    }
    Ok(Some(out))
}

pub fn seed_defaults(body: &mut Body) -> Result<bool, CommandError> {
    if !body.volumes.is_empty() {
        return Ok(false);
    }
    let (volumes, points) = ffb_body::place_defaults::default_volumes(body);
    if volumes.is_empty() {
        return Err(CommandError::new("invalid_geometry", "this body has no skeleton with arms, legs and a neck, so no default volumes can be made; add them with place.volume_set"));
    }
    body.volumes = volumes;
    body.points = points;
    Ok(true)
}

pub fn volume_at(body: &Body, v: &Value) -> Option<usize> {
    match v {
        Value::Number(n) => n.as_u64().and_then(|id| body.volumes.iter().position(|x| u64::from(x.id) == id)),
        Value::String(name) => body.volumes.iter().position(|x| x.name.eq_ignore_ascii_case(name.trim())),
        _ => None,
    }
}

pub fn point_at(body: &Body, v: &Value) -> Option<usize> {
    match v {
        Value::Number(n) => n.as_u64().and_then(|id| body.points.iter().position(|x| u64::from(x.id) == id)),
        Value::String(name) => body.points.iter().position(|x| x.name.eq_ignore_ascii_case(name.trim())),
        _ => None,
    }
}

pub fn volume_partner(body: &Body, i: usize) -> Option<usize> {
    let name = &body.volumes[i].name;
    let twin = match (name.strip_suffix("_L"), name.strip_suffix("_R")) {
        (Some(stem), _) => format!("{stem}_R"),
        (_, Some(stem)) => format!("{stem}_L"),
        _ => return None,
    };
    body.volumes.iter().position(|v| v.name == twin)
}

pub fn point_partner(body: &Body, i: usize) -> Option<usize> {
    let p = &body.points[i];
    match p.mirror_of {
        Some(other) => body.points.iter().position(|x| x.id == other),
        None => body.points.iter().position(|x| x.mirror_of == Some(p.id)),
    }
}

pub fn next_volume_id(body: &Body) -> u32 {
    body.volumes.iter().map(|v| v.id).max().unwrap_or(0) + 1
}

pub fn next_point_id(body: &Body) -> u32 {
    body.points.iter().map(|v| v.id).max().unwrap_or(0) + 1
}

pub fn body_change(s: &Session, i: usize, before: Body) -> BodyChange {
    BodyChange { id: s.bodies[i].id, index: i, before: Some(Box::new(before)), after: Some(Box::new(light_copy(&s.bodies[i]))), light: true }
}

pub fn record_body(s: &mut Session, i: usize, label: &str, before: Body) {
    let change = body_change(s, i, before);
    s.record(Edit::Bodies { label: label.into(), changes: vec![change] });
}

pub fn frame_of(rig: &Rig, body: &Body, point: &PlacementPoint) -> Option<PointFrame> {
    let volume = body.volumes.iter().find(|v| v.id == point.volume)?;
    Some(rig.point(volume, point))
}

pub fn round(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn round3(v: [f64; 3]) -> [f64; 3] {
    [round(v[0]), round(v[1]), round(v[2])]
}

pub fn volume_json(rig: &Rig, v: &PlacementVolume) -> Value {
    let f = rig.volume(v);
    json!({
        "id": v.id,
        "name": v.name,
        "kind": v.kind,
        "joints": v.joints,
        "radius": v.radius,
        "height": round(f.height),
        "start": round3(f.start),
        "end": round3(f.end),
        "axis": round3(f.axis),
    })
}

pub fn point_json(rig: &Rig, body: &Body, p: &PlacementPoint) -> Value {
    let mut out = json!({
        "id": p.id,
        "name": p.name,
        "volume": p.volume,
        "around": p.around,
        "along": p.along,
        "offset": p.offset,
        "wrap": p.wrap,
        "mirror_of": p.mirror_of,
    });
    if let Some(f) = frame_of(rig, body, p) {
        out["position"] = json!(round3(f.position));
        out["surface"] = json!(round3(f.surface));
        out["normal"] = json!(round3(f.normal));
        out["up"] = json!(round3(f.up));
        out["bend_radius"] = json!(round(f.bend_radius));
    }
    out
}
