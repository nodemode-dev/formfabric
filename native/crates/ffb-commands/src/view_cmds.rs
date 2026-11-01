use ffb_core::Units;
use serde_json::{json, Value};

use crate::{Command, CommandError, Edit, Group, Session, SET};

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {"units": {"type": "string", "enum": ["mm", "cm", "in"], "description": "Length unit to switch to. Leave out to read the current unit."}},
        "required": [],
        "additionalProperties": false
    })
}

fn report(units: Units, changed: bool) -> Value {
    json!({"units": units.name(), "mm_per_unit": units.mm_per_unit(), "available": ["mm", "cm", "in"], "changed": changed})
}

fn units(s: &mut Session, args: &Value) -> Result<Value, CommandError> {
    let Some(text) = args.get("units").and_then(Value::as_str) else {
        return Ok(report(s.garment.units, false));
    };
    let to = Units::parse(text).ok_or_else(|| CommandError::new("bad_args", format!("units must be mm, cm or in, not {text}")))?;
    let from = s.garment.units;
    if from == to {
        return Ok(report(to, false));
    }
    s.garment.units = to;
    s.record(Edit::Units { from, to });
    Ok(report(to, true))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "view.units",
        mcp: "view_units",
        group: Group::View,
        label: "Units",
        description: "Get or set the display length unit (mm, cm or in). Geometry is always stored in millimetres; only the unit shown and typed changes. Setting it is an undo step.",
        keys: &[],
        hints: SET,
        schema,
        run: units,
    }]
}
