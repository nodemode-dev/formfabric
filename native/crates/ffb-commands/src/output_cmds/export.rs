use ffb_anim::cachefmt::Cache;
use ffb_io::export::uv::{self, Island};
use ffb_io::export::{self, Animation, Format, Object, Options, Thickness};
use serde_json::{json, Value};

use super::scene::{bad, collect, effective_islands, flag, grain_angle, materials, number, source_objects, PanelGeo};
use crate::{args, no_args, Command, CommandError, Group, Session, EXTERNAL, READ};

fn formats(_: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let mut list = export::formats();
    if let Some(items) = list.as_array_mut() {
        for item in items.iter_mut() {
            let id = item["id"].as_str().unwrap_or("").to_string();
            item["label"] = json!(match id.as_str() {
                "obj" => "Wavefront OBJ",
                "gltf" => "glTF 2.0",
                "glb" => "glTF 2.0 binary",
                "fbx" => "Autodesk FBX 7.4",
                "usd" => "USD (usda)",
                "mdd" => "MDD point cache",
                "pc2" => "PC2 point cache",
                _ => "Alembic",
            });
            item["ext"] = item["extension"].clone();
        }
    }
    Ok(json!({"formats": list,"max_bytes": export::MAX_BYTES, "max_frames": export::MAX_FRAMES}))
}

fn run_schema() -> Value {
    let mut props = export::format_options().as_object().cloned().unwrap_or_default();
    props.insert("format".into(), json!({"type": "string", "enum": ["obj", "gltf", "glb", "fbx", "usd", "mdd", "pc2", "abc"], "description": "Output format. abc (Alembic) is not available; the call explains what to use instead."}));
    props.insert("path".into(), json!({"type": "string", "description": "Absolute path of the file to write; the extension must match the format (.usda or .usd for usd). Companion files (mtl, bin, OBJ sequence frames) are written beside it."}));
    props.insert("panels".into(), json!({"type": "array", "items": {"type": "integer"}, "description": "Panel ids to export. Omit for every panel."}));
    props.insert("quads".into(), json!({"type": "boolean", "description": "Write the quad mesh made by quad.remesh (OBJ, FBX and USD keep quads; glTF is triangulated by the format). Defaults to true when the project mesh kind is quad and there is no thickness or animation. Cannot be combined with thickness or animation."}));
    props.insert("binary".into(), json!({"type": "boolean", "default": false, "description": "FBX only: binary FBX 7.4 instead of ASCII."}));
    json!({"type": "object", "properties": Value::Object(props), "required": ["format", "path"], "additionalProperties": false})
}

fn unit_factor(name: &str) -> Option<f64> {
    Some(match name {
        "mm" => 1.0,
        "cm" => 0.1,
        "m" => 0.001,
        "in" => 1.0 / 25.4,
        _ => return None,
    })
}

pub(super) struct Common {
    pub scale: f64,
    pub z_up: bool,
}

pub(super) fn common(a: &Value) -> Result<Common, CommandError> {
    let fallback = if matches!(a.get("format").and_then(Value::as_str), Some("gltf" | "glb")) { "m" } else { "mm" };
    let units = a.get("units").and_then(Value::as_str).unwrap_or(fallback);
    let factor = unit_factor(units).ok_or_else(|| bad(format!("unknown units {units}")))?;
    let multiplier = number(a, "scale", 1e-6, 1e6)?.unwrap_or(1.0);
    let z_up = match a.get("axis").and_then(Value::as_str).unwrap_or("y_up") {
        "y_up" => false,
        "z_up" => true,
        other => return Err(bad(format!("axis must be y_up or z_up, got {other}"))),
    };
    Ok(Common { scale: factor * multiplier, z_up })
}

pub(super) fn panel_filter(s: &Session, a: &Value) -> Result<Option<Vec<u32>>, CommandError> {
    let Some(list) = a.get("panels").and_then(Value::as_array) else { return Ok(None) };
    let mut out = Vec::new();
    for v in list {
        let id = v.as_u64().ok_or_else(|| bad("panels must be integer ids"))? as u32;
        if s.garment.index_of(id).is_none() {
            return Err(CommandError::new("not_found", format!("no panel with id {id}")));
        }
        if !out.contains(&id) {
            out.push(id);
        }
    }
    Ok(Some(out))
}

struct Frames {
    list: Vec<Vec<[f32; 3]>>,
    fps: f32,
    first: u32,
}

