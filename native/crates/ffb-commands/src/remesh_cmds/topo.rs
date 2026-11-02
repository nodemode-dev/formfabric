use ffb_pattern::path;
use ffb_remesh::topo_ghost::{ghosts, mirror, project_sewn};
use ffb_remesh::topology::{Mode, TopoError, Topology};
use ffb_remesh::measure;
use serde_json::{json, Value};

use super::out::{lift_points, round, QuadPanel};
use crate::{Command, CommandError, Edit, Group, Session, READ, WRITE};

const GHOST_TOL: f64 = 1.0;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn conv(e: TopoError) -> CommandError {
    CommandError::new(e.code, e.message)
}

pub(crate) fn load(garment: &ffb_pattern::Garment) -> Topology {
    garment.output.retopo.get("topology").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
}

fn store(garment: &mut ffb_pattern::Garment, topo: &Topology) {
    if !garment.output.retopo.is_object() {
        garment.output.retopo = json!({});
    }
    if topo.is_empty() {
        if let Some(map) = garment.output.retopo.as_object_mut() {
            map.remove("topology");
        }
    } else {
        garment.output.retopo["topology"] = serde_json::to_value(topo).unwrap_or(Value::Null);
    }
}

pub(crate) fn modes(garment: &ffb_pattern::Garment) -> Vec<(u32, Mode)> {
    load(garment).remeshed.into_iter().filter(|(id, _)| garment.index_of(*id).is_some()).collect()
}

fn edit(s: &mut Session, label: &str, f: impl FnOnce(&mut Session, &mut Topology) -> Result<Value, CommandError>) -> Result<Value, CommandError> {
    let before = s.garment.clone();
    let mut topo = load(&s.garment);
    let mut out = f(s, &mut topo)?;
    store(&mut s.garment, &topo);
    let changed = s.garment != before;
    if changed {
        let after = s.garment.clone();
        s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
    }
    out["changed"] = json!(changed);
    Ok(out)
}

fn panel_id(a: &Value, key: &str) -> Result<u32, CommandError> {
    a.get(key).and_then(Value::as_u64).filter(|p| *p >= 1 && *p <= u64::from(u32::MAX)).map(|p| p as u32).ok_or_else(|| bad(format!("{key} must be a panel id")))
}

fn existing_panel(s: &Session, a: &Value) -> Result<u32, CommandError> {
    let id = panel_id(a, "panel")?;
    if s.garment.index_of(id).is_none() {
        return Err(CommandError::new("not_found", format!("no panel with id {id}")));
    }
    Ok(id)
}

fn infer_panel(s: &Session, topo: &Topology, a: &Value) -> Result<u32, CommandError> {
    if a.get("panel").is_some_and(|v| !v.is_null()) {
        return existing_panel(s, a);
    }
    if let Some(o) = &topo.open {
        return Ok(o.panel);
    }
    let ids: Vec<u32> = topo.panels.keys().copied().collect();
    if ids.len() == 1 {
        return Ok(ids[0]);
    }
    Err(bad("panel is required when several panels have topology"))
}

fn xy(v: &Value, name: &str) -> Result<[f64; 2], CommandError> {
    let l = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad(format!("{name} must be [x, y]")))?;
    match (l[0].as_f64(), l[1].as_f64()) {
        (Some(x), Some(y)) if x.is_finite() && y.is_finite() => Ok([x, y]),
        _ => Err(bad(format!("{name} must be finite numbers"))),
    }
}

fn nat(a: &Value, key: &str) -> Result<u32, CommandError> {
    a.get(key).and_then(Value::as_u64).filter(|x| *x <= u64::from(u32::MAX)).map(|x| x as u32).ok_or_else(|| bad(format!("{key} must be a point index")))
}

fn mirrors_json(s: &Session, panel: u32, pos: [f64; 2]) -> Vec<Value> {
    mirror(&s.garment, panel, pos, GHOST_TOL).iter().map(|m| json!({"panel": m.panel, "pos": [round(m.pos[0], 4), round(m.pos[1], 4)], "seam": m.seam, "fraction": round(m.fraction, 6), "partner_fraction": round(m.partner_fraction, 6)})).collect()
}

