use ffb_pattern::trim::{geometry as mesh_of, next_hardware_id};
use ffb_pattern::{Anchor, Hardware, StyleKind};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

fn add_schema() -> Value {
    obj(
        json!({
            "style": ident("Style id of the right kind, from style.list."),
            "panel": ident("Panel the hardware is attached to."),
            "point": xy("Anchor point on the panel; it must lie inside the outline."),
            "space": space(),
            "rotation": num("Rotation about the surface normal, 0 to 360 degrees.", -360.0, 360.0, Some("deg"))
        }),
        &["style", "panel", "point"],
    )
}

pub fn describe(h: &Hardware) -> Value {
    serde_json::to_value(h).unwrap_or(Value::Null)
}

fn place(s: &mut Session, a: &Value, kind: StyleKind, label: &str) -> Result<Value, CommandError> {
    let style = style_of(&s.garment, required_id(a, "style")?, &[kind])?.id;
    let panel = panel_of(&s.garment, required_id(a, "panel")?)?;
    let point = optional_vec2(a, "point")?.ok_or_else(|| bad("point is required"))?;
    let local = local_point(panel, point, text(a, "space") != Some("local"));
    if !inside(panel, local) {
        return Err(geometry(format!("the point lies outside panel {}", panel.id)));
    }
    let panel_id = panel.id;
    let before = s.garment.clone();
    let hardware = Hardware { id: next_hardware_id(&s.garment), style, anchor: Anchor { panel: panel_id, point: local }, rotation: number_or(a, "rotation", 0.0), fastened_to: None };
    let out = describe(&hardware);
    s.garment.hardware.push(hardware);
    record(s, label, before);
    Ok(out)
}

fn button(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    place(s, a, StyleKind::Button, "Add Button")
}

fn buttonhole(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    place(s, a, StyleKind::Buttonhole, "Add Buttonhole")
}

fn fasten_schema() -> Value {
    obj(
        json!({
            "button": ident("Hardware id of the button."),
            "buttonhole": ident("Hardware id of the buttonhole it fastens to."),
            "unfasten": {"type": "boolean", "default": false, "description": "Release the button from its buttonhole instead."}
        }),
        &["button"],
    )
}

fn kind_of_hardware(s: &Session, id: u32) -> Result<(usize, StyleKind), CommandError> {
    let index = s.garment.hardware.iter().position(|h| h.id == id).ok_or_else(|| missing(format!("no hardware with id {id}")))?;
    let kind = s.garment.styles.iter().find(|x| x.id == s.garment.hardware[index].style).map(|x| x.kind).ok_or_else(|| missing("the style of the hardware is gone"))?;
    Ok((index, kind))
}

fn fasten(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (bi, bk) = kind_of_hardware(s, required_id(a, "button")?)?;
    if bk != StyleKind::Button {
        return Err(bad("button must be the hardware id of a button"));
    }
    let before = s.garment.clone();
    let button_id = s.garment.hardware[bi].id;
    if flag(a, "unfasten").unwrap_or(false) {
        let hole = s.garment.hardware[bi].fastened_to.take();
        if let Some(h) = s.garment.hardware.iter_mut().find(|h| Some(h.id) == hole) {
            h.fastened_to = None;
        }
        if hole.is_none() {
            return Ok(json!({"changed": false, "button": button_id}));
        }
        record(s, "Unfasten Button", before);
        return Ok(json!({"changed": true, "button": button_id, "buttonhole": hole}));
    }
    let (hi, hk) = kind_of_hardware(s, required_id(a, "buttonhole")?)?;
    if hk != StyleKind::Buttonhole {
        return Err(bad("buttonhole must be the hardware id of a buttonhole"));
    }
    let hole_id = s.garment.hardware[hi].id;
    for h in s.garment.hardware.iter_mut() {
        if h.id != button_id && h.fastened_to == Some(hole_id) {
            h.fastened_to = None;
        }
    }
    let old = s.garment.hardware[bi].fastened_to.replace(hole_id);
    if let Some(h) = s.garment.hardware.iter_mut().find(|h| Some(h.id) == old && h.id != hole_id) {
        h.fastened_to = None;
    }
    s.garment.hardware[hi].fastened_to = Some(button_id);
    if s.garment == before {
        return Ok(json!({"changed": false, "button": button_id, "buttonhole": hole_id}));
    }
    record(s, "Fasten Button", before);
    Ok(json!({"changed": true, "button": button_id, "buttonhole": hole_id}))
}

