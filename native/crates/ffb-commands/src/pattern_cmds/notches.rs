use ffb_pattern::{notch, path, Notch, NotchKind, Panel};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::{check_index, edit_panel};
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

const KINDS: [&str; 4] = ["slit", "v", "t", "u"];

fn kind_of(a: &Value) -> Result<Option<NotchKind>, CommandError> {
    Ok(match text(a, "kind") {
        None => None,
        Some("slit") => Some(NotchKind::Slit),
        Some("v") => Some(NotchKind::V),
        Some("t") => Some(NotchKind::T),
        Some("u") => Some(NotchKind::U),
        Some(other) => return Err(bad(format!("kind must be slit, v, t or u, not {other}"))),
    })
}

fn position_fields(props: &mut Value) {
    props["segment"] = sc::index("Outline segment the notch sits on; segment i runs from point i to point i+1. Optional with at.");
    props["t"] = sc::ratio("Curve parameter between 0 and 1 along the segment.", 0.0, 1.0, None);
    props["length"] = sc::mm("Distance from the start of the segment along the curve.", 0.0, sc::SIZE, None);
    props["at"] = sc::xy("Place the notch at the spot on the outline nearest to this position.");
    props["space"] = sc::space();
}

fn look_fields(props: &mut Value) {
    let d = Notch::default();
    props["kind"] = sc::text("slit: a short cut into the allowance. v: a V-shaped wedge. t: a T-shaped cut. u: a U-shaped cut.", &KINDS);
    props["depth"] = sc::mm("How far the notch reaches from the outline.", 0.5, 100.0, Some(d.depth));
    props["width"] = sc::mm("Width of the notch across its opening; ignored by slits.", 0.1, 100.0, Some(d.width));
    props["angle"] = sc::deg("Angle between the notch and the outline tangent; 90 is square to the edge.", 10.0, 170.0, Some(d.angle));
}

fn resolve(panel: &Panel, a: &Value, current: Option<(usize, f64)>) -> Result<Option<(usize, f64)>, CommandError> {
    let given = ["t", "length", "at"].iter().filter(|k| a.get(**k).is_some()).count();
    if given > 1 {
        return Err(bad("give only one of t, length or at"));
    }
    let total = path::seg_count(panel.outline.len(), true);
    let segment = count(a, "segment");
    if let Some(i) = segment {
        check_index(total, i, "segment")?;
    }
    if let Some(q) = optional_vec2(a, "at")? {
        let q = to_local(panel, space(a), q);
        let near = match segment {
            Some(i) => {
                let (t, point, dist) = path::seg(&panel.outline, i).nearest(q);
                path::Near { seg: i, t, point, dist }
            }
            None => path::nearest(&panel.outline, true, q).ok_or_else(|| bad("the outline has no segment"))?,
        };
        return Ok(Some((near.seg, near.t)));
    }
    let Some(i) = segment.or(current.map(|c| c.0)) else {
        return if given == 0 { Ok(current) } else { Err(bad("t and length need a segment")) };
    };
    if let Some(t) = number(a, "t") {
        return Ok(Some((i, t)));
    }
    if let Some(l) = number(a, "length") {
        let curve = path::seg(&panel.outline, i);
        if l > curve.length() + 1e-9 {
            return Err(bad(format!("length {l:.3} is longer than segment {i}, {:.3}", curve.length())));
        }
        return Ok(Some((i, curve.t_at_length(l))));
    }
    Ok(match (segment, current) {
        (Some(_), Some((_, t))) => Some((i, t)),
        _ => current,
    })
}

fn describe(panel: &Panel, n: &Notch) -> Value {
    let located = notch::locate(panel, n);
    let world = located.map(|(p, _)| panel.placement().to_world(p));
    json!({
        "id": n.id,
        "segment": n.segment,
        "t": n.t,
        "kind": n.kind,
        "depth": n.depth,
        "width": n.width,
        "angle": n.angle,
        "point": located.map(|l| l.0),
        "normal": located.map(|l| l.1),
        "world": world
    })
}

fn add_schema() -> Value {
    let mut props = json!({"panel": sc::panel()});
    position_fields(&mut props);
    look_fields(&mut props);
    sc::obj(props, &["panel"])
}

