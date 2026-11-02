use ffb_pattern::StyleLine;
use serde_json::{json, Value};

use super::common::{bad, id_arg, id_list, missing, panel_index, ranged};
use super::fx::{self, LineFx, Mode};
use super::geo::{self, Locator};
use crate::sim_cmds::current_positions;
use crate::{Command, CommandError, Edit, Group, Session, READ, SET, WRITE};

const MAX_POINTS: usize = 2000;
const MAX_SAMPLES: usize = 4000;
const MAX_LINES: usize = 256;

fn fx_props() -> Value {
    json!({
        "mode": {"type": "string", "enum": ["none", "stiffen", "fold"], "description": "What the line does in the simulation. none draws only; stiffen multiplies stretch, shear and bending in a band along the line; fold adds a crease along the mesh edges that follow the line."},
        "stiffen": {"type": "number", "minimum": 1, "maximum": 1000, "description": "Stiffness multiplier of the band when mode is stiffen. Default 8."},
        "width": {"type": "number", "minimum": 0.5, "maximum": 500, "unit": "mm", "description": "Full width of the band or of the edge search for a fold. Default 12."},
        "fold_angle": {"type": "number", "minimum": 0, "maximum": 360, "unit": "deg", "description": "Rest dihedral angle of the crease when mode is fold. 180 is flat. Default 150."},
        "fold_strength": {"type": "number", "minimum": 0, "maximum": 1, "description": "Crease stiffness 0 to 1 when mode is fold. Default 0.5."}
    })
}

fn schema_add() -> Value {
    let mut props = json!({
        "panel": {"type": "integer", "minimum": 1, "description": "Panel id."},
        "points": {"type": "array", "minItems": 2, "maxItems": MAX_POINTS, "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "unit": "mm", "description": "Polyline in panel space as [x, y] pairs."},
        "closed": {"type": "boolean", "default": false, "description": "Join the last point to the first."}
    });
    if let (Some(p), Some(extra)) = (props.as_object_mut(), fx_props().as_object()) {
        p.extend(extra.clone());
    }
    json!({"type": "object", "properties": props, "required": ["panel", "points"], "additionalProperties": false})
}

fn schema_set() -> Value {
    let mut props = json!({
        "id": {"type": "integer", "minimum": 1, "description": "Style line id as listed by styleline.list."},
        "panel": {"type": "integer", "minimum": 1, "description": "Move the line to this panel."},
        "points": {"type": "array", "minItems": 2, "maxItems": MAX_POINTS, "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "unit": "mm", "description": "Replace the polyline (panel space)."},
        "closed": {"type": "boolean"}
    });
    if let (Some(p), Some(extra)) = (props.as_object_mut(), fx_props().as_object()) {
        p.extend(extra.clone());
    }
    json!({"type": "object", "properties": props, "required": ["id"], "additionalProperties": false})
}

fn schema_id() -> Value {
    json!({"type": "object", "properties": {"id": {"type": "integer", "minimum": 1}}, "required": ["id"], "additionalProperties": false})
}

fn schema_list() -> Value {
    json!({"type": "object", "properties": {"panel": {"type": "integer", "minimum": 1, "description": "Only the lines of this panel."}}, "additionalProperties": false})
}

fn schema_list3d() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ids": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Only these lines. Default all."},
            "step": {"type": "number", "minimum": 1, "maximum": 200, "default": 10, "unit": "mm", "description": "Longest panel-space distance between returned points; segments are subdivided so the line follows the surface."},
            "lift": {"type": "number", "minimum": 0, "maximum": 20, "default": 0.5, "unit": "mm", "description": "Raise the points along the surface normal so the line does not z-fight with the cloth."}
        },
        "additionalProperties": false
    })
}

