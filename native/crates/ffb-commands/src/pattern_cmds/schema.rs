use serde_json::{json, Map, Value};

pub const COORD: f64 = 100_000.0;
pub const SIZE: f64 = 20_000.0;

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

pub fn coord(description: &str, default: Option<f64>) -> Value {
    mm(description, -COORD, COORD, default)
}

pub fn size(description: &str) -> Value {
    mm(description, 0.1, SIZE, None)
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

pub fn text(description: &str, options: &[&str]) -> Value {
    if options.is_empty() {
        json!({"type": "string", "description": description})
    } else {
        json!({"type": "string", "enum": options, "description": description})
    }
}

pub fn flag(description: &str, default: bool) -> Value {
    json!({"type": "boolean", "default": default, "description": description})
}

pub fn panel() -> Value {
    int("Panel id as listed by pattern.get.", 1.0, 4_294_967_295.0, None)
}

pub fn inner() -> Value {
    int("Inner shape id inside the panel. Leave out to edit the panel outline.", 1.0, 4_294_967_295.0, None)
}

pub fn index(description: &str) -> Value {
    int(description, 0.0, 100_000.0, None)
}

pub fn ids(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "integer", "minimum": 1}, "minItems": 1, "description": description})
}

pub fn indices(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "integer", "minimum": 0}, "minItems": 1, "description": description})
}

pub fn space() -> Value {
    json!({
        "type": "string",
        "enum": ["world", "local"],
        "default": "world",
        "description": "world: coordinates on the pattern plane the user sees. local: coordinates inside the panel before its position, rotation and mirror apply."
    })
}

pub fn points(description: &str, min: usize) -> Value {
    json!({
        "type": "array",
        "minItems": min,
        "items": {"oneOf": [
            {"type": "array", "items": {"type": "number", "minimum": -COORD, "maximum": COORD}, "minItems": 2, "maxItems": 2},
            {"type": "object", "properties": {
                "p": xy("Point position."),
                "kind": text("corner: sharp vertex. curve: handles are computed from the neighbours. smooth: handles are kept collinear.", &["corner", "curve", "smooth"]),
                "h_in": xy("Incoming handle as an offset from the point."),
                "h_out": xy("Outgoing handle as an offset from the point.")
            }, "required": ["p"]}
        ]},
        "unit": "mm",
        "description": description
    })
}

pub fn name(description: &str) -> Value {
    json!({"type": "string", "minLength": 1, "maxLength": 64, "description": description})
}

pub fn fold_fields(props: &mut Value) {
    props["fold_angle"] = deg("Fold angle of the inner line: 180 is flat, lower folds to the front, higher to the back.", 0.0, 360.0, Some(180.0));
    props["fold_strength"] = ratio("How firmly the fold holds its angle, 0 for none.", 0.0, 20.0, Some(0.0));
}
