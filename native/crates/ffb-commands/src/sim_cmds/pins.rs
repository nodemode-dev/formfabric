use ffb_pattern::Pin;
use serde_json::{json, Value};

use super::assemble::nearest_vertex;
use super::build::ensure;
use super::runtime::lock;
use crate::{Command, CommandError, Edit, Group, Session, SET, WRITE};

pub(crate) const MAX_PINS: usize = 10_000;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
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

fn id_arg(a: &Value, key: &str) -> Result<u32, CommandError> {
    a.get(key).and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad(format!("{key} must be a positive integer id")))
}

fn pin_json(p: &Pin) -> Value {
    json!({"id": p.id, "panel": p.panel, "point": p.point, "target": p.target})
}

fn record(s: &mut Session, label: &str, edit: impl FnOnce(&mut Vec<Pin>) -> Result<(), CommandError>) -> Result<bool, CommandError> {
    let mut pins = s.garment.pins.clone();
    edit(&mut pins)?;
    if pins == s.garment.pins {
        return Ok(false);
    }
    let before = std::mem::replace(&mut s.garment.pins, pins.clone());
    s.record(Edit::Pins { label: label.into(), before, after: pins });
    Ok(true)
}

fn add_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id as listed by pattern.get."},
            "point": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Panel-local point; the nearest mesh vertex is pinned. Give point or vertex."},
            "vertex": {"type": "integer", "minimum": 0, "description": "Vertex index in the panel mesh as streamed for that panel."},
            "target": {"type": ["array", "null"], "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "Where the pinned vertex is held in 3D. Leave out to hold it where it is now (the simulated position when a simulation exists, else the placed position)."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel_id = id_arg(a, "panel")?;
    let index = s.garment.index_of(panel_id).ok_or_else(|| missing(format!("no panel with id {panel_id}")))?;
    let mesh = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
    let (point, local) = match (numbers::<2>(a, "point")?, a.get("vertex").and_then(Value::as_u64)) {
        (Some(p), None) => (p, nearest_vertex(&mesh.mesh, p).ok_or_else(|| missing("the panel mesh is empty"))?),
        (None, Some(v)) => {
            let v = v as usize;
            (*mesh.mesh.positions.get(v).ok_or_else(|| bad(format!("the panel mesh has {} vertices", mesh.mesh.positions.len())))?, v)
        }
        _ => return Err(bad("give exactly one of point and vertex")),
    };
    let target = match numbers::<3>(a, "target")? {
        Some(t) => Some(t),
        None => {
            let c = lock(&s.sim.core);
            let found = c.slices.iter().find(|sl| sl.id == panel_id && sl.count == mesh.mesh.positions.len());
            match (c.stepper.as_ref(), found) {
                (Some(st), Some(sl)) => st.positions().get(sl.start + local).map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]),
                _ => None,
            }
        }
    };
    if let Some(existing) = s.garment.pins.iter().find(|p| p.panel == panel_id && nearest_vertex(&mesh.mesh, p.point) == Some(local)) {
        return Ok(json!({"pin": pin_json(existing), "vertex": local, "count": s.garment.pins.len(), "changed": false}));
    }
    if s.garment.pins.len() >= MAX_PINS {
        return Err(bad(format!("a garment holds at most {MAX_PINS} pins")));
    }
    let mut created = 0;
    record(s, "Add pin", |pins| {
        created = pins.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        pins.push(Pin { id: created, panel: panel_id, point, target });
        Ok(())
    })?;
    Ok(json!({"pin": {"id": created, "panel": panel_id, "point": point, "target": target}, "vertex": local, "count": s.garment.pins.len(), "changed": true}))
}

fn remove_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"id": {"type": "integer", "minimum": 1, "description": "Pin id as returned by pin.add."}},
        "required": ["id"],
        "additionalProperties": false
    })
}

fn remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    if !s.garment.pins.iter().any(|p| p.id == id) {
        return Err(missing(format!("no pin with id {id}")));
    }
    record(s, "Remove pin", |pins| {
        pins.retain(|p| p.id != id);
        Ok(())
    })?;
    Ok(json!({"removed": id, "count": s.garment.pins.len(), "changed": true}))
}

