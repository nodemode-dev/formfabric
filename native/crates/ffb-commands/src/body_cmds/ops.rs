use ffb_body::import::{self, Options, Units};
use ffb_body::math::{self, Q};
use ffb_body::resize::{self, Targets};
use ffb_body::{light_copy, measure, pose, skin, Body, BodyChange};
use serde_json::{json, Map, Value};

use super::stream::OBJECT_BASE;
use crate::{Command, CommandError, Edit, Group, Session, DESTROY, EXTERNAL, SET, WRITE};

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub(super) fn id_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"id": {"type": "integer", "description": "Body id. Defaults to the first body."}},
        "required": [],
        "additionalProperties": false
    })
}

pub(super) fn pick(s: &Session, a: &Value) -> Result<usize, CommandError> {
    match a.get("id").and_then(Value::as_u64) {
        Some(id) => s.bodies.iter().position(|b| u64::from(b.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no body with id {id}"))),
        None if a.get("id").is_some_and(|v| !v.is_null()) => Err(bad("id must be a whole number")),
        None if s.bodies.is_empty() => Err(CommandError::new("not_found", "there are no bodies; use body.load_default or body.import")),
        None => Ok(0),
    }
}

pub(super) fn summary(b: &Body) -> Value {
    let posed: Vec<&String> = b.pose.iter().filter(|(_, q)| (math::qnorm(**q)[3].abs() - 1.0).abs() > 1e-6).map(|(k, _)| k).collect();
    json!({
        "id": b.id,
        "name": b.name,
        "object": OBJECT_BASE + b.id,
        "source": b.source,
        "license": b.license,
        "vertices": b.positions.len(),
        "triangles": b.triangles.len(),
        "joints": b.joints.len(),
        "visible": b.visible,
        "skin_offset": b.skin_offset,
        "friction": b.friction,
        "posed_joints": posed,
        "measurements": b.measurements,
    })
}

fn next_id(s: &Session) -> u32 {
    s.bodies.iter().map(|b| b.id).max().unwrap_or(0) + 1
}

fn record(s: &mut Session, label: &str, changes: Vec<BodyChange>) {
    s.record(Edit::Bodies { label: label.into(), changes });
}

fn add(s: &mut Session, label: &str, mut body: Body) -> Value {
    body.id = next_id(s);
    let out = summary(&body);
    let change = BodyChange { id: body.id, index: s.bodies.len(), before: None, after: Some(Box::new(body.clone())), light: false };
    s.bodies.push(body);
    record(s, label, vec![change]);
    out
}

fn load_default(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = a.get("name").and_then(Value::as_str).unwrap_or(ffb_body::FORM_NAME);
    let body = if name.trim().eq_ignore_ascii_case(ffb_body::FORM_NAME) {
        ffb_body::basic_form()
    } else {
        let entry = super::library::find(name).ok_or_else(|| CommandError::new("not_found", format!("no default body named {name}; body.list shows the library")))?;
        let options = Options { units: Units::Auto, up: math::IDENTITY, ground: true, name: Some(entry.name.clone()) };
        let mut body = import::read(&entry.path.to_string_lossy(), &options).map_err(|e| CommandError::new("import_failed", e))?;
        body.license = entry.license.clone();
        body.source = Some(entry.path.to_string_lossy().into_owned());
        body
    };
    Ok(add(s, "Load Body", body))
}

fn import_body(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = crate::args::file_path(a, "path", None)?;
    let path = path.as_str();
    if let Some(name) = a.get("name").and_then(Value::as_str) {
        crate::args::clean_name("name", name)?;
    }
    if !std::path::Path::new(path).exists() {
        return Err(CommandError::new("not_found", format!("no file at {path}")));
    }
    let units = match a.get("units").and_then(Value::as_str) {
        Some(u) => Units::parse(u).ok_or_else(|| bad("units must be auto, mm, cm, m, in or ft"))?,
        None => Units::Auto,
    };
    let up = match a.get("up").and_then(Value::as_str) {
        Some(u) => import::up_rotation(u).ok_or_else(|| bad("up must be y, -y, z, -z, x or -x"))?,
        None => math::IDENTITY,
    };
    let options = Options { units, up, ground: a.get("ground").and_then(Value::as_bool).unwrap_or(true), name: a.get("name").and_then(Value::as_str).map(str::to_string) };
    let mut body = import::read(path, &options).map_err(|e| CommandError::new("import_failed", e))?;
    body.source = Some(path.to_string());
    Ok(add(s, "Import Body", body))
}

fn remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let body = s.bodies.remove(i);
    let out = json!({"removed": body.id, "name": body.name});
    let change = BodyChange { id: body.id, index: i, before: Some(Box::new(body)), after: None, light: false };
    record(s, "Remove Body", vec![change]);
    Ok(out)
}

fn light_change(s: &mut Session, i: usize, label: &str, edit: impl FnOnce(&mut Body)) -> bool {
    let before = light_copy(&s.bodies[i]);
    edit(&mut s.bodies[i]);
    if before == light_copy(&s.bodies[i]) {
        return false;
    }
    let after = light_copy(&s.bodies[i]);
    let change = BodyChange { id: s.bodies[i].id, index: i, before: Some(Box::new(before)), after: Some(Box::new(after)), light: true };
    record(s, label, vec![change]);
    true
}

fn show(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let visible = a.get("visible").and_then(Value::as_bool).unwrap_or(true);
    let indices: Vec<usize> = if a.get("id").is_some_and(|v| !v.is_null()) { vec![pick(s, a)?] } else { (0..s.bodies.len()).collect() };
    let mut changes = Vec::new();
    for i in indices {
        let before = light_copy(&s.bodies[i]);
        s.bodies[i].visible = visible;
        if before.visible != visible {
            let after = light_copy(&s.bodies[i]);
            changes.push(BodyChange { id: s.bodies[i].id, index: i, before: Some(Box::new(before)), after: Some(Box::new(after)), light: true });
        }
    }
    let changed = changes.len();
    if changed > 0 {
        record(s, if visible { "Show Body" } else { "Hide Body" }, changes);
    }
    Ok(json!({"visible": visible, "changed": changed}))
}

fn range(a: &Value, key: &str, lo: f64, hi: f64) -> Result<Option<f64>, CommandError> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => {
            let x = v.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi).ok_or_else(|| bad(format!("{key} must be a number from {lo} to {hi}")))?;
            Ok(Some(x))
        }
    }
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let offset = range(a, "skin_offset", 0.0, 100.0)?;
    let friction = range(a, "friction", 0.0, 10.0)?;
    let name = a.get("name").and_then(Value::as_str).map(str::to_string);
    if let Some(name) = &name {
        crate::args::clean_name("name", name)?;
    }
    let changed = light_change(s, i, "Body Settings", |b| {
        if let Some(x) = offset {
            b.skin_offset = x;
        }
        if let Some(x) = friction {
            b.friction = x;
        }
        if let Some(n) = name {
            b.name = n;
        }
    });
    let mut out = summary(&s.bodies[i]);
    out["changed"] = json!(changed);
    Ok(out)
}

