use ffb_pattern::geom::{center, union, Bounds, V2};
use ffb_pattern::xform::{self, Mirror, Xform};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::{commit, edit_panel, index_of, replace};
use crate::{Command, CommandError, Edit, Group, Session, DESTROY, SET, WRITE};

fn transform_schema() -> Value {
    sc::obj(
        json!({
            "panels": sc::ids("Panel ids to transform together."),
            "translate": sc::xy("Move by this offset."),
            "rotate": sc::deg("Rotate counter-clockwise about the pivot.", -360.0, 360.0, Some(0.0)),
            "scale": {
                "type": ["number", "array"],
                "minimum": 0.01,
                "maximum": 100,
                "items": {"type": "number", "minimum": 0.01, "maximum": 100},
                "minItems": 2,
                "maxItems": 2,
                "description": "Scale factor about the pivot: one number for uniform scaling or [sx, sy] along the pattern axes. Uneven scaling bakes the panel rotation into its outline."
            },
            "mirror": sc::text("x flips left to right across the vertical line through the pivot, y flips top to bottom across the horizontal line.", &["x", "y"]),
            "pivot": sc::xy("Pivot for rotate, scale and mirror. Defaults to the centre of the selected panels.")
        }),
        &["panels"],
    )
}

fn scale_of(a: &Value) -> Result<Option<V2>, CommandError> {
    match a.get("scale") {
        None | Some(Value::Null) => Ok(None),
        Some(v) if v.is_number() => Ok(v.as_f64().map(|k| [k, k])),
        Some(v) => vec2(v, "scale").map(Some),
    }
}

fn transform(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(transform_schema, a)?;
    let ids = id_list(a, "panels")?;
    let (translate, scale, rotate) = (optional_vec2(a, "translate")?, scale_of(a)?, number_or(a, "rotate", 0.0));
    let mirror = text(a, "mirror").map(|m| if m == "x" { Mirror::X } else { Mirror::Y });
    if translate.is_none() && scale.is_none() && mirror.is_none() && rotate == 0.0 {
        return Err(bad("give at least one of translate, rotate, scale or mirror"));
    }
    let indices = ids.iter().map(|&id| index_of(s, id)).collect::<Result<Vec<_>, _>>()?;
    let pivot = match optional_vec2(a, "pivot")? {
        Some(p) => p,
        None => {
            let bounds: Option<Bounds> = indices.iter().filter_map(|&i| s.garment.panels[i].world_bounds()).reduce(union);
            bounds.map_or([0.0, 0.0], center)
        }
    };
    let x = Xform { mirror, scale, rotate, translate: translate.unwrap_or([0.0, 0.0]), pivot };
    let mut edits = Vec::new();
    for &i in &indices {
        let before = s.garment.panels[i].clone();
        xform::apply(&mut s.garment.panels[i], &x);
        edits.push(replace(i, "Transform", before, s.garment.panels[i].clone()));
    }
    commit(s, "Transform", edits);
    Ok(json!({"panels": ids, "pivot": pivot}))
}

fn duplicate_schema() -> Value {
    sc::obj(
        json!({
            "panels": sc::ids("Panel ids to copy."),
            "count": sc::int("Number of copies of each panel.", 1.0, 50.0, Some(1.0)),
            "offset": sc::xy("Shift between one copy and the next, starting from the original. Defaults to [20, 20].")
        }),
        &["panels"],
    )
}