fn points_of(a: &Value) -> Result<Option<Vec<[f32; 2]>>, CommandError> {
    let Some(v) = a.get("points").filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() >= 2 && l.len() <= MAX_POINTS).ok_or_else(|| bad(format!("points must hold 2 to {MAX_POINTS} [x, y] pairs")))?;
    let mut out = Vec::with_capacity(list.len());
    for item in list {
        let pair = item.as_array().filter(|p| p.len() == 2).ok_or_else(|| bad("each point must be [x, y]"))?;
        let mut xy = [0.0f32; 2];
        for (slot, n) in xy.iter_mut().zip(pair) {
            *slot = n.as_f64().filter(|x| x.is_finite() && x.abs() <= 1e6).ok_or_else(|| bad("point coordinates must be finite numbers within 1e6 mm"))? as f32;
        }
        out.push(xy);
    }
    Ok(Some(out))
}

fn apply_fx(a: &Value, base: LineFx) -> Result<LineFx, CommandError> {
    let mut out = base;
    if let Some(m) = a.get("mode").filter(|v| !v.is_null()) {
        out.mode = m.as_str().and_then(Mode::parse).ok_or_else(|| bad("mode must be none, stiffen or fold"))?;
    }
    if let Some(x) = ranged(a, "stiffen", 1.0, 1000.0)? {
        out.stiffen = x;
    }
    if let Some(x) = ranged(a, "width", 0.5, 500.0)? {
        out.width = x;
    }
    if let Some(x) = ranged(a, "fold_angle", 0.0, 360.0)? {
        out.fold_angle = x;
    }
    if let Some(x) = ranged(a, "fold_strength", 0.0, 1.0)? {
        out.fold_strength = x;
    }
    Ok(out)
}

fn closed_arg(a: &Value) -> Result<Option<bool>, CommandError> {
    a.get("closed").filter(|v| !v.is_null()).map(|v| v.as_bool().ok_or_else(|| bad("closed must be true or false"))).transpose()
}

fn describe(g: &ffb_pattern::Garment, l: &StyleLine) -> Value {
    let f = fx::line_fx(g, l.id);
    let len: f64 = geo::segments(&l.points.iter().map(|p| [f64::from(p[0]), f64::from(p[1])]).collect::<Vec<_>>(), l.closed).iter().map(|s| (s.1[0] - s.0[0]).hypot(s.1[1] - s.0[1])).sum();
    let mut v = f.to_json();
    if let Some(o) = v.as_object_mut() {
        o.insert("id".into(), json!(l.id));
        o.insert("panel".into(), json!(l.panel));
        o.insert("points".into(), json!(l.points));
        o.insert("closed".into(), json!(l.closed));
        o.insert("length".into(), json!(len));
    }
    v
}

