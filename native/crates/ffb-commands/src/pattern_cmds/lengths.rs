use ffb_pattern::geom::dist;
use ffb_pattern::measure2::{self, How};
use ffb_pattern::{path, Panel};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::{check_index, edit_panel, index_of, path_of};
use crate::{Command, CommandError, Group, Session, READ, SET};

fn lengths_schema() -> Value {
    sc::obj(
        json!({
            "panel": sc::panel(),
            "inner": sc::inner(),
            "segments": sc::indices("Only these segments count toward selected_total; the list still shows every segment.")
        }),
        &[],
    )
}

fn path_report(panel: &Panel, inner: Option<u32>, picked: &[usize]) -> Result<Value, CommandError> {
    let (pts, closed) = panel.path(inner).ok_or_else(|| missing(format!("panel {} has no inner shape {}", panel.id, inner.unwrap_or(0))))?;
    let segs = path::segments(pts, closed);
    if let Some(&i) = picked.iter().find(|&&i| i >= segs.len()) {
        return Err(bad(format!("segment {i} is out of range, there are {}", segs.len())));
    }
    let mut running = 0.0;
    let mut rows = Vec::with_capacity(segs.len());
    for (i, s) in segs.iter().enumerate() {
        let length = s.length();
        running += length;
        let notches: Vec<Value> = if inner.is_none() {
            panel.notches.iter().filter(|n| n.segment == i).map(|n| json!({"id": n.id, "t": n.t, "at": s.length_to(n.t)})).collect()
        } else {
            Vec::new()
        };
        rows.push(json!({
            "index": i, "length": length, "chord": dist(s.p[0], s.p[3]), "curved": s.curved,
            "start": s.p[0], "end": s.p[3], "cumulative": running, "notches": notches
        }));
    }
    let selected: f64 = picked.iter().map(|&i| segs[i].length()).sum();
    Ok(json!({
        "panel": panel.id, "name": panel.name, "inner": inner, "closed": closed,
        "total": running, "selected_total": if picked.is_empty() { Value::Null } else { json!(selected) },
        "segments": rows
    }))
}

fn segment_lengths(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(lengths_schema, a)?;
    let picked = if a.get("segments").is_some() { index_list(a, "segments")? } else { Vec::new() };
    let inner = optional_id(a, "inner");
    match optional_id(a, "panel") {
        Some(id) => Ok(json!({"panels": [path_report(&s.garment.panels[index_of(s, id)?], inner, &picked)?]})),
        None if inner.is_some() || !picked.is_empty() => Err(bad("inner and segments need a panel")),
        None => Ok(json!({"panels": s.garment.panels.iter().map(|p| path_report(p, None, &[])).collect::<Result<Vec<_>, _>>()?})),
    }
}

fn match_schema() -> Value {
    sc::obj(
        json!({
            "panel": sc::panel(),
            "inner": sc::inner(),
            "segments": sc::indices("Consecutive segments of the target to resize together. For an outline the run may wrap past the last segment."),
            "reference_panel": sc::panel(),
            "reference_inner": sc::inner(),
            "reference_segments": sc::indices("Segments of the reference whose lengths add up to the wanted length. The reference panel defaults to the target panel."),
            "length": sc::mm("Wanted length instead of a reference.", 0.1, sc::SIZE, None),
            "ease": sc::mm("Added to the wanted length; negative shortens, as for easing a sleeve cap into an armhole.", -sc::SIZE, sc::SIZE, Some(0.0)),
            "how": sc::text("move_end: slide the end point of the run along its chord until the run has the wanted length, the shape of the points before it unchanged. scale: scale the run about its first point, keeping its shape.", &["move_end", "scale"])
        }),
        &["panel", "segments"],
    )
}

fn match_length(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(match_schema, a)?;
    let id = required_id(a, "panel")?;
    let inner = optional_id(a, "inner");
    let picked = index_list(a, "segments")?;
    let how = if text(a, "how") == Some("scale") { How::Scale } else { How::MoveEnd };
    let wanted = match (number(a, "length"), a.get("reference_segments").is_some()) {
        (Some(l), false) => l,
        (None, true) => {
            let source = index_of(s, optional_id(a, "reference_panel").unwrap_or(id))?;
            let panel = &s.garment.panels[source];
            let (pts, closed) = panel.path(optional_id(a, "reference_inner")).ok_or_else(|| missing("the reference has no such inner shape"))?;
            let total = path::seg_count(pts.len(), closed);
            let list = index_list(a, "reference_segments")?;
            list.iter().try_for_each(|&i| check_index(total, i, "reference segment"))?;
            list.iter().map(|&i| path::seg(pts, i).length()).sum()
        }
        _ => return Err(bad("give exactly one of length or reference_segments")),
    };
    let target = wanted + number_or(a, "ease", 0.0);
    edit_panel(s, id, "Match Length", |panel| {
        let (pts, closed) = path_of(panel, inner)?;
        let total = path::seg_count(pts.len(), closed);
        picked.iter().try_for_each(|&i| check_index(total, i, "segment"))?;
        let (first, count) = measure2::contiguous(&picked, total, closed).ok_or_else(|| bad("segments must be consecutive"))?;
        let fit = measure2::fit(pts, closed, first, count, target, how).map_err(geometry)?;
        let end = (first + count) % pts.len();
        Ok(json!({
            "first": first, "count": count, "target": target, "before": fit.before, "after": fit.after,
            "error": fit.after - target, "factor": fit.factor, "shift": fit.shift, "end_point": pts[end].p
        }))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.segment_lengths",
            mcp: "pattern_segment_lengths",
            group: Group::Pattern,
            label: "Segment Lengths",
            description: "List the length of every segment of a panel outline or inner shape in mm, with chord, end points, running total and the notches on each segment at their distance along it. With segments, also return the sum of those segments. Without panel it lists every panel outline. Does not change the document.",
            keys: &[],
            hints: READ,
            schema: lengths_schema,
            run: segment_lengths,
        },
        Command {
            id: "pattern.match_length",
            mcp: "pattern_match_length",
            group: Group::Pattern,
            label: "Match Length",
            description: "Make a run of consecutive segments as long as the sum of reference segments (of this or another panel) or a given length, plus an optional ease. move_end slides the end point of the run along its chord; scale scales the run about its first point. Returns the lengths before and after.",
            keys: &[],
            hints: SET,
            schema: match_schema,
            run: match_length,
        },
    ]
}