fn cache_frames(s: &Session, a: &Value, geos: &[PanelGeo], built: &export::Built) -> Result<Frames, CommandError> {
    let anim = &s.garment.anim;
    let wanted = a.get("cache").and_then(Value::as_u64).map(|x| x as u32).or(anim.active_cache).or_else(|| anim.caches.first().map(|c| c.id));
    let layer = wanted.and_then(|id| anim.caches.iter().find(|c| c.id == id)).ok_or_else(|| CommandError::new("not_found", "there is no cache layer to export; record one with anim.record"))?;
    let (_, bytes) = s.unknown.iter().find(|(n, _)| *n == layer.entry).ok_or_else(|| CommandError::new("not_found", format!("the cache data {} is not in the session", layer.entry)))?;
    let cache = Cache::parse(bytes).map_err(|e| CommandError::new("invalid_cache", e))?;
    let mut slots = Vec::new();
    for geo in geos {
        let at = cache.header.panels.iter().position(|p| p.id == geo.id).ok_or_else(|| CommandError::new("invalid_cache", format!("panel {} is not in the cache", geo.id)))?;
        if cache.header.panels[at].vertices as usize != geo.pos3.len() {
            return Err(CommandError::new("invalid_cache", format!("panel {} has {} vertices now but {} in the cache; re-record the cache", geo.id, geo.pos3.len(), cache.header.panels[at].vertices)));
        }
        slots.push(at);
    }
    let total = cache.header.frames;
    if total == 0 {
        return Err(CommandError::new("invalid_cache", "the cache has no frames"));
    }
    let start = number(a, "start", 0.0, f64::from(total - 1))?.map_or(0, |x| x as u32);
    let end = number(a, "end", 0.0, f64::from(total - 1))?.map_or(total - 1, |x| x as u32);
    let step = number(a, "step", 1.0, f64::from(total))?.map_or(1, |x| x as u32);
    if end < start {
        return Err(bad("end must not be before start"));
    }
    let count = ((end - start) / step + 1) as usize;
    if count > export::MAX_FRAMES {
        return Err(bad(format!("{count} frames is over the limit of {}; raise step", export::MAX_FRAMES)));
    }
    let mut list = Vec::with_capacity(count);
    let mut frame = start;
    while frame <= end {
        let mut raw = Vec::new();
        for slot in &slots {
            let f = cache.panel_frame(frame, *slot).ok_or_else(|| CommandError::new("invalid_cache", "a cache frame is missing"))?;
            raw.extend(f.chunks_exact(3).map(|c| [c[0], c[1], c[2]]));
        }
        list.push(built.frame(&raw).map_err(|e| CommandError::new("invalid_cache", e))?);
        frame += step;
    }
    let base_fps = if cache.header.fps > 0.0 { cache.header.fps } else { anim.fps };
    Ok(Frames { list, fps: base_fps / step as f32, first: layer.start + cache.header.start + start })
}

pub(super) fn write_files(dir: &std::path::Path, files: &[export::File]) -> Result<Vec<Value>, CommandError> {
    let mut written = Vec::new();
    for f in files {
        let path = dir.join(&f.name);
        let text = path.to_str().ok_or_else(|| bad("the path is not valid text"))?.to_string();
        args::write_atomic(&text, &f.bytes)?;
        written.push(json!({"path": text, "bytes": f.bytes.len()}));
    }
    Ok(written)
}

