use ffb_fabric::presets;
use ffb_fabric::units::{self, FIELDS};
use ffb_fabric::validate;
use ffb_pattern::Garment;
use serde_json::{json, Value};

use super::common::*;
use super::schema as sc;
use crate::{Command, CommandError, Group, Session, DESTROY, READ, SET, WRITE};

fn preset_json(p: &presets::Preset) -> Value {
    let mut v = serde_json::to_value(p).unwrap_or(Value::Null);
    let sliders: serde_json::Map<String, Value> = FIELDS.iter().map(|f| (f.key.to_string(), json!(units::get(&p.physical, f.key).map_or(0.0, |x| units::to_slider(f, x))))).collect();
    v["sliders"] = Value::Object(sliders);
    v
}

fn presets_schema() -> Value {
    sc::obj(json!({"id": {"type": "string", "description": "Preset id to return only that preset."}}), &[])
}

fn presets_run(_: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let all = presets::all();
    let chosen: Vec<&presets::Preset> = match a.get("id").and_then(Value::as_str) {
        Some(id) => vec![all.iter().find(|p| p.id == id).ok_or_else(|| missing(format!("no preset named {id}")))?],
        None => all.iter().collect(),
    };
    let fields: Vec<Value> = FIELDS.iter().map(|f| serde_json::to_value(f).unwrap_or(Value::Null)).collect();
    Ok(json!({
        "default": presets::DEFAULT_ID,
        "presets": chosen.into_iter().map(preset_json).collect::<Vec<_>>(),
        "fields": fields,
        "slider_max": units::SLIDER_MAX,
        "sources": presets::SOURCES.iter().map(|(k, v)| json!({"key": k, "citation": v})).collect::<Vec<_>>(),
        "data_file": "encyclopedia/16-FABRIC-DATA.md (planning sidecar)"
    }))
}

pub(super) fn fabric_json(g: &Garment, f: &ffb_fabric::Fabric) -> Value {
    let mut v = serde_json::to_value(f).unwrap_or(Value::Null);
    v["panels"] = json!(usage(g, f.id));
    v["is_default"] = json!(g.default_fabric == Some(f.id));
    let active = g.active_colorway.and_then(|c| g.colorways.iter().find(|w| w.id == c));
    v["color"] = json!(active.and_then(|w| w.fabric_colors.get(&f.id)).copied().unwrap_or(f.look.front.color));
    v
}

fn list_run(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let g = &s.garment;
    Ok(json!({
        "fabrics": g.fabrics.iter().map(|f| fabric_json(g, f)).collect::<Vec<_>>(),
        "default_fabric": g.default_fabric,
        "colorways": g.colorways,
        "active_colorway": g.active_colorway
    }))
}

fn create_schema() -> Value {
    sc::obj(
        json!({
            "name": sc::name("Fabric name. Defaults to the preset name."),
            "preset": {"type": "string", "description": "Preset id from fabric.presets. Defaults to the default preset."},
            "physical": sc::physical(true),
            "look": sc::look()
        }),
        &[],
    )
}

fn create_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let preset = match a.get("preset").and_then(Value::as_str) {
        Some(id) => presets::find(id).ok_or_else(|| bad(format!("no preset named {id}; list them with fabric.presets")))?,
        None => presets::default_preset(),
    };
    let name = text(a, "name")?;
    mutate(s, "New Fabric", |g| {
        let id = next_fabric_id(g);
        let mut f = presets::instantiate(&preset, id, name.as_deref());
        if let Some(p) = a.get("physical") {
            validate::physical_patch(&mut f.physical, p).map_err(bad)?;
        }
        if let Some(l) = a.get("look") {
            validate::look_patch(&mut f.look, l).map_err(bad)?;
        }
        g.fabrics.push(f);
        ensure_default(g);
        Ok(json!({"fabric": fabric_json(g, &g.fabrics[g.fabrics.len() - 1])}))
    })
}

