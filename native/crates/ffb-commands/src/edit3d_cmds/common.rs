use serde_json::Value;

use crate::{CommandError, Session};

pub(super) fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub(super) fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub(super) fn numbers<const N: usize>(a: &Value, key: &str) -> Result<Option<[f64; N]>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() == N).ok_or_else(|| bad(format!("{key} must hold {N} numbers")))?;
    let mut out = [0.0; N];
    for (slot, item) in out.iter_mut().zip(list) {
        *slot = item.as_f64().filter(|x| x.is_finite() && x.abs() <= 1e7).ok_or_else(|| bad(format!("{key} must hold {N} finite numbers")))?;
    }
    Ok(Some(out))
}

pub(super) fn id_of(v: &Value, key: &str) -> Result<u32, CommandError> {
    v.as_u64().and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad(format!("{key} must be a positive integer id")))
}

pub(super) fn id_arg(a: &Value, key: &str) -> Result<u32, CommandError> {
    id_of(a.get(key).unwrap_or(&Value::Null), key)
}

pub(super) fn id_list(a: &Value, key: &str) -> Result<Option<Vec<u32>>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().ok_or_else(|| bad(format!("{key} must be a list of ids")))?;
    list.iter().map(|x| id_of(x, key)).collect::<Result<Vec<_>, _>>().map(Some)
}

pub(super) fn ranged(a: &Value, key: &str, min: f64, max: f64) -> Result<Option<f64>, CommandError> {
    match a.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_f64().filter(|x| x.is_finite() && *x >= min && *x <= max).map(Some).ok_or_else(|| bad(format!("{key} must be a number from {min} to {max}"))),
    }
}

pub(super) fn panel_index(s: &Session, id: u32) -> Result<usize, CommandError> {
    s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))
}