fn notch_add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(add_schema, a)?;
    let kind = kind_of(a)?.unwrap_or_default();
    edit_panel(s, required_id(a, "panel")?, "Add Notch", |panel| {
        let (segment, t) = resolve(panel, a, None)?.ok_or_else(|| bad("give one of t, length or at"))?;
        let d = Notch::default();
        let n = Notch {
            id: notch::next_id(panel),
            segment,
            t: t.clamp(0.0, 1.0),
            kind,
            depth: number_or(a, "depth", d.depth),
            width: number_or(a, "width", d.width),
            angle: number_or(a, "angle", d.angle),
        };
        let report = describe(panel, &n);
        panel.notches.push(n);
        Ok(report)
    })
}

fn set_schema() -> Value {
    let mut props = json!({"panel": sc::panel(), "notch": sc::int("Notch id as listed in the panel's notches.", 1.0, 4_294_967_295.0, None)});
    position_fields(&mut props);
    look_fields(&mut props);
    for key in ["depth", "width", "angle"] {
        if let Some(m) = props[key].as_object_mut() {
            m.remove("default");
        }
    }
    sc::obj(props, &["panel", "notch"])
}

fn notch_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(set_schema, a)?;
    let (id, kind) = (required_id(a, "notch")?, kind_of(a)?);
    edit_panel(s, required_id(a, "panel")?, "Edit Notch", |panel| {
        let at = panel.notches.iter().position(|n| n.id == id).ok_or_else(|| missing(format!("panel {} has no notch {id}", panel.id)))?;
        let current = (panel.notches[at].segment, panel.notches[at].t);
        let moved = resolve(panel, a, Some(current))?;
        let n = &mut panel.notches[at];
        if let Some((segment, t)) = moved {
            n.segment = segment;
            n.t = t.clamp(0.0, 1.0);
        }
        n.kind = kind.unwrap_or(n.kind);
        n.depth = number_or(a, "depth", n.depth);
        n.width = number_or(a, "width", n.width);
        n.angle = number_or(a, "angle", n.angle);
        let done = panel.notches[at].clone();
        Ok(describe(panel, &done))
    })
}

fn delete_schema() -> Value {
    sc::obj(
        json!({
            "panel": sc::panel(),
            "notch": sc::int("Notch id to remove.", 1.0, 4_294_967_295.0, None),
            "notches": sc::ids("Several notch ids to remove."),
            "all": sc::flag("Remove every notch of the panel.", false)
        }),
        &["panel"],
    )
}

fn notch_delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(delete_schema, a)?;
    let all = flag(a, "all").unwrap_or(false);
    let mut ids = if a.get("notches").is_some() { id_list(a, "notches")? } else { Vec::new() };
    ids.extend(optional_id(a, "notch"));
    if all == !ids.is_empty() {
        return Err(bad("give exactly one of notch, notches or all"));
    }
    edit_panel(s, required_id(a, "panel")?, "Delete Notch", |panel| {
        if let Some(id) = ids.iter().find(|id| !panel.notches.iter().any(|n| n.id == **id)) {
            return Err(missing(format!("panel {} has no notch {id}", panel.id)));
        }
        let before = panel.notches.len();
        panel.notches.retain(|n| !all && !ids.contains(&n.id));
        Ok(json!({"removed": before - panel.notches.len(), "remaining": panel.notches.len()}))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.notch_add",
            mcp: "pattern_notch_add",
            group: Group::Pattern,
            label: "Add Notch",
            description: "Add a notch to the panel outline at curve parameter t of a segment, at a distance along a segment (length), or at the spot nearest a position (at). Kinds slit, v, t and u with depth, width and angle. Notches are stored by segment and t, so they keep their place when points are added, segments split or the panel moves. Returns the notch id, its local and world position and its outward normal.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: notch_add,
        },
        Command {
            id: "pattern.notch_set",
            mcp: "pattern_notch_set",
            group: Group::Pattern,
            label: "Edit Notch",
            description: "Change a notch: move it with segment and t, length or at, or change its kind, depth, width and angle. Only the given values change.",
            keys: &[],
            hints: SET,
            schema: set_schema,
            run: notch_set,
        },
        Command {
            id: "pattern.notch_delete",
            mcp: "pattern_notch_delete",
            group: Group::Pattern,
            label: "Delete Notch",
            description: "Remove one notch by id, several by notches, or every notch of the panel with all.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: notch_delete,
        },
    ]
}
