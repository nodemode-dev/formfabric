use ffb_pattern::sew::side_length;
use ffb_pattern::trim::next_zipper_id;
use ffb_pattern::{Garment, StyleKind, Zipper};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

fn describe(garment: &Garment, z: &Zipper) -> Value {
    let (la, lb) = (side_length(garment, &z.a), side_length(garment, &z.b));
    let mut out = serde_json::to_value(z).unwrap_or(Value::Null);
    out["length_a"] = json!(la);
    out["length_b"] = json!(lb);
    out["mismatch"] = json!(la.zip(lb).map(|(x, y)| y - x));
    out
}

fn closed_prop() -> Value {
    num("How far the zipper is closed, 0 (open) to 1 (closed), measured from the start of the spans, which is the direction the zipper zips.", 0.0, 1.0, None)
}

fn add_schema() -> Value {
    obj(
        json!({
            "style": ident("Style id of a zipper style."),
            "a": spans("Spans of the first side, listed in the direction the zipper zips. A span with t0 above t1 walks against its line."),
            "b": spans("Spans of the second side, listed in the same zipping direction."),
            "closed": closed_prop()
        }),
        &["style", "a", "b"],
    )
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let style = style_of(&s.garment, required_id(a, "style")?, &[StyleKind::Zipper])?.id;
    let (ea, eb) = (parse_spans(&s.garment, a, "a")?, parse_spans(&s.garment, a, "b")?);
    let closed = number_or(a, "closed", 1.0);
    if !(0.0..=1.0).contains(&closed) {
        return Err(bad("closed must be between 0 and 1"));
    }
    let before = s.garment.clone();
    let zipper = Zipper { id: next_zipper_id(&s.garment), style, a: ea, b: eb, closed };
    s.garment.zippers.push(zipper.clone());
    let out = describe(&s.garment, &zipper);
    record(s, "Add Zipper", before);
    Ok(out)
}

fn set_schema() -> Value {
    obj(json!({"zipper": ident("Zipper id."), "closed": closed_prop(), "style": ident("Move the zipper to another zipper style.")}), &["zipper"])
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = required_id(a, "zipper")?;
    let index = s.garment.zippers.iter().position(|z| z.id == id).ok_or_else(|| missing(format!("no zipper with id {id}")))?;
    let closed = number(a, "closed");
    if closed.is_some_and(|c| !(0.0..=1.0).contains(&c)) {
        return Err(bad("closed must be between 0 and 1"));
    }
    let style = a.get("style").map(|_| style_of(&s.garment, id_or_zero(a), &[StyleKind::Zipper]).map(|x| x.id)).transpose()?;
    let before = s.garment.clone();
    let z = &mut s.garment.zippers[index];
    z.closed = closed.unwrap_or(z.closed);
    z.style = style.unwrap_or(z.style);
    if s.garment == before {
        return Ok(json!({"changed": false, "zipper": describe(&s.garment, &s.garment.zippers[index])}));
    }
    record(s, "Set Zipper", before);
    Ok(json!({"changed": true, "zipper": describe(&s.garment, &s.garment.zippers[index])}))
}

fn id_or_zero(a: &Value) -> u32 {
    id(a, "style").unwrap_or(0)
}

fn delete_schema() -> Value {
    obj(json!({"zipper": ident("Zipper id.")}), &["zipper"])
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = required_id(a, "zipper")?;
    if !s.garment.zippers.iter().any(|z| z.id == id) {
        return Err(missing(format!("no zipper with id {id}")));
    }
    let before = s.garment.clone();
    s.garment.zippers.retain(|z| z.id != id);
    record(s, "Delete Zipper", before);
    Ok(json!({"changed": true, "removed": id, "remaining": s.garment.zippers.len()}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "zipper.add",
            mcp: "zipper_add",
            group: Group::Sew,
            label: "Add Zipper",
            description: "Add a zipper between two sides, each a list of spans listed in the direction the zipper zips. The closed part binds like a seam, the open part does not. Reports both side lengths and the mismatch.",
            keys: &["Shift+Z"],
            hints: WRITE,
            schema: add_schema,
            run: add,
        },
        Command {
            id: "zipper.set",
            mcp: "zipper_set",
            group: Group::Sew,
            label: "Set Zipper",
            description: "Set how far a zipper is closed (0 to 1) or change its style.",
            keys: &[],
            hints: SET,
            schema: set_schema,
            run: set,
        },
        Command {
            id: "zipper.delete",
            mcp: "zipper_delete",
            group: Group::Sew,
            label: "Delete Zipper",
            description: "Remove a zipper.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
    ]
}
