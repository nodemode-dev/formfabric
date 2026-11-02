use ffb_pattern::trim::{next_hardware_id, next_style_id};
use std::io::Read;

use ffb_pattern::{Anchor, Hardware, SharedMesh, Style, StyleKind};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, EXTERNAL};

const MAX_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TRIANGLES: usize = 20_000;
#[cfg(not(target_arch = "wasm32"))]
const FBX_MEMORY: usize = 512 * 1024 * 1024;
const MAX_EXTENT: f64 = 2_000.0;

type Mesh = (Vec<[f64; 3]>, Vec<[u32; 3]>);

fn resolve(token: &str, count: usize) -> Result<usize, CommandError> {
    let head = token.split('/').next().unwrap_or("");
    let n: i64 = head.parse().map_err(|_| bad(format!("bad face index {token}")))?;
    let index = if n < 0 { count as i64 + n } else { n - 1 };
    if index < 0 || index as usize >= count {
        return Err(bad(format!("face index {token} is outside the {count} vertices read so far")));
    }
    Ok(index as usize)
}

pub fn parse_obj(source: &str, scale: f64, up_y: bool) -> Result<Mesh, CommandError> {
    let mut raw: Vec<[f64; 3]> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();
    for line in source.lines() {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let v: Vec<f64> = words.take(3).filter_map(|w| w.parse::<f64>().ok()).collect();
                if v.len() != 3 || v.iter().any(|x| !x.is_finite()) {
                    return Err(bad("a vertex line needs three finite numbers"));
                }
                raw.push([v[0], v[1], v[2]]);
            }
            Some("f") => {
                let ids = words.map(|w| resolve(w, raw.len())).collect::<Result<Vec<_>, _>>()?;
                if ids.len() < 3 {
                    return Err(bad("a face needs at least three vertices"));
                }
                for k in 1..ids.len() - 1 {
                    faces.push([ids[0], ids[k], ids[k + 1]]);
                }
                if faces.len() > MAX_TRIANGLES {
                    return Err(bad(format!("the mesh has more than {MAX_TRIANGLES} triangles")));
                }
            }
            _ => {}
        }
    }
    if faces.is_empty() {
        return Err(bad("the file holds no faces"));
    }
    conform(&raw, &faces, scale, up_y)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_fbx(bytes: &[u8], scale: f64) -> Result<Mesh, CommandError> {
    let raw = ffb_body::import::fbx::read_memory(bytes, FBX_MEMORY).map_err(bad)?;
    if raw.triangles.len() > MAX_TRIANGLES {
        return Err(bad(format!("the mesh has more than {MAX_TRIANGLES} triangles")));
    }
    let points: Vec<[f64; 3]> = raw.positions.iter().map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]).collect();
    let faces: Vec<[usize; 3]> = raw.triangles.iter().filter(|t| t.iter().all(|i| (*i as usize) < points.len())).map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).collect();
    if faces.is_empty() {
        return Err(bad("the file holds no faces"));
    }
    conform(&points, &faces, scale, true)
}

#[cfg(target_arch = "wasm32")]
fn parse_fbx(_bytes: &[u8], _scale: f64) -> Result<Mesh, CommandError> {
    Err(bad("FBX import is not available in the browser build"))
}

fn conform(raw: &[[f64; 3]], faces: &[[usize; 3]], scale: f64, up_y: bool) -> Result<Mesh, CommandError> {
    let mut remap: Vec<Option<u32>> = vec![None; raw.len()];
    let mut positions: Vec<[f64; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    for face in faces {
        let mut tri = [0u32; 3];
        for (slot, i) in tri.iter_mut().zip(*face) {
            *slot = *remap[i].get_or_insert_with(|| {
                let [x, y, z] = raw[i];
                positions.push(if up_y { [x * scale, -z * scale, y * scale] } else { [x * scale, y * scale, z * scale] });
                positions.len() as u32 - 1
            });
        }
        if tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2] {
            triangles.push(tri);
        }
    }
    if triangles.is_empty() {
        return Err(bad("every face is degenerate"));
    }
    let lo = positions.iter().fold([f64::MAX; 3], |m, p| [m[0].min(p[0]), m[1].min(p[1]), m[2].min(p[2])]);
    let hi = positions.iter().fold([f64::MIN; 3], |m, p| [m[0].max(p[0]), m[1].max(p[1]), m[2].max(p[2])]);
    if (0..3).any(|k| hi[k] - lo[k] > MAX_EXTENT || !(hi[k] - lo[k]).is_finite()) {
        return Err(bad(format!("the mesh is larger than {MAX_EXTENT} mm after scaling; check scale")));
    }
    let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, lo[2]];
    let round = |x: f64| (x * 100.0).round() / 100.0;
    for p in positions.iter_mut() {
        *p = [round(p[0] - centre[0]), round(p[1] - centre[1]), round(p[2] - centre[2])];
    }
    Ok((positions, triangles))
}

