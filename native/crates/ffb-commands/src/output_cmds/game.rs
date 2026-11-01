use ffb_io::export::thick::border_edges;
use ffb_io::export::{self, Format, Options, SrcObject, Thickness};
use ffb_remesh::{decimate, DecimateOptions};
use serde_json::{json, Value};

use super::export::{common, write_files};
use super::scene::{bad, collect, effective_islands, flag, materials, number, uvs_for};
use crate::{args, Command, CommandError, Group, Session, EXTERNAL};

fn schema() -> Value {
    let mut props = serde_json::Map::new();
    props.insert("path".into(), json!({"type": "string", "description": "Absolute output path. The extension picks the format: .obj, .fbx, .glb, .gltf or .usda."}));
    props.insert("target_tris".into(), json!({"type": "integer", "minimum": 12, "maximum": 5_000_000, "default": 5000, "description": "Triangle budget for the whole merged mesh. Panel borders are locked, so a budget below the border triangles cannot be reached; the result reports what was achieved."}));
    props.insert("quads".into(), json!({"type": "boolean", "default": false, "description": "Reserved. Output is triangulated; use quad.remesh for quad topology."}));
    props.insert("thickness".into(), json!({"type": "boolean", "default": false, "description": "Extrude thickness (thickness.set) after decimation."}));
    props.insert("units".into(), json!({"type": "string", "enum": ["mm", "cm", "m", "in"], "default": "mm"}));
    props.insert("scale".into(), json!({"type": "number", "default": 1.0}));
    props.insert("axis".into(), json!({"type": "string", "enum": ["y_up", "z_up"], "default": "y_up"}));
    json!({"type": "object", "properties": Value::Object(props), "required": ["path"], "additionalProperties": false})
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::file_path(a, "path", None)?;
    let location = std::path::Path::new(&path);
    let extension = location.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let format = match extension.as_str() {
        "obj" => Format::Obj,
        "fbx" => Format::Fbx,
        "glb" => Format::Glb,
        "gltf" => Format::Gltf,
        "usda" | "usd" => Format::Usd,
        _ => return Err(bad("path must end in .obj, .fbx, .glb, .gltf or .usda")),
    };
    let stem = location.file_stem().and_then(|x| x.to_str()).filter(|x| !x.is_empty()).ok_or_else(|| bad("path needs a file name"))?.to_string();
    let dir = location.parent().filter(|p| !p.as_os_str().is_empty()).ok_or_else(|| bad("path has no folder"))?;
    let target = number(a, "target_tris", 12.0, 5_000_000.0)?.map_or(5000, |x| x as usize);
    let common = common(a)?;
    let geos = collect(s, None)?;
    let total: usize = geos.iter().map(|g| g.indices.len() / 3).sum();
    let (islands, _) = effective_islands(&s.garment, &geos);
    let thick_on = flag(a, "thickness").unwrap_or(false);
    let thickness = thick_on.then_some(Thickness { amount: s.garment.output.thickness.amount, sides: s.garment.output.thickness.sides });
    let mats = materials(&s.garment, &geos, thickness.is_some());
    let mut objects: Vec<SrcObject> = Vec::new();
    let mut rows = Vec::new();
    for (i, geo) in geos.iter().enumerate() {
        let tris: Vec<[u32; 3]> = geo.indices.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
        let share = ((target as f64) * (tris.len() as f64) / (total.max(1) as f64)).round() as usize;
        let mut options = DecimateOptions::new(share.max(2));
        let mut lock = vec![false; geo.pos3.len()];
        for (x, y) in border_edges(&geo.indices) {
            lock[x as usize] = true;
            lock[y as usize] = true;
        }
        options.lock = lock;
        let result = decimate(&geo.pos3, &tris, &options);
        let full_uv = uvs_for(geo, &islands[i], "packed");
        let uvs: Vec<[f32; 2]> = result.origin.iter().map(|o| full_uv[*o as usize]).collect();
        rows.push(json!({"panel": geo.id, "before": tris.len(), "after": result.triangles.len()}));
        objects.push(SrcObject {
            name: geo.name.clone(),
            positions: result.positions,
            uvs,
            indices: result.triangles.iter().flatten().copied().collect(),
            front: mats.per_panel[i][0],
            back: mats.per_panel[i][1],
            side: mats.per_panel[i][2],
        });
    }
    let built = export::build(&objects, mats.materials.clone(), thickness.as_ref()).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let options = Options { stem, scale: common.scale, z_up: common.z_up, merged: true, fps: s.garment.anim.fps, meters_per_unit: 0.001 / common.scale, ..Options::default() };
    let files = export::write(format, &built.scene, None, &options).map_err(|e| CommandError::new("export_failed", e))?;
    let written = write_files(dir, &files)?;
    let achieved: usize = objects.iter().map(|o| o.indices.len() / 3).sum();
    Ok(json!({
        "path": path,
        "files": written,
        "triangles_before": total,
        "triangles_target": target,
        "triangles_achieved": achieved,
        "triangles_written": built.scene.triangle_count(),
        "reached": achieved <= target + target / 10,
        "panels": rows,
        "vertices": built.scene.vertex_count(),
        "objects": 1,
        "quads_applied": false,
        "decimator": "ffb-remesh quadric edge collapse with locked panel borders"
    }))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "gameready.run",
        mcp: "gameready_run",
        group: Group::Export,
        label: "Game Ready Export",
        description: "Write one merged, packed-UV mesh reduced to a triangle budget with the quadric decimator of the remesh crate (panel borders locked so seams and silhouette stay). Materials are kept; the format follows the extension (.obj, .fbx, .glb, .gltf, .usda). Reports the triangles achieved against the target. Output is triangulated; quads is reserved.",
        keys: &[],
        hints: EXTERNAL,
        schema,
        run,
    }]
}