fn move_schema() -> Value {
    obj(
        json!({
            "hardware": ident("Hardware id."),
            "panel": ident("Move to this panel; needs point."),
            "point": xy("New anchor point; it must lie inside the outline."),
            "space": space(),
            "rotation": num("New rotation about the surface normal.", -360.0, 360.0, Some("deg"))
        }),
        &["hardware"],
    )
}

fn move_to(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = required_id(a, "hardware")?;
    let index = s.garment.hardware.iter().position(|h| h.id == id).ok_or_else(|| missing(format!("no hardware with id {id}")))?;
    let point = optional_vec2(a, "point")?;
    if point.is_none() && (a.get("panel").is_some() || a.get("rotation").is_none()) {
        return Err(bad("give point (with an optional panel) or rotation"));
    }
    let before = s.garment.clone();
    let mut anchor = s.garment.hardware[index].anchor.clone();
    if let Some(p) = point {
        let panel = panel_of(&s.garment, id_of(a, &anchor))?;
        anchor.point = local_point(panel, p, text(a, "space") != Some("local"));
        if !inside(panel, anchor.point) {
            return Err(geometry(format!("the point lies outside panel {}", panel.id)));
        }
        anchor.panel = panel.id;
    }
    let h = &mut s.garment.hardware[index];
    h.anchor = anchor;
    h.rotation = number_or(a, "rotation", h.rotation);
    let out = describe(h);
    if s.garment == before {
        return Ok(json!({"changed": false, "hardware": out}));
    }
    record(s, "Move Hardware", before);
    Ok(json!({"changed": true, "hardware": out}))
}

fn id_of(a: &Value, anchor: &Anchor) -> u32 {
    id(a, "panel").unwrap_or(anchor.panel)
}

fn delete_schema() -> Value {
    obj(
        json!({
            "hardware": ident("Hardware id."),
            "hardware_ids": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Several hardware ids."}
        }),
        &[],
    )
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mut ids = if a.get("hardware_ids").is_some() { id_list(a, "hardware_ids")? } else { Vec::new() };
    ids.extend(id(a, "hardware"));
    if ids.is_empty() {
        return Err(bad("give hardware or hardware_ids"));
    }
    if let Some(gone) = ids.iter().find(|i| !s.garment.hardware.iter().any(|h| h.id == **i)) {
        return Err(missing(format!("no hardware with id {gone}")));
    }
    let before = s.garment.clone();
    let styles: Vec<u32> = s.garment.hardware.iter().filter(|h| ids.contains(&h.id)).map(|h| h.style).collect();
    s.garment.hardware.retain(|h| !ids.contains(&h.id));
    for h in s.garment.hardware.iter_mut() {
        if h.fastened_to.is_some_and(|x| ids.contains(&x)) {
            h.fastened_to = None;
        }
    }
    let used: Vec<u32> = s.garment.hardware.iter().map(|h| h.style).collect();
    s.garment.styles.retain(|x| !(styles.contains(&x.id) && x.kind == StyleKind::Trim && mesh_of(x).is_some() && !used.contains(&x.id)));
    record(s, "Delete Hardware", before);
    Ok(json!({"changed": true, "removed": ids.len(), "remaining": s.garment.hardware.len()}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "button.add",
            mcp: "button_add",
            group: Group::Sew,
            label: "Add Button",
            description: "Place a button of a button style at a point inside a panel. In the simulation it is a small rigid disc attached to the panel triangle that holds the anchor.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: button,
        },
        Command {
            id: "buttonhole.add",
            mcp: "buttonhole_add",
            group: Group::Sew,
            label: "Add Buttonhole",
            description: "Place a buttonhole of a buttonhole style at a point inside a panel.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: buttonhole,
        },
        Command {
            id: "button.fasten",
            mcp: "button_fasten",
            group: Group::Sew,
            label: "Fasten Button",
            description: "Fasten a button to a buttonhole: during simulation the two anchor points are held together by one pair constraint. A button holds one buttonhole and the reverse. unfasten releases the button.",
            keys: &[],
            hints: SET,
            schema: fasten_schema,
            run: fasten,
        },
        Command {
            id: "hardware.move",
            mcp: "hardware_move",
            group: Group::Sew,
            label: "Move Hardware",
            description: "Move a button, buttonhole or trim to another point or panel, or turn it.",
            keys: &[],
            hints: SET,
            schema: move_schema,
            run: move_to,
        },
        Command {
            id: "hardware.delete",
            mcp: "hardware_delete",
            group: Group::Sew,
            label: "Delete Hardware",
            description: "Remove buttons, buttonholes or trims. A fastened partner is released. An imported trim mesh that nothing else uses is removed with it.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
    ]
}
