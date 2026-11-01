use ffb_io::export::uv::{self, Island, DEFAULT_PADDING, MAX_TILES};
use serde_json::{json, Value};

use super::scene::{auto_layout, bad, collect, effective_islands, flag, number, tile_count, to_stored};
use crate::{no_args, Command, CommandError, Edit, Group, Session, READ, SET, WRITE};

fn report(s: &mut Session) -> Result<Value, CommandError> {
    let geos = collect(s, None)?;
    let (islands, stored) = effective_islands(&s.garment, &geos);
    let mut rows = Vec::new();
    let mut area = 0.0f64;
    for (geo, island) in geos.iter().zip(&islands) {
        let size = uv::island_size(&geo.aligned, island.rotation);
        let w = size[0] * f64::from(island.scale);
        let h = size[1] * f64::from(island.scale);
        area += w * h;
        rows.push(json!({
            "panel": geo.id,
            "name": geo.name,
            "offset": island.offset,
            "rotation": island.rotation,
            "scale": island.scale,
            "tile": island.tile,
            "udim": 1001 + island.tile,
            "size_mm": [size[0], size[1]],
            "uv_size": [w, h],
            "bbox": [island.offset[0], island.offset[1], island.offset[0] + w as f32, island.offset[1] + h as f32],
            "outline": [[island.offset[0], island.offset[1]], [island.offset[0] + w as f32, island.offset[1]], [island.offset[0] + w as f32, island.offset[1] + h as f32], [island.offset[0], island.offset[1] + h as f32]],
            "vertices": geo.aligned.len()
        }));
    }
    let tiles = tile_count(&islands);
    let padding = if s.garment.output.uv_padding > 0.0 { s.garment.output.uv_padding } else { DEFAULT_PADDING };
    Ok(json!({"islands": rows, "padding": padding, "tiles": tiles, "coverage": area / f64::from(tiles), "stored": stored}))
}

fn get(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    report(s)
}

fn layout_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "padding": {"type": "number", "minimum": 0.0, "maximum": 0.1, "description": "Gap between islands and the tile border as a fraction of a tile (0.004 is 4 pixels on a 1024 map)."},
            "tiles": {"type": "integer", "minimum": 1, "maximum": MAX_TILES, "default": 1, "description": "Number of UDIM tiles along U (1001, 1002, ...) the islands may use."},
            "rotate": {"type": "boolean", "default": false, "description": "Allow quarter turns for a tighter fit. Off keeps every island grain aligned."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn commit(s: &mut Session, label: &str, islands: Vec<Island>, padding: Option<f32>) -> bool {
    let before = s.garment.clone();
    let mut after = before.clone();
    after.output.islands = islands.iter().map(to_stored).collect();
    if let Some(p) = padding {
        after.output.uv_padding = p;
    }
    if after.output == before.output {
        return false;
    }
    s.garment = after.clone();
    s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
    true
}

fn layout(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let padding = number(a, "padding", 0.0, 0.1)?.map_or(if s.garment.output.uv_padding > 0.0 { s.garment.output.uv_padding } else { DEFAULT_PADDING }, |x| x as f32);
    let tiles = number(a, "tiles", 1.0, f64::from(MAX_TILES))?.map_or(1, |x| x as u32);
    let geos = collect(s, None)?;
    let mut islands = auto_layout(&geos, padding, tiles);
    if flag(a, "rotate").unwrap_or(false) {
        let landscape = |g: &super::scene::PanelGeo| {
            let r = uv::island_size(&g.aligned, 0.0);
            r[1] > r[0] * 1.2
        };
        let rotations: Vec<f32> = geos.iter().map(|g| if landscape(g) { 90.0 } else { 0.0 }).collect();
        let sizes: Vec<(u32, [f64; 2])> = geos.iter().zip(&rotations).map(|(g, r)| (g.id, uv::island_size(&g.aligned, *r))).collect();
        let rotated = uv::pack(&sizes, &rotations, padding, tiles);
        if rotated.first().is_some_and(|r| r.scale > islands[0].scale) {
            islands = rotated;
        }
    }
    let changed = commit(s, "UV layout", islands, Some(padding));
    let mut out = report(s)?;
    out["changed"] = json!(changed);
    Ok(out)
}

fn island_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id."},
            "offset": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "UV position of the island's lower left corner inside its tile."},
            "rotation": {"type": "number", "description": "Rotation of the island in degrees, applied after grain alignment."},
            "scale": {"type": "number", "exclusiveMinimum": 0, "description": "UV units per millimetre."},
            "tile": {"type": "integer", "minimum": 0, "maximum": MAX_TILES - 1, "description": "UDIM tile index along U (0 is 1001)."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn island_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = a.get("panel").and_then(Value::as_u64).ok_or_else(|| bad("panel must be a panel id"))? as u32;
    let geos = collect(s, None)?;
    if !geos.iter().any(|g| g.id == id) {
        return Err(CommandError::new("not_found", format!("no panel with id {id}")));
    }
    let (mut islands, _) = effective_islands(&s.garment, &geos);
    let target = islands.iter_mut().find(|i| i.panel == id).ok_or_else(|| CommandError::new("not_found", format!("no panel with id {id}")))?;
    if let Some(v) = a.get("offset").filter(|v| !v.is_null()) {
        let list = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad("offset must be [u, v]"))?;
        let u = list[0].as_f64().filter(|x| x.is_finite() && x.abs() < 1e6).ok_or_else(|| bad("offset must be finite numbers"))?;
        let w = list[1].as_f64().filter(|x| x.is_finite() && x.abs() < 1e6).ok_or_else(|| bad("offset must be finite numbers"))?;
        target.offset = [u as f32, w as f32];
    }
    if let Some(r) = number(a, "rotation", -3600.0, 3600.0)? {
        target.rotation = r as f32;
    }
    if let Some(sc) = number(a, "scale", 1e-9, 1e3)? {
        target.scale = sc as f32;
    }
    if let Some(t) = number(a, "tile", 0.0, f64::from(MAX_TILES - 1))? {
        target.tile = t as u32;
    }
    let changed = commit(s, "UV island", islands, None);
    let mut out = report(s)?;
    out["changed"] = json!(changed);
    Ok(out)
}

