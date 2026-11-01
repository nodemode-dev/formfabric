mod args;
mod exchange_cmds;
mod fabric_cmds;
mod file_cmds;
mod history;
mod history_cmds;
mod mesh_cmds;
mod pattern_cmds;
mod pattern_tools_cmds;
mod system_cmds;
mod view_cmds;

use std::sync::LazyLock;

use ffb_pattern::Garment;
use serde::Serialize;
use serde_json::{json, Value};

pub use file_cmds::{open_data, save_data};
pub use history::{Edit, History, DEFAULT_STEPS, MAX_STEPS};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    System,
    File,
    History,
    View,
    Pattern,
    Sew,
    Sim,
    Body,
    Fabric,
    Anim,
    Export,
}

impl Group {
    pub const ALL: [Group; 11] = [
        Group::System,
        Group::File,
        Group::History,
        Group::View,
        Group::Pattern,
        Group::Sew,
        Group::Sim,
        Group::Body,
        Group::Fabric,
        Group::Anim,
        Group::Export,
    ];
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Hints {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

pub const READ: Hints = Hints { read_only_hint: true, destructive_hint: false, idempotent_hint: true, open_world_hint: false };
pub const WRITE: Hints = Hints { read_only_hint: false, destructive_hint: false, idempotent_hint: false, open_world_hint: false };
pub const SET: Hints = Hints { read_only_hint: false, destructive_hint: false, idempotent_hint: true, open_world_hint: false };
pub const DESTROY: Hints = Hints { read_only_hint: false, destructive_hint: true, idempotent_hint: false, open_world_hint: false };
pub const EXTERNAL: Hints = Hints { read_only_hint: false, destructive_hint: false, idempotent_hint: false, open_world_hint: true };

pub struct Command {
    pub id: &'static str,
    pub mcp: &'static str,
    pub group: Group,
    pub label: &'static str,
    pub description: &'static str,
    pub keys: &'static [&'static str],
    pub hints: Hints,
    pub schema: fn() -> Value,
    pub run: fn(&mut Session, &Value) -> Result<Value, CommandError>,
}

#[derive(Debug, Default)]
pub struct Session {
    pub garment: Garment,
    pub history: History,
    pub path: Option<String>,
    pub unknown: Vec<(String, Vec<u8>)>,
    pub epoch: u64,
}

impl Session {
    pub fn reset(&mut self, garment: Garment) {
        self.garment = garment;
        self.history = History::default();
        self.path = None;
        self.unknown.clear();
        self.epoch += 1;
    }

    pub fn record(&mut self, edit: Edit) {
        self.history.record(edit);
        self.epoch += 1;
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

pub(crate) fn no_args() -> Value {
    json!({"type": "object", "properties": {}, "required": [], "additionalProperties": false})
}

static COMMANDS: LazyLock<Vec<Command>> = LazyLock::new(|| {
    let mut all = Vec::new();
    all.extend(system_cmds::commands());
    all.extend(file_cmds::commands());
    all.extend(history_cmds::commands());
    all.extend(view_cmds::commands());
    all.extend(pattern_cmds::commands());
    all.extend(pattern_tools_cmds::commands());
    all.extend(exchange_cmds::commands());
    all.extend(mesh_cmds::commands());
    all.extend(fabric_cmds::commands());
    all
});

pub fn commands() -> &'static [Command] {
    &COMMANDS
}

pub fn find(name: &str) -> Option<&'static Command> {
    commands().iter().find(|c| c.id == name || c.mcp == name)
}

pub fn dispatch(session: &mut Session, name: &str, args: &Value) -> Result<Value, CommandError> {
    let command = find(name).ok_or_else(|| CommandError::new("unknown_command", format!("no command named {name}")))?;
    let mut args = if args.is_null() { json!({}) } else { args.clone() };
    if !args.is_object() {
        return Err(CommandError::new("bad_args", "arguments must be a JSON object"));
    }
    let schema = (command.schema)();
    args::coerce_scalars(&schema, &mut args);
    args::validate(&schema, &args)?;
    (command.run)(session, &args)
}

pub fn reply(result: Result<Value, CommandError>) -> Value {
    match result {
        Ok(result) => json!({"ok": true, "result": result}),
        Err(err) => json!({"ok": false, "error": err}),
    }
}

pub fn dispatch_json(session: &mut Session, name: &str, args_json: &str) -> Value {
    let parsed: Result<Value, _> = if args_json.trim().is_empty() { Ok(json!({})) } else { serde_json::from_str(args_json) };
    match parsed {
        Ok(args) => reply(dispatch(session, name, &args)),
        Err(e) => reply(Err(CommandError::new("bad_args", format!("arguments are not JSON: {e}")))),
    }
}

pub fn catalogue() -> Value {
    let commands: Vec<Value> = commands()
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.mcp,
                "group": c.group,
                "label": c.label,
                "description": c.description,
                "keys": c.keys,
                "annotations": c.hints,
                "inputSchema": (c.schema)(),
            })
        })
        .collect();
    json!({"engine": ffb_core::ENGINE_NAME, "version": ffb_core::ENGINE_VERSION, "commands": commands})
}
