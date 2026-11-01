use ffb_pattern::dart::{self, DartSpec};
use ffb_pattern::geom::{add, mul, rotate, unit, V2};
use ffb_pattern::slash::{self, Side, SlashSpec};
use ffb_pattern::spiral::{self, SpiralSpec};
use ffb_pattern::trace::{self, Traced};
use ffb_pattern::unfold;
use ffb_pattern::{path, Panel};
use serde_json::{json, Value};

use super::support::*;
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

fn world_of(panel: &Panel, p: V2) -> V2 {
    panel.placement().to_world(p)
}

fn dart_schema() -> Value {
    obj(
        json!({
            "panel": panel_id(),
            "segment": int("Outline segment that carries the dart opening. Segment i runs from point i to point i+1. For an internal dart it only locates the base.", 0.0, 100_000.0, None),
            "t": ratio("Position of the dart centre along the segment, 0 at its start and 1 at its end.", 0.0, 1.0, Some(0.5)),
            "width": size("Dart opening (intake) measured along the segment, centred on t. For an internal dart the width of its base.", None),
            "length": size("Dart length from the middle of the opening to the apex, along the inward normal. Give length or apex.", None),
            "apex": xy("Apex point instead of a length."),
            "internal": switch("Create a closed inner dart shape with role dart instead of cutting the outline.", false),
            "center": xy("Internal dart only: base centre, used instead of segment and t."),
            "angle": deg("Internal dart only: direction of the apex from the base centre, used with center and length.", -360.0, 360.0, None),
            "inset": mm("Internal dart only: distance the base is moved inside the outline from the segment point.", 0.0, 1000.0, Some(5.0)),
            "space": space()
        }),
        &["panel", "width"],
    )
}

fn dart_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(dart_schema, a)?;
    let id = required_id(a, "panel")?;
    let internal = flag(a, "internal", false);
    let world = world_or_local(a);
    let width = number_or(a, "width", 0.0);
    let (length, apex_arg, center_arg) = (number(a, "length"), optional_vec2(a, "apex")?, optional_vec2(a, "center")?);
    if length.is_none() && apex_arg.is_none() {
        return Err(bad("give a length or an apex point"));
    }
    let segment = count(a, "segment");
    let label = if internal { "Internal Dart" } else { "Dart" };
    edit_source(s, id, label, |panel, next| {
        let apex = apex_arg.map(|p| to_local(panel, world, p));
        if !internal {
            let segment = segment.ok_or_else(|| bad("segment is required for a dart on the outline"))?;
            let spec = DartSpec { segment, t: number_or(a, "t", 0.5), width, length, apex };
            let r = dart::insert(panel, &spec).map_err(geometry)?;
            return Ok(json!({
                "opening": [world_of(panel, r.opening[0]), world_of(panel, r.opening[1])],
                "apex": world_of(panel, r.apex), "legs": r.legs, "opening_width": r.chord, "area_removed": r.area_removed
            }));
        }
        let mut inward = None;
        let base = match (center_arg, segment) {
            (Some(c), _) => to_local(panel, world, c),
            (None, Some(sg)) => {
                if sg >= panel.outline.len() {
                    return Err(bad(format!("segment {sg} is out of range, the outline has {}", panel.outline.len())));
                }
                let seg = path::seg(&panel.outline, sg);
                let t = number_or(a, "t", 0.5);
                let tangent = unit(seg.tangent(t)).ok_or_else(|| geometry("the segment has no direction"))?;
                let normal = [-tangent[1], tangent[0]];
                inward = Some(normal);
                add(seg.at(t), mul(normal, number_or(a, "inset", 5.0)))
            }
            (None, None) => return Err(bad("an internal dart needs a segment or a center")),
        };
        let apex = match apex {
            Some(p) => p,
            None => {
                let length = length.unwrap_or(0.0);
                let dir = match number(a, "angle") {
                    Some(deg) => {
                        let w = rotate([1.0, 0.0], deg);
                        if world { panel.placement().vec_to_local(w) } else { w }
                    }
                    None => inward.ok_or_else(|| bad("give an angle or an apex with a center"))?,
                };
                add(base, mul(dir, length))
            }
        };
        let shape = *next;
        *next += 1;
        dart::internal(panel, shape, base, apex, width).map_err(geometry)?;
        Ok(json!({"inner": shape, "base": world_of(panel, base), "apex": world_of(panel, apex), "length": ffb_pattern::geom::dist(base, apex)}))
    })
}