fn check_shape(points: &[[f32; 2]], closed: bool) -> Result<(), CommandError> {
    if closed && points.len() < 3 {
        return Err(bad("a closed style line needs at least 3 points"));
    }
    let mut total = 0.0f64;
    for w in points.windows(2) {
        total += f64::from(w[1][0] - w[0][0]).hypot(f64::from(w[1][1] - w[0][1]));
    }
    if total < 0.01 {
        return Err(bad("the style line has no length"));
    }
    Ok(())
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel = id_arg(a, "panel")?;
    panel_index(s, panel)?;
    let points = points_of(a)?.ok_or_else(|| bad("points is required"))?;
    let closed = closed_arg(a)?.unwrap_or(false);
    check_shape(&points, closed)?;
    let fxv = apply_fx(a, LineFx::default())?;
    if s.garment.style_lines.len() >= MAX_LINES {
        return Err(bad(format!("a garment holds at most {MAX_LINES} style lines; remove some first")));
    }
    let before = Box::new(s.garment.clone());
    let id = s.garment.style_lines.iter().map(|l| l.id).max().unwrap_or(0) + 1;
    let line = StyleLine { id, panel, points, closed };
    s.garment.style_lines.push(line.clone());
    if fxv != LineFx::default() {
        fx::put(&mut s.garment, "lines", &id.to_string(), Some(fxv.to_json()));
    }
    let after = Box::new(s.garment.clone());
    s.record(Edit::Garment { label: "Add style line".into(), before, after });
    Ok(describe(&s.garment, &line))
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let at = s.garment.style_lines.iter().position(|l| l.id == id).ok_or_else(|| missing(format!("no style line with id {id}")))?;
    let mut line = s.garment.style_lines[at].clone();
    if a.get("panel").is_some_and(|v| !v.is_null()) {
        let panel = id_arg(a, "panel")?;
        panel_index(s, panel)?;
        line.panel = panel;
    }
    if let Some(p) = points_of(a)? {
        line.points = p;
    }
    if let Some(c) = closed_arg(a)? {
        line.closed = c;
    }
    check_shape(&line.points, line.closed)?;
    let old_fx = fx::line_fx(&s.garment, id);
    let new_fx = apply_fx(a, old_fx)?;
    let changed = line != s.garment.style_lines[at] || new_fx != old_fx;
    if changed {
        let before = Box::new(s.garment.clone());
        s.garment.style_lines[at] = line.clone();
        let stored = if new_fx == LineFx::default() { None } else { Some(new_fx.to_json()) };
        fx::put(&mut s.garment, "lines", &id.to_string(), stored);
        let after = Box::new(s.garment.clone());
        s.record(Edit::Garment { label: "Edit style line".into(), before, after });
    }
    let mut out = describe(&s.garment, &line);
    if let Some(o) = out.as_object_mut() {
        o.insert("changed".into(), json!(changed));
    }
    Ok(out)
}

fn remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let at = s.garment.style_lines.iter().position(|l| l.id == id).ok_or_else(|| missing(format!("no style line with id {id}")))?;
    let before = Box::new(s.garment.clone());
    s.garment.style_lines.remove(at);
    fx::put(&mut s.garment, "lines", &id.to_string(), None);
    let after = Box::new(s.garment.clone());
    s.record(Edit::Garment { label: "Remove style line".into(), before, after });
    Ok(json!({"removed": id, "remaining": s.garment.style_lines.len()}))
}

fn list(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let only = match a.get("panel").filter(|v| !v.is_null()) {
        Some(_) => Some(id_arg(a, "panel")?),
        None => None,
    };
    let lines: Vec<Value> = s.garment.style_lines.iter().filter(|l| only.is_none_or(|p| l.panel == p)).map(|l| describe(&s.garment, l)).collect();
    Ok(json!({"count": lines.len(), "lines": lines}))
}