fn add_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel to draw on. Only one panel can hold an open shape at a time."},
            "pos": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "Point in panel coordinates, x y in mm."},
            "uv": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "Point as fractions 0 to 1 of the panel's bounding box, used when pos is left out."},
            "snap": {"type": "number", "minimum": 0, "maximum": 1000, "default": 0, "description": "Snap radius in mm. The point snaps to the nearest existing point or ghost point inside it, else onto a sewn edge inside it. 0 places the point exactly where given."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn add_point(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel = existing_panel(s, a)?;
    let index = s.garment.index_of(panel).unwrap_or(0);
    let mut pos = if let Some(p) = a.get("pos").filter(|v| !v.is_null()) {
        xy(p, "pos")?
    } else if let Some(u) = a.get("uv").filter(|v| !v.is_null()) {
        let uv = xy(u, "uv")?;
        let b = path::bounds(&s.garment.panels[index].outline, true).ok_or_else(|| CommandError::new("invalid_state", "the panel has no outline"))?;
        [b[0][0] + uv[0] * (b[1][0] - b[0][0]), b[0][1] + uv[1] * (b[1][1] - b[0][1])]
    } else {
        return Err(bad("give pos or uv"));
    };
    let snap = a.get("snap").and_then(Value::as_f64).filter(|x| x.is_finite() && *x >= 0.0).unwrap_or(0.0);
    edit(s, "Topology point", |s, topo| {
        let mut snapped = Value::Null;
        if snap > 0.0 {
            let mut best: Option<(f64, [f64; 2], &str)> = None;
            let mut offer = |p: [f64; 2], kind: &'static str| {
                let d = (p[0] - pos[0]).hypot(p[1] - pos[1]);
                if d <= snap && best.is_none_or(|b| d < b.0) {
                    best = Some((d, p, kind));
                }
            };
            if let Some(t) = topo.panels.get(&panel) {
                for p in &t.points {
                    offer(*p, "point");
                }
            }
            for g in ghosts(&s.garment, topo, GHOST_TOL).iter().filter(|g| g.panel == panel) {
                offer(g.pos, "ghost");
            }
            if let Some((_, p, kind)) = best {
                pos = p;
                snapped = json!(kind);
            } else if let Some(p) = project_sewn(&s.garment, panel, pos, snap) {
                pos = p;
                snapped = json!("edge");
            }
        }
        let (idx, created) = topo.add_point(panel, pos).map_err(conv)?;
        let open = topo.open.as_ref().map_or(0, |o| o.points.len());
        Ok(json!({"panel": panel, "index": idx, "created": created, "pos": [round(pos[0], 4), round(pos[1], 4)], "snapped": snapped, "open_points": open, "ghosts": mirrors_json(s, panel, pos)}))
    })
}

fn empty_schema() -> Value {
    json!({"type": "object", "properties": {}, "required": [], "additionalProperties": false})
}

fn close(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    edit(s, "Topology face", |_, topo| {
        let (panel, face) = topo.close().map_err(conv)?;
        Ok(json!({"panel": panel, "face": face}))
    })
}

fn finish(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    edit(s, "Topology line", |_, topo| {
        let (panel, points) = topo.finish().map_err(conv)?;
        Ok(json!({"panel": panel, "points": points}))
    })
}

fn delete_last(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    edit(s, "Topology remove last point", |_, topo| {
        let (panel, left) = topo.delete_last().map_err(conv)?;
        Ok(json!({"panel": panel, "open_points": left}))
    })
}

fn cancel(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    edit(s, "Topology cancel", |_, topo| Ok(json!({"panel": topo.cancel()})))
}

fn merge_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel of both points; left out when only one panel has topology."},
            "a": {"type": "integer", "minimum": 0, "description": "First point index as topology.list reports it."},
            "b": {"type": "integer", "minimum": 0, "description": "Second point index."}
        },
        "required": ["a", "b"],
        "additionalProperties": false
    })
}

fn merge(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (pa, pb) = (nat(a, "a")?, nat(a, "b")?);
    let topo0 = load(&s.garment);
    let panel = infer_panel(s, &topo0, a)?;
    edit(s, "Topology merge", |_, topo| {
        topo.merge(panel, pa, pb).map_err(conv)?;
        let t = topo.panels.get(&panel);
        Ok(json!({"panel": panel, "points": t.map_or(0, |t| t.points.len()), "faces": t.map_or(0, |t| t.faces.len())}))
    })
}

fn merge_all_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "distance": {"type": "number", "exclusiveMinimum": 0, "maximum": 1000, "description": "Points closer than this (mm) merge to their centre."},
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Panels to merge in. Leave out for every panel with topology."}
        },
        "required": ["distance"],
        "additionalProperties": false
    })
}

