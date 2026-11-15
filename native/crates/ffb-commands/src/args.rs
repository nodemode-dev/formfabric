use serde_json::{json, Value};

use crate::CommandError;

pub(crate) fn string<'a>(args: &'a Value, key: &str) -> Result<&'a str, CommandError> {
    args.get(key).and_then(Value::as_str).ok_or_else(|| CommandError::new("bad_args", format!("{key} must be a string")))
}

pub(crate) fn file_path(args: &Value, key: &str, extension: Option<&str>) -> Result<String, CommandError> {
    let bad = |m: String| CommandError::new("bad_args", m);
    let path = string(args, key)?;
    if path.is_empty() || path.contains('\0') {
        return Err(bad(format!("{key} must be a file path")));
    }
    let location = std::path::Path::new(path);
    if !location.is_absolute() {
        return Err(bad(format!("{key} must be an absolute path, got {path}")));
    }
    if let Some(wanted) = extension {
        let found = location.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !found.eq_ignore_ascii_case(wanted) {
            return Err(bad(format!("{key} must end in .{wanted}, got {path}")));
        }
    }
    Ok(path.to_string())
}

pub(crate) fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), CommandError> {
    use std::io::Write;
    if cfg!(target_arch = "wasm32") {
        return Err(CommandError::new("unsupported", "this build cannot write files"));
    }
    let io = |e: std::io::Error| CommandError::new("io", format!("could not write {path}: {e}"));
    let target = std::path::Path::new(path);
    let folder = target.parent().filter(|p| !p.as_os_str().is_empty()).ok_or_else(|| CommandError::new("bad_args", format!("{path} has no folder")))?;
    let name = target.file_name().and_then(|n| n.to_str()).ok_or_else(|| CommandError::new("bad_args", format!("{path} has no file name")))?;
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
    let temp = folder.join(format!(".{name}.{}.{nanos:x}.{}.tmp", std::process::id(), COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let written = std::fs::OpenOptions::new().write(true).create_new(true).open(&temp).and_then(|mut f| f.write_all(bytes).and_then(|_| f.sync_all()));
    if let Err(e) = written.and_then(|_| std::fs::rename(&temp, target)) {
        let _ = std::fs::remove_file(&temp);
        return Err(io(e));
    }
    Ok(())
}

pub(crate) fn has_control(text: &str) -> bool {
    text.chars().any(|c| c.is_control())
}

pub(crate) fn clean_name(key: &str, text: &str) -> Result<(), CommandError> {
    if has_control(text) {
        return Err(CommandError::new("bad_args", format!("{key} must not contain control characters")));
    }
    Ok(())
}

fn types_of(spec: &Value) -> Vec<&str> {
    match spec.get("type") {
        Some(Value::String(t)) => vec![t.as_str()],
        Some(Value::Array(ts)) => ts.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn coerce_scalars(schema: &Value, args: &mut Value) {
    let Some(map) = args.as_object_mut() else { return };
    let Some(props) = schema.get("properties").and_then(Value::as_object) else { return };
    for (key, value) in map.iter_mut() {
        let Some(text) = value.as_str().map(str::trim) else { continue };
        let Some(spec) = props.get(key) else { continue };
        let types = types_of(spec);
        if types.contains(&"string") {
            continue;
        }
        if types.contains(&"integer") || types.contains(&"number") {
            if let Ok(x) = text.parse::<f64>() {
                if x.is_finite() {
                    *value = if types.contains(&"integer") && !types.contains(&"number") && x.fract() == 0.0 { json!(x as i64) } else { json!(x) };
                    continue;
                }
            }
        }
        if types.contains(&"boolean") && (text == "true" || text == "false") {
            *value = json!(text == "true");
        }
    }
}

fn matches(types: &[&str], value: &Value) -> bool {
    types.is_empty()
        || types.iter().any(|t| match *t {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64() || value.as_f64().is_some_and(|x| x.fract() == 0.0),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => true,
        })
}

pub(crate) fn validate(schema: &Value, args: &Value) -> Result<(), CommandError> {
    let bad = |m: String| Err(CommandError::new("bad_args", m));
    let Some(map) = args.as_object() else { return bad("arguments must be a JSON object".into()) };
    let props = schema.get("properties").and_then(Value::as_object);
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            if !map.contains_key(key) {
                return bad(format!("missing required argument {key}"));
            }
        }
    }
    for (key, value) in map {
        let Some(spec) = props.and_then(|p| p.get(key)) else {
            if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
                return bad(format!("unknown argument {key}"));
            }
            continue;
        };
        if !matches(&types_of(spec), value) {
            return bad(format!("{key} has the wrong type"));
        }
        if let Some(options) = spec.get("enum").and_then(Value::as_array) {
            if !options.contains(value) {
                return bad(format!("{key} must be one of {}", options.iter().map(Value::to_string).collect::<Vec<_>>().join(", ")));
            }
        }
    }
    Ok(())
}
