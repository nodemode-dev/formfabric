use ffb_pattern::geom::{cross, dist, mul, sub, unit};
use ffb_pattern::link::{derived_of, has_links, instance_axis, sync};
use ffb_pattern::polygon;
use ffb_pattern::{LinkKind, Panel, PanelLink};
use serde_json::{json, Value};

use super::support::*;
use crate::{Command, CommandError, Edit, Group, Session, SET, WRITE};

fn new_name(a: &Value, fallback: String) -> Result<String, CommandError> {
    match text(a, "name") {
        None => Ok(fallback),
        Some(raw) => {
            crate::args::clean_name("name", raw)?;
            let t = raw.trim();
            if t.is_empty() || t.chars().count() > 64 {
                return Err(bad("name must be 1 to 64 characters"));
            }
            Ok(t.to_string())
        }
    }
}

fn derived_panel(s: &Session, source: &Panel, link: PanelLink, name: String) -> Panel {
    let id = s.garment.next_id();
    let mut next = id + 1;
    let mut panel = Panel {
        id,
        name,
        link: Some(link),
        fabric: source.fabric,
        layer: source.layer,
        particle_distance: source.particle_distance,
        ..Panel::default()
    };
    sync(source, &mut panel, &mut next);
    panel
}

fn push_panel(s: &mut Session, label: &str, panel: Panel) -> Edit {
    let index = s.garment.panels.len();
    s.garment.panels.push(panel.clone());
    Edit::Panel { label: label.into(), index, before: None, after: Some(Box::new(panel)) }
}

fn mirror_schema() -> Value {
    obj(
        json!({
            "panel": panel_id(),
            "axis": json!({"type": "array", "items": xy("Axis end point."), "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Mirror axis as two points [[x, y], [x, y]] in world mm. Give axis or axis_segment."}),
            "axis_segment": int("Use the straight line through this outline segment of the panel as the axis (for example the centre front).", 0.0, 100_000.0, None),
            "gap": mm("Extra distance between the panel and its mirror image across the axis.", 0.0, SIZE, Some(0.0)),
            "name": json!({"type": "string", "minLength": 1, "maxLength": 64, "description": "Name of the mirror copy. Defaults to the source name and mirror."})
        }),
        &["panel"],
    )
}

fn mirror_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(mirror_schema, a)?;
    let id = required_id(a, "panel")?;
    let index = source_index(s, id)?;
    let source = s.garment.panels[index].clone();
    let place = source.placement();
    let (p, q) = match (a.get("axis").and_then(Value::as_array), count(a, "axis_segment")) {
        (Some(list), None) if list.len() == 2 => (vec2(&list[0], "axis")?, vec2(&list[1], "axis")?),
        (None, Some(k)) => {
            let n = source.outline.len();
            if k >= n {
                return Err(bad(format!("axis_segment {k} is out of range, the outline has {n}")));
            }
            (place.to_world(source.outline[k].p), place.to_world(source.outline[(k + 1) % n].p))
        }
        _ => return Err(bad("give exactly one of axis and axis_segment")),
    };
    let dir = unit(sub(q, p)).ok_or_else(|| bad("the axis has no length"))?;
    let outline = source.world_outline(0.1);
    let centre = polygon::centroid(&outline);
    let side = cross(dir, sub(centre, p));
    if side.abs() < 1e-6 {
        return Err(geometry("the axis passes through the middle of the panel; choose an axis beside it"));
    }
    let gap = number_or(a, "gap", 0.0);
    let away = mul([-dir[1], dir[0]], if side > 0.0 { -gap / 2.0 } else { gap / 2.0 });
    let axis = [[p[0] + away[0], p[1] + away[1]], [q[0] + away[0], q[1] + away[1]]];
    let link = PanelLink { kind: LinkKind::Mirror, source: id, axis: Some(axis) };
    let panel = derived_panel(s, &source, link, new_name(a, format!("{} mirror", source.name))?);
    let new_id = panel.id;
    let edit = push_panel(s, "Mirror Edit", panel);
    s.record(edit);
    Ok(json!({"id": new_id, "source": id, "axis": axis, "gap": gap}))
}

fn instance_schema() -> Value {
    obj(
        json!({
            "panel": panel_id(),
            "count": int("Number of linked copies after the source. Running it again on the same panel changes the number and interval of its existing copies.", 1.0, 50.0, Some(1.0)),
            "interval": xy("Shift between one copy and the next, in world mm. Defaults to the panel width plus 20 mm along x, or to the interval already in use."),
            "name": json!({"type": "string", "minLength": 1, "maxLength": 64, "description": "Base name of new copies. Defaults to the source name and the copy number."})
        }),
        &["panel"],
    )
}

fn ordinal(p: &Panel) -> f64 {
    p.link.as_ref().and_then(|l| l.axis).map_or(0.0, |x| x[1][0])
}

