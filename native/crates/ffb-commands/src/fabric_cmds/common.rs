use ffb_fabric::presets;
use ffb_fabric::Fabric;
use ffb_pattern::Garment;
use serde_json::{json, Value};

use crate::{CommandError, Edit, Session};

pub fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub fn next_fabric_id(g: &Garment) -> u32 {
    g.fabrics.iter().map(|f| f.id).max().unwrap_or(0) + 1
}

pub fn next_colorway_id(g: &Garment) -> u32 {
    g.colorways.iter().map(|c| c.id).max().unwrap_or(0) + 1
}

pub fn ensure_default(g: &mut Garment) -> bool {
    if g.fabrics.is_empty() {
        let id = next_fabric_id(g);
        g.fabrics.push(presets::instantiate(&presets::default_preset(), id, None));
        g.default_fabric = Some(id);
        return true;
    }
    if !g.default_fabric.is_some_and(|d| g.fabrics.iter().any(|f| f.id == d)) {
        g.default_fabric = g.fabrics.first().map(|f| f.id);
    }
    false
}

pub fn fabric_index(g: &Garment, id: u32) -> Result<usize, CommandError> {
    g.fabrics.iter().position(|f| f.id == id).ok_or_else(|| missing(format!("no fabric with id {id}")))
}

pub fn fabric_of(g: &Garment, id: u32) -> Result<&Fabric, CommandError> {
    fabric_index(g, id).map(|i| &g.fabrics[i])
}

pub fn id_arg(a: &Value, key: &str) -> Result<u32, CommandError> {
    a.get(key).and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad(format!("{key} must be a positive integer id")))
}

pub fn opt_id(a: &Value, key: &str) -> Result<Option<u32>, CommandError> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => id_arg(a, key).map(Some),
    }
}

pub fn id_list(a: &Value, key: &str) -> Result<Vec<u32>, CommandError> {
    let items = a.get(key).and_then(Value::as_array).ok_or_else(|| bad(format!("{key} must be an array of ids")))?;
    let mut out = Vec::new();
    for item in items {
        let id = item.as_u64().and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad(format!("{key} holds a value that is not a positive integer id")))?;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    if out.is_empty() {
        return Err(bad(format!("{key} needs at least one id")));
    }
    Ok(out)
}

pub fn text(a: &Value, key: &str) -> Result<Option<String>, CommandError> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => ffb_fabric::validate::name(s).map(Some).map_err(bad),
        Some(_) => Err(bad(format!("{key} must be a string"))),
    }
}

pub fn mutate<F>(s: &mut Session, label: &str, f: F) -> Result<Value, CommandError>
where
    F: FnOnce(&mut Garment) -> Result<Value, CommandError>,
{
    let before = s.garment.clone();
    let mut after = before.clone();
    let mut out = f(&mut after)?;
    let changed = after != before;
    if changed {
        s.garment = after.clone();
        s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
    }
    out["changed"] = json!(changed);
    Ok(out)
}

pub fn usage(g: &Garment, id: u32) -> Vec<u32> {
    g.panels.iter().filter(|p| p.fabric == Some(id)).map(|p| p.id).collect()
}
