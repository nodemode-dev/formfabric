use ffb_pattern::viz::{Fur, Light, LightKind, SavedView};
use serde_json::{json, Value};

use super::store::edit;
use crate::{args, Command, CommandError, Group, Session, READ, WRITE};

const MAX_LIGHTS: usize = 16;
const MAX_VIEWS: usize = 64;
const MAX_FUR: usize = 64;
const MAX_CAMERA_BYTES: usize = 4096;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

fn number(a: &Value, key: &str, lo: f64, hi: f64) -> Result<Option<f32>, CommandError> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => {
            let x = v.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi).ok_or_else(|| bad(format!("{key} must be a number from {lo} to {hi}")))?;
            Ok(Some(x as f32))
        }
    }
}

fn vec3(a: &Value, key: &str, lo: f64, hi: f64) -> Result<Option<[f32; 3]>, CommandError> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => {
            let list = v.as_array().filter(|l| l.len() == 3).ok_or_else(|| bad(format!("{key} must be three numbers")))?;
            let mut out = [0.0f32; 3];
            for (o, x) in out.iter_mut().zip(list) {
                *o = x.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi).ok_or_else(|| bad(format!("{key} entries must be numbers from {lo} to {hi}")))? as f32;
            }
            Ok(Some(out))
        }
    }
}

fn id_of(a: &Value) -> Result<u32, CommandError> {
    a.get("id").and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok()).ok_or_else(|| bad("id must be a positive integer"))
}

fn vec3_schema(what: &str) -> Value {
    json!({"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "description": what})
}

fn stage_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "hdri": {"type": ["string", "null"], "description": "Environment image name or path; null clears it."},
            "hdri_rotation": {"type": "number", "description": "Environment rotation in degrees."},
            "exposure": {"type": "number", "minimum": -10, "maximum": 10, "description": "Exposure in stops."},
            "ground": {"type": "boolean", "description": "Show the ground plane."},
            "background": {"description": "Background: a CSS colour string, a colour array, or an object the viewer understands."}
        },
        "additionalProperties": false
    })
}

fn stage_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let rotation = number(a, "hdri_rotation", -3600.0, 3600.0)?;
    let exposure = number(a, "exposure", -10.0, 10.0)?;
    let hdri = match a.get("hdri") {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::String(t)) => {
            args::clean_name("hdri", t)?;
            let lower = t.to_ascii_lowercase();
            if t.len() > 1024 || ["http:", "https:", "data:", "blob:", "file:", "ftp:"].iter().any(|p| lower.starts_with(p)) {
                return Err(bad("hdri must be a local file path of at most 1024 bytes; URLs are not accepted"));
            }
            Some(Some(t.clone()))
        }
        Some(_) => return Err(bad("hdri must be a string or null")),
    };
    let ground = match a.get("ground") {
        None => None,
        Some(v) => Some(v.as_bool().ok_or_else(|| bad("ground must be true or false"))?),
    };
    let background = a.get("background").cloned();
    if background.as_ref().is_some_and(|b| b.to_string().len() > 1024) {
        return Err(bad("background is too large"));
    }
    edit(s, "Stage", |g| {
        let st = &mut g.viz.stage;
        if let Some(h) = hdri {
            st.hdri = h;
        }
        if let Some(r) = rotation {
            st.hdri_rotation = r;
        }
        if let Some(e) = exposure {
            st.exposure = e;
        }
        if let Some(v) = ground {
            st.ground = v;
        }
        if let Some(b) = background {
            st.background = b;
        }
        Ok(json!(st))
    })
}