fn slash_schema() -> Value {
    obj(
        json!({
            "panel": panel_id(),
            "from": xy("First point of the cut line. The line runs through from and to and is used between the outline crossings nearest to them."),
            "to": xy("Second point of the cut line."),
            "spread": mm("Opening of the cut at its free end, measured as a straight distance. Give spread or angle. Negative closes the cut.", -SIZE, SIZE, None),
            "angle": deg("Rotation of the moving part in degrees instead of a distance. Positive opens the cut.", -180.0, 180.0, None),
            "hinge": choice("Which end of the cut stays joined. The part rotates about it.", &["from", "to"], Some("from")),
            "side": choice("Which part moves, seen from the hinge looking along the cut: left, right, or both by half the spread each.", &["left", "right", "both"], Some("left")),
            "smooth": switch("Join the opened end with a curve that continues the neighbouring outline tangents. false uses a straight edge.", true),
            "space": space()
        }),
        &["panel", "from", "to"],
    )
}

fn slash_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(slash_schema, a)?;
    let id = required_id(a, "panel")?;
    let world = world_or_local(a);
    let (spread, angle) = (number(a, "spread"), number(a, "angle"));
    if spread.is_some() == angle.is_some() {
        return Err(bad("give exactly one of spread and angle"));
    }
    let (from, to) = (optional_vec2(a, "from")?.ok_or_else(|| bad("from is required"))?, optional_vec2(a, "to")?.ok_or_else(|| bad("to is required"))?);
    let side = match text(a, "side") {
        Some("right") => Side::Right,
        Some("both") => Side::Both,
        _ => Side::Left,
    };
    edit_source(s, id, "Slash and Spread", |panel, _| {
        let spec = SlashSpec { from: to_local(panel, world, from), to: to_local(panel, world, to), hinge_from: text(a, "hinge") != Some("to"), angle, spread, side, smooth: flag(a, "smooth", true) };
        let r = slash::spread(panel, &spec).map_err(geometry)?;
        Ok(json!({"hinge": world_of(panel, r.hinge), "far_end": world_of(panel, r.far), "angle": r.angle, "cut_length": r.length, "opening": r.opening, "area_added": r.area_added}))
    })
}

fn unfold_schema() -> Value {
    obj(
        json!({
            "panel": panel_id(),
            "segment": int("Straight outline segment to unfold about, usually the centre front or centre back. It disappears; the half is mirrored across it.", 0.0, 100_000.0, None)
        }),
        &["panel", "segment"],
    )
}

fn unfold_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(unfold_schema, a)?;
    let id = required_id(a, "panel")?;
    let segment = count(a, "segment").unwrap_or(0);
    edit_source(s, id, "Unfold", |panel, next| {
        let r = unfold::unfold(panel, segment, next).map_err(geometry)?;
        Ok(json!({"points": r.points, "inner_mirrored": r.inner_mirrored, "notches_mirrored": r.notches_mirrored, "axis": [world_of(panel, r.axis[0]), world_of(panel, r.axis[1])]}))
    })
}

fn panel_name(a: &Value, fallback: String) -> Result<String, CommandError> {
    match text(a, "name") {
        None => Ok(fallback),
        Some(raw) => {
            let t = raw.trim();
            if t.is_empty() || t.chars().count() > 64 {
                return Err(bad("name must be 1 to 64 characters"));
            }
            Ok(t.to_string())
        }
    }
}

