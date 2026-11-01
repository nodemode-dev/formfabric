use serde_json::{json, Value};

use crate::{no_args, Command, CommandError, Group, Session, READ, WRITE};

fn state(s: &Session) -> Value {
    json!({"can_undo": s.history.can_undo(), "can_redo": s.history.can_redo(), "position": s.history.position(), "steps": s.history.len()})
}

fn step(s: &mut Session, undo: bool) -> Result<Value, CommandError> {
    let result = if undo { s.history.undo(&mut s.garment) } else { s.history.redo(&mut s.garment) };
    match result {
        Ok(Some(label)) => {
            s.epoch += 1;
            let mut out = state(s);
            out["label"] = json!(label);
            Ok(out)
        }
        Ok(None) => Err(CommandError::new(if undo { "nothing_to_undo" } else { "nothing_to_redo" }, if undo { "there is nothing to undo" } else { "there is nothing to redo" })),
        Err(message) => Err(CommandError::new("history_conflict", message)),
    }
}

fn undo(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    step(s, true)
}

fn redo(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    step(s, false)
}

fn list(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let steps: Vec<Value> = s.history.labels().into_iter().enumerate().map(|(i, (label, applied))| json!({"index": i, "label": label, "applied": applied})).collect();
    let mut out = state(s);
    out["limit"] = json!(s.history.limit());
    out["grouping"] = json!(s.history.grouping());
    out["list"] = json!(steps);
    Ok(out)
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "history.undo",
            mcp: "history_undo",
            group: Group::History,
            label: "Undo",
            description: "Undo the last step of the pattern history (a grouped drag counts as one step). Fails with nothing_to_undo when the history is at its start.",
            keys: &["Mod+Z"],
            hints: WRITE,
            schema: no_args,
            run: undo,
        },
        Command {
            id: "history.redo",
            mcp: "history_redo",
            group: Group::History,
            label: "Redo",
            description: "Redo the step last undone. Fails with nothing_to_redo when there is none; a new edit after an undo drops the redo steps.",
            keys: &["Mod+Shift+Z"],
            hints: WRITE,
            schema: no_args,
            run: redo,
        },
        Command {
            id: "history.list",
            mcp: "history_list",
            group: Group::History,
            label: "History",
            description: "Every history step with its label and whether it is applied, the current position, the step limit and whether undo and redo are available.",
            keys: &[],
            hints: READ,
            schema: no_args,
            run: list,
        },
    ]
}