fn schema() -> Value {
    obj(
        json!({
            "path": {"type": "string", "description": "Absolute path of an .obj or .fbx file, at most 8 MB and 20,000 triangles after triangulation. FBX (ASCII or binary) reads every mesh in the file as one trim in world space, takes its unit and up axis from the file header (millimetres and Y up after conversion) and ignores animation and skin."},
            "panel": ident("Panel the trim is attached to."),
            "point": xy("Anchor point on the panel; it must lie inside the outline."),
            "space": space(),
            "rotation": num("Rotation about the surface normal.", -360.0, 360.0, Some("deg")),
            "name": {"type": "string", "description": "Name of the trim style. Defaults to the file name."},
            "scale": num("Millimetres per unit of the file (OBJ), or an extra multiplier on the FBX header unit. Default 1.", 0.000001, 1_000_000.0, None),
            "up": {"type": "string", "enum": ["z", "y"], "default": "z", "description": "OBJ: axis of the file that points out of the cloth. FBX: ignored, the up axis of the file header points out of the cloth."},
            "collision": {"type": "boolean", "default": true, "description": "Whether the trim collides with the cloth."}
        }),
        &["path", "panel", "point"],
    )
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = crate::args::file_path(a, "path", None)?;
    let fbx = match std::path::Path::new(&path).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("obj") => false,
        Some("fbx") => true,
        _ => return Err(bad(format!("path must end in .obj or .fbx, got {path}"))),
    };
    let io = |e: std::io::Error| CommandError::new("io", format!("could not read {path}: {e}"));
    let file = std::fs::File::open(&path).map_err(io)?;
    let meta = file.metadata().map_err(io)?;
    if !meta.is_file() {
        return Err(bad("the path is not a regular file"));
    }
    if meta.len() > MAX_BYTES {
        return Err(bad("the file is larger than 8 MB"));
    }
    let mut bytes = Vec::new();
    std::io::Read::take(file, MAX_BYTES + 1).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(bad("the file is larger than 8 MB"));
    }
    let panel = panel_of(&s.garment, required_id(a, "panel")?)?;
    let point = optional_vec2(a, "point")?.ok_or_else(|| bad("point is required"))?;
    let local = local_point(panel, point, text(a, "space") != Some("local"));
    if !inside(panel, local) {
        return Err(geometry(format!("the point lies outside panel {}", panel.id)));
    }
    let panel_id = panel.id;
    let scale = number_or(a, "scale", 1.0);
    let up_y = fbx || text(a, "up") == Some("y");
    let (positions, triangles) = if fbx { parse_fbx(&bytes, scale)? } else { parse_obj(&String::from_utf8_lossy(&bytes), scale, up_y)? };
    let span = |k: usize| positions.iter().map(|p| p[k]).fold(f64::MIN, f64::max) - positions.iter().map(|p| p[k]).fold(f64::MAX, f64::min);
    let (width, thickness) = (span(0).max(span(1)), span(2));
    let stem = std::path::Path::new(&path).file_stem().and_then(|n| n.to_str()).unwrap_or("Trim").to_string();
    let name = clean_text(a, "name")?.unwrap_or(stem);
    let style_id = next_style_id(&s.garment);
    let source = std::path::Path::new(&path).file_name().and_then(|n| n.to_str()).unwrap_or("trim.obj").to_string();
    let params = json!({
        "source": source, "scale": scale, "up": if up_y { "y" } else { "z" }, "collision": flag(a, "collision").unwrap_or(true),
        "width": width, "thickness": thickness, "weight": 2.0, "stiffness": 1.0
    });
    let mesh = Some(SharedMesh(std::sync::Arc::new(json!({"positions": positions, "triangles": triangles}))));
    let before = s.garment.clone();
    s.garment.styles.push(Style { id: style_id, kind: StyleKind::Trim, name, params, mesh });
    let hardware = Hardware { id: next_hardware_id(&s.garment), style: style_id, anchor: Anchor { panel: panel_id, point: local }, rotation: number_or(a, "rotation", 0.0), fastened_to: None };
    let out = json!({"hardware": hardware.id, "style": style_id, "vertices": positions.len(), "triangles": triangles.len(), "width": width, "thickness": thickness, "anchor": {"panel": panel_id, "point": local}});
    s.garment.hardware.push(hardware);
    record(s, "Import Trim", before);
    Ok(out)
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "trim.import",
        mcp: "trim_import",
        group: Group::Sew,
        label: "Import Trim",
        description: "Import an OBJ or FBX file as a rigid trim glued to a panel point. The mesh is recentred (x and y on the anchor, base at z 0), scaled to millimetres and stored inside the document as a trim style, so the project does not depend on the file afterwards. One undo step.",
        keys: &[],
        hints: EXTERNAL,
        schema,
        run,
    }]
}