fn add_panel(s: &mut Session, label: &str, a: &Value, traced: Traced, base_name: &str, template: Option<&Panel>, mut edits: Vec<Edit>) -> Result<Value, CommandError> {
    let id = s.garment.next_id();
    let mut panel = Panel {
        id,
        name: panel_name(a, format!("{base_name} {id}"))?,
        outline: traced.outline,
        position: traced.position,
        particle_distance: number_or(a, "particle_distance", template.map_or(20.0, |t| t.particle_distance)),
        ..Panel::default()
    };
    if let Some(t) = template {
        panel.layer = t.layer;
        panel.fabric = t.fabric;
        panel.grain = trace::world_grain(t);
    }
    let index = s.garment.panels.len();
    let report = json!({"id": id, "index": index, "name": panel.name, "position": panel.position, "area": traced.area, "perimeter": path::length(&panel.outline, true)});
    s.garment.panels.push(panel.clone());
    edits.push(Edit::Panel { label: label.into(), index, before: None, after: Some(Box::new(panel)) });
    commit(s, label, edits);
    Ok(report)
}

fn spiral_schema() -> Value {
    obj(
        json!({
            "center": xy("Centre of the spiral on the pattern plane."),
            "inner_radius": size("Radius of the strip centre line where the spiral starts.", None),
            "outer_radius": size("Radius of the strip centre line where the spiral ends.", None),
            "turns": ratio("Number of turns from start to end.", 0.25, 20.0, Some(3.0)),
            "width": size("Width of the strip.", None),
            "direction": choice("Winding direction outwards.", &["ccw", "cw"], Some("ccw")),
            "start_angle": deg("Angle of the start of the strip.", -360.0, 360.0, Some(0.0)),
            "name": json!({"type": "string", "minLength": 1, "maxLength": 64, "description": "Panel name. Defaults to Spiral and the new id."}),
            "particle_distance": mm("Simulation particle spacing for this panel.", 1.0, 200.0, Some(20.0))
        }),
        &["inner_radius", "outer_radius", "width"],
    )
}

fn spiral_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(spiral_schema, a)?;
    let spec = SpiralSpec {
        center: optional_vec2(a, "center")?.unwrap_or([0.0, 0.0]),
        inner: number_or(a, "inner_radius", 0.0),
        outer: number_or(a, "outer_radius", 0.0),
        turns: number_or(a, "turns", 3.0),
        width: number_or(a, "width", 0.0),
        ccw: text(a, "direction") != Some("cw"),
        start: number_or(a, "start_angle", 0.0),
        per_turn: 24,
    };
    let world = spiral::outline(&spec).map_err(geometry)?;
    let traced = trace::localize(world).map_err(geometry)?;
    add_panel(s, "Spiral", a, traced, "Spiral", None, Vec::new())
}

fn trace_schema() -> Value {
    obj(
        json!({
            "panel": panel_id(),
            "inner": int("Closed inner shape id to trace into a new panel.", 1.0, 4_294_967_295.0, None),
            "at": xy("World point: traces the smallest closed inner shape of the panel that contains it."),
            "points": json!({"type": "array", "minItems": 3, "items": xy("Region vertex in world mm."), "unit": "mm", "description": "A closed region given by its vertices in world mm, in any winding. The panel is then optional and only supplies fabric, layer and grain."}),
            "remove": switch("Remove the traced inner shape from the source panel.", false),
            "name": json!({"type": "string", "minLength": 1, "maxLength": 64, "description": "Panel name. Defaults to Traced and the new id."}),
            "particle_distance": mm("Simulation particle spacing for the new panel. Defaults to the source panel's.", 1.0, 200.0, None)
        }),
        &[],
    )
}

