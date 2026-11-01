use ffb_fabric::units::{Scale, FIELDS};
use ffb_fabric::validate::{LOOK_KINDS, MAP_NAME_MAX, MAP_SLOTS, NAME_MAX, OFFSET_RANGE, ROTATION_RANGE, SCALE_RANGE};
use serde_json::{json, Map, Value};

pub fn obj(props: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

pub fn fabric_id(description: &str) -> Value {
    json!({"type": "integer", "minimum": 1, "maximum": 4_294_967_295u64, "description": description})
}

pub fn ids(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "integer", "minimum": 1}, "minItems": 1, "description": description})
}

pub fn name(description: &str) -> Value {
    json!({"type": "string", "minLength": 1, "maxLength": NAME_MAX, "description": description})
}

pub fn path(description: &str) -> Value {
    json!({"type": "string", "minLength": 1, "description": description})
}

pub fn rgba(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "number", "minimum": 0, "maximum": 1}, "minItems": 3, "maxItems": 4, "description": description})
}

pub fn physical(partial: bool) -> Value {
    let mut props = Map::new();
    for f in FIELDS.iter() {
        let kind = if f.scale == Scale::Log { "logarithmic slider" } else { "linear slider" };
        props.insert(
            f.key.into(),
            json!({"type": "number", "unit": f.unit, "minimum": f.min, "maximum": f.max, "description": format!("{} ({}; {} mapping)", f.description, f.unit, kind)}),
        );
    }
    let description = if partial { "Physical properties to change; fields left out keep their value. SI based units." } else { "Physical properties in SI based units." };
    let mut v = obj(Value::Object(props), &[]);
    v["description"] = json!(description);
    v
}

fn pair(unit: &str, lo: f64, hi: f64, description: &str) -> Value {
    json!({"type": "array", "items": {"type": "number", "minimum": lo, "maximum": hi}, "minItems": 2, "maxItems": 2, "unit": unit, "description": description})
}

pub fn surface() -> Value {
    let unit = |description: &str| json!({"type": "number", "minimum": 0, "maximum": 1, "description": description});
    let maps: Map<String, Value> = MAP_SLOTS.iter().map(|s| (s.to_string(), json!({"type": ["string", "null"], "maxLength": MAP_NAME_MAX, "description": format!("Image for the {} map, or null for none.", s.replace('_', " "))}))).collect();
    obj(
        json!({
            "color": rgba("Base colour as sRGB [r, g, b, a], each 0 to 1."),
            "roughness": unit("Surface roughness, 0 mirror to 1 fully rough."),
            "metalness": unit("Metalness, 0 dielectric to 1 metal."),
            "sheen": unit("Soft fibre sheen at grazing angles, 0 to 1."),
            "opacity": unit("Opacity, 0 clear to 1 solid."),
            "maps": obj(Value::Object(maps), &[]),
            "map_scale": pair("mm", SCALE_RANGE[0], SCALE_RANGE[1], "Size of one map repeat on the cloth, [width, height] in millimetres."),
            "map_rotation": {"type": "number", "unit": "deg", "minimum": -ROTATION_RANGE, "maximum": ROTATION_RANGE, "description": "Map rotation in degrees."},
            "map_offset": pair("mm", -OFFSET_RANGE, OFFSET_RANGE, "Map offset [x, y] in millimetres.")
        }),
        &[],
    )
}

pub fn look() -> Value {
    let mut back = surface();
    back["type"] = json!(["object", "null"]);
    back["description"] = json!("Back surface fields to change; null returns to using the front surface.");
    let mut side = back.clone();
    side["description"] = json!("Side surface fields to change; null returns to using the front surface.");
    let mut front = surface();
    front["description"] = json!("Front surface fields to change.");
    let mut v = obj(json!({"kind": {"type": "string", "enum": LOOK_KINDS, "description": "Shading model of the cloth."}, "front": front, "back": back, "side": side}), &[]);
    v["description"] = json!("Look fields to change; fields left out keep their value.");
    v
}
