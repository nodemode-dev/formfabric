use ffb_mesh::layout::quaternion;
use ffb_pattern::Place3D;
use serde_json::{json, Value};

use crate::{Command, CommandError, Edit, Group, Session, SET, WRITE};

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn numbers<const N: usize>(a: &Value, key: &str) -> Result<Option<[f64; N]>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() == N).ok_or_else(|| bad(format!("{key} must hold {N} numbers")))?;
    let mut out = [0.0; N];
    for (slot, item) in out.iter_mut().zip(list) {
        *slot = item.as_f64().filter(|x| x.is_finite() && x.abs() <= 1e7).ok_or_else(|| bad(format!("{key} must hold {N} finite numbers")))?;
    }
    Ok(Some(out))
}

fn panel_id(a: &Value) -> Result<u32, CommandError> {
    a.get("panel").and_then(Value::as_f64).filter(|x| x.fract() == 0.0 && *x >= 1.0).map(|x| x as u32).ok_or_else(|| bad("panel must be a panel id"))
}

fn place_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id as listed by pattern.get."},
            "origin": {"type": "array", "items": {"type": "number", "minimum": -10_000_000, "maximum": 10_000_000}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "Where the panel-local origin goes in 3D, x y z in mm, y up."},
            "rotation": {"type": "array", "items": {"type": "number"}, "minItems": 4, "maxItems": 4, "default": [0, 0, 0, 1], "description": "Quaternion x y z w applied to the panel-local points (x, y, 0). It is normalised; identity keeps the panel facing +z."},
            "bend": {"type": "number", "minimum": -1000, "maximum": 1000, "default": 0, "description": "Pre-bend curvature stored with the placement for the solver."}
        },
        "required": ["panel", "origin"],
        "additionalProperties": false
    })
}

fn place(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = panel_id(a)?;
    let index = s.garment.index_of(id).ok_or_else(|| CommandError::new("not_found", format!("no panel with id {id}")))?;
    let origin = numbers::<3>(a, "origin")?.ok_or_else(|| bad("origin is required"))?;
    let rotation = match numbers::<4>(a, "rotation")? {
        Some(q) if q.iter().all(|x| *x == 0.0) => return Err(CommandError::new("bad_args", "rotation must not be the zero quaternion")),
        Some(q) => quaternion(q),
        None => [0.0, 0.0, 0.0, 1.0],
    };
    let bend = a.get("bend").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(0.0);
    let before = s.garment.panels[index].clone();
    let mut after = before.clone();
    after.place3d = Some(Place3D { origin, rotation, bend, pose: None });
    let changed = after != before;
    let place3d = after.place3d.clone();
    if changed {
        s.garment.panels[index] = after.clone();
        s.record(Edit::Panel { label: "Arrange panel".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
    }
    Ok(json!({"panel": id, "place3d": place3d, "changed": changed}))
}

fn reset_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Panel ids to return to the default flat layout. Leave out for every panel."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn reset(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let wanted: Option<Vec<u32>> = match a.get("panels").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => {
            let list = v.as_array().ok_or_else(|| bad("panels must be a list of panel ids"))?;
            let mut ids = Vec::with_capacity(list.len());
            for item in list {
                let id = item.as_f64().filter(|x| x.fract() == 0.0 && *x >= 1.0).ok_or_else(|| bad("panels must hold whole panel ids"))? as u32;
                if s.garment.index_of(id).is_none() {
                    return Err(CommandError::new("not_found", format!("no panel with id {id}")));
                }
                ids.push(id);
            }
            Some(ids)
        }
    };
    let mut edits = Vec::new();
    let mut cleared = Vec::new();
    for index in 0..s.garment.panels.len() {
        let panel = &s.garment.panels[index];
        if panel.place3d.is_none() || wanted.as_ref().is_some_and(|w| !w.contains(&panel.id)) {
            continue;
        }
        let before = panel.clone();
        let mut after = before.clone();
        after.place3d = None;
        cleared.push(after.id);
        s.garment.panels[index] = after.clone();
        edits.push(Edit::Panel { label: "Reset arrangement".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
    }
    let changed = !edits.is_empty();
    match edits.len() {
        0 => {}
        1 => s.record(edits.remove(0)),
        _ => s.record(Edit::Many { label: "Reset arrangement".into(), edits }),
    }
    Ok(json!({"reset": cleared, "changed": changed}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "arrange.place",
            mcp: "arrange_place",
            group: Group::Sim,
            label: "Place Panel in 3D",
            description: "Set where a panel sits in 3D: an origin in mm and a quaternion rotation applied to the panel-local points (x, y, 0). Panels without a placement use the default flat layout, x and y from the pattern plane and z 0. One undo step; the stream positions for that panel update.",
            keys: &[],
            hints: SET,
            schema: place_schema,
            run: place,
        },
        Command {
            id: "arrange.reset",
            mcp: "arrange_reset",
            group: Group::Sim,
            label: "Reset Arrangement",
            description: "Clear the 3D placement of the given panels, or of all panels, so they return to the default flat layout. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: reset_schema,
            run: reset,
        },
    ]
}