fn update_schema() -> Value {
    sc::obj(
        json!({
            "id": sc::fabric_id("Fabric id from fabric.list."),
            "name": sc::name("New fabric name."),
            "physical": sc::physical(true),
            "look": sc::look()
        }),
        &["id"],
    )
}

fn update_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let name = text(a, "name")?;
    mutate(s, "Edit Fabric", |g| {
        let i = fabric_index(g, id)?;
        let mut f = g.fabrics[i].clone();
        if let Some(n) = name {
            f.name = n;
        }
        if let Some(p) = a.get("physical") {
            validate::physical_patch(&mut f.physical, p).map_err(bad)?;
        }
        if let Some(l) = a.get("look") {
            validate::look_patch(&mut f.look, l).map_err(bad)?;
        }
        g.fabrics[i] = f;
        Ok(json!({"fabric": fabric_json(g, &g.fabrics[i])}))
    })
}

fn duplicate_schema() -> Value {
    sc::obj(json!({"id": sc::fabric_id("Fabric to copy."), "name": sc::name("Name of the copy. Defaults to the original name with copy added.")}), &["id"])
}

fn duplicate_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let name = text(a, "name")?;
    mutate(s, "Duplicate Fabric", |g| {
        let mut f = fabric_of(g, id)?.clone();
        let new_id = next_fabric_id(g);
        let wanted = name.unwrap_or_else(|| format!("{} copy", f.name));
        f.name = wanted.chars().take(validate::NAME_MAX).collect();
        f.id = new_id;
        for w in g.colorways.iter_mut() {
            if let Some(c) = w.fabric_colors.get(&id).copied() {
                w.fabric_colors.insert(new_id, c);
            }
        }
        g.fabrics.push(f);
        Ok(json!({"fabric": fabric_json(g, &g.fabrics[g.fabrics.len() - 1])}))
    })
}

fn delete_schema() -> Value {
    sc::obj(
        json!({
            "id": sc::fabric_id("Fabric to delete."),
            "reassign": sc::fabric_id("Fabric that takes over the panels using the deleted one. Required when panels use it.")
        }),
        &["id"],
    )
}

fn delete_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let reassign = opt_id(a, "reassign")?;
    mutate(s, "Delete Fabric", |g| {
        let i = fabric_index(g, id)?;
        if g.fabrics.len() == 1 {
            return Err(bad("the garment needs at least one fabric; create another before deleting this one"));
        }
        if reassign == Some(id) {
            return Err(bad("reassign must name a different fabric"));
        }
        if let Some(r) = reassign {
            fabric_index(g, r)?;
        }
        let users = usage(g, id);
        if !users.is_empty() && reassign.is_none() {
            return Err(bad(format!("fabric {id} is assigned to {} panels; pass reassign with the id of the fabric that should take them", users.len())));
        }
        for p in g.panels.iter_mut().filter(|p| p.fabric == Some(id)) {
            p.fabric = reassign;
        }
        g.fabrics.remove(i);
        for w in g.colorways.iter_mut() {
            w.fabric_colors.remove(&id);
        }
        if g.default_fabric == Some(id) {
            g.default_fabric = reassign.or_else(|| g.fabrics.first().map(|f| f.id));
        }
        Ok(json!({"deleted": id, "reassigned": users.len(), "default_fabric": g.default_fabric}))
    })
}

fn assign_schema() -> Value {
    sc::obj(
        json!({
            "fabric": sc::fabric_id("Fabric to assign. Leave out for the default fabric."),
            "panels": sc::ids("Panel ids that take the fabric."),
            "clear": {"type": "boolean", "default": false, "description": "Remove the explicit assignment so the panels follow the default fabric."}
        }),
        &["panels"],
    )
}

