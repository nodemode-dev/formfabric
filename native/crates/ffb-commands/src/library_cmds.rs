pub(crate) mod store;

use ffb_anim::clip::Clip;
use ffb_pattern::{Style, StyleKind};
use serde_json::{json, Map, Value};

use crate::{args, b64, dispatch, Command, CommandError, Group, Session, DESTROY, EXTERNAL, READ, WRITE};
use store::Kind;

const KINDS: [&str; 7] = ["fabric", "trim", "style", "component", "garment", "pose", "motion"];
const MAGIC: &[u8; 8] = b"FFMOTION";
const MAX_HEADER: usize = 1 << 20;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn kind_arg(a: &Value) -> Result<Kind, CommandError> {
    let text = args::string(a, "kind")?;
    Kind::parse(text).ok_or_else(|| bad(format!("kind must be one of {}", KINDS.join(", "))))
}

fn name_arg(a: &Value) -> Result<String, CommandError> {
    let name = args::string(a, "name")?;
    ffb_fabric::validate::name(name).map_err(bad)
}

fn from_id(a: &Value) -> Result<Option<u64>, CommandError> {
    match a.get("from") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n.as_u64().map(Some).ok_or_else(|| bad("from must be an id")),
        Some(Value::String(t)) => t.trim().parse().map(Some).map_err(|_| bad("from must be an id")),
        Some(_) => Err(bad("from must be an id")),
    }
}

fn need_id(a: &Value, what: &str) -> Result<u64, CommandError> {
    from_id(a)?.ok_or_else(|| bad(format!("from must be the id of the {what} to save")))
}

fn thumbnail(a: &Value) -> Result<Option<Vec<u8>>, CommandError> {
    match a.get("thumbnail").and_then(Value::as_str) {
        Some(t) => b64::png_from_base64(t).map(Some).map_err(bad),
        None => Ok(None),
    }
}

fn roots_run(_: &mut Session, _: &Value) -> Result<Value, CommandError> {
    Ok(json!({"roots": store::roots().iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(), "default": store::default_root().to_string_lossy(), "data_dir": store::data_dir().to_string_lossy()}))
}

fn set_roots_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "paths": {"type": "array", "items": {"type": "string"}, "maxItems": 8, "description": "Absolute folders chosen by the user. The first one receives new items. System folders, the filesystem root and the home folder are refused. An empty list returns to the default [data dir]/library."},
            "confirmed": {"type": "boolean", "description": "Must be true, and only after the user has chosen and confirmed these folders."}
        },
        "required": ["paths", "confirmed"],
        "additionalProperties": false
    })
}

fn set_roots_run(_: &mut Session, a: &Value) -> Result<Value, CommandError> {
    if a.get("confirmed").and_then(Value::as_bool) != Some(true) {
        return Err(CommandError::new("confirmation_required", "library folders change only after the user chose and confirmed them; send confirmed true then"));
    }
    let paths: Vec<String> = a.get("paths").and_then(Value::as_array).ok_or_else(|| bad("paths must be an array of folders"))?.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    if paths.len() != a["paths"].as_array().map_or(0, Vec::len) {
        return Err(bad("paths must hold strings"));
    }
    let roots = store::set_roots(&paths)?;
    Ok(json!({"roots": roots.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()}))
}

fn list_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": KINDS, "description": "Only this kind. Leave out for all."},
            "query": {"type": "string", "description": "Case-insensitive text that must appear in the name or a tag."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn list_run(_: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kind = match a.get("kind") {
        None | Some(Value::Null) => None,
        Some(_) => Some(kind_arg(a)?),
    };
    let query = a.get("query").and_then(Value::as_str).map(|q| q.trim().to_lowercase()).filter(|q| !q.is_empty());
    let mut items: Vec<(String, String, Value)> = store::all_items()
        .into_iter()
        .filter(|(_, i)| kind.is_none_or(|k| i.kind == k.name()))
        .filter(|(_, i)| query.as_ref().is_none_or(|q| i.name.to_lowercase().contains(q) || i.tags.iter().any(|t| t.to_lowercase().contains(q))))
        .map(|(root, i)| (i.kind.clone(), i.name.to_lowercase(), store::item_json(&root, &i)))
        .collect();
    items.sort_by(|x, y| (&x.0, &x.1).cmp(&(&y.0, &y.1)));
    Ok(json!({"items": items.into_iter().map(|t| t.2).collect::<Vec<_>>(), "roots": store::roots().iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()}))
}

fn add_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": KINDS},
            "name": {"type": "string", "minLength": 1, "maxLength": 64, "description": "Item name."},
            "from": {"type": ["integer", "string"], "description": "What to save: a fabric id, a trim or style id (style.list), a component id (component.list), a body id for a pose (default the first body) or a motion id. Leave out for kind garment, which saves the open project."},
            "tags": {"type": "array", "items": {"type": "string"}, "maxItems": 16},
            "thumbnail": {"type": "string", "description": "Optional PNG as base64, at most 1 MiB."}
        },
        "required": ["kind", "name"],
        "additionalProperties": false
    })
}