fn duplicate(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(duplicate_schema, a)?;
    let ids = id_list(a, "panels")?;
    let offset = optional_vec2(a, "offset")?.unwrap_or([20.0, 20.0]);
    let copies = count(a, "count").unwrap_or(1);
    let sources = ids.iter().map(|&id| index_of(s, id).map(|i| s.garment.panels[i].clone())).collect::<Result<Vec<_>, _>>()?;
    let mut next = s.garment.next_id();
    let mut next_print = ffb_pattern::trim::next_print_id(&s.garment);
    let mut edits = Vec::new();
    let mut created = Vec::new();
    for k in 1..=copies {
        for src in &sources {
            let mut copy = src.clone();
            copy.id = next;
            next += 1;
            for inner in copy.inner.iter_mut() {
                inner.id = next;
                next += 1;
            }
            for print in copy.prints.iter_mut() {
                print.id = next_print;
                next_print = next_print.saturating_add(1);
            }
            copy.name = if copies > 1 { format!("{} copy {k}", src.name) } else { format!("{} copy", src.name) };
            copy.position = [src.position[0] + offset[0] * k as f64, src.position[1] + offset[1] * k as f64];
            created.push(copy.id);
            edits.push(Edit::Panel { label: "Duplicate".into(), index: s.garment.panels.len(), before: None, after: Some(Box::new(copy.clone())) });
            s.garment.panels.push(copy);
        }
    }
    commit(s, "Duplicate", edits);
    Ok(json!({"created": created}))
}

fn delete_schema() -> Value {
    sc::obj(json!({"panels": sc::ids("Panel ids to remove, with their inner shapes.")}), &["panels"])
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(delete_schema, a)?;
    let ids = id_list(a, "panels")?;
    let mut indices = ids.iter().map(|&id| index_of(s, id)).collect::<Result<Vec<_>, _>>()?;
    indices.sort_unstable_by(|x, y| y.cmp(x));
    let mut edits = Vec::new();
    for i in indices {
        let gone = s.garment.panels.remove(i);
        edits.push(Edit::Panel { label: "Delete".into(), index: i, before: Some(Box::new(gone)), after: None });
    }
    commit(s, "Delete", edits);
    Ok(json!({"deleted": ids, "panels": s.garment.panels.len()}))
}

fn rename_schema() -> Value {
    sc::obj(json!({"panel": sc::panel(), "name": sc::name("New panel name.")}), &["panel", "name"])
}

fn rename(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(rename_schema, a)?;
    let new = name(a, "name")?.ok_or_else(|| bad("name is required"))?;
    edit_panel(s, required_id(a, "panel")?, "Rename", |p| {
        p.name = new;
        Ok(json!({"name": p.name}))
    })
}

fn props_schema() -> Value {
    let mut props = json!({
        "panels": sc::ids("Panel ids to change."),
        "particle_distance": sc::mm("Simulation particle spacing.", 0.8, 700.0, None),
        "pressure": sc::ratio("Inflation pressure of the panel in the solver's units, along the panel normal; 0 is none.", -1_000_000.0, 1_000_000.0, None),
        "layer": sc::int("Layer order for overlapping panels; higher lies on top.", -100.0, 100.0, None),
        "grain": sc::deg("Grain line direction inside the panel.", -360.0, 360.0, None),
        "position": sc::xy("Absolute panel position on the pattern plane."),
        "rotation": sc::deg("Absolute panel rotation.", -360.0, 360.0, None),
        "inner": sc::inner()
    });
    sc::fold_fields(&mut props);
    props["fold_angle"].as_object_mut().into_iter().for_each(|m| {
        m.remove("default");
    });
    props["fold_strength"].as_object_mut().into_iter().for_each(|m| {
        m.remove("default");
    });
    props["inner"]["description"] = json!("Inner line id whose fold is set by fold_angle and fold_strength; needs exactly one panel.");
    sc::obj(props, &["panels"])
}

fn set_props(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(props_schema, a)?;
    let ids = id_list(a, "panels")?;
    let inner = optional_id(a, "inner");
    let fold = number(a, "fold_angle").is_some() || number(a, "fold_strength").is_some();
    if (inner.is_some() || fold) && (inner.is_none() || ids.len() != 1) {
        return Err(bad("fold settings need inner and exactly one panel"));
    }
    let position = optional_vec2(a, "position")?;
    let indices = ids.iter().map(|&id| index_of(s, id)).collect::<Result<Vec<_>, _>>()?;
    let mut edits = Vec::new();
    for &i in &indices {
        let before = s.garment.panels[i].clone();
        let p = &mut s.garment.panels[i];
        if let Some(id) = inner {
            let panel_id = p.id;
            let target = p.inner.iter_mut().find(|n| n.id == id).ok_or_else(|| missing(format!("panel {panel_id} has no inner shape {id}")))?;
            target.fold_angle = number_or(a, "fold_angle", target.fold_angle);
            target.fold_strength = number_or(a, "fold_strength", target.fold_strength);
        }
        p.particle_distance = number_or(a, "particle_distance", p.particle_distance);
        p.pressure = number_or(a, "pressure", p.pressure);
        p.layer = number(a, "layer").map_or(p.layer, |x| x as i32);
        p.grain = number_or(a, "grain", p.grain);
        p.rotation = number_or(a, "rotation", p.rotation);
        p.position = position.unwrap_or(p.position);
        if *p != before {
            edits.push(replace(i, "Panel Properties", before, p.clone()));
        }
    }
    let changed = !edits.is_empty();
    commit(s, "Panel Properties", edits);
    Ok(json!({"panels": ids, "changed": changed}))
}