fn merge_all(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let distance = a.get("distance").and_then(Value::as_f64).filter(|d| d.is_finite() && *d > 0.0 && *d <= 1000.0).ok_or_else(|| bad("distance must be a positive number of mm"))?;
    let only: Option<Vec<u32>> = match a.get("panels").filter(|v| !v.is_null()) {
        Some(v) => Some(v.as_array().ok_or_else(|| bad("panels must be a list"))?.iter().filter_map(|x| x.as_u64().map(|x| x as u32)).collect()),
        None => None,
    };
    edit(s, "Topology merge all", |_, topo| {
        let ids: Vec<u32> = topo.panels.keys().copied().filter(|id| only.as_ref().is_none_or(|o| o.contains(id))).collect();
        let mut merged = 0usize;
        for id in ids {
            merged += topo.merge_all(id, distance);
        }
        Ok(json!({"merged_points": merged}))
    })
}

fn divide_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "edge": {"type": "object", "properties": {"panel": {"type": "integer", "minimum": 1}, "a": {"type": "integer", "minimum": 0}, "b": {"type": "integer", "minimum": 0}}, "required": ["a", "b"], "additionalProperties": false, "description": "The edge between points a and b. panel may be left out when only one panel has topology."},
            "count": {"type": "integer", "minimum": 2, "maximum": 32, "default": 2, "description": "Number of parts the edge splits into. The loop of quads across it splits the same way."}
        },
        "required": ["edge"],
        "additionalProperties": false
    })
}

fn divide(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let e = a.get("edge").filter(|v| v.is_object()).ok_or_else(|| bad("edge must be {panel, a, b}"))?;
    let (pa, pb) = (nat(e, "a")?, nat(e, "b")?);
    let count = a.get("count").and_then(Value::as_u64).unwrap_or(2) as usize;
    let topo0 = load(&s.garment);
    let panel = infer_panel(s, &topo0, e)?;
    edit(s, "Topology divide", |_, topo| {
        let d = topo.divide(panel, pa, pb, count).map_err(conv)?;
        Ok(json!({"panel": panel, "edges_cut": d.edges_cut, "new_points": d.new_points, "faces_before": d.faces_before, "faces_after": d.faces_after}))
    })
}

fn delete_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1},
            "face": {"type": "integer", "minimum": 0, "description": "Face index to remove."},
            "point": {"type": "integer", "minimum": 0, "description": "Point index to remove with the faces and lines that use it."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel = existing_panel(s, a)?;
    edit(s, "Topology delete", |_, topo| {
        if a.get("face").is_some() {
            topo.delete_face(panel, nat(a, "face")? as usize).map_err(conv)?;
        } else if a.get("point").is_some() {
            topo.delete_point(panel, nat(a, "point")?).map_err(conv)?;
        } else {
            return Err(bad("give face or point"));
        }
        Ok(json!({"panel": panel}))
    })
}

fn clear_schema() -> Value {
    json!({"type": "object", "properties": {"panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Panels to clear. Leave out for all."}}, "required": [], "additionalProperties": false})
}

fn clear(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let only: Option<Vec<u32>> = a.get("panels").and_then(Value::as_array).map(|l| l.iter().filter_map(|x| x.as_u64().map(|x| x as u32)).collect());
    edit(s, "Clear topology", |_, topo| {
        let mut ids: Vec<u32> = topo.panels.keys().chain(topo.remeshed.keys()).copied().collect();
        ids.sort_unstable();
        ids.dedup();
        let mut cleared = Vec::new();
        for id in ids.into_iter().filter(|id| only.as_ref().is_none_or(|o| o.contains(id))) {
            if topo.clear(id) {
                cleared.push(id);
            }
        }
        Ok(json!({"cleared": cleared}))
    })
}

