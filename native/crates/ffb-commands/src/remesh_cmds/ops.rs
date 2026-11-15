use ffb_pattern::output::MeshKind;
use ffb_pattern::Garment;
use serde_json::{json, Value};

use super::out::{density_of, now_ms, pure_flag, quad_meshes, regularize_flag, round, run, seam_report, summary};
use crate::{Command, CommandError, Edit, Group, Session, READ, SET, WRITE};

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn selection(garment: &Garment, args: &Value) -> Result<Vec<u32>, CommandError> {
    let Some(list) = args.get("panels").filter(|v| !v.is_null()) else {
        return Ok(garment.panels.iter().map(|p| p.id).collect());
    };
    let list = list.as_array().ok_or_else(|| bad("panels must be a list of panel ids"))?;
    let mut out: Vec<u32> = Vec::new();
    for v in list {
        let id = v.as_f64().filter(|x| x.fract() == 0.0 && *x >= 1.0 && *x <= f64::from(u32::MAX)).ok_or_else(|| bad("panels must hold whole panel ids"))? as u32;
        if garment.index_of(id).is_none() {
            return Err(CommandError::new("not_found", format!("no panel with id {id}")));
        }
        if !out.contains(&id) {
            out.push(id);
        }
    }
    Ok(out)
}

fn panels_schema() -> Value {
    json!({"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Panel ids as listed by pattern.get. Leave out for every panel."})
}

fn commit(s: &mut Session, label: &str, before: &Garment) -> bool {
    if s.garment == *before {
        return false;
    }
    let after = s.garment.clone();
    s.record(Edit::Garment { label: label.into(), before: Box::new(before.clone()), after: Box::new(after) });
    true
}

