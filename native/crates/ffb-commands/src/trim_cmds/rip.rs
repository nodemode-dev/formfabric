use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, SET};

fn rip_schema() -> Value {
    obj(json!({"seams": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Seam ids as listed by sew.list."}}), &["seams"])
}

fn rip(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = id_list(a, "seams")?;
    if let Some(gone) = ids.iter().find(|i| !s.garment.seams.iter().any(|x| x.id == **i)) {
        return Err(missing(format!("no seam with id {gone}")));
    }
    let before = s.garment.clone();
    for id in &ids {
        if !s.garment.ripped.contains(id) {
            s.garment.ripped.push(*id);
        }
    }
    if s.garment == before {
        return Ok(json!({"changed": false, "ripped": s.garment.ripped}));
    }
    record(s, "Rip Seam", before);
    Ok(json!({"changed": true, "ripped": s.garment.ripped}))
}

fn mend_schema() -> Value {
    obj(
        json!({
            "seams": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Seam ids to close again."},
            "all": {"type": "boolean", "default": false, "description": "Close every ripped seam."}
        }),
        &[],
    )
}

fn mend(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let all = flag(a, "all").unwrap_or(false);
    let ids = if a.get("seams").is_some() { id_list(a, "seams")? } else { Vec::new() };
    if all == !ids.is_empty() {
        return Err(bad("give exactly one of seams or all"));
    }
    let before = s.garment.clone();
    s.garment.ripped.retain(|r| !all && !ids.contains(r));
    if s.garment == before {
        return Ok(json!({"changed": false, "ripped": s.garment.ripped}));
    }
    record(s, "Mend Seam", before);
    Ok(json!({"changed": true, "ripped": s.garment.ripped}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "sew.rip",
            mcp: "sew_rip",
            group: Group::Sew,
            label: "Rip Seam",
            description: "Open seams: the seams stay in the document but the simulation leaves them out until they are mended.",
            keys: &[],
            hints: SET,
            schema: rip_schema,
            run: rip,
        },
        Command {
            id: "sew.mend",
            mcp: "sew_mend",
            group: Group::Sew,
            label: "Mend Seam",
            description: "Close ripped seams again, the listed ones or all.",
            keys: &[],
            hints: SET,
            schema: mend_schema,
            run: mend,
        },
    ]
}
