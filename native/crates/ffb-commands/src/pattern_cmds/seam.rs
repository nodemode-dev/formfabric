use ffb_pattern::offset::{self, Spec, MITER_LIMIT};
use ffb_pattern::{path, polygon, CornerStyle, Inner, InnerKind, InnerRole, Panel, Point};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::{commit, edit_panel, index_of, replace};
use crate::{Command, CommandError, Edit, Group, Session, SET, WRITE};

const CORNERS: [&str; 4] = ["miter", "square", "round", "mirror"];
const REPORT_TOL: f64 = 0.02;

fn corner_style(a: &Value) -> Result<Option<CornerStyle>, CommandError> {
    Ok(match text(a, "corners") {
        None => None,
        Some("miter") => Some(CornerStyle::Miter),
        Some("square") => Some(CornerStyle::Square),
        Some("round") => Some(CornerStyle::Round),
        Some("mirror") => Some(CornerStyle::Mirror),
        Some(other) => return Err(bad(format!("corners must be miter, square, round or mirror, not {other}"))),
    })
}

fn corner_schema() -> Value {
    sc::text(
        "How a corner of the offset line is finished where the outline turns away from the offset. miter: the offset edges meet in a sharp point, cut back to a square corner beyond the miter limit. square: each edge is extended by the offset distance and cut square. round: a circular arc around the corner. mirror: a chamfer cut across the corner at the offset distance from it, as for a folded mitred corner.",
        &CORNERS,
    )
}

fn per_segment_schema(description: &str) -> Value {
    json!({
        "type": "array",
        "items": {"type": ["number", "null"], "minimum": -1000.0, "maximum": 1000.0, "unit": "mm"},
        "unit": "mm",
        "description": description
    })
}

