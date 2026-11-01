use std::collections::BTreeMap;

use ffb_fabric::Colorway;
use ffb_pattern::Garment;
use serde_json::{json, Value};

use super::common::*;
use super::schema as sc;
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

fn colors_schema(nullable: bool) -> Value {
    let mut item = sc::rgba("Colour as sRGB [r, g, b, a], each 0 to 1.");
    if nullable {
        item["type"] = json!(["array", "null"]);
    }
    json!({
        "type": "object",
        "additionalProperties": item,
        "description": if nullable { "Colour per fabric id (keys are fabric ids); null removes the entry so the fabric's own colour shows." } else { "Colour per fabric id (keys are fabric ids). Fabrics left out use their own front colour." }
    })
}

fn rgba(v: &Value, path: &str) -> Result<[f32; 4], CommandError> {
    let items = v.as_array().filter(|a| a.len() == 3 || a.len() == 4).ok_or_else(|| bad(format!("{path} must be [r, g, b, a] with values 0 to 1")))?;
    let mut out = [1.0f32; 4];
    for (i, item) in items.iter().enumerate() {
        let x = item.as_f64().filter(|x| x.is_finite() && (0.0..=1.0).contains(x)).ok_or_else(|| bad(format!("{path} must hold numbers between 0 and 1")))?;
        out[i] = x as f32;
    }
    Ok(out)
}

type ColorEdits = Vec<(u32, Option<[f32; 4]>)>;

fn read_colors(g: &Garment, a: &Value, nullable: bool) -> Result<ColorEdits, CommandError> {
    let Some(v) = a.get("fabric_colors") else { return Ok(Vec::new()) };
    let map = v.as_object().ok_or_else(|| bad("fabric_colors must be an object keyed by fabric id"))?;
    let mut out = Vec::new();
    for (key, value) in map {
        let id: u32 = key.parse().map_err(|_| bad(format!("fabric_colors key {key} is not a fabric id")))?;
        fabric_index(g, id)?;
        if value.is_null() {
            if !nullable {
                return Err(bad(format!("fabric_colors.{key} must be a colour")));
            }
            out.push((id, None));
        } else {
            out.push((id, Some(rgba(value, &format!("fabric_colors.{key}"))?)));
        }
    }
    Ok(out)
}

fn colorway_index(g: &Garment, id: u32) -> Result<usize, CommandError> {
    g.colorways.iter().position(|c| c.id == id).ok_or_else(|| missing(format!("no colorway with id {id}")))
}

fn create_schema() -> Value {
    sc::obj(
        json!({
            "name": sc::name("Colorway name."),
            "from": sc::fabric_id("Colorway id to copy the colours from. Leave out to start from each fabric's front colour."),
            "fabric_colors": colors_schema(false)
        }),
        &["name"],
    )
}

fn create_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = text(a, "name")?.ok_or_else(|| bad("name is required"))?;
    let from = opt_id(a, "from")?;
    mutate(s, "New Colorway", |g| {
        ensure_default(g);
        let mut colors: BTreeMap<u32, [f32; 4]> = match from {
            Some(f) => g.colorways[colorway_index(g, f)?].fabric_colors.clone(),
            None => BTreeMap::new(),
        };
        for f in &g.fabrics {
            colors.entry(f.id).or_insert(f.look.front.color);
        }
        for (id, c) in read_colors(g, a, false)? {
            if let Some(c) = c {
                colors.insert(id, c);
            }
        }
        let id = next_colorway_id(g);
        let w = Colorway { id, name, fabric_colors: colors };
        g.colorways.push(w.clone());
        Ok(json!({"colorway": w}))
    })
}

fn update_schema() -> Value {
    sc::obj(json!({"id": sc::fabric_id("Colorway id."), "name": sc::name("New name."), "fabric_colors": colors_schema(true)}), &["id"])
}

fn update_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let name = text(a, "name")?;
    mutate(s, "Edit Colorway", |g| {
        let i = colorway_index(g, id)?;
        let edits = read_colors(g, a, true)?;
        let w = &mut g.colorways[i];
        if let Some(n) = name {
            w.name = n;
        }
        for (fabric, color) in edits {
            match color {
                Some(c) => {
                    w.fabric_colors.insert(fabric, c);
                }
                None => {
                    w.fabric_colors.remove(&fabric);
                }
            }
        }
        Ok(json!({"colorway": w}))
    })
}

fn id_schema() -> Value {
    sc::obj(json!({"id": sc::fabric_id("Colorway id.")}), &["id"])
}

fn delete_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    mutate(s, "Delete Colorway", |g| {
        let i = colorway_index(g, id)?;
        g.colorways.remove(i);
        if g.active_colorway == Some(id) {
            g.active_colorway = None;
        }
        Ok(json!({"deleted": id, "active_colorway": g.active_colorway}))
    })
}

fn activate_schema() -> Value {
    sc::obj(json!({"id": sc::fabric_id("Colorway to show. Leave out to show each fabric's own colour.")}), &[])
}

fn activate_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = opt_id(a, "id")?;
    mutate(s, "Activate Colorway", |g| {
        if let Some(i) = id {
            colorway_index(g, i)?;
        }
        g.active_colorway = id;
        Ok(json!({"active_colorway": id}))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "colorway.create",
            mcp: "colorway_create",
            group: Group::Fabric,
            label: "New Colorway",
            description: "Add a colorway: one colour per fabric, copied from another colorway or from each fabric's front colour, with optional overrides.",
            keys: &[],
            hints: WRITE,
            schema: create_schema,
            run: create_run,
        },
        Command {
            id: "colorway.update",
            mcp: "colorway_update",
            group: Group::Fabric,
            label: "Edit Colorway",
            description: "Rename a colorway or change the colour it gives some fabrics. A null colour removes that fabric's entry.",
            keys: &[],
            hints: SET,
            schema: update_schema,
            run: update_run,
        },
        Command {
            id: "colorway.delete",
            mcp: "colorway_delete",
            group: Group::Fabric,
            label: "Delete Colorway",
            description: "Delete a colorway. If it was active, the fabrics show their own colours again.",
            keys: &[],
            hints: DESTROY,
            schema: id_schema,
            run: delete_run,
        },
        Command {
            id: "colorway.activate",
            mcp: "colorway_activate",
            group: Group::Fabric,
            label: "Activate Colorway",
            description: "Show a colorway: its colour per fabric replaces the fabric's front colour in views and renders. Without id the colorway is switched off.",
            keys: &[],
            hints: SET,
            schema: activate_schema,
            run: activate_run,
        },
    ]
}