fn quad_swap(s: &mut Session, geos: &[PanelGeo], islands: &[Island], mode: &str, scene: &mut export::Scene) -> usize {
    let meshes = crate::quad_meshes(s);
    let mut total = 0;
    for (i, geo) in geos.iter().enumerate() {
        let Some(qp) = meshes.iter().find(|q| q.panel == geo.id).filter(|q| !q.quads.is_empty() && q.positions.len() == q.positions2d.len()) else { continue };
        let Some(index) = s.garment.index_of(geo.id) else { continue };
        let aligned = uv::aligned(&qp.positions2d, grain_angle(&s.garment.panels[index]));
        let uvs = match mode {
            "tiled" => uv::tiled_uvs(&aligned, geo.map_scale),
            "none" => vec![[0.0, 0.0]; aligned.len()],
            _ => uv::island_uvs(&aligned, &islands[i]),
        };
        let material = scene.objects[i].groups.first().map_or(0, |g| g.material);
        let mut object = Object {
            name: scene.objects[i].name.clone(),
            positions: qp.positions.clone(),
            normals: Vec::new(),
            uvs,
            indices: qp.tris.iter().flatten().copied().collect(),
            quads: qp.quads.clone(),
            groups: vec![export::Group { material, first: 0, count: qp.tris.len() }],
        };
        object.normals = export::vertex_normals(&object.positions, &object.triangulated().indices);
        total += qp.quads.len();
        scene.objects[i] = object;
    }
    total
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = args::string(a, "format")?;
    if name == "abc" {
        return Err(CommandError::new("unsupported", "Alembic (.abc) needs an Ogawa writer, which is not implemented in this build. Export USD (usda) or glTF for animation, or MDD/PC2 point caches, and convert in the target application."));
    }
    let format = Format::parse(name).ok_or_else(|| bad(format!("unknown format {name}")))?;
    let path = args::file_path(a, "path", None)?;
    let location = std::path::Path::new(&path);
    let extension = location.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !format.accepts(extension) {
        return Err(bad(format!("path must end in .{} for {}, got {path}", format.extension(), format.id())));
    }
    let stem = location.file_stem().and_then(|x| x.to_str()).filter(|x| !x.is_empty()).ok_or_else(|| bad("path needs a file name"))?.to_string();
    let dir = location.parent().filter(|p| !p.as_os_str().is_empty()).ok_or_else(|| bad("path has no folder"))?.to_path_buf();
    let common = common(a)?;
    let filter = panel_filter(s, a)?;
    let geos = collect(s, filter.as_deref())?;
    let thick_on = flag(a, "thickness").unwrap_or(s.garment.output.thickness.enabled);
    let thickness = thick_on.then_some(Thickness { amount: s.garment.output.thickness.amount, sides: s.garment.output.thickness.sides });
    let uv_mode = a.get("uv").and_then(Value::as_str).unwrap_or("packed");
    if !["packed", "tiled", "none"].contains(&uv_mode) {
        return Err(bad("uv must be packed, tiled or none"));
    }
    let (islands, _) = effective_islands(&s.garment, &geos);
    let want_materials = flag(a, "materials").unwrap_or(true);
    let mats = materials(&s.garment, &geos, thickness.is_some());
    let objects = source_objects(&geos, &islands, &mats, uv_mode);
    let mut built = export::build(&objects, mats.materials.clone(), thickness.as_ref()).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let want_animation = flag(a, "animation").unwrap_or(false) || matches!(format, Format::Mdd | Format::Pc2);
    let quad_mode = flag(a, "quads").unwrap_or(s.garment.output.mesh == ffb_pattern::output::MeshKind::Quad && thickness.is_none() && !want_animation) && !matches!(format, Format::Mdd | Format::Pc2);
    if quad_mode {
        if thickness.is_some() || want_animation {
            return Err(bad("quads cannot be combined with thickness or animation; pass quads false, or turn thickness off and export without a cache"));
        }
        if quad_swap(s, &geos, &islands, uv_mode, &mut built.scene) == 0 {
            return Err(CommandError::new("not_found", "there is no quad mesh to export; run quad.remesh first or pass quads false"));
        }
    }
    let frames = if want_animation { Some(cache_frames(s, a, &geos, &built)?) } else { None };
    let animation = frames.as_ref().map(|f| Animation { frames: f.list.clone() });
    let options = Options {
        stem,
        scale: common.scale,
        z_up: common.z_up,
        merged: flag(a, "single_object").unwrap_or(false),
        materials: want_materials,
        uv: uv_mode != "none",
        fps: frames.as_ref().map_or(s.garment.anim.fps, |f| f.fps),
        start_frame: frames.as_ref().map_or(0, |f| f.first),
        binary_fbx: flag(a, "binary").unwrap_or(false),
        meters_per_unit: 0.001 / common.scale,
    };
    let files = export::write(format, &built.scene, animation.as_ref(), &options).map_err(|e| CommandError::new("export_failed", e))?;
    let written = write_files(&dir, &files)?;
    Ok(json!({
        "format": format.id(),
        "path": path,
        "files": written,
        "objects": if options.merged { 1 } else { built.scene.objects.len() },
        "vertices": built.scene.vertex_count(),
        "triangles": built.scene.triangle_count(),
        "quads": built.scene.quad_count(),
        "materials": built.scene.materials.len(),
        "frames": animation.as_ref().map_or(0, |a| a.frames.len()),
        "thickness": thickness.is_some(),
        "uv": uv_mode
    }))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "export.formats",
            mcp: "export_formats",
            group: Group::Export,
            label: "Export Formats",
            description: "List the export formats (obj with mtl, gltf, glb, fbx 7.4 ASCII or binary, usd as usda text, mdd, pc2; abc is reported unavailable) with their file sets, how animation is stored and the option schema of each.",
            keys: &[],
            hints: READ,
            schema: no_args,
            run: formats,
        },
        Command {
            id: "export.run",
            mcp: "export_run",
            group: Group::Export,
            label: "Export",
            description: "Write the draped garment to a 3D file. Per panel objects or one merged object, UVs from the stored island layout (or tiled for repeating fabric textures), normals, fabric materials (base colour, roughness, metalness, opacity, texture paths), optional thickness (back face and side strips from thickness.set), scale, units and axis. With animation true the active cache layer (or cache, start, end, step) is written as morph targets (glTF), blend shapes (FBX), time sampled points (USD), an OBJ sequence, or a point cache (MDD, PC2). The path must be absolute with the format's extension; files are written atomically and capped at 1 GiB.",
            keys: &[],
            hints: EXTERNAL,
            schema: run_schema,
            run,
        },
    ]
}