fn per_segment(a: &Value) -> Result<Option<Vec<Option<f64>>>, CommandError> {
    let Some(list) = a.get("per_segment") else { return Ok(None) };
    let items = list.as_array().ok_or_else(|| bad("per_segment must be a list"))?;
    items
        .iter()
        .map(|v| if v.is_null() { Ok(None) } else { v.as_f64().map(Some).ok_or_else(|| bad("per_segment holds numbers or null")) })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn points_json(points: &[Point]) -> Value {
    serde_json::to_value(points).unwrap_or(Value::Null)
}

fn outline_report(points: &[Point]) -> Value {
    let poly = path::flatten(points, true, REPORT_TOL);
    json!({"outline": points_json(points), "length": path::length(points, true), "area": polygon::signed_area(&poly).abs()})
}

fn seam_schema() -> Value {
    sc::obj(
        json!({
            "panels": sc::ids("Panels to give or change the seam allowance of."),
            "width": sc::mm("Allowance width on every segment without its own override. A new allowance starts at 10 mm.", 0.0, 500.0, None),
            "corners": corner_schema(),
            "per_segment": {
                "type": "array",
                "items": {"type": ["number", "null"], "minimum": 0.0, "maximum": 500.0, "unit": "mm"},
                "unit": "mm",
                "description": "Allowance width per outline segment, counted from segment 0; null uses the general width, and a shorter list leaves the rest on the general width. Needs exactly one panel."
            },
            "clear": sc::flag("Remove the seam allowance from the panels.", false)
        }),
        &["panels"],
    )
}

fn seam_allowance(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(seam_schema, a)?;
    let ids = id_list(a, "panels")?;
    let clear = flag(a, "clear").unwrap_or(false);
    let (width, corners, list) = (number(a, "width"), corner_style(a)?, per_segment(a)?);
    if clear && (width.is_some() || corners.is_some() || list.is_some()) {
        return Err(bad("clear cannot be combined with width, corners or per_segment"));
    }
    if list.is_some() && ids.len() != 1 {
        return Err(bad("per_segment needs exactly one panel"));
    }
    if list.iter().flatten().flatten().any(|w| *w < 0.0) {
        return Err(bad("per_segment widths must not be negative"));
    }
    let indices = ids.iter().map(|&id| index_of(s, id)).collect::<Result<Vec<_>, _>>()?;
    let mut staged = Vec::new();
    let mut report = Vec::new();
    for &i in &indices {
        let before = s.garment.panels[i].clone();
        let mut after = before.clone();
        if clear {
            after.seam_allowance = None;
        } else {
            let mut sa = after.seam_allowance.take().unwrap_or_default();
            sa.width = width.unwrap_or(sa.width);
            sa.corners = corners.unwrap_or(sa.corners);
            if let Some(items) = &list {
                if items.len() > after.outline.len() {
                    return Err(bad(format!("per_segment has {} entries, the outline has {} segments", items.len(), after.outline.len())));
                }
                sa.per_segment = items.clone();
            }
            after.seam_allowance = Some(sa);
        }
        let mut entry = json!({"panel": after.id, "seam_allowance": after.seam_allowance});
        match offset::seam_outline(&after) {
            Some(Ok(outline)) => {
                if let (Some(map), Some(extra)) = (entry.as_object_mut(), outline_report(&outline).as_object()) {
                    map.extend(extra.clone());
                }
            }
            Some(Err(e)) => return Err(geometry(format!("panel {}: {e}", after.id))),
            None => {}
        }
        report.push(entry);
        staged.push((i, before, after));
    }
    let mut edits = Vec::new();
    for (i, before, after) in staged {
        if before != after {
            s.garment.panels[i] = after.clone();
            edits.push(replace(i, "Seam Allowance", before, after));
        }
    }
    let changed = !edits.is_empty();
    commit(s, "Seam Allowance", edits);
    Ok(json!({"panels": report, "changed": changed}))
}

fn offset_schema() -> Value {
    sc::obj(
        json!({
            "panel": sc::panel(),
            "distance": sc::mm("Offset distance. Positive grows the outline outward, negative retracts it inward.", -1000.0, 1000.0, None),
            "corners": corner_schema(),
            "miter_limit": sc::ratio("Longest miter, as a multiple of the distance from the corner; beyond it a miter corner is cut square.", 1.0, 20.0, Some(MITER_LIMIT)),
            "count": sc::int("Number of offsets, each one distance further than the last.", 1.0, 20.0, Some(1.0)),
            "as": sc::text("inner: closed inner lines inside the panel, which needs a negative distance. panel: new panels with the offset outline, placed over the source.", &["inner", "panel"]),
            "per_segment": per_segment_schema("Offset distance per outline segment, counted from segment 0, with the same sign as distance; null uses distance. Segments the list does not reach use distance."),
            "name": sc::name("Name of the new panel. Defaults to the source name and offset.")
        }),
        &["panel", "distance"],
    )
}

fn offset_outline(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(offset_schema, a)?;
    let distance = number(a, "distance").filter(|d| d.abs() > 1e-9).ok_or_else(|| bad("distance must not be zero"))?;
    let as_panel = text(a, "as") == Some("panel");
    if !as_panel && distance > 0.0 {
        return Err(bad("an inner line must lie inside the panel; give a negative distance or use as panel"));
    }
    let times = count(a, "count").unwrap_or(1);
    let id = required_id(a, "panel")?;
    let index = index_of(s, id)?;
    let source = s.garment.panels[index].clone();
    let corners = corner_style(a)?.unwrap_or_default();
    let limit = number_or(a, "miter_limit", MITER_LIMIT);
    let overrides = per_segment(a)?.unwrap_or_default();
    if overrides.len() > source.outline.len() {
        return Err(bad(format!("per_segment has {} entries, the outline has {} segments", overrides.len(), source.outline.len())));
    }
    if overrides.iter().flatten().any(|w| w.signum() != distance.signum()) {
        return Err(bad("per_segment distances must have the same sign as distance"));
    }
    let mut outlines = Vec::new();
    for k in 1..=times {
        let widths = (0..source.outline.len()).map(|i| overrides.get(i).copied().flatten().unwrap_or(distance) * k as f64).collect();
        let spec = Spec { widths, corners, miter_limit: limit };
        outlines.push(offset::offset_closed(&source.outline, &spec).map_err(|e| geometry(format!("offset {k}: {e}")))?);
    }
    let mut next = s.garment.next_id();
    let mut created = Vec::new();
    let mut summary = Vec::new();
    for outline in &outlines {
        summary.push(outline_report(outline));
    }
    if as_panel {
        let mut edits = Vec::new();
        for (k, outline) in outlines.into_iter().enumerate() {
            let label = name(a, "name")?.unwrap_or_else(|| format!("{} offset {}", source.name, distance * (k + 1) as f64));
            let panel = Panel {
                id: next,
                name: label,
                outline,
                position: source.position,
                rotation: source.rotation,
                mirror: source.mirror,
                grain: source.grain,
                particle_distance: source.particle_distance,
                layer: source.layer,
                fabric: source.fabric,
                ..Panel::default()
            };
            created.push(next);
            next += 1;
            let at = s.garment.panels.len();
            s.garment.panels.push(panel.clone());
            edits.push(Edit::Panel { label: "Offset Outline".into(), index: at, before: None, after: Some(Box::new(panel)) });
        }
        commit(s, "Offset Outline", edits);
        return Ok(json!({"created": created, "kind": "panel", "offsets": summary, "changed": true}));
    }
    let shapes: Vec<Inner> = outlines
        .into_iter()
        .map(|points| {
            let shape = Inner { id: next, kind: InnerKind::Polygon, points, closed: true, role: InnerRole::Shape, ..Inner::default() };
            created.push(next);
            next += 1;
            shape
        })
        .collect();
    let mut out = edit_panel(s, id, "Offset Outline", |panel| {
        panel.inner.extend(shapes);
        Ok(json!({}))
    })?;
    out["created"] = json!(created);
    out["kind"] = json!("inner");
    out["offsets"] = json!(summary);
    Ok(out)
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.seam_allowance",
            mcp: "pattern_seam_allowance",
            group: Group::Pattern,
            label: "Seam Allowance",
            description: "Give panels a seam allowance stored on the panel: a general width in mm, a corner style (miter, square, round, mirror) and optional widths for single outline segments. clear removes it. The allowance outline is derived from the panel outline and follows later edits; the reply carries the derived outline, its length and area. Notches stay on the sewing outline.",
            keys: &[],
            hints: SET,
            schema: seam_schema,
            run: seam_allowance,
        },
        Command {
            id: "pattern.offset_outline",
            mcp: "pattern_offset_outline",
            group: Group::Pattern,
            label: "Offset Outline",
            description: "Offset a panel outline by a distance in mm (positive outward, negative inward), with Bezier segments refitted as cubic curves, per-segment distances, corner styles miter, square, round or mirror, and loops from inward offsets removed. Creates closed inner lines (inward only) or new panels laid over the source. Count makes several offsets in one step.",
            keys: &[],
            hints: WRITE,
            schema: offset_schema,
            run: offset_outline,
        },
    ]
}