fn thickness_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "enabled": {"type": "boolean", "description": "Extrude the cloth at export (and in bake and game ready output)."},
            "amount": {"type": "number", "minimum": 0.01, "maximum": 50.0, "unit": "mm", "description": "Distance of the back face from the front face along the vertex normals."},
            "sides": {"type": "boolean", "description": "Add side strips along panel borders so the solid is closed."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn thickness_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let before = s.garment.clone();
    let mut after = before.clone();
    if let Some(v) = flag(a, "enabled") {
        after.output.thickness.enabled = v;
    }
    if let Some(v) = number(a, "amount", 0.01, 50.0)? {
        after.output.thickness.amount = v as f32;
    }
    if let Some(v) = flag(a, "sides") {
        after.output.thickness.sides = v;
    }
    let changed = after.output != before.output;
    if changed {
        s.garment = after.clone();
        s.record(Edit::Garment { label: "Thickness".into(), before: Box::new(before), after: Box::new(after) });
    }
    let t = &s.garment.output.thickness;
    Ok(json!({"changed": changed, "thickness": {"enabled": t.enabled, "amount": t.amount, "sides": t.sides}}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "uv.get",
            mcp: "uv_get",
            group: Group::Export,
            label: "UV Islands",
            description: "List the UV island of every panel: offset, rotation, scale (UV units per millimetre), UDIM tile, size in millimetres, and the tile count and coverage. UVs come from each panel's 2D mesh in millimetres rotated so the grain line runs along V. When no layout was stored the automatic layout is reported with stored false.",
            keys: &[],
            hints: READ,
            schema: no_args,
            run: get,
        },
        Command {
            id: "uv.layout",
            mcp: "uv_layout",
            group: Group::Export,
            label: "Pack UV Islands",
            description: "Pack all panel islands with a shelf packer at one common scale (so texel density is equal), with padding and up to 16 UDIM tiles along U. Stored in the project and used by export, bake and game ready output. One undo step.",
            keys: &[],
            hints: SET,
            schema: layout_schema,
            run: layout,
        },
        Command {
            id: "uv.island.set",
            mcp: "uv_island_set",
            group: Group::Export,
            label: "Set UV Island",
            description: "Move, rotate, rescale or change the UDIM tile of one panel's UV island. Panels without a stored island get the automatic layout first. One undo step.",
            keys: &[],
            hints: SET,
            schema: island_schema,
            run: island_set,
        },
        Command {
            id: "thickness.set",
            mcp: "thickness_set",
            group: Group::Export,
            label: "Thickness",
            description: "Set export thickness: on or off, amount in millimetres (back face offset along the normals) and whether border side strips close the solid. Applied when exporting, baking and writing game ready meshes; the simulation mesh is unchanged. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: thickness_schema,
            run: thickness_set,
        },
    ]
}
