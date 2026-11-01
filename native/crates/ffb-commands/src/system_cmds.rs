use serde_json::{json, Value};

use crate::{no_args, Command, CommandError, Group, Session, READ};

fn info(_: &mut Session, _: &Value) -> Result<Value, CommandError> {
    Ok(json!({"engine": ffb_core::ENGINE_NAME, "version": ffb_core::ENGINE_VERSION}))
}

fn capabilities(_: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let units: Vec<&str> = ffb_core::Units::ALL.iter().map(|u| u.name()).collect();
    let groups = serde_json::to_value(Group::ALL).map_err(|e| CommandError::new("internal", e.to_string()))?;
    Ok(json!({
        "engine": ffb_core::ENGINE_NAME,
        "version": ffb_core::ENGINE_VERSION,
        "commands": crate::commands().len(),
        "groups": groups,
        "units": units,
        "project": {"extension": ffb_io::EXTENSION, "mime": ffb_io::MIME, "schema": 1},
        "history": {"limit": crate::DEFAULT_STEPS, "max": crate::MAX_STEPS},
        "wasm": cfg!(target_arch = "wasm32"),
        "stream": false,
    }))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "system.info",
            mcp: "system_info",
            group: Group::System,
            label: "Engine Info",
            description: "The engine's name and version. Use it to confirm the engine is reachable before other calls.",
            keys: &[],
            hints: READ,
            schema: no_args,
            run: info,
        },
        Command {
            id: "system.capabilities",
            mcp: "system_capabilities",
            group: Group::System,
            label: "Capabilities",
            description: "What this engine build offers: command count and groups, the length units, the project file format, the history limits and whether it runs as WebAssembly.",
            keys: &[],
            hints: READ,
            schema: no_args,
            run: capabilities,
        },
    ]
}