fn light_schema(with_id: bool) -> Value {
    let mut props = json!({
        "kind": {"type": "string", "enum": ["directional", "point", "spot", "area"]},
        "position": vec3_schema("World position in mm."),
        "direction": vec3_schema("Direction the light points along."),
        "color": vec3_schema("Linear RGB from 0 to 1."),
        "intensity": {"type": "number", "minimum": 0, "maximum": 1000},
        "shadows": {"type": "boolean"}
    });
    let mut required: Vec<&str> = Vec::new();
    if with_id {
        props["id"] = json!({"type": "integer", "minimum": 1, "description": "Light id from viz.get."});
        required.push("id");
    }
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

fn kind_of(a: &Value) -> Result<Option<LightKind>, CommandError> {
    Ok(match a.get("kind").and_then(Value::as_str) {
        None => None,
        Some("directional") => Some(LightKind::Directional),
        Some("point") => Some(LightKind::Point),
        Some("spot") => Some(LightKind::Spot),
        Some("area") => Some(LightKind::Area),
        Some(other) => return Err(bad(format!("unknown light kind {other}"))),
    })
}

struct LightPatch {
    kind: Option<LightKind>,
    position: Option<[f32; 3]>,
    direction: Option<[f32; 3]>,
    color: Option<[f32; 3]>,
    intensity: Option<f32>,
    shadows: Option<bool>,
}

fn light_patch(a: &Value) -> Result<LightPatch, CommandError> {
    let shadows = match a.get("shadows") {
        None => None,
        Some(v) => Some(v.as_bool().ok_or_else(|| bad("shadows must be true or false"))?),
    };
    Ok(LightPatch {
        kind: kind_of(a)?,
        position: vec3(a, "position", -1.0e6, 1.0e6)?,
        direction: vec3(a, "direction", -1.0e6, 1.0e6)?,
        color: vec3(a, "color", 0.0, 1.0)?,
        intensity: number(a, "intensity", 0.0, 1000.0)?,
        shadows,
    })
}

fn apply_light(l: &mut Light, p: &LightPatch) {
    if let Some(k) = p.kind {
        l.kind = k;
    }
    if let Some(v) = p.position {
        l.position = v;
    }
    if let Some(v) = p.direction {
        l.direction = v;
    }
    if let Some(v) = p.color {
        l.color = v;
    }
    if let Some(v) = p.intensity {
        l.intensity = v;
    }
    if let Some(v) = p.shadows {
        l.shadows = v;
    }
}

fn next_id(ids: impl Iterator<Item = u32>) -> u32 {
    ids.max().unwrap_or(0) + 1
}

fn light_add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let patch = light_patch(a)?;
    edit(s, "Add Light", |g| {
        if g.viz.stage.lights.len() >= MAX_LIGHTS {
            return Err(bad(format!("at most {MAX_LIGHTS} lights")));
        }
        let mut light = Light { id: next_id(g.viz.stage.lights.iter().map(|l| l.id)), direction: [0.0, -1.0, 0.0], color: [1.0, 1.0, 1.0], intensity: 1.0, ..Light::default() };
        apply_light(&mut light, &patch);
        let out = json!(light);
        g.viz.stage.lights.push(light);
        Ok(out)
    })
}

fn light_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (id, patch) = (id_of(a)?, light_patch(a)?);
    edit(s, "Edit Light", |g| {
        let light = g.viz.stage.lights.iter_mut().find(|l| l.id == id).ok_or_else(|| missing(format!("no light with id {id}")))?;
        apply_light(light, &patch);
        Ok(json!(light))
    })
}

fn id_schema() -> Value {
    json!({"type": "object", "properties": {"id": {"type": "integer", "minimum": 1}}, "required": ["id"], "additionalProperties": false})
}

fn light_remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a)?;
    edit(s, "Remove Light", |g| {
        let at = g.viz.stage.lights.iter().position(|l| l.id == id).ok_or_else(|| missing(format!("no light with id {id}")))?;
        g.viz.stage.lights.remove(at);
        Ok(json!({"removed": id}))
    })
}

fn view_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 1, "maxLength": 80},
            "camera": {"type": "object", "description": "Camera and display state exactly as the viewer reports it."}
        },
        "required": ["name", "camera"],
        "additionalProperties": false
    })
}

