use ffb_pattern::trim::{next_stitch_id, next_style_id};
use ffb_pattern::{SeamEnd, Stitch, Style, StyleKind};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, WRITE};

fn describe(s: &Stitch) -> Value {
    serde_json::to_value(s).unwrap_or(Value::Null)
}

fn add_schema() -> Value {
    obj(
        json!({
            "style": ident("Style id of a topstitch or puckering style."),
            "spans": spans("Spans the stitch line follows, on outlines or inner lines."),
            "offset": num("Distance of the stitch line from the span, positive to the left of the walking direction. Default is the offset of the style.", -50.0, 50.0, Some("mm"))
        }),
        &["style", "spans"],
    )
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let style = style_of(&s.garment, required_id(a, "style")?, &[StyleKind::Topstitch, StyleKind::Puckering])?;
    let offset = number(a, "offset").unwrap_or_else(|| ffb_pattern::trim::param(style, "offset", 0.0));
    let ends = parse_spans(&s.garment, a, "spans")?;
    let stitch = Stitch { id: next_stitch_id(&s.garment), style: style.id, spans: ends, offset };
    let before = s.garment.clone();
    s.garment.stitches.push(stitch.clone());
    record(s, "Add Stitch", before);
    Ok(describe(&stitch))
}

fn delete_schema() -> Value {
    obj(
        json!({
            "stitch": ident("Stitch id."),
            "stitches": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Several stitch ids."},
            "all": {"type": "boolean", "default": false, "description": "Remove every stitch."}
        }),
        &[],
    )
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let all = flag(a, "all").unwrap_or(false);
    let mut ids = if a.get("stitches").is_some() { id_list(a, "stitches")? } else { Vec::new() };
    ids.extend(id(a, "stitch"));
    if all == !ids.is_empty() {
        return Err(bad("give exactly one of stitch, stitches or all"));
    }
    if let Some(gone) = ids.iter().find(|i| !s.garment.stitches.iter().any(|x| x.id == **i)) {
        return Err(missing(format!("no stitch with id {gone}")));
    }
    let before = s.garment.clone();
    let count = s.garment.stitches.len();
    s.garment.stitches.retain(|x| !all && !ids.contains(&x.id));
    let removed = count - s.garment.stitches.len();
    if removed == 0 {
        return Ok(json!({"changed": false, "removed": 0, "remaining": count}));
    }
    record(s, "Delete Stitch", before);
    Ok(json!({"changed": true, "removed": removed, "remaining": s.garment.stitches.len()}))
}

fn set_schema() -> Value {
    obj(
        json!({
            "stitch": ident("Stitch id."),
            "offset": num("New distance of the stitch line from the span, positive to the left of the walking direction.", -50.0, 50.0, Some("mm")),
            "style": ident("New style id, a topstitch or puckering style."),
            "spans": spans("New spans, replacing the old ones.")
        }),
        &["stitch"],
    )
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let target = required_id(a, "stitch")?;
    let index = s.garment.stitches.iter().position(|x| x.id == target).ok_or_else(|| missing(format!("no stitch with id {target}")))?;
    let offset = number(a, "offset");
    if offset.is_some_and(|o| !(-50.0..=50.0).contains(&o)) {
        return Err(bad("offset must be between -50 and 50 mm"));
    }
    let style = match id(a, "style") {
        Some(sid) => Some(style_of(&s.garment, sid, &[StyleKind::Topstitch, StyleKind::Puckering])?.id),
        None => None,
    };
    let ends = if a.get("spans").is_some() { Some(parse_spans(&s.garment, a, "spans")?) } else { None };
    if offset.is_none() && style.is_none() && ends.is_none() {
        return Err(bad("give offset, style or spans"));
    }
    let mut next = s.garment.stitches[index].clone();
    if let Some(o) = offset {
        next.offset = o;
    }
    if let Some(sid) = style {
        next.style = sid;
    }
    if let Some(e) = ends {
        next.spans = e;
    }
    if next == s.garment.stitches[index] {
        return Ok(json!({"changed": false, "stitch": describe(&next)}));
    }
    let before = s.garment.clone();
    s.garment.stitches[index] = next.clone();
    record(s, "Set Stitch", before);
    Ok(json!({"changed": true, "stitch": describe(&next)}))
}

fn fray_schema() -> Value {
    obj(
        json!({
            "spans": spans("Edge spans that fray."),
            "seam": ident("Seam id; both sides of the seam fray. Use instead of spans."),
            "amount": num("Length of loose threads for rendering.", 0.1, 30.0, Some("mm"))
        }),
        &["amount"],
    )
}

fn fray(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let amount = number(a, "amount").ok_or_else(|| bad("amount is required"))?;
    if !(0.1..=30.0).contains(&amount) {
        return Err(bad("amount must be between 0.1 and 30"));
    }
    let ends: Vec<SeamEnd> = match (a.get("spans").is_some(), id(a, "seam")) {
        (true, None) => parse_spans(&s.garment, a, "spans")?,
        (false, Some(seam)) => {
            let found = s.garment.seams.iter().find(|x| x.id == seam).ok_or_else(|| missing(format!("no seam with id {seam}")))?;
            found.a.iter().chain(found.b.iter()).cloned().collect()
        }
        _ => return Err(bad("give exactly one of spans or seam")),
    };
    let before = s.garment.clone();
    let name = format!("Fray {amount}");
    let style = match s.garment.styles.iter().find(|x| x.kind == StyleKind::Trim && x.name == name && x.params.get("fray").is_some()) {
        Some(found) => found.id,
        None => {
            let id = next_style_id(&s.garment);
            s.garment.styles.push(Style { id, kind: StyleKind::Trim, name, params: json!({"fray": amount}), mesh: None });
            id
        }
    };
    let stitch = Stitch { id: next_stitch_id(&s.garment), style, spans: ends, offset: 0.0 };
    s.garment.stitches.push(stitch.clone());
    record(s, "Add Fray", before);
    Ok(describe(&stitch))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "stitch.add",
            mcp: "stitch_add",
            group: Group::Sew,
            label: "Add Stitch",
            description: "Add a topstitch or puckering line along spans of a panel outline or inner line, at an offset in mm. Render data, with a slight stiffening along the line (the stiffen parameter of the style).",
            keys: &["Shift+S"],
            hints: WRITE,
            schema: add_schema,
            run: add,
        },
        Command {
            id: "stitch.set",
            mcp: "stitch_set",
            group: Group::Sew,
            label: "Set Stitch",
            description: "Change a stitch after creation: its offset in mm from the span, its topstitch or puckering style, or its spans. Give at least one. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: set_schema,
            run: set,
        },
        Command {
            id: "stitch.delete",
            mcp: "stitch_delete",
            group: Group::Sew,
            label: "Delete Stitch",
            description: "Remove one stitch by id, several by stitches, or every stitch with all. Fray lines are stitches too.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
        Command {
            id: "fray.add",
            mcp: "fray_add",
            group: Group::Sew,
            label: "Add Fray",
            description: "Fray an edge or a seam for rendering: stores a stitch whose style carries the fray length in mm. No effect on the simulation. Remove it with stitch.delete.",
            keys: &[],
            hints: WRITE,
            schema: fray_schema,
            run: fray,
        },
    ]
}
