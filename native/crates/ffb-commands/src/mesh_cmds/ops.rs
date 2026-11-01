use std::collections::BTreeMap;

use ffb_mesh::layout;
use ffb_pattern::Garment;
use serde_json::{json, Value};

use super::publish::sync_panel;
use super::state::{now_ms, Ensured};
use crate::{Command, CommandError, Group, Session, READ, SET};

const MAX_SEGMENT_COUNT: u64 = 100_000;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn round(x: f64, places: i32) -> f64 {
    let k = 10f64.powi(places);
    (x * k).round() / k
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
    json!({
        "type": "array",
        "items": {"type": "integer", "minimum": 1},
        "description": "Panel ids as listed by pattern.get. Leave out for every panel."
    })
}

fn summary(id: u32, ensured: &Ensured) -> Value {
    match &ensured.built {
        Ok(b) => json!({
            "panel": id,
            "vertices": b.mesh.positions.len(),
            "triangles": b.mesh.triangles.len(),
            "boundary": b.mesh.boundary.len(),
            "inner_chains": b.mesh.inner_edges.len(),
            "min_angle": round(b.quality.min_angle, 2),
            "build_ms": round(b.ms, 2),
            "cached": !ensured.rebuilt,
            "error": null
        }),
        Err(e) => json!({"panel": id, "vertices": 0, "triangles": 0, "boundary": 0, "inner_chains": 0, "min_angle": 0, "build_ms": 0, "cached": !ensured.rebuilt, "error": e}),
    }
}

fn build_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panels": panels_schema(),
            "force": {"type": "boolean", "default": false, "description": "Rebuild even when the cached mesh still matches the panel geometry and particle distance."},
            "segment_counts": {
                "type": "object",
                "additionalProperties": {"type": "integer", "minimum": 1, "maximum": MAX_SEGMENT_COUNT},
                "description": "Forced vertex count per outline segment for a single panel, keyed by segment index. This is how sewn seams get matching vertex counts. An empty object clears the forced counts. Needs exactly one panel."
            }
        },
        "required": [],
        "additionalProperties": false
    })
}

fn build(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = selection(&s.garment, a)?;
    let force = a.get("force").and_then(Value::as_bool).unwrap_or(false);
    let Session { garment, mesh, sink, .. } = s;
    if let Some(counts) = a.get("segment_counts").filter(|v| !v.is_null()) {
        let map = counts.as_object().ok_or_else(|| bad("segment_counts must be an object"))?;
        if ids.len() != 1 {
            return Err(bad("segment_counts needs exactly one panel in panels"));
        }
        let mut parsed: BTreeMap<usize, usize> = BTreeMap::new();
        for (key, value) in map {
            let segment: usize = key.parse().map_err(|_| bad(format!("segment index {key} is not a whole number")))?;
            let count = value.as_u64().filter(|c| (1..=MAX_SEGMENT_COUNT).contains(c)).ok_or_else(|| bad("segment counts must be whole numbers from 1 to 100000"))?;
            parsed.insert(segment, count as usize);
        }
        mesh.set_segment_counts(ids[0], parsed);
    }
    let start = now_ms();
    let sink = sink.clone();
    let mut rows = Vec::with_capacity(ids.len());
    let (mut vertices, mut triangles) = (0usize, 0usize);
    for id in &ids {
        let Some(index) = garment.index_of(*id) else { continue };
        let panel = &garment.panels[index];
        let ensured = mesh.ensure(panel, force);
        if let Ok(b) = &ensured.built {
            vertices += b.mesh.positions.len();
            triangles += b.mesh.triangles.len();
        }
        rows.push(summary(*id, &ensured));
        if let Some(sink) = &sink {
            sync_panel(panel, mesh, sink.as_ref(), false);
        }
    }
    Ok(json!({"panels": rows, "vertices": vertices, "triangles": triangles, "ms": round(now_ms() - start, 2), "published": sink.is_some()}))
}