fn trace_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(trace_schema, a)?;
    let given = [a.get("inner").is_some(), a.get("at").is_some(), a.get("points").is_some()].iter().filter(|x| **x).count();
    if given != 1 {
        return Err(bad("give exactly one of inner, at and points"));
    }
    let remove = flag(a, "remove", false);
    let source = match a.get("panel") {
        Some(_) => Some(index_of(s, required_id(a, "panel")?)?),
        None if a.get("points").is_some() => None,
        None => return Err(bad("panel is required with inner or at")),
    };
    let mut edits = Vec::new();
    let traced;
    let mut shape = None;
    if let Some(pts) = a.get("points").and_then(Value::as_array) {
        let mut world = Vec::with_capacity(pts.len());
        for p in pts {
            world.push(ffb_pattern::Point { p: vec2(p, "points")?, ..ffb_pattern::Point::default() });
        }
        traced = trace::localize(world).map_err(geometry)?;
    } else {
        let Some(i) = source else { return Err(bad("panel is required with inner or at")) };
        let src = &s.garment.panels[i];
        let wanted = match count(a, "inner") {
            Some(v) => v as u32,
            None => {
                let at = optional_vec2(a, "at")?.ok_or_else(|| bad("at must be [x, y]"))?;
                trace::region_at(src, at).ok_or_else(|| missing("no closed inner shape of the panel contains that point"))?
            }
        };
        traced = trace::from_inner(src, wanted).map_err(geometry)?;
        shape = Some(wanted);
    }
    if let (true, Some(i), Some(wanted)) = (remove, source, shape) {
        let i = source_index(s, s.garment.panels[i].id)?;
        let before = s.garment.panels[i].clone();
        s.garment.panels[i].inner.retain(|x| x.id != wanted);
        edits.push(replace(i, "Trace", before, s.garment.panels[i].clone()));
    }
    let template = source.map(|i| s.garment.panels[i].clone());
    let mut out = add_panel(s, "Trace", a, traced, "Traced", template.as_ref(), edits)?;
    out["inner"] = json!(shape);
    Ok(out)
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.dart",
            mcp: "pattern_dart",
            group: Group::Pattern,
            label: "Dart",
            description: "Cut a dart into a panel outline: the opening is width mm wide, centred at t on a segment, and its two equal legs meet at an apex length mm along the inward normal (or at a given apex point). The outline stays closed and valid or the command fails with invalid_geometry. With internal true it adds a closed triangular inner shape with role dart instead. Linked copies of the panel follow. Fails with linked_panel on a mirror copy or instance.",
            keys: &[],
            hints: WRITE,
            schema: dart_schema,
            run: dart_run,
        },
        Command {
            id: "pattern.slash_spread",
            mcp: "pattern_slash_spread",
            group: Group::Pattern,
            label: "Slash and Spread",
            description: "Cut a panel along a line from edge to edge and rotate the part on one side about the hinge end, by a spread distance at the free end or by an angle. The outline stays closed; the opened end is joined by a curve that continues the neighbouring tangents. Inner shapes and the grain line move with their part. Fails with invalid_geometry when the result would cross itself or an inner shape crosses the cut.",
            keys: &[],
            hints: WRITE,
            schema: slash_schema,
            run: slash_run,
        },
        Command {
            id: "pattern.unfold",
            mcp: "pattern_unfold",
            group: Group::Pattern,
            label: "Unfold",
            description: "Mirror a half panel across one of its straight outline segments into one symmetric outline; the segment disappears. Inner shapes, notches and seam allowance values are mirrored too (inner shapes lying on the axis are not doubled).",
            keys: &[],
            hints: WRITE,
            schema: unfold_schema,
            run: unfold_run,
        },
        Command {
            id: "pattern.spiral",
            mcp: "pattern_spiral",
            group: Group::Pattern,
            label: "Spiral",
            description: "Create a panel shaped as a flat spiral strip of a given width that winds from the inner radius to the outer radius over a number of turns. Fails with invalid_geometry when neighbouring turns would overlap.",
            keys: &[],
            hints: WRITE,
            schema: spiral_schema,
            run: spiral_run,
        },
        Command {
            id: "pattern.trace",
            mcp: "pattern_trace",
            group: Group::Pattern,
            label: "Trace",
            description: "Create a new panel from a closed inner shape (by inner id, or the smallest one containing the point at) or from a closed region given as vertices. The new panel keeps the traced geometry at the same place on the pattern plane. remove true also deletes the inner shape from the source panel.",
            keys: &[],
            hints: WRITE,
            schema: trace_schema,
            run: trace_run,
        },
    ]
}
