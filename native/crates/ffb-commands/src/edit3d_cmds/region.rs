use std::collections::HashSet;

use ffb_pattern::Pin;
use serde_json::{json, Value};

use super::common::{bad, id_arg, missing, numbers, panel_index};
use crate::sim_cmds::{current_positions, MAX_PINS};
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

type P3 = [f64; 3];

#[derive(Clone)]
enum Region {
    Box { min: P3, max: P3 },
    Lasso { polygon: Vec<[f64; 2]>, u: P3, v: P3 },
    Listed(HashSet<usize>),
}

const MAX_LISTED: usize = 10_000;
const MAX_PANELS: usize = 256;

fn pin_key(panel: u32, point: [f64; 2]) -> (u32, [u64; 2]) {
    (panel, [(point[0] + 0.0).to_bits(), (point[1] + 0.0).to_bits()])
}

fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: P3, b: P3) -> P3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: P3) -> Option<P3> {
    let l = dot(a, a).sqrt();
    (l > 1e-9).then(|| [a[0] / l, a[1] / l, a[2] / l])
}

fn inside(polygon: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut hit = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let (a, b) = (polygon[i], polygon[j]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            hit = !hit;
        }
        j = i;
    }
    hit
}

fn contains(region: &Region, index: usize, p: [f32; 3]) -> bool {
    let q = [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])];
    if !q.iter().all(|x| x.is_finite()) {
        return false;
    }
    match region {
        Region::Box { min, max } => (0..3).all(|k| q[k] >= min[k] && q[k] <= max[k]),
        Region::Lasso { polygon, u, v } => inside(polygon, [dot(q, *u), dot(q, *v)]),
        Region::Listed(list) => list.contains(&index),
    }
}

fn region(a: &Value, kind: &str) -> Result<Region, CommandError> {
    if let Some(list) = a.get("vertices").filter(|v| !v.is_null()) {
        let list = list.as_array().filter(|l| l.len() <= MAX_LISTED).ok_or_else(|| bad(format!("vertices must be a list of at most {MAX_LISTED} mesh vertex indices")))?;
        return list.iter().map(|x| x.as_u64().and_then(|x| usize::try_from(x).ok()).ok_or_else(|| bad("vertices must be whole numbers"))).collect::<Result<HashSet<_>, _>>().map(Region::Listed);
    }
    if kind == "box" {
        let (Some(min), Some(max)) = (numbers::<3>(a, "min")?, numbers::<3>(a, "max")?) else { return Err(bad("give vertices, or min and max")) };
        return Ok(Region::Box { min: [min[0].min(max[0]), min[1].min(max[1]), min[2].min(max[2])], max: [min[0].max(max[0]), min[1].max(max[1]), min[2].max(max[2])] });
    }
    let dir = numbers::<3>(a, "direction")?.and_then(unit).ok_or_else(|| bad("give vertices, or polygon and direction"))?;
    let points = a.get("polygon").and_then(Value::as_array).filter(|l| (3..=MAX_LISTED).contains(&l.len())).ok_or_else(|| bad(format!("polygon must hold 3 to {MAX_LISTED} points")))?;
    let helper = if dir[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let u = unit(cross(dir, helper)).ok_or_else(|| bad("direction is degenerate"))?;
    let v = cross(dir, u);
    let mut polygon = Vec::new();
    for item in points {
        let list = item.as_array().filter(|l| l.len() == 3).ok_or_else(|| bad("polygon points must hold 3 numbers"))?;
        let mut p = [0.0; 3];
        for (slot, x) in p.iter_mut().zip(list) {
            *slot = x.as_f64().filter(|x| x.is_finite()).ok_or_else(|| bad("polygon points must be finite"))?;
        }
        polygon.push([dot(p, u), dot(p, v)]);
    }
    Ok(Region::Lasso { polygon, u, v })
}

fn schema(kind: &str) -> Value {
    let mut props = json!({
        "panels": {"type": "array", "minItems": 1, "maxItems": MAX_PANELS, "items": {"type": "object", "properties": {"panel": {"type": "integer", "minimum": 1}, "vertices": {"type": "array", "maxItems": MAX_LISTED, "items": {"type": "integer", "minimum": 0}}}, "required": ["panel", "vertices"]}, "description": "Mesh vertices picked in the viewer for several panels at once, recorded as one undo step. Replaces panel, vertices and the 3D region."},
        "panel": {"type": "integer", "minimum": 1, "description": "Limit to this panel. Needed with vertices."},
        "vertices": {"type": "array", "maxItems": MAX_LISTED, "items": {"type": "integer", "minimum": 0}, "description": "Mesh vertex indices picked by the viewer (of the given panel). Replaces the 3D region."}
    });
    let extra = if kind == "box" {
        json!({
            "min": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "One corner of the 3D box."},
            "max": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "The opposite corner."}
        })
    } else {
        json!({
            "polygon": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3}, "minItems": 3, "maxItems": MAX_LISTED, "description": "Lasso outline in 3D, drawn on the view plane."},
            "direction": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "description": "View direction the outline is extruded along."}
        })
    };
    if let (Some(p), Some(e)) = (props.as_object_mut(), extra.as_object()) {
        p.extend(e.clone());
    }
    json!({"type": "object", "properties": props, "required": [], "additionalProperties": false})
}