fn view_save(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = args::string(a, "name")?.trim().to_string();
    args::clean_name("name", &name)?;
    if name.is_empty() {
        return Err(bad("name must not be empty"));
    }
    let camera = a.get("camera").filter(|c| c.is_object()).cloned().ok_or_else(|| bad("camera must be an object"))?;
    if camera.to_string().len() > MAX_CAMERA_BYTES {
        return Err(bad(format!("camera is larger than {MAX_CAMERA_BYTES} bytes")));
    }
    edit(s, "Save View", |g| {
        if let Some(v) = g.viz.views.iter_mut().find(|v| v.name == name) {
            v.camera = camera;
            return Ok(json!(v));
        }
        if g.viz.views.len() >= MAX_VIEWS {
            return Err(bad(format!("at most {MAX_VIEWS} saved views")));
        }
        let view = SavedView { id: next_id(g.viz.views.iter().map(|v| v.id)), name, camera };
        let out = json!(view);
        g.viz.views.push(view);
        Ok(out)
    })
}

fn view_remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a)?;
    edit(s, "Remove View", |g| {
        let at = g.viz.views.iter().position(|v| v.id == id).ok_or_else(|| missing(format!("no view with id {id}")))?;
        g.viz.views.remove(at);
        Ok(json!({"removed": id}))
    })
}

fn fur_schema(with_id: bool) -> Value {
    let mut props = json!({
        "panel": {"type": "integer", "minimum": 1, "description": "Panel id the fur grows on."},
        "length": {"type": "number", "minimum": 0, "maximum": 500, "description": "Strand length in mm."},
        "density": {"type": "number", "minimum": 0, "maximum": 100000, "description": "Strands per square centimetre."},
        "thickness": {"type": "number", "minimum": 0, "maximum": 10, "description": "Strand thickness in mm."},
        "curl": {"type": "number", "minimum": 0, "maximum": 1},
        "color": vec3_schema("Linear RGB from 0 to 1.")
    });
    let mut required: Vec<&str> = Vec::new();
    if with_id {
        props["id"] = json!({"type": "integer", "minimum": 1});
        required.push("id");
    } else {
        required.push("panel");
    }
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

struct FurPatch {
    panel: Option<u32>,
    length: Option<f32>,
    density: Option<f32>,
    thickness: Option<f32>,
    curl: Option<f32>,
    color: Option<[f32; 3]>,
}

fn fur_patch(s: &Session, a: &Value) -> Result<FurPatch, CommandError> {
    let panel = match a.get("panel") {
        None => None,
        Some(v) => {
            let id = v.as_u64().and_then(|x| u32::try_from(x).ok()).ok_or_else(|| bad("panel must be a panel id"))?;
            if s.garment.index_of(id).is_none() {
                return Err(missing(format!("no panel with id {id}")));
            }
            Some(id)
        }
    };
    Ok(FurPatch {
        panel,
        length: number(a, "length", 0.0, 500.0)?,
        density: number(a, "density", 0.0, 100_000.0)?,
        thickness: number(a, "thickness", 0.0, 10.0)?,
        curl: number(a, "curl", 0.0, 1.0)?,
        color: vec3(a, "color", 0.0, 1.0)?,
    })
}

fn apply_fur(f: &mut Fur, p: &FurPatch) {
    if let Some(v) = p.panel {
        f.panel = v;
    }
    if let Some(v) = p.length {
        f.length = v;
    }
    if let Some(v) = p.density {
        f.density = v;
    }
    if let Some(v) = p.thickness {
        f.thickness = v;
    }
    if let Some(v) = p.curl {
        f.curl = v;
    }
    if let Some(v) = p.color {
        f.color = v;
    }
}

fn fur_add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let patch = fur_patch(s, a)?;
    edit(s, "Add Fur", |g| {
        if g.viz.fur.len() >= MAX_FUR {
            return Err(bad(format!("at most {MAX_FUR} fur layers")));
        }
        let mut fur = Fur { id: next_id(g.viz.fur.iter().map(|f| f.id)), length: 10.0, density: 200.0, thickness: 0.1, color: [0.8, 0.8, 0.8], ..Fur::default() };
        apply_fur(&mut fur, &patch);
        let out = json!(fur);
        g.viz.fur.push(fur);
        Ok(out)
    })
}