fn style_bytes(s: &Session, kind: Kind, id: u64) -> Result<Vec<u8>, CommandError> {
    let style = s.garment.styles.iter().find(|x| u64::from(x.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no style with id {id}")))?;
    if (kind == Kind::Trim) != (style.kind == StyleKind::Trim) {
        return Err(bad(if kind == Kind::Trim { "that style is not a trim style; save it as kind style" } else { "that style is a trim style; save it as kind trim" }));
    }
    serde_json::to_vec(&json!({"format": "ffstyle", "schema": 1, "style": style})).map_err(|e| CommandError::new("save_failed", e.to_string()))
}

fn pose_bytes(s: &Session, id: Option<u64>) -> Result<Vec<u8>, CommandError> {
    let body = match id {
        Some(id) => s.bodies.iter().find(|b| u64::from(b.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no body with id {id}")))?,
        None => s.bodies.first().ok_or_else(|| CommandError::new("not_found", "there is no body to take a pose from"))?,
    };
    let joints: Map<String, Value> = body.pose.iter().map(|(k, q)| (k.clone(), json!(q))).collect();
    serde_json::to_vec(&json!({"format": "ffpose", "schema": 1, "body": body.name, "joints": joints})).map_err(|e| CommandError::new("save_failed", e.to_string()))
}

fn motion_bytes(s: &Session, id: u64) -> Result<Vec<u8>, CommandError> {
    let m = s.garment.anim.motions.iter().find(|m| u64::from(m.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no motion with id {id}")))?;
    if m.source.starts_with("morph:") {
        return Err(bad("a body morph is not a motion clip and cannot be saved to the library"));
    }
    let entry = format!("motion/{}.bin", m.id);
    let (_, data) = s.unknown.iter().find(|(n, _)| *n == entry).ok_or_else(|| CommandError::new("not_found", format!("motion {id} has no stored data")))?;
    let header = serde_json::to_vec(&json!({"format": "ffmotion", "schema": 1, "name": m.name, "source": m.source, "fps": m.fps, "frames": m.frames, "lead_in": m.lead_in, "retarget": m.retarget}))
        .map_err(|e| CommandError::new("save_failed", e.to_string()))?;
    let mut out = Vec::with_capacity(12 + header.len() + data.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(header.len() as u32).to_le_bytes());
    out.extend_from_slice(&header);
    out.extend_from_slice(data);
    Ok(out)
}

fn add_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kind = kind_arg(a)?;
    let name = name_arg(a)?;
    let tags = store::clean_tags(a)?;
    let thumb = thumbnail(a)?;
    let item = match kind {
        Kind::Fabric => {
            let id = need_id(a, "fabric")?;
            let r = store::reserve(kind, &name)?;
            dispatch(s, "fabric.export", &json!({"fabric": id, "path": r.abs.to_string_lossy()}))?;
            store::register(&r, &name, tags, thumb)?
        }
        Kind::Trim | Kind::Style => {
            let bytes = style_bytes(s, kind, need_id(a, "style")?)?;
            store::add_bytes(kind, &name, tags, &bytes, thumb)?
        }
        Kind::Component => {
            let id = need_id(a, "component")?;
            let bytes = crate::component_cmds::encode_component(&s.garment, u32::try_from(id).map_err(|_| bad("from must be a component id"))?)?;
            store::add_bytes(kind, &name, tags, &bytes, thumb)?
        }
        Kind::Garment => {
            let bytes = crate::file_cmds::save_data(s)?;
            store::add_bytes(kind, &name, tags, &bytes, thumb)?
        }
        Kind::Pose => {
            let bytes = pose_bytes(s, from_id(a)?)?;
            store::add_bytes(kind, &name, tags, &bytes, thumb)?
        }
        Kind::Motion => {
            let bytes = motion_bytes(s, need_id(a, "motion")?)?;
            store::add_bytes(kind, &name, tags, &bytes, thumb)?
        }
    };
    Ok(json!({"item": item}))
}

fn id_schema() -> Value {
    json!({"type": "object", "properties": {"id": {"type": "string", "minLength": 1, "description": "Library item id from library.list, for example fabric/denim."}}, "required": ["id"], "additionalProperties": false})
}

fn remove_run(_: &mut Session, a: &Value) -> Result<Value, CommandError> {
    store::remove(args::string(a, "id")?)
}

fn insert_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string", "minLength": 1, "description": "Library item id from library.list."},
            "body": {"type": "integer", "minimum": 1, "description": "Pose and motion only: the body to apply it to. Defaults to the first body."},
            "joins": {"type": "array", "items": {"type": "object"}, "description": "Component only: joins for component.insert, each {port, to: {component, port}, reversed, ease}."}
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn body_of(s: &Session, a: &Value) -> Result<u32, CommandError> {
    match a.get("body").and_then(Value::as_u64) {
        Some(id) => s.bodies.iter().find(|b| u64::from(b.id) == id).map(|b| b.id).ok_or_else(|| CommandError::new("not_found", format!("no body with id {id}"))),
        None => s.bodies.first().map(|b| b.id).ok_or_else(|| CommandError::new("not_found", "there is no body; use body.load_default first")),
    }
}

fn insert_style(s: &mut Session, bytes: &[u8], kind: Kind) -> Result<Value, CommandError> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| CommandError::new("open_failed", format!("not a style file: {e}")))?;
    if v.get("format").and_then(Value::as_str) != Some("ffstyle") {
        return Err(CommandError::new("open_failed", "not a style file"));
    }
    let mut style: Style = serde_json::from_value(v.get("style").cloned().unwrap_or(Value::Null)).map_err(|e| CommandError::new("open_failed", format!("the style is invalid: {e}")))?;
    if (kind == Kind::Trim) != (style.kind == StyleKind::Trim) {
        return Err(CommandError::new("open_failed", "the file holds a different kind of style than its library entry says"));
    }
    if !style.params.is_object() {
        return Err(CommandError::new("open_failed", "the style parameters are not an object"));
    }
    style.id = ffb_pattern::trim::next_style_id(&s.garment);
    let id = style.id;
    let label = if kind == Kind::Trim { "Insert Trim" } else { "Insert Style" };
    crate::component_cmds::mutate(s, label, |g| {
        g.styles.push(style.clone());
        Ok(json!({"style": id, "name": style.name, "style_kind": style.kind}))
    })
}

fn insert_pose(s: &mut Session, bytes: &[u8], a: &Value) -> Result<Value, CommandError> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| CommandError::new("open_failed", format!("not a pose file: {e}")))?;
    if v.get("format").and_then(Value::as_str) != Some("ffpose") {
        return Err(CommandError::new("open_failed", "not a pose file"));
    }
    let joints = v.get("joints").and_then(Value::as_object).ok_or_else(|| CommandError::new("open_failed", "the pose has no joints"))?;
    for (k, q) in joints {
        let ok = q.as_array().is_some_and(|l| l.len() == 4 && l.iter().all(|x| x.as_f64().is_some_and(f64::is_finite)));
        if !ok {
            return Err(CommandError::new("open_failed", format!("joint {k} is not a quaternion [x, y, z, w]")));
        }
    }
    let body = body_of(s, a)?;
    let known = s.bodies.iter().find(|b| b.id == body).map(|b| b.joints.iter().map(|j| j.name.clone()).collect::<Vec<_>>()).unwrap_or_default();
    let used: Map<String, Value> = joints.iter().filter(|(k, _)| known.contains(k)).map(|(k, q)| (k.clone(), q.clone())).collect();
    let skipped = joints.len() - used.len();
    let out = dispatch(s, "body.pose", &json!({"id": body, "joints": used, "replace": true}))?;
    Ok(json!({"body": body, "joints": used.len(), "skipped": skipped, "pose": out}))
}

fn insert_motion(s: &mut Session, bytes: &[u8], a: &Value) -> Result<Value, CommandError> {
    if bytes.len() < 12 || &bytes[..8] != MAGIC {
        return Err(CommandError::new("open_failed", "not a motion file"));
    }
    let n = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    if n > MAX_HEADER || bytes.len() < 12 + n {
        return Err(CommandError::new("open_failed", "the motion header is damaged"));
    }
    let header: Value = serde_json::from_slice(&bytes[12..12 + n]).map_err(|e| CommandError::new("open_failed", format!("the motion header is not JSON: {e}")))?;
    let clip_bytes = &bytes[12 + n..];
    let clip = Clip::decode(clip_bytes).map_err(|e| CommandError::new("open_failed", format!("the motion clip is invalid: {e}")))?;
    let body = body_of(s, a)?;
    let id = crate::anim_cmds::alloc_motion_id(s);
    let fps = s.garment.anim.fps;
    let name = header.get("name").and_then(Value::as_str).map(|n| n.chars().take(64).collect::<String>()).filter(|n| !n.is_empty() && !args::has_control(n)).unwrap_or_else(|| "Motion".into());
    let source = header.get("source").and_then(Value::as_str).unwrap_or("library").chars().take(200).collect::<String>();
    let frames = ((clip.duration() * fps).ceil() as u32 + 1).min(100_000);
    let lead_in = header.get("lead_in").and_then(Value::as_u64).unwrap_or(0).min(1000) as u32;
    let retarget = header.get("retarget").cloned().filter(Value::is_object).unwrap_or(Value::Null);
    let start = s.garment.anim.start;
    let first = s.garment.anim.motions.is_empty();
    crate::anim_cmds::put_entry(s, format!("motion/{id}.bin"), clip_bytes.to_vec());
    crate::anim_cmds::edit_garment(s, "Insert Motion", |g| {
        g.anim.motions.push(ffb_pattern::anim::Motion { id, name: name.clone(), source: source.clone(), body, start, frames, fps: clip.fps, lead_in, retarget: retarget.clone() });
        if first {
            g.anim.end = (start + lead_in + frames.saturating_sub(1)).min(100_000);
        }
        Ok(())
    })?;
    Ok(json!({"motion": id, "name": name, "body": body, "frames": frames}))
}

fn insert_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let reference = args::string(a, "id")?;
    let (path, item) = store::locate(reference, &Kind::ALL)?;
    let kind = match &item {
        Some(i) => Kind::parse(&i.kind).unwrap_or(Kind::Garment),
        None => Kind::ALL.into_iter().find(|k| path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(k.extension()))).unwrap_or(Kind::Garment),
    };
    let mut out = match kind {
        Kind::Fabric => dispatch(s, "fabric.import", &json!({"path": path.to_string_lossy()}))?,
        Kind::Garment => return Err(CommandError::new("unsupported", format!("a garment is opened, not inserted: call file.open with path {}", path.display()))),
        Kind::Trim | Kind::Style => insert_style(s, &store::read_bytes(&path)?, kind)?,
        Kind::Component => crate::component_cmds::insert_from_bytes(s, &store::read_bytes(&path)?, a.get("joins").unwrap_or(&Value::Null))?,
        Kind::Pose => insert_pose(s, &store::read_bytes(&path)?, a)?,
        Kind::Motion => insert_motion(s, &store::read_bytes(&path)?, a)?,
    };
    if let Some(o) = out.as_object_mut() {
        o.insert("inserted".into(), json!(kind.name()));
        o.insert("from".into(), json!(reference));
    }
    Ok(out)
}