fn instance_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(instance_schema, a)?;
    let id = required_id(a, "panel")?;
    let index = source_index(s, id)?;
    let source = s.garment.panels[index].clone();
    let mut existing: Vec<usize> = derived_of(&s.garment, id).into_iter().filter(|&i| s.garment.panels[i].link.as_ref().is_some_and(|l| l.kind == LinkKind::Instance)).collect();
    existing.sort_by(|&x, &y| ordinal(&s.garment.panels[x]).total_cmp(&ordinal(&s.garment.panels[y])));
    let known = existing.first().and_then(|&i| s.garment.panels[i].link.as_ref()).and_then(|l| l.axis).map(|x| x[0]);
    let width = source.world_bounds().map_or(100.0, |b| b[1][0] - b[0][0]);
    let interval = optional_vec2(a, "interval")?.or(known).unwrap_or([width + 20.0, 0.0]);
    if dist(interval, [0.0, 0.0]) < 1e-6 {
        return Err(bad("interval must not be zero"));
    }
    let want = count(a, "count").unwrap_or(1);
    let mut edits = Vec::new();
    let (mut created, mut updated, mut removed) = (Vec::new(), Vec::new(), Vec::new());
    let mut next = s.garment.next_id();
    for k in 1..=want {
        let link = PanelLink { kind: LinkKind::Instance, source: id, axis: Some(instance_axis(interval, k)) };
        match existing.get(k - 1) {
            Some(&i) => {
                let before = s.garment.panels[i].clone();
                let mut after = before.clone();
                after.link = Some(link);
                sync(&source, &mut after, &mut next);
                if after != before {
                    updated.push(after.id);
                    s.garment.panels[i] = after.clone();
                    edits.push(replace(i, "Instance", before, after));
                }
            }
            None => {
                let panel = derived_panel(s, &source, link, new_name(a, format!("{} {}", source.name, k))?);
                created.push(panel.id);
                edits.push(push_panel(s, "Instance", panel));
            }
        }
    }
    if existing.len() > want {
        let mut extra: Vec<usize> = existing[want..].to_vec();
        extra.sort_unstable_by(|x, y| y.cmp(x));
        for i in extra {
            let gone = s.garment.panels.remove(i);
            removed.push(gone.id);
            edits.push(Edit::Panel { label: "Instance".into(), index: i, before: Some(Box::new(gone)), after: None });
        }
    }
    commit(s, "Instance", edits);
    Ok(json!({"source": id, "created": created, "updated": updated, "removed": removed, "interval": interval}))
}

fn unlink_schema() -> Value {
    obj(json!({"panel": panel_id()}), &["panel"])
}

fn unlink_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(unlink_schema, a)?;
    let id = required_id(a, "panel")?;
    let index = index_of(s, id)?;
    let targets = if s.garment.panels[index].link.is_some() { vec![index] } else { derived_of(&s.garment, id) };
    let mut edits = Vec::new();
    let mut ids = Vec::new();
    for i in targets {
        let before = s.garment.panels[i].clone();
        s.garment.panels[i].link = None;
        ids.push(before.id);
        edits.push(replace(i, "Unlink", before, s.garment.panels[i].clone()));
    }
    commit(s, "Unlink", edits);
    Ok(json!({"unlinked": ids, "changed": !ids.is_empty()}))
}

pub(crate) fn after_change(s: &mut Session, command: &str) {
    if command.starts_with("history.") || command.starts_with("file.") || !has_links(&s.garment) {
        return;
    }
    let mut next = s.garment.next_id();
    for i in 0..s.garment.panels.len() {
        let Some(link) = s.garment.panels[i].link.clone() else { continue };
        let before = s.garment.panels[i].clone();
        let mut after = before.clone();
        match s.garment.index_of(link.source) {
            Some(si) if si != i && s.garment.panels[si].link.is_none() => {
                let source = &s.garment.panels[si];
                sync(source, &mut after, &mut next);
            }
            Some(_) => continue,
            None => after.link = None,
        }
        if after != before {
            s.garment.panels[i] = after.clone();
            s.history.amend_last(replace(i, "Linked Edit", before, after));
        }
    }
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.mirror_link",
            mcp: "pattern_mirror_link",
            group: Group::Pattern,
            label: "Mirror Edit",
            description: "Create a linked mirror copy of a panel across an axis (two points, or a straight outline segment such as the centre front), optionally set apart by a gap. The copy is derived: after every change to the source it is regenerated, inside the same undo step. Geometry edits made directly on the copy are discarded; pattern.dart, pattern.slash_spread, pattern.unfold and pattern.mirror_link fail with linked_panel on it. Name, fabric, layer, particle distance and 3D placement of the copy stay editable. Use pattern.unlink to make it independent.",
            keys: &[],
            hints: WRITE,
            schema: mirror_schema,
            run: mirror_run,
        },
        Command {
            id: "pattern.instance",
            mcp: "pattern_instance",
            group: Group::Pattern,
            label: "Instances",
            description: "Create count linked copies of a panel, each shifted by the interval from the previous one (the first by one interval from the source). Copies are derived exactly like a mirror copy. Running it again on the same source changes the count and interval of its existing instances.",
            keys: &[],
            hints: WRITE,
            schema: instance_schema,
            run: instance_run,
        },
        Command {
            id: "pattern.unlink",
            mcp: "pattern_unlink",
            group: Group::Pattern,
            label: "Unlink",
            description: "Make a mirror copy or instance an ordinary independent panel that keeps its current geometry. Given a source panel it unlinks every copy derived from it.",
            keys: &[],
            hints: SET,
            schema: unlink_schema,
            run: unlink_run,
        },
    ]
}