fn fur_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (id, patch) = (id_of(a)?, fur_patch(s, a)?);
    edit(s, "Edit Fur", |g| {
        let fur = g.viz.fur.iter_mut().find(|f| f.id == id).ok_or_else(|| missing(format!("no fur with id {id}")))?;
        apply_fur(fur, &patch);
        Ok(json!(fur))
    })
}

fn fur_remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a)?;
    edit(s, "Remove Fur", |g| {
        let at = g.viz.fur.iter().position(|f| f.id == id).ok_or_else(|| missing(format!("no fur with id {id}")))?;
        g.viz.fur.remove(at);
        Ok(json!({"removed": id}))
    })
}

fn get(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    Ok(json!(s.garment.viz))
}

fn light_add_schema() -> Value {
    light_schema(false)
}

fn light_set_schema() -> Value {
    light_schema(true)
}

fn fur_add_schema() -> Value {
    fur_schema(false)
}

fn fur_set_schema() -> Value {
    fur_schema(true)
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command { id: "viz.get", mcp: "viz_get", group: Group::View, label: "Visualization State", description: "Read Garment.viz: stage, lights, saved views, fur, the active map and the compare and measure definitions.", keys: &[], hints: READ, schema: crate::no_args, run: get },
        Command { id: "stage.set", mcp: "stage_set", group: Group::View, label: "Stage", description: "Set the environment image, its rotation, exposure in stops, the ground plane and the background. Only the given fields change. An undo step.", keys: &[], hints: WRITE, schema: stage_schema, run: stage_set },
        Command { id: "light.add", mcp: "light_add", group: Group::View, label: "Add Light", description: "Add a light (directional, point, spot or area) to the stage. Returns it with its id. At most 16. An undo step.", keys: &[], hints: WRITE, schema: light_add_schema, run: light_add },
        Command { id: "light.set", mcp: "light_set", group: Group::View, label: "Edit Light", description: "Change the given fields of one light. An undo step.", keys: &[], hints: WRITE, schema: light_set_schema, run: light_set },
        Command { id: "light.remove", mcp: "light_remove", group: Group::View, label: "Remove Light", description: "Remove one light by id. An undo step.", keys: &[], hints: WRITE, schema: id_schema, run: light_remove },
        Command { id: "view.save", mcp: "view_save", group: Group::View, label: "Save View", description: "Save the camera and display state under a name; saving over a name replaces that view. An undo step.", keys: &[], hints: WRITE, schema: view_schema, run: view_save },
        Command { id: "view.remove", mcp: "view_remove", group: Group::View, label: "Remove View", description: "Remove one saved view by id. An undo step.", keys: &[], hints: WRITE, schema: id_schema, run: view_remove },
        Command { id: "fur.add", mcp: "fur_add", group: Group::View, label: "Add Fur", description: "Grow fur strands on a panel (length mm, density per cm2, thickness mm, curl 0 to 1, colour). An undo step.", keys: &[], hints: WRITE, schema: fur_add_schema, run: fur_add },
        Command { id: "fur.set", mcp: "fur_set", group: Group::View, label: "Edit Fur", description: "Change the given fields of one fur layer. An undo step.", keys: &[], hints: WRITE, schema: fur_set_schema, run: fur_set },
        Command { id: "fur.remove", mcp: "fur_remove", group: Group::View, label: "Remove Fur", description: "Remove one fur layer by id. An undo step.", keys: &[], hints: WRITE, schema: id_schema, run: fur_remove },
    ]
}
