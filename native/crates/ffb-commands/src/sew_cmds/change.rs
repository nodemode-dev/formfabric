use serde_json::{json, Value};

use super::common::*;
use super::create::check_ranges;
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

fn seam_pick(props: &mut Value) {
    props["seam"] = panel_id("Seam id as listed by sew.list.");
    props["seams"] = json!({"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Several seam ids."});
}

fn targets(s: &Session, a: &Value) -> Result<Vec<u32>, CommandError> {
    let mut ids = if a.get("seams").is_some() { id_list(a, "seams")? } else { Vec::new() };
    if let Some(one) = id(a, "seam") {
        if !ids.contains(&one) {
            ids.push(one);
        }
    }
    if ids.is_empty() {
        return Err(bad("give seam or seams"));
    }
    for seam in &ids {
        seam_index(s, *seam)?;
    }
    Ok(ids)
}

fn edit_schema() -> Value {
    let mut props = json!({"seam": panel_id("Seam id as listed by sew.list."), "a": spans("Replace side a with these spans."), "b": spans("Replace side b with these spans.")});
    seam_fields(&mut props, false);
    obj(props, &["seam"])
}

fn edit(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check_ranges(a)?;
    let index = seam_index(s, required_id(a, "seam")?)?;
    let before = s.garment.clone();
    let mut seam = s.garment.seams[index].clone();
    if a.get("a").is_some() {
        seam.a = span_list(&s.garment, a, "a")?;
    }
    if a.get("b").is_some() {
        seam.b = span_list(&s.garment, a, "b")?;
    }
    apply_fields(&mut seam, a)?;
    if seam == s.garment.seams[index] {
        return Ok(json!({"changed": false, "seam": describe(&s.garment, &seam)}));
    }
    s.garment.seams[index] = seam.clone();
    record(s, "Edit Seam", before);
    Ok(json!({"changed": true, "seam": describe(&s.garment, &seam)}))
}

fn flip_schema() -> Value {
    let mut props = json!({});
    seam_pick(&mut props);
    obj(props, &[])
}

fn flip(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = targets(s, a)?;
    let before = s.garment.clone();
    for seam in s.garment.seams.iter_mut().filter(|x| ids.contains(&x.id)) {
        seam.reversed = !seam.reversed;
    }
    record(s, "Flip Seam", before);
    let flipped: Vec<Value> = s.garment.seams.iter().filter(|x| ids.contains(&x.id)).map(|x| json!({"id": x.id, "reversed": x.reversed})).collect();
    Ok(json!({"changed": true, "seams": flipped}))
}

fn delete_schema() -> Value {
    let mut props = json!({"all": {"type": "boolean", "default": false, "description": "Remove every seam."}});
    seam_pick(&mut props);
    obj(props, &[])
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let all = flag(a, "all").unwrap_or(false);
    let ids = if all {
        s.garment.seams.iter().map(|x| x.id).collect()
    } else {
        targets(s, a)?
    };
    if ids.is_empty() {
        return Ok(json!({"changed": false, "removed": 0, "remaining": 0}));
    }
    let before = s.garment.clone();
    s.garment.seams.retain(|x| !ids.contains(&x.id));
    record(s, "Delete Seam", before);
    Ok(json!({"changed": true, "removed": ids.len(), "remaining": s.garment.seams.len()}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "sew.edit",
            mcp: "sew_edit",
            group: Group::Sew,
            label: "Edit Seam",
            description: "Change any field of a seam: its sides (spans), reversed, fold (turned, flat, custom), fold angle and strength, ratio, tension and side. Only the given fields change. A turned seam locks the fold angle to 180 and the strength to 0.",
            keys: &["B"],
            hints: SET,
            schema: edit_schema,
            run: edit,
        },
        Command {
            id: "sew.flip",
            mcp: "sew_flip",
            group: Group::Sew,
            label: "Flip Seam",
            description: "Reverse the direction of side b of one or several seams, so the start of side a is paired with the other end of side b.",
            keys: &[],
            hints: WRITE,
            schema: flip_schema,
            run: flip,
        },
        Command {
            id: "sew.delete",
            mcp: "sew_delete",
            group: Group::Sew,
            label: "Delete Seam",
            description: "Remove one seam, several, or all of them. Forced mesh vertex counts that existed for the seam are released.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
    ]
}