fn assign_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panels = id_list(a, "panels")?;
    let fabric = opt_id(a, "fabric")?;
    let clear = a.get("clear").and_then(Value::as_bool).unwrap_or(false);
    mutate(s, "Assign Fabric", |g| {
        if let Some(missing_id) = panels.iter().find(|p| g.index_of(**p).is_none()) {
            return Err(missing(format!("no panel with id {missing_id}")));
        }
        ensure_default(g);
        let target = if clear {
            None
        } else {
            let id = fabric.or(g.default_fabric).ok_or_else(|| bad("the garment has no default fabric"))?;
            fabric_index(g, id)?;
            Some(id)
        };
        let mut count = 0;
        for p in g.panels.iter_mut().filter(|p| panels.contains(&p.id)) {
            if p.fabric != target {
                p.fabric = target;
                count += 1;
            }
        }
        Ok(json!({"fabric": target, "panels": panels, "assigned": count}))
    })
}

fn default_schema() -> Value {
    sc::obj(json!({"id": sc::fabric_id("Fabric that panels without an explicit assignment use.")}), &["id"])
}

fn default_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    mutate(s, "Set Default Fabric", |g| {
        fabric_index(g, id)?;
        g.default_fabric = Some(id);
        Ok(json!({"default_fabric": id}))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "fabric.presets",
            mcp: "fabric_presets",
            group: Group::Fabric,
            label: "Fabric Presets",
            description: "List formfabric's own fabric presets with physical values in SI based units, look, source keys, whether each is provisional, and the unit and range of every physical field with the 0 to 99 slider mapping.",
            keys: &[],
            hints: READ,
            schema: presets_schema,
            run: presets_run,
        },
        Command {
            id: "fabric.list",
            mcp: "fabric_list",
            group: Group::Fabric,
            label: "Fabrics",
            description: "List the garment's fabrics with the panels using each, the default fabric, the colorways and the active colorway. The color of each fabric already accounts for the active colorway.",
            keys: &[],
            hints: READ,
            schema: crate::no_args,
            run: list_run,
        },
        Command {
            id: "fabric.create",
            mcp: "fabric_create",
            group: Group::Fabric,
            label: "New Fabric",
            description: "Add a fabric to the garment from a preset, with optional partial physical and look values. Out-of-range values are refused with bad_args. The first fabric becomes the default.",
            keys: &[],
            hints: WRITE,
            schema: create_schema,
            run: create_run,
        },
        Command {
            id: "fabric.update",
            mcp: "fabric_update",
            group: Group::Fabric,
            label: "Edit Fabric",
            description: "Change a fabric's name, physical properties (SI based units, ranges in the schema) and look. Only the fields given change; every value is range checked and the whole call is refused with bad_args if one is out of range.",
            keys: &[],
            hints: SET,
            schema: update_schema,
            run: update_run,
        },
        Command {
            id: "fabric.duplicate",
            mcp: "fabric_duplicate",
            group: Group::Fabric,
            label: "Duplicate Fabric",
            description: "Copy a fabric, including its colorway colours, under a new id.",
            keys: &[],
            hints: WRITE,
            schema: duplicate_schema,
            run: duplicate_run,
        },
        Command {
            id: "fabric.delete",
            mcp: "fabric_delete",
            group: Group::Fabric,
            label: "Delete Fabric",
            description: "Delete a fabric. The last fabric cannot be deleted, and a fabric used by panels needs reassign naming the fabric that takes them over.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete_run,
        },
        Command {
            id: "fabric.assign",
            mcp: "fabric_assign",
            group: Group::Fabric,
            label: "Assign Fabric",
            description: "Assign a fabric to panels, or clear the assignment so they follow the default fabric. A garment with no fabric gets the default fabric first.",
            keys: &[],
            hints: SET,
            schema: assign_schema,
            run: assign_run,
        },
        Command {
            id: "fabric.set_default",
            mcp: "fabric_set_default",
            group: Group::Fabric,
            label: "Set Default Fabric",
            description: "Choose the fabric that panels without an explicit assignment use.",
            keys: &[],
            hints: SET,
            schema: default_schema,
            run: default_run,
        },
    ]
}
