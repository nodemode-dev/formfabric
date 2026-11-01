use serde_json::{json, Value};

use super::common::{bad, numbers, ranged};
use crate::{Command, CommandError, Edit, Group, Session, SET};

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "enabled": {"type": "boolean"},
            "kind": {"type": "string", "enum": ["directional", "radial"], "description": "directional blows along direction; radial blows away from origin."},
            "origin": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "Where the wind starts. Decay is measured from here."},
            "direction": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "description": "Wind direction (directional)."},
            "strength": {"type": "number", "minimum": 0, "maximum": 100, "unit": "m/s", "description": "Wind speed at the origin."},
            "decay": {"type": "number", "minimum": 0, "maximum": 100, "description": "Weakening with distance from the origin. 0 is none; 100 drops the speed to 37 percent per metre."},
            "turbulence": {"type": "number", "minimum": 0, "maximum": 100, "description": "Percent of irregularity in speed and direction (deterministic noise in space and time)."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let before = s.garment.clone();
    let mut wind = before.wind.clone();
    if let Some(v) = a.get("enabled").filter(|v| !v.is_null()) {
        wind.enabled = v.as_bool().ok_or_else(|| bad("enabled must be true or false"))?;
    }
    if let Some(v) = a.get("kind").filter(|v| !v.is_null()) {
        let kind = v.as_str().filter(|k| ["directional", "radial"].contains(k)).ok_or_else(|| bad("kind must be directional or radial"))?;
        wind.kind = kind.into();
    }
    if let Some(v) = numbers::<3>(a, "origin")? {
        wind.origin = v;
    }
    if let Some(v) = numbers::<3>(a, "direction")? {
        if v.iter().all(|x| x.abs() < 1e-9) {
            return Err(bad("direction must not be zero"));
        }
        wind.direction = v;
    }
    if let Some(v) = ranged(a, "strength", 0.0, 100.0)? {
        wind.strength = v;
    }
    if let Some(v) = ranged(a, "decay", 0.0, 100.0)? {
        wind.decay = v;
    }
    if let Some(v) = ranged(a, "turbulence", 0.0, 100.0)? {
        wind.turbulence = v;
    }
    if wind.kind.is_empty() {
        wind.kind = "directional".into();
    }
    let changed = wind != before.wind;
    if changed {
        let mut after = before.clone();
        after.wind = wind.clone();
        s.garment = after.clone();
        s.record(Edit::Garment { label: "Wind".into(), before: Box::new(before), after: Box::new(after) });
    }
    Ok(json!({
        "changed": changed,
        "wind": {"enabled": wind.enabled, "kind": wind.kind, "origin": wind.origin, "direction": wind.direction, "strength": wind.strength, "decay": wind.decay, "turbulence": wind.turbulence}
    }))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "wind.set",
        mcp: "wind_set",
        group: Group::Sim,
        label: "Wind",
        description: "Set the wind: on or off, kind (directional or radial), origin, direction, strength in m/s, decay with distance from the origin, turbulence. The simulation applies it as a force on every triangle (pressure on the face, so it also needs the face to be turned toward the wind). One undo step.",
        keys: &[],
        hints: SET,
        schema,
        run,
    }]
}
