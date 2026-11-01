use serde_json::{json, Value};

use crate::{args, no_args, Command, CommandError, Group, Session, DESTROY, EXTERNAL};

pub fn save_data(session: &Session) -> Result<Vec<u8>, CommandError> {
    ffb_io::save(&session.garment, &session.unknown, ffb_core::ENGINE_VERSION).map_err(|e| CommandError::new("save_failed", e))
}

pub fn open_data(session: &mut Session, bytes: Vec<u8>) -> Result<Value, CommandError> {
    let project = ffb_io::open(bytes).map_err(|e| CommandError::new("open_failed", e))?;
    let panels = project.garment.panels.len();
    session.reset(project.garment);
    session.unknown = project.unknown;
    Ok(json!({"panels": panels, "units": session.garment.units.name()}))
}

fn path_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"path": {"type": "string", "description": "File path of the .ffbrc project."}},
        "required": ["path"],
        "additionalProperties": false
    })
}

fn new(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    s.reset(crate::fabric_cmds::new_garment());
    Ok(json!({"panels": 0, "units": s.garment.units.name()}))
}

fn save(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::string(a, "path")?.to_string();
    let bytes = save_data(s)?;
    std::fs::write(&path, &bytes).map_err(|e| CommandError::new("io", format!("could not write {path}: {e}")))?;
    s.path = Some(path.clone());
    Ok(json!({"path": path, "bytes": bytes.len()}))
}

fn open(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::string(a, "path")?.to_string();
    let bytes = std::fs::read(&path).map_err(|e| CommandError::new(if e.kind() == std::io::ErrorKind::NotFound { "not_found" } else { "io" }, format!("could not read {path}: {e}")))?;
    let mut out = open_data(s, bytes)?;
    s.path = Some(path.clone());
    out["path"] = json!(path);
    Ok(out)
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "file.new",
            mcp: "file_new",
            group: Group::File,
            label: "New Project",
            description: "Discard the current garment and history and start an empty project in millimetres. Unsaved work is lost.",
            keys: &["Mod+N"],
            hints: DESTROY,
            schema: no_args,
            run: new,
        },
        Command {
            id: "file.save",
            mcp: "file_save",
            group: Group::File,
            label: "Save Project",
            description: "Write the garment to a .ffbrc project file (a zip with manifest.json and garment.json). Entries of an opened project this engine does not know are kept.",
            keys: &["Mod+S"],
            hints: EXTERNAL,
            schema: path_schema,
            run: save,
        },
        Command {
            id: "file.open",
            mcp: "file_open",
            group: Group::File,
            label: "Open Project",
            description: "Replace the current garment with the one in a .ffbrc project file and clear the history. Unsaved work is lost.",
            keys: &["Mod+O"],
            hints: DESTROY,
            schema: path_schema,
            run: open,
        },
    ]
}