fn resize_body(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let g = |k: &str| range(a, k, 100.0, 5000.0);
    let chest = match (g("chest")?, g("bust")?) {
        (Some(c), Some(b)) if (c - b).abs() > 0.01 => return Err(bad("chest and bust are the same measurement; give one")),
        (c, b) => c.or(b),
    };
    let targets = Targets { height: g("height")?, inseam: g("inseam")?, chest, waist: g("waist")?, hip: g("hip")?, shoulder_width: g("shoulder_width")?, arm_length: g("arm_length")? };
    if !targets.any() {
        return Err(bad("give at least one target: height, inseam, chest, waist, hip, shoulder_width or arm_length"));
    }
    let before = s.bodies[i].clone();
    let report = resize::resize(&mut s.bodies[i], &targets).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let after = s.bodies[i].clone();
    let change = BodyChange { id: after.id, index: i, before: Some(Box::new(before)), after: Some(Box::new(after)), light: false };
    record(s, "Resize Body", vec![change]);
    Ok(json!({"id": s.bodies[i].id, "iterations": report.iterations, "before": report.before, "after": report.after}))
}

fn quaternion(name: &str, v: &Value) -> Result<Q, CommandError> {
    let nums = |v: &Value, n: usize| -> Option<Vec<f32>> {
        let list = v.as_array()?;
        let out: Vec<f32> = list.iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect();
        (out.len() == n && list.len() == n && out.iter().all(|x| x.is_finite())).then_some(out)
    };
    if let Some(q) = nums(v, 4) {
        return Ok(math::qnorm([q[0], q[1], q[2], q[3]]));
    }
    if let Some(e) = v.get("euler").and_then(|e| nums(e, 3)) {
        return Ok(math::qeuler([e[0], e[1], e[2]]));
    }
    if let (Some(axis), Some(angle)) = (v.get("axis").and_then(|x| nums(x, 3)), v.get("angle").and_then(Value::as_f64)) {
        return Ok(math::qaxis([axis[0], axis[1], axis[2]], (angle as f32).to_radians()));
    }
    Err(bad(format!("joint {name} needs [x, y, z, w], {{\"euler\": [x, y, z]}} in degrees or {{\"axis\": [x, y, z], \"angle\": degrees}}")))
}

