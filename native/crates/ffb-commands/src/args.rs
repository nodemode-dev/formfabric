use serde_json::{json, Value};

use crate::CommandError;

pub(crate) fn string<'a>(args: &'a Value, key: &str) -> Result<&'a str, CommandError> {
    args.get(key).and_then(Value::as_str).ok_or_else(|| CommandError::new("bad_args", format!("{key} must be a string")))
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
