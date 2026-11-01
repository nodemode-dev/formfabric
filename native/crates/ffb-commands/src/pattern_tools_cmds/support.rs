use ffb_pattern::geom::{finite, V2};
use ffb_pattern::{LinkKind, Panel};
use serde_json::{json, Map, Value};

use crate::{CommandError, Edit, Session};

pub const COORD: f64 = 100_000.0;
pub const SIZE: f64 = 20_000.0;

pub fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub fn geometry(message: impl Into<String>) -> CommandError {
    CommandError::new("invalid_geometry", message)
}

pub fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub fn linked(panel: &Panel) -> CommandError {
    let (what, source) = panel.link.as_ref().map_or(("linked copy", 0), |l| (if l.kind == LinkKind::Mirror { "mirror copy" } else { "instance" }, l.source));
    CommandError::new("linked_panel", format!("panel {} is a {what} of panel {source}; its geometry follows the source. Edit the source panel, or run pattern.unlink on it first", panel.id))
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

pub fn flag(a: &Value, key: &str, default: bool) -> bool {
    a.get(key).and_then(Value::as_bool).unwrap_or(default)
}

pub fn text<'a>(a: &'a Value, key: &str) -> Option<&'a str> {
    a.get(key).and_then(Value::as_str)
}

pub fn required_id(a: &Value, key: &str) -> Result<u32, CommandError> {
    count(a, key).map(|x| x as u32).ok_or_else(|| bad(format!("{key} is required")))
}

pub fn vec2(v: &Value, name: &str) -> Result<V2, CommandError> {
    let list = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad(format!("{name} must be [x, y]")))?;
    match [list[0].as_f64(), list[1].as_f64()] {
        [Some(x), Some(y)] if finite([x, y]) => Ok([x, y]),
        _ => Err(bad(format!("{name} must hold two finite numbers"))),
    }
}

pub fn optional_vec2(a: &Value, key: &str) -> Result<Option<V2>, CommandError> {
    a.get(key).filter(|v| !v.is_null()).map(|v| vec2(v, key)).transpose()
}

pub fn world_or_local(a: &Value) -> bool {
    text(a, "space") != Some("local")
}

pub fn to_local(panel: &Panel, world: bool, p: V2) -> V2 {
    if world {
        panel.placement().to_local(p)
    } else {
        p
    }
}

pub fn obj(props: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

fn scalar(kind: &str, unit: Option<&str>, description: &str, lo: f64, hi: f64, default: Option<f64>) -> Value {
    let mut m = Map::new();
    m.insert("type".into(), json!(kind));
    if let Some(u) = unit {
        m.insert("unit".into(), json!(u));
    }
    m.insert("minimum".into(), json!(lo));
    m.insert("maximum".into(), json!(hi));
    if let Some(d) = default {
        m.insert("default".into(), json!(d));
    }
    m.insert("description".into(), json!(description));
    Value::Object(m)
}

pub fn mm(description: &str, lo: f64, hi: f64, default: Option<f64>) -> Value {
    scalar("number", Some("mm"), description, lo, hi, default)
}

pub fn size(description: &str, default: Option<f64>) -> Value {
    mm(description, 0.1, SIZE, default)
}

pub fn deg(description: &str, lo: f64, hi: f64, default: Option<f64>) -> Value {
    scalar("number", Some("deg"), description, lo, hi, default)
}

pub fn ratio(description: &str, lo: f64, hi: f64, default: Option<f64>) -> Value {
    scalar("number", None, description, lo, hi, default)
}

pub fn int(description: &str, lo: f64, hi: f64, default: Option<f64>) -> Value {
    scalar("integer", None, description, lo, hi, default)
}

pub fn xy(description: &str) -> Value {
    json!({
        "type": "array",
        "items": {"type": "number", "minimum": -COORD, "maximum": COORD},
        "minItems": 2,
        "maxItems": 2,
        "unit": "mm",
        "description": description
    })
}

pub fn choice(description: &str, options: &[&str], default: Option<&str>) -> Value {
    let mut v = json!({"type": "string", "enum": options, "description": description});
    if let Some(d) = default {
        v["default"] = json!(d);
    }
    v
}

pub fn switch(description: &str, default: bool) -> Value {
    json!({"type": "boolean", "default": default, "description": description})
}

pub fn panel_id() -> Value {
    int("Panel id as listed by pattern.get.", 1.0, 4_294_967_295.0, None)
}

pub fn space() -> Value {
    json!({
        "type": "string",
        "enum": ["world", "local"],
        "default": "world",
        "description": "world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply."
    })
}

pub fn index_of(s: &Session, id: u32) -> Result<usize, CommandError> {
    s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))
}

pub fn source_index(s: &Session, id: u32) -> Result<usize, CommandError> {
    let i = index_of(s, id)?;
    if s.garment.panels[i].link.is_some() {
        return Err(linked(&s.garment.panels[i]));
    }
    Ok(i)
}

pub fn replace(index: usize, label: &str, before: Panel, after: Panel) -> Edit {
    Edit::Panel { label: label.into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) }
}

pub fn commit(s: &mut Session, label: &str, mut edits: Vec<Edit>) {
    match edits.len() {
        0 => {}
        1 => s.record(edits.remove(0)),
        _ => s.record(Edit::Many { label: label.into(), edits }),
    }
}

pub fn edit_source<F>(s: &mut Session, id: u32, label: &str, f: F) -> Result<Value, CommandError>
where
    F: FnOnce(&mut Panel, &mut u32) -> Result<Value, CommandError>,
{
    let index = source_index(s, id)?;
    let before = s.garment.panels[index].clone();
    let mut after = before.clone();
    let mut next = s.garment.next_id();
    let mut out = f(&mut after, &mut next)?;
    let changed = after != before;
    if changed {
        s.garment.panels[index] = after.clone();
        s.record(replace(index, label, before, after));
    }
    out["changed"] = json!(changed);
    Ok(out)
}
