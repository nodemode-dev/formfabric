use ffb_fabric::validate;
use serde_json::{json, Value};

use super::common::*;
use super::library::fabric_json;
use super::schema as sc;
use crate::{Command, CommandError, Group, Session, SET, WRITE};

fn copy_schema() -> Value {
    sc::obj(json!({"id": sc::fabric_id("Fabric to copy."), "name": sc::name("Name of the copy. Defaults to the original name with copy added.")}), &["id"])
}

fn copy_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "id")?;
    let name = text(a, "name")?;
    mutate(s, "Copy Fabric", |g| {
        let at = fabric_index(g, id)?;
        let mut f = g.fabrics[at].clone();
        let new_id = next_fabric_id(g);
        let wanted = name.unwrap_or_else(|| format!("{} copy", f.name));
        f.name = wanted.chars().take(validate::NAME_MAX).collect();
        f.id = new_id;
        for w in g.colorways.iter_mut() {
            if let Some(c) = w.fabric_colors.get(&id).copied() {
                w.fabric_colors.insert(new_id, c);
            }
        }
        g.fabrics.insert(at + 1, f);
        Ok(json!({"fabric": fabric_json(g, &g.fabrics[at + 1])}))
    })
}

fn merge_schema() -> Value {
    sc::obj(
        json!({
            "ids": {"type": "array", "minItems": 2, "maxItems": 64, "items": sc::fabric_id("Fabric id."), "description": "Two or more fabrics. The first one supplies the physical values and look of the new fabric."},
            "name": sc::name("Name of the new fabric. Defaults to the first name with merged added.")
        }),
        &["ids"],
    )
}

fn merge_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mut ids = id_list(a, "ids")?;
    let mut seen = Vec::new();
    ids.retain(|id| {
        let fresh = !seen.contains(id);
        seen.push(*id);
        fresh
    });
    if ids.len() < 2 {
        return Err(bad("ids needs at least two different fabrics"));
    }
    let name = text(a, "name")?;
    mutate(s, "Merge Fabrics", |g| {
        let first = fabric_index(g, ids[0])?;
        for id in &ids {
            fabric_index(g, *id)?;
        }
        let new_id = next_fabric_id(g);
        let mut f = g.fabrics[first].clone();
        let wanted = name.unwrap_or_else(|| format!("{} merged", f.name));
        f.name = wanted.chars().take(validate::NAME_MAX).collect();
        f.id = new_id;
        let moved = g.panels.iter().filter(|p| p.fabric.is_some_and(|x| ids.contains(&x))).count();
        for p in g.panels.iter_mut().filter(|p| p.fabric.is_some_and(|x| ids.contains(&x))) {
            p.fabric = Some(new_id);
        }
        for w in g.colorways.iter_mut() {
            if let Some(c) = w.fabric_colors.get(&ids[0]).copied() {
                w.fabric_colors.insert(new_id, c);
            }
            for id in &ids {
                w.fabric_colors.remove(id);
            }
        }
        if g.default_fabric.is_some_and(|d| ids.contains(&d)) {
            g.default_fabric = Some(new_id);
        }
        let at = g.fabrics.iter().position(|x| ids.contains(&x.id)).unwrap_or(0);
        g.fabrics.retain(|x| !ids.contains(&x.id));
        g.fabrics.insert(at.min(g.fabrics.len()), f);
        let index = fabric_index(g, new_id)?;
        Ok(json!({"fabric": fabric_json(g, &g.fabrics[index]), "merged": ids, "reassigned": moved}))
    })
}

fn order_schema() -> Value {
    sc::obj(json!({"ids": {"type": "array", "minItems": 1, "maxItems": 4096, "items": sc::fabric_id("Fabric id."), "description": "Fabric ids in the wanted order. Fabrics left out follow in their current order."}}), &["ids"])
}

fn order_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = id_list(a, "ids")?;
    mutate(s, "Order Fabrics", |g| {
        for id in &ids {
            fabric_index(g, *id)?;
        }
        let mut rest = std::mem::take(&mut g.fabrics);
        let mut ordered = Vec::with_capacity(rest.len());
        for id in &ids {
            if let Some(i) = rest.iter().position(|f| f.id == *id) {
                ordered.push(rest.remove(i));
            }
        }
        ordered.extend(rest);
        g.fabrics = ordered;
        Ok(json!({"order": g.fabrics.iter().map(|f| f.id).collect::<Vec<_>>()}))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "fabric.copy",
            mcp: "fabric_copy",
            group: Group::Fabric,
            label: "Copy Fabric",
            description: "Copy a fabric to a new fabric with its own id, placed right after the original, with the same colorway colours. Panels keep their assignment. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: copy_schema,
            run: copy_run,
        },
        Command {
            id: "fabric.merge",
            mcp: "fabric_merge",
            group: Group::Fabric,
            label: "Merge Fabrics",
            description: "Merge two or more fabrics into one new fabric that takes the values of the first. Panels that used any of them move to the new fabric, the originals are removed and the default follows. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: merge_schema,
            run: merge_run,
        },
        Command {
            id: "fabric.order",
            mcp: "fabric_order",
            group: Group::Fabric,
            label: "Order Fabrics",
            description: "Set the order of the fabric list. Ids not listed follow in their current order. Panels and the default fabric are not touched. One undo step.",
            keys: &[],
            hints: SET,
            schema: order_schema,
            run: order_run,
        },
    ]
}