fn pose_body(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let mut next = if a.get("replace").and_then(Value::as_bool).unwrap_or(false) || a.get("name").is_some() { std::collections::BTreeMap::new() } else { s.bodies[i].pose.clone() };
    if let Some(name) = a.get("name").and_then(Value::as_str) {
        next = pose::named(&s.bodies[i], name).map_err(bad)?;
    }
    if let Some(map) = a.get("joints") {
        let map = map.as_object().ok_or_else(|| bad("joints must be an object of joint name to rotation"))?;
        for (k, v) in map {
            if !s.bodies[i].joints.iter().any(|j| &j.name == k) {
                return Err(CommandError::new("not_found", format!("no joint named {k}")));
            }
            next.insert(k.clone(), quaternion(k, v)?);
        }
    } else if a.get("name").is_none() && a.get("replace").and_then(Value::as_bool) != Some(true) {
        return Err(bad("give a pose name (rest, A, T) or joints"));
    }
    let changed = light_change(s, i, "Pose Body", |b| {
        b.pose = next;
        b.measurements = measure::measure(b);
    });
    let b = &s.bodies[i];
    let posed: Map<String, Value> = b.pose.iter().map(|(k, q)| (k.clone(), json!(q))).collect();
    Ok(json!({"id": b.id, "changed": changed, "pose": posed, "measurements": b.measurements, "vertices": skin::posed_positions(b).len()}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "body.load_default",
            mcp: "body_load_default",
            group: Group::Body,
            label: "Add Body",
            description: "Add a default body to the project: the built-in Basic Form (formfabric's own parametric body, about 20k triangles, 20 joints, skin weights) or a library body from assets/bodies by name. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: || json!({"type": "object", "properties": {"name": {"type": "string", "description": "Body name; default Basic Form. body.list shows the library."}}, "required": [], "additionalProperties": false}),
            run: load_default,
        },
        Command {
            id: "body.import",
            mcp: "body_import",
            group: Group::Body,
            label: "Import Body",
            description: "Import a body from an OBJ, glTF, GLB or FBX file (FBX on native builds only), convert to millimetres with y up and stand it on the ground. Skins and skeletons are read from glTF and FBX; Alembic is deferred. One undo step.",
            keys: &[],
            hints: EXTERNAL,
            schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path of the body."},
                        "units": {"type": "string", "enum": ["auto", "mm", "cm", "m", "in", "ft"], "description": "Units of the file; auto reads glTF as metres and guesses OBJ from its height."},
                        "up": {"type": "string", "enum": ["y", "-y", "z", "-z", "x", "-x"], "description": "Up axis of the file; default y."},
                        "name": {"type": "string", "description": "Name for the body; default the file name."},
                        "ground": {"type": "boolean", "description": "Stand the body on y = 0; default true."}
                    },
                    "required": ["path"],
                    "additionalProperties": false
                })
            },
            run: import_body,
        },
        Command {
            id: "body.remove",
            mcp: "body_remove",
            group: Group::Body,
            label: "Remove Body",
            description: "Remove a body from the project. One undo step.",
            keys: &[],
            hints: DESTROY,
            schema: id_schema,
            run: remove,
        },
        Command {
            id: "body.show",
            mcp: "body_show",
            group: Group::Body,
            label: "Show Body",
            description: "Show or hide a body (all bodies when no id is given). A hidden body is removed from the 3D view but stays in the project.",
            keys: &[],
            hints: SET,
            schema: || json!({"type": "object", "properties": {"id": {"type": "integer"}, "visible": {"type": "boolean", "description": "Default true."}}, "required": [], "additionalProperties": false}),
            run: show,
        },
        Command {
            id: "body.set",
            mcp: "body_set",
            group: Group::Body,
            label: "Body Collision",
            description: "Set a body's collision skin offset in millimetres (0 to 100, default 3), friction (0 to 10) or name.",
            keys: &[],
            hints: SET,
            schema: || {
                json!({
                    "type": "object",
                    "properties": {"id": {"type": "integer"}, "skin_offset": {"type": "number"}, "friction": {"type": "number"}, "name": {"type": "string"}},
                    "required": [],
                    "additionalProperties": false
                })
            },
            run: set,
        },
        Command {
            id: "body.resize",
            mcp: "body_resize",
            group: Group::Body,
            label: "Body Editor",
            description: "Deform a body to target measurements in millimetres: height and inseam (vertical scale), chest (or bust), waist and hip (cross-sections scaled around the vertical axis with smooth falloff), shoulder width and arm length. The skeleton moves with its region. Targets are met within 1 mm. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "height": {"type": "number"},
                        "inseam": {"type": "number"},
                        "chest": {"type": "number"},
                        "bust": {"type": "number", "description": "Same as chest."},
                        "waist": {"type": "number"},
                        "hip": {"type": "number"},
                        "shoulder_width": {"type": "number"},
                        "arm_length": {"type": "number"}
                    },
                    "required": [],
                    "additionalProperties": false
                })
            },
            run: resize_body,
        },
        Command {
            id: "body.pose",
            mcp: "body_pose",
            group: Group::Body,
            label: "Pose Body",
            description: "Pose a body with linear blend skinning: a named pose (rest, A, T) or joint rotations as quaternions [x, y, z, w], {euler: degrees} or {axis, angle}, applied on top of the rest pose. Joints given alone merge into the current pose; replace clears it first. One undo step.",
            keys: &[],
            hints: SET,
            schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "integer"},
                        "name": {"type": "string", "description": "rest, A or T."},
                        "joints": {"type": "object", "description": "Joint name to rotation."},
                        "replace": {"type": "boolean"}
                    },
                    "required": [],
                    "additionalProperties": false
                })
            },
            run: pose_body,
        },
    ]
}
