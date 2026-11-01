use ffb_pattern::geom::{finite, V2};
use ffb_pattern::{Panel, Point, PointKind};
use serde_json::Value;

use crate::CommandError;

pub fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub fn geometry(message: impl Into<String>) -> CommandError {
    CommandError::new("invalid_geometry", message)
}

pub fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

fn pick<'a>(spec: &'a Value, value: &Value) -> &'a Value {
    let wanted = match value {
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        _ => return spec,
    };
    spec.get("oneOf")
        .and_then(Value::as_array)
        .and_then(|alts| alts.iter().find(|alt| alt.get("type").and_then(Value::as_str) == Some(wanted)))
        .unwrap_or(spec)
}

fn check_value(spec: &Value, value: &Value, name: &str) -> Result<(), CommandError> {
    let spec = pick(spec, value);
    match value {
        Value::Number(n) => {
            let x = n.as_f64().unwrap_or(f64::NAN);
            if let Some(lo) = spec.get("minimum").and_then(Value::as_f64).filter(|lo| x < *lo) {
                return Err(bad(format!("{name} must be at least {lo}")));
            }
            if let Some(hi) = spec.get("maximum").and_then(Value::as_f64).filter(|hi| x > *hi) {
                return Err(bad(format!("{name} must be at most {hi}")));
            }
        }
        Value::Array(items) => {
            if let Some(lo) = spec.get("minItems").and_then(Value::as_u64).filter(|lo| (items.len() as u64) < *lo) {
                return Err(bad(format!("{name} needs at least {lo} items")));
            }
            if let Some(hi) = spec.get("maxItems").and_then(Value::as_u64).filter(|hi| items.len() as u64 > *hi) {
                return Err(bad(format!("{name} takes at most {hi} items")));
            }
            if let Some(item) = spec.get("items") {
                items.iter().try_for_each(|e| check_value(item, e, name))?;
            }
        }
        Value::Object(map) => {
            if let Some(required) = spec.get("required").and_then(Value::as_array) {
                if let Some(key) = required.iter().filter_map(Value::as_str).find(|k| !map.contains_key(*k)) {
                    return Err(bad(format!("{name} is missing {key}")));
                }
            }
            if let Some(props) = spec.get("properties").and_then(Value::as_object) {
                for (key, v) in map {
                    if let Some(p) = props.get(key) {
                        check_value(p, v, key)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn check(schema: fn() -> Value, args: &Value) -> Result<(), CommandError> {
    check_value(&schema(), args, "arguments")
}

pub fn number(a: &Value, key: &str) -> Option<f64> {
    a.get(key).and_then(Value::as_f64)
}

pub fn number_or(a: &Value, key: &str, default: f64) -> f64 {
    number(a, key).unwrap_or(default)
}

pub fn count(a: &Value, key: &str) -> Option<usize> {
    a.get(key).and_then(Value::as_f64).map(|x| x as usize)
}

pub fn flag(a: &Value, key: &str) -> Option<bool> {
    a.get(key).and_then(Value::as_bool)
}

pub fn text<'a>(a: &'a Value, key: &str) -> Option<&'a str> {
    a.get(key).and_then(Value::as_str)
}

pub fn required_count(a: &Value, key: &str) -> Result<usize, CommandError> {
    count(a, key).ok_or_else(|| bad(format!("{key} is required")))
}

pub fn required_id(a: &Value, key: &str) -> Result<u32, CommandError> {
    required_count(a, key).map(|x| x as u32)
}

pub fn optional_id(a: &Value, key: &str) -> Option<u32> {
    count(a, key).map(|x| x as u32)
}

pub fn vec2(v: &Value, name: &str) -> Result<V2, CommandError> {
    let list = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad(format!("{name} must be [x, y]")))?;
    let p = [list[0].as_f64(), list[1].as_f64()];
    match p {
        [Some(x), Some(y)] if finite([x, y]) => Ok([x, y]),
        _ => Err(bad(format!("{name} must hold two finite numbers"))),
    }
}

pub fn optional_vec2(a: &Value, key: &str) -> Result<Option<V2>, CommandError> {
    a.get(key).filter(|v| !v.is_null()).map(|v| vec2(v, key)).transpose()
}

pub fn id_list(a: &Value, key: &str) -> Result<Vec<u32>, CommandError> {
    let list = a.get(key).and_then(Value::as_array).ok_or_else(|| bad(format!("{key} must be a list of ids")))?;
    let mut out: Vec<u32> = Vec::new();
    for v in list {
        let id = v.as_f64().filter(|x| x.fract() == 0.0 && *x >= 0.0).ok_or_else(|| bad(format!("{key} must hold whole numbers")))? as u32;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    if out.is_empty() {
        return Err(bad(format!("{key} must not be empty")));
    }
    Ok(out)
}

pub fn index_list(a: &Value, key: &str) -> Result<Vec<usize>, CommandError> {
    id_list(a, key).map(|v| v.into_iter().map(|x| x as usize).collect())
}

fn kind_of(text: &str) -> Result<PointKind, CommandError> {
    match text {
        "corner" => Ok(PointKind::Corner),
        "curve" => Ok(PointKind::Curve),
        "smooth" => Ok(PointKind::Smooth),
        other => Err(bad(format!("kind must be corner, curve or smooth, not {other}"))),
    }
}

pub fn point_kind(a: &Value, key: &str) -> Result<Option<PointKind>, CommandError> {
    text(a, key).map(kind_of).transpose()
}

fn point(v: &Value) -> Result<Point, CommandError> {
    if v.is_array() {
        return Ok(Point { p: vec2(v, "point")?, ..Point::default() });
    }
    let p = vec2(v.get("p").ok_or_else(|| bad("a point needs p"))?, "p")?;
    let handle = |key: &str| optional_vec2(v, key);
    Ok(Point { p, kind: point_kind(v, "kind")?.unwrap_or_default(), h_in: handle("h_in")?, h_out: handle("h_out")? })
}

pub fn point_list(a: &Value, key: &str, min: usize) -> Result<Vec<Point>, CommandError> {
    let list = a.get(key).and_then(Value::as_array).ok_or_else(|| bad(format!("{key} must be a list of points")))?;
    if list.len() < min {
        return Err(bad(format!("{key} needs at least {min} points")));
    }
    list.iter().map(point).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    World,
    Local,
}

pub fn space(a: &Value) -> Space {
    if text(a, "space") == Some("local") {
        Space::Local
    } else {
        Space::World
    }
}

pub fn to_local(panel: &Panel, space: Space, p: V2) -> V2 {
    match space {
        Space::World => panel.placement().to_local(p),
        Space::Local => p,
    }
}

pub fn vec_to_local(panel: &Panel, space: Space, v: V2) -> V2 {
    match space {
        Space::World => panel.placement().vec_to_local(v),
        Space::Local => v,
    }
}

pub fn name(a: &Value, key: &str) -> Result<Option<String>, CommandError> {
    let Some(raw) = text(a, key) else { return Ok(None) };
    let t = raw.trim();
    if t.is_empty() || t.chars().count() > 64 {
        return Err(bad(format!("{key} must be 1 to 64 characters")));
    }
    Ok(Some(t.to_string()))
}
