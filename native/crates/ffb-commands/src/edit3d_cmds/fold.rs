use ffb_mesh::layout;
use ffb_pattern::Place3D;
use serde_json::{json, Value};

use super::common::{bad, id_arg, panel_index, ranged};
use crate::sim_cmds::rigid_fit;
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

type V = [f64; 3];

fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1},
            "line": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Fold line as two panel-local points; it extends across the whole panel."},
            "angle": {"type": "number", "minimum": -180, "maximum": 180, "unit": "deg", "description": "Fold angle. Positive turns the folded part toward the front of the panel; 180 lays it flat over the rest."},
            "side": {"type": "string", "enum": ["left", "right"], "default": "left", "description": "Which part moves, looking along the line from its first to its second point in the panel plane."}
        },
        "required": ["panel", "line", "angle"],
        "additionalProperties": false
    })
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "panel")?;
    let index = panel_index(s, id)?;
    let angle = ranged(a, "angle", -180.0, 180.0)?.ok_or_else(|| bad("angle is required"))?.to_radians();
    let left = match a.get("side").and_then(Value::as_str).unwrap_or("left") {
        "left" => true,
        "right" => false,
        _ => return Err(bad("side must be left or right")),
    };
    let ends = a.get("line").and_then(Value::as_array).filter(|l| l.len() == 2).ok_or_else(|| bad("line must hold 2 points"))?;
    let mut pts = [[0.0; 2]; 2];
    for (slot, e) in pts.iter_mut().zip(ends) {
        let l = e.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad("line points must hold 2 numbers"))?;
        for (c, x) in slot.iter_mut().zip(l) {
            *c = x.as_f64().filter(|x| x.is_finite()).ok_or_else(|| bad("line points must be finite"))?;
        }
    }
    let len2 = ((pts[1][0] - pts[0][0]).powi(2) + (pts[1][1] - pts[0][1]).powi(2)).sqrt();
    if len2 < 1e-6 {
        return Err(bad("the fold line needs two different points"));
    }
    let dir2 = [(pts[1][0] - pts[0][0]) / len2, (pts[1][1] - pts[0][1]) / len2];
    let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
    let m = &built.mesh;
    let before = s.garment.panels[index].clone();
    let n = m.positions.len();
    let unfolded: Vec<[f32; 3]> = match before.place3d.as_ref().and_then(|p| p.pose.as_ref()).filter(|p| p.len() == n) {
        Some(pose) => pose.clone(),
        None => layout::positions_3d(m, &before).chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
    };
    let world = |i: usize| -> V { unfolded[i].map(f64::from) };
    let signed: Vec<f64> = m.positions.iter().map(|p| dir2[0] * (p[1] - pts[0][1]) - dir2[1] * (p[0] - pts[0][0])).collect();
    let mut hits: Vec<(f64, V)> = Vec::new();
    for t in &m.triangles {
        for k in 0..3 {
            let (i, j) = (t[k] as usize, t[(k + 1) % 3] as usize);
            if (signed[i] >= 0.0) != (signed[j] >= 0.0) {
                let f = signed[i] / (signed[i] - signed[j]);
                let along = (m.positions[i][0] + f * (m.positions[j][0] - m.positions[i][0]) - pts[0][0]) * dir2[0] + (m.positions[i][1] + f * (m.positions[j][1] - m.positions[i][1]) - pts[0][1]) * dir2[1];
                let (p, q) = (world(i), world(j));
                hits.push((along, [p[0] + f * (q[0] - p[0]), p[1] + f * (q[1] - p[1]), p[2] + f * (q[2] - p[2])]));
            }
        }
    }
    let first = hits.iter().min_by(|x, y| x.0.total_cmp(&y.0)).ok_or_else(|| bad("the fold line does not cross the panel"))?.1;
    let last = hits.iter().max_by(|x, y| x.0.total_cmp(&y.0)).map(|h| h.1).unwrap_or(first);
    let axis_raw = sub(last, first);
    let axis_len = dot(axis_raw, axis_raw).sqrt();
    if axis_len < 1e-6 {
        return Err(bad("the fold line does not cross the panel"));
    }
    let axis = [axis_raw[0] / axis_len, axis_raw[1] / axis_len, axis_raw[2] / axis_len];
    let (sin, cos) = angle.sin_cos();
    let mut folded = unfolded.clone();
    let mut moved = 0usize;
    for (i, out) in folded.iter_mut().enumerate() {
        let on_side = if left { signed[i] > 1e-9 } else { signed[i] < -1e-9 };
        if !on_side {
            continue;
        }
        let r = sub(world(i), first);
        let c = cross(axis, r);
        let d = dot(axis, r);
        for k in 0..3 {
            out[k] = (first[k] + r[k] * cos + c[k] * sin + axis[k] * d * (1.0 - cos)) as f32;
        }
        moved += 1;
    }
    if moved == 0 {
        return Err(bad("no vertex lies on that side of the line"));
    }
    let mut after = before.clone();
    match after.place3d.as_mut() {
        Some(p) => p.pose = Some(folded),
        None => {
            if layout::mirrored_flat(&before) {
                return Err(bad("place the panel in 3D first (a mirrored flat panel cannot hold a pose)"));
            }
            let rest: Vec<[f32; 2]> = m.positions.iter().map(|p| [p[0] as f32, p[1] as f32]).collect();
            let (origin, rotation) = rigid_fit(&rest, &unfolded, &m.triangles).ok_or_else(|| bad("the panel shape could not be placed"))?;
            after.place3d = Some(Place3D { origin, rotation, bend: 0.0, pose: Some(folded) });
        }
    }
    s.garment.panels[index] = after.clone();
    s.record(Edit::Panel { label: "Fold place".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
    Ok(json!({"panel": id, "moved": moved, "axis": [first, last], "angle": a.get("angle")}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "fold.place",
        mcp: "fold_place",
        group: Group::Sim,
        label: "Fold place",
        description: "Fold a panel before draping: the part of the panel on one side of a fold line is rotated about that line in the placed shape, by an angle. The folded shape is saved as the panel's pose and the next simulation starts from it. One undo step.",
        keys: &["Shift+D"],
        hints: WRITE,
        schema,
        run,
    }]
}