fn make(s: &mut Session, a: &Value, kind: &str) -> Result<Value, CommandError> {
    let jobs: Vec<(u32, Region)> = if let Some(list) = a.get("panels").filter(|v| !v.is_null()) {
        let list = list.as_array().filter(|l| !l.is_empty() && l.len() <= MAX_PANELS).ok_or_else(|| bad(format!("panels must be a list of 1 to {MAX_PANELS} {{panel, vertices}}")))?;
        let mut jobs = Vec::new();
        for item in list {
            let id = id_arg(item, "panel")?;
            panel_index(s, id)?;
            jobs.push((id, region(item, kind)?));
        }
        jobs
    } else {
        let region = region(a, kind)?;
        let only = match a.get("panel").filter(|v| !v.is_null()) {
            Some(_) => Some(id_arg(a, "panel")?),
            None => None,
        };
        if matches!(region, Region::Listed(_)) && only.is_none() {
            return Err(bad("vertices needs panel"));
        }
        let ids: Vec<u32> = match only {
            Some(id) => {
                panel_index(s, id)?;
                vec![id]
            }
            None => s.garment.panels.iter().filter(|p| !p.state.disabled).map(|p| p.id).collect(),
        };
        ids.into_iter().map(|id| (id, region.clone())).collect()
    };
    let mut pins = s.garment.pins.clone();
    let mut next = pins.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    let (mut added, mut skipped) = (0usize, 0usize);
    let mut keys: HashSet<(u32, [u64; 2])> = pins.iter().map(|q| pin_key(q.panel, q.point)).collect();
    for (id, region) in jobs {
        let index = panel_index(s, id)?;
        let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
        let flat2 = built.mesh.positions.clone();
        let placed = current_positions(s, id)?;
        for (i, p) in placed.iter().enumerate() {
            if !contains(&region, i, *p) {
                continue;
            }
            if !keys.insert(pin_key(id, flat2[i])) {
                skipped += 1;
                continue;
            }
            if pins.len() >= MAX_PINS {
                return Err(bad(format!("a garment holds at most {MAX_PINS} pins")));
            }
            pins.push(Pin { id: next, panel: id, point: flat2[i], target: Some([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]) });
            next += 1;
            added += 1;
        }
    }
    if added == 0 {
        return Err(missing("no vertex lies in the region"));
    }
    let label = if kind == "box" { "Pin box" } else { "Pin lasso" };
    let before = std::mem::replace(&mut s.garment.pins, pins.clone());
    s.record(Edit::Pins { label: label.into(), before, after: pins });
    Ok(json!({"added": added, "skipped": skipped, "count": s.garment.pins.len()}))
}

fn pin_box(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    make(s, a, "box")
}

fn pin_lasso(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    make(s, a, "lasso")
}

fn box_schema() -> Value {
    schema("box")
}

fn lasso_schema() -> Value {
    schema("lasso")
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command { id: "pin.box", mcp: "pin_box", group: Group::Sim, label: "Pin (Box)", description: "Pin every vertex inside a 3D box, or a list of mesh vertices picked in the viewer, at its current position. One pin per vertex, all alike; one undo step.", keys: &["Shift+X"], hints: WRITE, schema: box_schema, run: pin_box },
        Command { id: "pin.lasso", mcp: "pin_lasso", group: Group::Sim, label: "Pin (Lasso)", description: "Pin every vertex whose projection along the view direction falls inside a lasso outline, or a list of mesh vertices picked in the viewer, at its current position. One pin per vertex, all alike; one undo step.", keys: &["Shift+L"], hints: WRITE, schema: lasso_schema, run: pin_lasso },
    ]
}
