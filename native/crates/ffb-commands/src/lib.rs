mod anim_cmds;
mod args;
mod exchange_cmds;
mod output_cmds;
mod remesh_cmds;
mod viz_cmds;
mod fabric_cmds;
mod file_cmds;
mod history;
mod history_cmds;
mod hooks;
mod body_cmds;
mod mesh_cmds;
mod place_cmds;
mod sew_cmds;
mod sim_cmds;
mod edit3d_cmds;
mod trim_cmds;
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
pub use body_cmds::StreamState as BodyStream;

pub fn publish_all(session: &mut Session) {
    mesh_cmds::publish_all(session);
    body_cmds::publish_all(session);
}

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

pub trait FrameSink: Send + Sync {
    fn topology(&self, object: u32, indices: &[u32], panel_ids: &[u16]);
    fn positions(&self, object: u32, positions: &[f32]);
}

#[derive(Default)]
pub struct Session {
    pub garment: Garment,
    pub history: History,
    pub path: Option<String>,
    pub unknown: Vec<(String, Vec<u8>)>,
    pub epoch: u64,
    pub sink: Option<std::sync::Arc<dyn FrameSink>>,
    pub mesh: mesh_cmds::MeshState,
    pub sim: sim_cmds::SimState,
    pub bodies: Vec<ffb_body::Body>,
    pub body_stream: body_cmds::StreamState,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("panels", &self.garment.panels.len())
            .field("path", &self.path)
            .field("epoch", &self.epoch)
            .field("sink", &self.sink.is_some())
            .finish()
    }
}

impl Session {
    pub fn reset(&mut self, garment: Garment) {
        self.garment = garment;
        self.history = History::default();
        self.path = None;
        self.unknown.clear();
        self.epoch += 1;
    }

    pub fn record(&mut self, mut edit: Edit) {
        self.drop_stale_poses(&mut edit);
        self.history.record(edit);
        self.epoch += 1;
    }

    fn drop_stale_poses(&mut self, edit: &mut Edit) {
        match edit {
            Edit::Panel { before: Some(before), after: Some(after), .. } => {
                if stale_pose(before, after) {
                    clear_pose(after);
                    if let Some(live) = self.garment.panels.iter_mut().find(|p| p.id == after.id) {
                        clear_pose(live);
                    }
                }
            }
            Edit::Garment { before, after, .. } => {
                for panel in after.panels.iter_mut() {
                    let Some(old) = before.panels.iter().find(|p| p.id == panel.id) else { continue };
                    if stale_pose(old, panel) {
                        clear_pose(panel);
                        if let Some(live) = self.garment.panels.iter_mut().find(|p| p.id == panel.id) {
                            clear_pose(live);
                        }
                    }
                }
            }
            Edit::Many { edits, .. } => edits.iter_mut().for_each(|e| self.drop_stale_poses(e)),
            _ => {}
        }
    }
}

fn stale_pose(before: &ffb_pattern::Panel, after: &ffb_pattern::Panel) -> bool {
    fn carried(p: &ffb_pattern::Panel) -> Option<&Vec<[f32; 3]>> {
        p.place3d.as_ref().and_then(|q| q.pose.as_ref())
    }
    carried(after).is_some() && carried(before) == carried(after) && !before.same_mesh_shape(after)
}

fn clear_pose(panel: &mut ffb_pattern::Panel) {
    if let Some(place) = panel.place3d.as_mut() {
        place.pose = None;
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
    all.extend(sew_cmds::commands());
    all.extend(sim_cmds::commands());
    all.extend(body_cmds::commands());
    all.extend(place_cmds::commands());
    all.extend(edit3d_cmds::commands());
    all.extend(trim_cmds::commands());
    all.extend(anim_cmds::commands());
    all.extend(output_cmds::commands());
    all.extend(remesh_cmds::commands());
    all.extend(viz_cmds::commands());
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
    let before = session.epoch;
    let result = (command.run)(session, &args)?;
    if session.epoch != before {
        hooks::after_change(session, command.id);
    }
    Ok(result)
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