fn remesh_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panels": panels_schema(),
            "density": {"type": "number", "minimum": 0.1, "maximum": 8, "description": "Quad density relative to each panel's particle distance: the target quad edge is particle_distance divided by density, so 1 keeps the simulation resolution and 2 halves the edge. Leave out to reuse the stored density (1 when none)."},
            "pure_quads": {"type": "boolean", "default": false, "description": "Split every triangle and quad once so the result holds only quads (edge counts double, seams stay matched). Needed for thick export."},
            "regularize": {"type": "boolean", "default": false, "description": "Run the integer edge regularity pass (a min-cost flow over lattice offsets) before quad extraction; falls back to the plain extraction when the flow is not full. Leave out to reuse the stored setting."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn remesh(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = selection(&s.garment, a)?;
    if s.garment.panels.is_empty() {
        return Err(CommandError::new("invalid_state", "there are no panels to remesh"));
    }
    let density = match a.get("density").filter(|v| !v.is_null()) {
        Some(v) => v.as_f64().filter(|x| x.is_finite() && (0.1..=8.0).contains(x)).ok_or_else(|| bad("density must be a number from 0.1 to 8"))?,
        None => density_of(s),
    };
    let pure = a.get("pure_quads").and_then(Value::as_bool).unwrap_or_else(|| pure_flag(s));
    let regular = a.get("regularize").and_then(Value::as_bool).unwrap_or_else(|| regularize_flag(s));
    let before = s.garment.clone();
    let start = now_ms();
    if !s.garment.output.retopo.is_object() {
        s.garment.output.retopo = json!({});
    }
    s.garment.output.retopo["regularize"] = json!(regular);
    let rows = run(s, &ids, density, pure);
    if rows.iter().all(|r| !r["error"].is_null()) {
        let first = rows.first().and_then(|r| r["error"].as_str()).unwrap_or("remesh failed").to_string();
        s.garment = before;
        return Err(CommandError::new("invalid_geometry", first));
    }
    s.garment.output.mesh = MeshKind::Quad;
    s.garment.output.quad_density = density as f32;
    if !s.garment.output.retopo.is_object() {
        s.garment.output.retopo = json!({});
    }
    s.garment.output.retopo["pure_quads"] = json!(pure);
    let changed = commit(s, "Quad remesh", &before);
    let skipped: Vec<u32> = s.garment.panels.iter().map(|p| p.id).filter(|id| !ids.contains(id) && !s.remesh.meshes.contains_key(id)).collect();
    let seams = seam_report(s);
    Ok(json!({"panels": rows, "density": round(density, 4), "pure_quads": pure, "mesh": "quad", "seams": seams, "panels_without_quads": skipped, "ms": round(now_ms() - start, 1), "changed": changed}))
}

fn clear(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = selection(&s.garment, a)?;
    let before = s.garment.clone();
    let mut cleared = Vec::new();
    for id in &ids {
        if s.remesh.meshes.remove(id).is_some() {
            cleared.push(*id);
        }
    }
    let all_gone = s.garment.panels.iter().all(|p| !s.remesh.meshes.contains_key(&p.id));
    if all_gone {
        s.garment.output.mesh = MeshKind::Triangle;
    }
    let changed = commit(s, "Clear quad mesh", &before);
    Ok(json!({"cleared": cleared, "mesh": if all_gone { "triangle" } else { "quad" }, "changed": changed}))
}

fn get_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "With include_mesh, the panel whose quad mesh to return."},
            "include_mesh": {"type": "boolean", "default": false, "description": "Also return the panel's quad mesh: 2D positions, 3D positions (mm), quads, triangles and the boundary vertex list with segment, index and count."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn get(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mut panels = Vec::new();
    for p in &s.garment.panels {
        if let Some(e) = s.remesh.meshes.get(&p.id) {
            panels.push(summary(p.id, e));
        }
    }
    let guides = s.garment.output.retopo.get("guides").cloned().unwrap_or_else(|| json!([]));
    let mut out = json!({
        "mesh": match s.garment.output.mesh { MeshKind::Quad => "quad", MeshKind::Triangle => "triangle" },
        "density": round(density_of(s), 4),
        "pure_quads": pure_flag(s),
        "guides": guides,
        "panels": panels
    });
    if a.get("include_mesh").and_then(Value::as_bool).unwrap_or(false) {
        let id = a.get("panel").and_then(Value::as_u64).filter(|p| *p >= 1).ok_or_else(|| bad("include_mesh needs a panel id"))? as u32;
        if s.garment.index_of(id).is_none() {
            return Err(CommandError::new("not_found", format!("no panel with id {id}")));
        }
        let pieces = quad_meshes(s);
        let q = pieces.into_iter().find(|q| q.panel == id).ok_or_else(|| CommandError::new("invalid_state", "that panel has no quad mesh; run quad.remesh first"))?;
        out["mesh_data"] = json!({
            "panel": id,
            "positions2d": q.positions2d.iter().map(|p| [round(p[0], 4), round(p[1], 4)]).collect::<Vec<_>>(),
            "positions3d": q.positions.iter().map(|p| [round(f64::from(p[0]), 3), round(f64::from(p[1]), 3), round(f64::from(p[2]), 3)]).collect::<Vec<_>>(),
            "quads": q.quads,
            "triangles": q.tris,
            "boundary": q.boundary.iter().map(|b| json!({"vertex": b.0, "segment": b.1, "index": b.2, "count": b.3})).collect::<Vec<_>>()
        });
    }
    Ok(out)
}

fn set_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "guides": {
                "type": "array",
                "maxItems": 256,
                "items": {
                    "type": "object",
                    "properties": {
                        "panel": {"type": "integer", "minimum": 1},
                        "points": {"type": "array", "minItems": 2, "maxItems": 4096, "items": {"type": "array", "items": {"type": "number", "minimum": -1000000, "maximum": 1000000}, "minItems": 2, "maxItems": 2}, "description": "Guide curve polyline in panel space, x y in mm."}
                    },
                    "required": ["panel", "points"],
                    "additionalProperties": false
                },
                "description": "User guide curves. The orientation field aligns to them and the quad lattice runs along them. An empty list removes all guides."
            }
        },
        "required": ["guides"],
        "additionalProperties": false
    })
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let list = a.get("guides").and_then(Value::as_array).ok_or_else(|| bad("guides must be a list"))?;
    let mut out: Vec<Value> = Vec::new();
    for g in list {
        let panel = g.get("panel").and_then(Value::as_u64).filter(|p| *p >= 1 && *p <= u64::from(u32::MAX)).ok_or_else(|| bad("each guide needs a panel id"))? as u32;
        if s.garment.index_of(panel).is_none() {
            return Err(CommandError::new("not_found", format!("no panel with id {panel}")));
        }
        let points = g.get("points").and_then(Value::as_array).filter(|p| p.len() >= 2).ok_or_else(|| bad("each guide needs at least 2 points"))?;
        let mut line: Vec<Value> = Vec::with_capacity(points.len());
        for p in points {
            let xy = p.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad("guide points must be [x, y]"))?;
            let (x, y) = (xy[0].as_f64().filter(|v| v.is_finite()), xy[1].as_f64().filter(|v| v.is_finite()));
            let (Some(x), Some(y)) = (x, y) else { return Err(bad("guide points must be finite numbers")) };
            line.push(json!([x, y]));
        }
        out.push(json!({"panel": panel, "points": line}));
    }
    let before = s.garment.clone();
    if !s.garment.output.retopo.is_object() {
        s.garment.output.retopo = json!({});
    }
    s.garment.output.retopo["guides"] = json!(out);
    let changed = commit(s, "Retopo guides", &before);
    Ok(json!({"guides": out.len(), "changed": changed, "stale": s.garment.output.mesh == MeshKind::Quad && changed}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "quad.remesh",
            mcp: "quad_remesh",
            group: Group::Export,
            label: "Quad Remesh",
            description: "Remesh panels into a quad-dominant mesh in pattern space: a 4-RoSy orientation field smoothed with alignment to the outline, the grain and any retopo guides, an Instant Meshes style position field, lattice extraction, a triangulated border band paired into quads, then cleanup (valence 2 removal, diagonal collapse to remove singularity pairs, smoothing). Quads respect the panel outline exactly and sewn seams get equal vertex counts on both sides. Density sets the edge length relative to the particle distance. Results live in session memory and the garment records mesh kind quad and the density; one undo step. Returns percent quads, valence histogram, irregular vertices, angle deviation from 90 degrees and a seam match report.",
            keys: &[],
            hints: SET,
            schema: remesh_schema,
            run: remesh,
        },
        Command {
            id: "quad.clear",
            mcp: "quad_clear",
            group: Group::Export,
            label: "Clear Quad Mesh",
            description: "Drop the stored quad meshes of the given panels, or all panels. When no panel keeps a quad mesh the garment goes back to the triangle mesh kind. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: remesh_clear_schema,
            run: clear,
        },
        Command {
            id: "retopo.get",
            mcp: "retopo_get",
            group: Group::Export,
            label: "Get Retopo",
            description: "The retopology state: mesh kind, density, whether pure quads are on, the user guide curves and the per-panel quad statistics of the stored results.",
            keys: &[],
            hints: READ,
            schema: get_schema,
            run: get,
        },
        Command {
            id: "retopo.set",
            mcp: "retopo_set",
            group: Group::Export,
            label: "Set Retopo Guides",
            description: "Replace the user guide curves (panel space polylines in mm) that the orientation field aligns to. Guides apply at the next quad.remesh, or when the quad mesh is next read for output. One undo step.",
            keys: &[],
            hints: SET,
            schema: set_schema,
            run: set,
        },
    ]
}

fn remesh_clear_schema() -> Value {
    json!({"type": "object", "properties": {"panels": panels_schema()}, "required": [], "additionalProperties": false})
}