fn drag_schema() -> Value {
    sc::obj(json!({"label": sc::name("Name of the undo step the drag becomes. Defaults to Drag.")}), &[])
}

fn begin_drag(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(drag_schema, a)?;
    s.history.begin_group(&name(a, "label")?.unwrap_or_else(|| "Drag".into()));
    Ok(json!({"grouping": true}))
}

fn end_drag(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let was = s.history.grouping();
    s.history.end_group();
    s.epoch += 1;
    Ok(json!({"grouping": false, "was_grouping": was, "position": s.history.position(), "steps": s.history.len()}))
}

fn no_args() -> Value {
    sc::obj(json!({}), &[])
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.transform",
            mcp: "pattern_transform",
            group: Group::Pattern,
            label: "Transform",
            description: "Mirror, scale, rotate and move whole panels in this order about a pivot (default: the centre of the selection). Rotation and translation change the panel placement; uneven scale and mirror rewrite the outline and keep it counter-clockwise. One undo step for all panels.",
            keys: &["A"],
            hints: WRITE,
            schema: transform_schema,
            run: transform,
        },
        Command {
            id: "pattern.duplicate",
            mcp: "pattern_duplicate",
            group: Group::Pattern,
            label: "Duplicate",
            description: "Copy panels with their inner shapes under new ids, shifted by offset per copy. Returns the created panel ids.",
            keys: &["Mod+D"],
            hints: WRITE,
            schema: duplicate_schema,
            run: duplicate,
        },
        Command {
            id: "pattern.delete",
            mcp: "pattern_delete",
            group: Group::Pattern,
            label: "Delete Panels",
            description: "Remove panels and their inner shapes. Undo restores them at the same positions in the list.",
            keys: &["Delete"],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
        Command {
            id: "pattern.rename",
            mcp: "pattern_rename",
            group: Group::Pattern,
            label: "Rename Panel",
            description: "Give a panel a new name of 1 to 64 characters.",
            keys: &["F2"],
            hints: SET,
            schema: rename_schema,
            run: rename,
        },
        Command {
            id: "pattern.set_props",
            mcp: "pattern_set_props",
            group: Group::Pattern,
            label: "Panel Properties",
            description: "Set particle distance, layer, grain direction, position or rotation on one or more panels, and the fold angle (0 to 360, 180 flat) and strength (0 to 20) of one inner line. Only the given values change.",
            keys: &[],
            hints: SET,
            schema: props_schema,
            run: set_props,
        },
        Command {
            id: "pattern.begin_drag",
            mcp: "pattern_begin_drag",
            group: Group::Pattern,
            label: "Begin Drag",
            description: "Open an undo group so many edits, such as repeated pattern.move_point calls during a mouse drag, become one history step. Close it with pattern.end_drag; undo and redo also close it.",
            keys: &[],
            hints: SET,
            schema: drag_schema,
            run: begin_drag,
        },
        Command {
            id: "pattern.end_drag",
            mcp: "pattern_end_drag",
            group: Group::Pattern,
            label: "End Drag",
            description: "Close the open undo group and commit its edits as a single history step. Safe to call when no group is open.",
            keys: &[],
            hints: SET,
            schema: no_args,
            run: end_drag,
        },
    ]
}