fn remesh_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Panels whose drawn faces become the export quad mesh. Leave out for every panel with faces."},
            "mode": {"type": "string", "enum": ["replace", "duplicate"], "default": "replace", "description": "replace swaps the panel's remesh result for the drawn faces; duplicate keeps the remesh result and adds the drawn mesh beside it."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn remesh(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mode = match a.get("mode").and_then(Value::as_str).unwrap_or("replace") {
        "replace" => Mode::Replace,
        "duplicate" => Mode::Duplicate,
        _ => return Err(bad("mode must be replace or duplicate")),
    };
    let only: Option<Vec<u32>> = a.get("panels").filter(|v| !v.is_null()).and_then(Value::as_array).map(|l| l.iter().filter_map(|x| x.as_u64().map(|x| x as u32)).collect());
    edit(s, "Topology remesh", |s, topo| {
        let ids: Vec<u32> = topo.panels.iter().filter(|(id, t)| !t.faces.is_empty() && only.as_ref().is_none_or(|o| o.contains(id)) && s.garment.index_of(**id).is_some()).map(|(id, _)| *id).collect();
        if ids.is_empty() {
            return Err(CommandError::new("invalid_state", "no panel has topology faces; draw and close a face first"));
        }
        let mut rows = Vec::new();
        for id in &ids {
            let mesh = topo.to_quad_mesh(*id).map_err(conv)?;
            let m = measure(&mesh);
            rows.push(json!({"panel": id, "mode": match mode { Mode::Replace => "replace", Mode::Duplicate => "duplicate" }, "vertices": m.vertices, "quads": m.quads, "triangles": m.triangles, "percent_quads": round(m.percent_quads, 2)}));
            topo.remeshed.insert(*id, mode);
            if mode == Mode::Replace {
                s.remesh.meshes.remove(id);
            }
        }
        s.garment.output.mesh = ffb_pattern::output::MeshKind::Quad;
        Ok(json!({"panels": rows, "mesh": "quad"}))
    })
}

fn list_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Only this panel. Leave out for all panels with topology."},
            "tolerance": {"type": "number", "exclusiveMinimum": 0, "maximum": 100, "default": 1, "description": "Distance in mm within which a point counts as on a sewn edge."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn list(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let tol = a.get("tolerance").and_then(Value::as_f64).filter(|t| t.is_finite() && *t > 0.0).unwrap_or(GHOST_TOL);
    let only = a.get("panel").and_then(Value::as_u64).map(|p| p as u32);
    let topo = load(&s.garment);
    let mut panels = Vec::new();
    for (id, t) in &topo.panels {
        let Some(i) = s.garment.index_of(*id) else { continue };
        if only.is_some_and(|o| o != *id) {
            continue;
        }
        let rem = topo.remeshed.get(id).map(|m| match m { Mode::Replace => "replace", Mode::Duplicate => "duplicate" });
        panels.push(json!({"panel": id, "name": s.garment.panels[i].name, "points": t.points.iter().map(|p| [round(p[0], 4), round(p[1], 4)]).collect::<Vec<_>>(), "faces": t.faces, "lines": t.lines, "remeshed": rem}));
    }
    let ghost: Vec<Value> = ffb_remesh::topo_ghost::ghosts(&s.garment, &topo, tol)
        .iter()
        .filter(|g| only.is_none_or(|o| o == g.panel))
        .map(|g| json!({"panel": g.panel, "pos": [round(g.pos[0], 4), round(g.pos[1], 4)], "from": {"panel": g.from_panel, "point": g.from_point}, "seam": g.seam, "fraction": round(g.fraction, 6), "partner_fraction": round(g.partner_fraction, 6), "occupied": g.occupied}))
        .collect();
    Ok(json!({"panels": panels, "open": topo.open.as_ref().map(|o| json!({"panel": o.panel, "points": o.points})), "ghosts": ghost}))
}

pub(crate) fn topology_panels(s: &mut Session) -> Vec<QuadPanel> {
    let topo = load(&s.garment);
    let mut out = Vec::new();
    for (id, mode) in modes(&s.garment) {
        let Ok(mesh) = topo.to_quad_mesh(id) else { continue };
        let Some((points, flip)) = lift_points(s, id, &mesh.positions) else { continue };
        let base = s.garment.panels.iter().find(|p| p.id == id).map(|p| p.name.clone()).unwrap_or_default();
        let wind = |q: [u32; 4]| if flip { [q[3], q[2], q[1], q[0]] } else { q };
        let wind3 = |t: [u32; 3]| if flip { [t[2], t[1], t[0]] } else { t };
        out.push(QuadPanel {
            panel: id,
            name: if mode == Mode::Duplicate { format!("{base} topology") } else { base },
            positions2d: mesh.positions.clone(),
            positions: points.iter().map(|p| [p[0] as f32, p[1] as f32, p[2] as f32]).collect(),
            quads: mesh.quads.iter().copied().map(wind).collect(),
            tris: mesh.tris.iter().copied().map(wind3).collect(),
            boundary: Vec::new(),
        });
    }
    out
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "topology.add_point",
            mcp: "topology_add_point",
            group: Group::Export,
            label: "Add Topology Point",
            description: "Add a point to the open topology shape of a panel, in panel coordinates (pos, mm) or as fractions of the panel box (uv). Points snap to existing points and ghost points inside the snap radius, else onto a sewn edge. A point on a sewn edge is mirrored at the same arc length fraction onto the partner edge of the seam as a ghost point; the reply lists those ghosts. One panel at a time can hold an open shape.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: add_point,
        },
        Command {
            id: "topology.close",
            mcp: "topology_close",
            group: Group::Export,
            label: "Close Topology Shape",
            description: "Close the open shape into a face. A closed loop becomes a face only with 3 or 4 points; the face is stored counter-clockwise and must have area and no crossing edges.",
            keys: &[],
            hints: WRITE,
            schema: empty_schema,
            run: close,
        },
        Command {
            id: "topology.finish",
            mcp: "topology_finish",
            group: Group::Export,
            label: "Finish Topology Line",
            description: "Finish the open shape as an open line of 2 or more points without closing it into a face.",
            keys: &[],
            hints: WRITE,
            schema: empty_schema,
            run: finish,
        },
        Command {
            id: "topology.delete_last",
            mcp: "topology_delete_last",
            group: Group::Export,
            label: "Remove Last Topology Point",
            description: "Remove the last point of the open shape. A point that no face or line uses is dropped.",
            keys: &[],
            hints: WRITE,
            schema: empty_schema,
            run: delete_last,
        },
        Command {
            id: "topology.cancel",
            mcp: "topology_cancel",
            group: Group::Export,
            label: "Cancel Topology Shape",
            description: "Drop the open shape and the points only it used. Does nothing when no shape is open.",
            keys: &[],
            hints: WRITE,
            schema: empty_schema,
            run: cancel,
        },
        Command {
            id: "topology.merge",
            mcp: "topology_merge",
            group: Group::Export,
            label: "Merge Topology Points",
            description: "Merge two topology points of a panel into one at their midpoint. Faces that collapse are dropped or become triangles.",
            keys: &[],
            hints: WRITE,
            schema: merge_schema,
            run: merge,
        },
        Command {
            id: "topology.merge_all",
            mcp: "topology_merge_all",
            group: Group::Export,
            label: "Merge Topology Points by Distance",
            description: "Merge every group of topology points closer than the distance (mm) into one point at the group centre.",
            keys: &[],
            hints: WRITE,
            schema: merge_all_schema,
            run: merge_all,
        },
        Command {
            id: "topology.divide",
            mcp: "topology_divide",
            group: Group::Export,
            label: "Divide Topology Edge",
            description: "Split an edge into count parts and cut the loop of quads across it the same way, so the faces beside it stay quads. A triangle ends the loop with a fan. Fails without changing anything when the loop would cross itself.",
            keys: &[],
            hints: WRITE,
            schema: divide_schema,
            run: divide,
        },
        Command {
            id: "topology.delete",
            mcp: "topology_delete",
            group: Group::Export,
            label: "Delete Topology Element",
            description: "Delete a face, or a point with the faces and lines that use it, from a panel's topology.",
            keys: &[],
            hints: WRITE,
            schema: delete_schema,
            run: delete,
        },
        Command {
            id: "topology.clear",
            mcp: "topology_clear",
            group: Group::Export,
            label: "Clear Topology",
            description: "Remove the drawn topology and its remesh mode from the given panels, or all panels.",
            keys: &[],
            hints: WRITE,
            schema: clear_schema,
            run: clear,
        },
        Command {
            id: "topology.remesh",
            mcp: "topology_remesh",
            group: Group::Export,
            label: "Remesh from Topology",
            description: "Turn the drawn faces of the given panels into their export quad mesh and set the mesh kind to quad. replace swaps the panel's automatic remesh for the drawn faces; duplicate keeps the automatic result and adds the drawn mesh beside it. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: remesh_schema,
            run: remesh,
        },
        Command {
            id: "topology.list",
            mcp: "topology_list",
            group: Group::Export,
            label: "List Topology",
            description: "The drawn topology per panel (points in panel mm, faces, lines, remesh mode), the open shape, and the ghost points: mirrors of points on sewn edges at the same arc length fraction on the partner edge.",
            keys: &[],
            hints: READ,
            schema: list_schema,
            run: list,
        },
    ]
}