fn sub(p: [f32; 3], q: [f32; 3]) -> [f32; 3] {
    [p[0] - q[0], p[1] - q[1], p[2] - q[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn list3d(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let wanted = id_list(a, "ids")?;
    let step = ranged(a, "step", 1.0, 200.0)?.unwrap_or(10.0);
    let lift = ranged(a, "lift", 0.0, 20.0)?.unwrap_or(0.5) as f32;
    let lines: Vec<StyleLine> = s.garment.style_lines.iter().filter(|l| wanted.as_ref().is_none_or(|w| w.contains(&l.id))).cloned().collect();
    if let Some(w) = &wanted {
        if let Some(missing_id) = w.iter().find(|id| !lines.iter().any(|l| l.id == **id)) {
            return Err(missing(format!("no style line with id {missing_id}")));
        }
    }
    let mut panels: Vec<u32> = lines.iter().map(|l| l.panel).collect();
    panels.sort_unstable();
    panels.dedup();
    let mut out = Vec::new();
    let mut skipped = Vec::new();
    for panel in panels {
        let Ok(index) = panel_index(s, panel) else {
            skipped.extend(lines.iter().filter(|l| l.panel == panel).map(|l| l.id));
            continue;
        };
        let mesh = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?.mesh.clone();
        let pos3 = current_positions(s, panel)?;
        if pos3.len() != mesh.positions.len() {
            return Err(CommandError::new("invalid_state", "the panel mesh and its current positions differ in size"));
        }
        let locator = Locator::new(&mesh.positions, &mesh.triangles);
        for line in lines.iter().filter(|l| l.panel == panel) {
            let pts: Vec<[f64; 2]> = line.points.iter().map(|p| [f64::from(p[0]), f64::from(p[1])]).collect();
            let mut samples: Vec<[f64; 2]> = Vec::new();
            for (k, (p, q)) in geo::segments(&pts, line.closed).into_iter().enumerate() {
                let len = (q[0] - p[0]).hypot(q[1] - p[1]);
                let n = ((len / step).ceil() as usize).max(1);
                if k == 0 {
                    samples.push(p);
                }
                for i in 1..=n {
                    let t = i as f64 / n as f64;
                    samples.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
                }
                if samples.len() > MAX_SAMPLES {
                    return Err(bad(format!("style line {} would need more than {MAX_SAMPLES} points; raise step", line.id)));
                }
            }
            let mut points = Vec::with_capacity(samples.len());
            for p in samples {
                let Some(hit) = locator.locate(p) else { continue };
                let t = mesh.triangles[hit.tri];
                let c = [pos3[t[0] as usize], pos3[t[1] as usize], pos3[t[2] as usize]];
                let mut at = [0.0f32; 3];
                for (corner, w) in c.iter().zip(hit.bary) {
                    for k in 0..3 {
                        at[k] += corner[k] * w as f32;
                    }
                }
                let n = cross(sub(c[1], c[0]), sub(c[2], c[0]));
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if l > 1e-12 {
                    for k in 0..3 {
                        at[k] += n[k] / l * lift;
                    }
                }
                points.push(at);
            }
            let length: f64 = points.windows(2).map(|w| f64::from(sub(w[1], w[0]).iter().map(|x| x * x).sum::<f32>()).sqrt()).sum();
            out.push(json!({"id": line.id, "panel": line.panel, "closed": line.closed, "mode": fx::line_fx(&s.garment, line.id).mode.name(), "points": points, "length": length}));
        }
    }
    out.sort_by_key(|v| v["id"].as_u64().unwrap_or(0));
    Ok(json!({"count": out.len(), "lines": out, "skipped": skipped}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "styleline.add",
            mcp: "styleline_add",
            group: Group::Pattern,
            label: "Add style line",
            description: "Add a style line to a panel: a polyline in panel space (mm) that shows on the 3D garment. With mode stiffen or fold it also stiffens a band or creases the cloth along the line in the next simulation build. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: schema_add,
            run: add,
        },
        Command {
            id: "styleline.set",
            mcp: "styleline_set",
            group: Group::Pattern,
            label: "Edit style line",
            description: "Change a style line: its points, closed flag, panel, or what it does in the simulation (mode none, stiffen or fold with width, stiffness, crease angle and strength). One undo step.",
            keys: &[],
            hints: SET,
            schema: schema_set,
            run: set,
        },
        Command {
            id: "styleline.remove",
            mcp: "styleline_remove",
            group: Group::Pattern,
            label: "Remove style line",
            description: "Delete a style line. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: schema_id,
            run: remove,
        },
        Command {
            id: "styleline.list",
            mcp: "styleline_list",
            group: Group::Pattern,
            label: "List style lines",
            description: "List the style lines with their panel, points in panel space, closed flag, length and simulation mode.",
            keys: &[],
            hints: READ,
            schema: schema_list,
            run: list,
        },
        Command {
            id: "styleline.list3d",
            mcp: "styleline_list3d",
            group: Group::Pattern,
            label: "Style lines in 3D",
            description: "Return each style line as a 3D polyline in world mm, mapped through the current draped panel mesh (simulation positions, applied pose or flat placement) by a barycentric lookup of every sample point in its panel mesh.",
            keys: &[],
            hints: READ,
            schema: schema_list3d,
            run: list3d,
        },
    ]
}
