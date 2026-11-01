use ffb_pattern::sew::next_tack_id;
use ffb_pattern::{Tack, TackEnd};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, WRITE};

fn end_of(s: &Session, a: &Value, tag: &str, world: bool) -> Result<TackEnd, CommandError> {
    let key = |name: &str| format!("{name}_{tag}");
    let body = optional_vec3(a, &key("body_point"))?;
    match (id(a, &key("panel")), optional_vec2(a, &key("point"))?, body) {
        (Some(panel), Some(point), None) => {
            let p = s.garment.panels.iter().find(|p| p.id == panel).ok_or_else(|| missing(format!("no panel with id {panel}")))?;
            let local = if world { p.placement().to_local(point) } else { point };
            Ok(TackEnd { panel: Some(panel), point: local, body_point: None })
        }
        (None, None, Some(b)) => Ok(TackEnd { panel: None, point: [0.0, 0.0], body_point: Some(b) }),
        _ => Err(bad(format!("end {tag} needs panel_{tag} with point_{tag}, or body_point_{tag}"))),
    }
}

fn add_schema() -> Value {
    obj(
        json!({
            "panel_a": panel_id("Panel of end a."),
            "point_a": xy("Point of end a on its panel."),
            "body_point_a": xyz("End a on a body instead of a panel, in body space."),
            "panel_b": panel_id("Panel of end b."),
            "point_b": xy("Point of end b on its panel."),
            "body_point_b": xyz("End b on a body instead of a panel, in body space."),
            "length": num("Rest distance the two ends are held at; 0 holds them together.", 0.0, 100_000.0, Some("mm")),
            "space": space()
        }),
        &[],
    )
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let world = text(a, "space") != Some("local");
    let (ea, eb) = (end_of(s, a, "a", world)?, end_of(s, a, "b", world)?);
    if ea.panel.is_none() && eb.panel.is_none() {
        return Err(bad("a tack needs at least one end on a panel"));
    }
    let length = number_or(a, "length", 0.0);
    if !(0.0..=100_000.0).contains(&length) {
        return Err(bad("length must be between 0 and 100000"));
    }
    let before = s.garment.clone();
    let tack = Tack { id: next_tack_id(&s.garment), a: ea, b: eb, length };
    let out = serde_json::to_value(&tack).unwrap_or(Value::Null);
    s.garment.tacks.push(tack);
    record(s, "Add Tack", before);
    Ok(out)
}

fn delete_schema() -> Value {
    obj(
        json!({
            "tack": panel_id("Tack id as listed by sew.list."),
            "tacks": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Several tack ids."},
            "all": {"type": "boolean", "default": false, "description": "Remove every tack."}
        }),
        &[],
    )
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let all = flag(a, "all").unwrap_or(false);
    let mut ids = if a.get("tacks").is_some() { id_list(a, "tacks")? } else { Vec::new() };
    ids.extend(id(a, "tack"));
    if all == !ids.is_empty() {
        return Err(bad("give exactly one of tack, tacks or all"));
    }
    if let Some(missing_id) = ids.iter().find(|i| !s.garment.tacks.iter().any(|t| t.id == **i)) {
        return Err(missing(format!("no tack with id {missing_id}")));
    }
    let before = s.garment.clone();
    let count = s.garment.tacks.len();
    s.garment.tacks.retain(|t| !all && !ids.contains(&t.id));
    let removed = count - s.garment.tacks.len();
    if removed == 0 {
        return Ok(json!({"changed": false, "removed": 0, "remaining": count}));
    }
    record(s, "Delete Tack", before);
    Ok(json!({"changed": true, "removed": removed, "remaining": s.garment.tacks.len()}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "tack.add",
            mcp: "tack_add",
            group: Group::Sew,
            label: "Add Tack",
            description: "Add a temporary point-to-point constraint between two panel points, or between a panel point and a body point. Points are in world or local space and are stored panel-local.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: add,
        },
        Command {
            id: "tack.delete",
            mcp: "tack_delete",
            group: Group::Sew,
            label: "Delete Tack",
            description: "Remove one tack by id, several by tacks, or every tack with all.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
    ]
}