fn get_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id as listed by pattern.get."},
            "include_3d": {"type": "boolean", "default": true, "description": "Also return the 3D positions (flat x y z in mm, from the arrangement or the default flat layout) and the triangle indices as streamed."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn get(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = a.get("panel").and_then(Value::as_f64).filter(|x| x.fract() == 0.0 && *x >= 1.0).ok_or_else(|| bad("panel must be a panel id"))? as u32;
    let Session { garment, mesh, .. } = s;
    let index = garment.index_of(id).ok_or_else(|| CommandError::new("not_found", format!("no panel with id {id}")))?;
    let panel = &garment.panels[index];
    let ensured = mesh.ensure(panel, false);
    let built = ensured.built.clone().map_err(|e| CommandError::new("invalid_geometry", e))?;
    let m = &built.mesh;
    let mut out = json!({
        "panel": id,
        "particle_distance": panel.particle_distance,
        "vertices": m.positions.len(),
        "positions": m.positions.iter().map(|p| [round(p[0], 4), round(p[1], 4)]).collect::<Vec<_>>(),
        "triangles": m.triangles,
        "boundary": m.boundary,
        "boundary_param": m.boundary_param.iter().map(|(seg, t)| json!([seg, round(*t, 6)])).collect::<Vec<_>>(),
        "inner_edges": m.inner_edges.iter().map(|(inner, chain)| json!({"inner": inner, "vertices": chain})).collect::<Vec<_>>(),
        "build_ms": round(built.ms, 2),
        "cached": !ensured.rebuilt
    });
    if a.get("include_3d").and_then(Value::as_bool).unwrap_or(true) {
        out["layout"] = json!(if panel.place3d.is_some() { "place3d" } else { "flat" });
        out["positions3d"] = json!(layout::positions_3d(m, panel).iter().map(|x| round(f64::from(*x), 3)).collect::<Vec<_>>());
        out["indices"] = json!(layout::indices(m, layout::mirrored_flat(panel)));
    }
    Ok(out)
}

fn stats_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"panels": panels_schema()},
        "required": [],
        "additionalProperties": false
    })
}

fn stats(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = selection(&s.garment, a)?;
    let Session { garment, mesh, sink, .. } = s;
    let mut rows = Vec::with_capacity(ids.len());
    let (mut vertices, mut triangles) = (0usize, 0usize);
    for id in &ids {
        let Some(index) = garment.index_of(*id) else { continue };
        let ensured = mesh.ensure(&garment.panels[index], false);
        match &ensured.built {
            Ok(b) => {
                vertices += b.mesh.positions.len();
                triangles += b.mesh.triangles.len();
                let q = &b.quality;
                rows.push(json!({
                    "panel": id,
                    "vertices": q.vertices,
                    "triangles": q.triangles,
                    "boundary": b.mesh.boundary.len(),
                    "inner_chains": b.mesh.inner_edges.len(),
                    "min_angle": round(q.min_angle, 2),
                    "mean_min_angle": round(q.mean_min_angle, 2),
                    "below_20": q.below_20,
                    "histogram": {"bin_deg": 10, "counts": q.histogram, "labels": ["0-10", "10-20", "20-30", "30-40", "40-50", "50-60"]},
                    "edge": {"min": round(q.edge_min, 3), "mean": round(q.edge_mean, 3), "max": round(q.edge_max, 3)},
                    "area": round(q.area, 2),
                    "build_ms": round(b.ms, 2),
                    "forced_segments": mesh.forced.get(id).map(|f| f.len()).unwrap_or(0),
                    "error": null
                }));
            }
            Err(e) => rows.push(json!({"panel": id, "vertices": 0, "triangles": 0, "error": e})),
        }
    }
    Ok(json!({
        "panels": rows,
        "vertices": vertices,
        "triangles": triangles,
        "cache": {"entries": mesh.cache.len(), "builds": mesh.builds, "hits": mesh.hits},
        "sink": sink.is_some()
    }))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "mesh.build",
            mcp: "mesh_build",
            group: Group::Sim,
            label: "Build Mesh",
            description: "Triangulate panels for simulation: constrained Delaunay at each panel's particle distance with the outline resampled to that edge length, fold and dart inner lines as constraint edges and interior refinement to a 20 degree minimum angle. Meshes are cached per panel by geometry and particle distance, so only changed panels rebuild unless force is set. Returns per-panel vertex and triangle counts and the time taken. Optional segment_counts forces the vertex count of outline segments of one panel so sewn seams match.",
            keys: &[],
            hints: SET,
            schema: build_schema,
            run: build,
        },
        Command {
            id: "mesh.get",
            mcp: "mesh_get",
            group: Group::View,
            label: "Get Mesh",
            description: "One panel's mesh: panel-local 2D positions in mm, triangles, the boundary vertex loop with the outline segment and t of each boundary vertex, inner line vertex chains, and by default the 3D positions and indices the stream carries. Builds the mesh when it is not cached.",
            keys: &[],
            hints: READ,
            schema: get_schema,
            run: get,
        },
        Command {
            id: "mesh.stats",
            mcp: "mesh_stats",
            group: Group::View,
            label: "Mesh Stats",
            description: "Mesh statistics per panel: vertex, triangle and boundary counts, minimum angle, a histogram of per-triangle minimum angles in 10 degree bins, edge length range, area, build time and cache counters.",
            keys: &[],
            hints: READ,
            schema: stats_schema,
            run: stats,
        },
    ]
}