pub(crate) fn read_component(reference: &str) -> Result<Vec<u8>, CommandError> {
    let (path, _) = store::locate(reference, &[Kind::Component])?;
    store::read_bytes(&path)
}

pub(crate) fn write_component(name: &str, tags: Vec<String>, bytes: &[u8], thumb: Option<Vec<u8>>, path: Option<&str>) -> Result<Value, CommandError> {
    let Some(path) = path else { return store::add_bytes(Kind::Component, name, tags, bytes, thumb) };
    let target = std::path::PathBuf::from(args::file_path(&json!({"p": path}), "p", Some(Kind::Component.extension()))?);
    let folder = target.parent().ok_or_else(|| bad("path has no folder"))?;
    if !store::would_be_inside_roots(folder) {
        return Err(CommandError::new("outside_library", format!("{path} is not inside a library folder")));
    }
    std::fs::create_dir_all(folder).map_err(|e| CommandError::new("io", format!("{}: {e}", folder.display())))?;
    let (root, canon_folder) = store::inside_roots(folder).ok_or_else(|| CommandError::new("outside_library", format!("{path} is not inside a library folder")))?;
    let file = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let rel = canon_folder.strip_prefix(&root).map(|p| p.join(&file)).map_err(|_| bad("path is outside the library"))?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    if rel.matches('/').count() != 1 || !rel.starts_with("component/") {
        return Err(bad("component files live in the component folder of a library root"));
    }
    if bytes.len() as u64 > store::MAX_ITEM {
        return Err(CommandError::new("too_large", "the component file is too large"));
    }
    args::write_atomic(&canon_folder.join(&file).to_string_lossy(), bytes)?;
    let slug = file.trim_end_matches(&format!(".{}", Kind::Component.extension())).to_string();
    store::register(&store::Reserved { root: root.clone(), kind: Kind::Component, slug, file: rel, abs: canon_folder.join(&file) }, name, tags, thumb)
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "library.list",
            mcp: "library_list",
            group: Group::File,
            label: "Library",
            description: "List the user library: fabrics (.ffab), trim and style files (.ffstyle), components (.ffcomp), garments (.ffbrc), poses (.ffpose) and motions (.ffmotion) from every configured library folder, with name, tags, thumbnail path and size. Files placed in the kind folders by hand are listed too.",
            keys: &[],
            hints: READ,
            schema: list_schema,
            run: list_run,
        },
        Command {
            id: "library.add",
            mcp: "library_add",
            group: Group::File,
            label: "Add to Library",
            description: "Save a fabric, trim style, style, component, pose, motion or the open garment into the first library folder, with tags and an optional PNG thumbnail. Files are written atomically; the item can be inserted again with library.insert.",
            keys: &[],
            hints: EXTERNAL,
            schema: add_schema,
            run: add_run,
        },
        Command {
            id: "library.insert",
            mcp: "library_insert",
            group: Group::File,
            label: "Insert from Library",
            description: "Bring a library fabric, trim, style, component, pose or motion into the open project. One undo step. A garment is not inserted: the error names the path for file.open.",
            keys: &[],
            hints: WRITE,
            schema: insert_schema,
            run: insert_run,
        },
        Command {
            id: "library.remove",
            mcp: "library_remove",
            group: Group::File,
            label: "Remove from Library",
            description: "Delete a library item, its thumbnail and index entry from disk. Only files inside the library folders can be removed. Cannot be undone.",
            keys: &[],
            hints: DESTROY,
            schema: id_schema,
            run: remove_run,
        },
        Command {
            id: "library.roots",
            mcp: "library_roots",
            group: Group::File,
            label: "Library Folders",
            description: "The folders the library reads and writes: the configured list, or the default [data dir]/library.",
            keys: &[],
            hints: READ,
            schema: crate::no_args,
            run: roots_run,
        },
        Command {
            id: "library.set_roots",
            mcp: "library_set_roots",
            group: Group::File,
            label: "Set Library Folders",
            description: "Set the library folders (absolute paths the user chose and confirmed, at most 8; created when missing after the checks). System folders, the filesystem root, the home folder and any folder holding the data folder are refused. The first receives new items. The list is stored in [data dir]/library_roots.json. An empty list returns to the default folder.",
            keys: &[],
            hints: DESTROY,
            schema: set_roots_schema,
            run: set_roots_run,
        },
    ]
}