fn clear_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"panel": {"type": "integer", "minimum": 1, "description": "Only clear the pins of this panel. Leave out to clear every pin."}},
        "required": [],
        "additionalProperties": false
    })
}

fn clear(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel = match a.get("panel").filter(|v| !v.is_null()) {
        Some(_) => Some(id_arg(a, "panel")?),
        None => None,
    };
    let removed = s.garment.pins.iter().filter(|p| panel.is_none_or(|id| p.panel == id)).count();
    let changed = record(s, "Clear pins", |pins| {
        pins.retain(|p| panel.is_some_and(|id| p.panel != id));
        Ok(())
    })?;
    Ok(json!({"removed": removed, "count": s.garment.pins.len(), "changed": changed, "pins": s.garment.pins.iter().map(pin_json).collect::<Vec<_>>()}))
}

fn grab_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id. With vertex, the vertex index is in that panel's mesh; with point, the nearest vertex is taken."},
            "vertex": {"type": "integer", "minimum": 0, "description": "Vertex index in the panel mesh when panel is given, else an index into the whole cloth."},
            "point": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Panel-local point (needs panel)."},
            "target": {"type": ["array", "null"], "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "Where to hold the grabbed vertex in 3D. Null or missing releases the grab. Send again to drag."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn grab(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let target = numbers::<3>(a, "target")?;
    let panel = match a.get("panel").filter(|v| !v.is_null()) {
        Some(_) => Some(id_arg(a, "panel")?),
        None => None,
    };
    if target.is_some() {
        ensure(s)?;
    }
    let wanted: Option<usize> = match (a.get("vertex").and_then(Value::as_u64), numbers::<2>(a, "point")?) {
        (Some(v), None) => Some(v as usize),
        (None, Some(p)) => {
            let id = panel.ok_or_else(|| bad("point needs panel"))?;
            let index = s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))?;
            let mesh = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
            Some(nearest_vertex(&mesh.mesh, p).ok_or_else(|| missing("the panel mesh is empty"))?)
        }
        (None, None) => None,
        _ => return Err(bad("give at most one of vertex and point")),
    };
    let mut c = lock(&s.sim.core);
    c.sink = s.sink.clone();
    let global = match (wanted, panel) {
        (Some(v), Some(id)) => {
            let slice = c.slices.iter().find(|sl| sl.id == id).ok_or_else(|| missing(format!("panel {id} is not in the simulation")))?;
            if v >= slice.count {
                return Err(bad(format!("panel {id} has {} vertices", slice.count)));
            }
            Some(slice.start + v)
        }
        (Some(v), None) => Some(v),
        (None, _) => c.grabbed.map(|g| g as usize),
    };
    let Some(vertex) = global else {
        return Ok(json!({"grabbed": false, "vertex": null}));
    };
    let total = c.base.positions.len();
    if vertex >= total {
        return Err(bad(format!("the cloth has {total} vertices")));
    }
    let Some(stepper) = c.stepper.as_mut() else {
        return Ok(json!({"grabbed": false, "vertex": vertex}));
    };
    stepper.grab(vertex as u32, target.map(|t| [t[0] as f32, t[1] as f32, t[2] as f32]));
    c.grabbed = target.map(|_| vertex as u32);
    Ok(json!({"grabbed": target.is_some(), "vertex": vertex}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command { id: "pin.add", mcp: "pin_add", group: Group::Sim, label: "Add pin", description: "Pin a mesh vertex of a panel in 3D. Pins are part of the document and an undo step; they hold the vertex where it is (or at target) during simulation.", keys: &[], hints: WRITE, schema: add_schema, run: add },
        Command { id: "pin.remove", mcp: "pin_remove", group: Group::Sim, label: "Remove pin", description: "Remove one pin by id.", keys: &[], hints: WRITE, schema: remove_schema, run: remove },
        Command { id: "pin.clear", mcp: "pin_clear", group: Group::Sim, label: "Clear pins", description: "Remove every pin, or those of one panel.", keys: &[], hints: WRITE, schema: clear_schema, run: clear },
        Command { id: "sim.grab", mcp: "sim_grab", group: Group::Sim, label: "Grab", description: "Hold a vertex at a 3D target while the simulation runs, as the mouse drag does. A null or missing target releases the grab. Not saved and not an undo step.", keys: &[], hints: SET, schema: grab_schema, run: grab },
    ]
}
