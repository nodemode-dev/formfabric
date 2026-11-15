use ffb_pattern::edit::{self, Which};
use ffb_pattern::geom::{add, len};
use ffb_pattern::{notch, path};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::{check_index, edit_panel, min_points, path_of};
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

fn target(mut props: Value) -> Value {
    props["panel"] = sc::panel();
    props["inner"] = sc::inner();
    props
}

fn move_schema() -> Value {
    sc::obj(
        target(json!({
            "index": sc::index("Point to move."),
            "indices": sc::indices("Several points to move together by delta; counts from 0."),
            "to": sc::xy("New position of the single point."),
            "delta": sc::xy("Offset to add to every selected point."),
            "space": sc::space()
        })),
        &["panel"],
    )
}

fn move_point(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(move_schema, a)?;
    let space = space(a);
    let indices = match (count(a, "index"), a.get("indices")) {
        (Some(i), None) => vec![i],
        (None, Some(_)) => index_list(a, "indices")?,
        _ => return Err(bad("give exactly one of index or indices")),
    };
    let (to, delta) = (optional_vec2(a, "to")?, optional_vec2(a, "delta")?);
    if to.is_some() == delta.is_some() {
        return Err(bad("give exactly one of to or delta"));
    }
    if to.is_some() && indices.len() != 1 {
        return Err(bad("to moves a single point; use delta for several"));
    }
    edit_panel(s, required_id(a, "panel")?, "Move Point", |panel| {
        let moved = to.map(|t| to_local(panel, space, t));
        let step = delta.map(|d| vec_to_local(panel, space, d));
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        for &i in &indices {
            check_index(pts.len(), i, "point")?;
            pts[i].p = moved.or_else(|| step.map(|d| add(pts[i].p, d))).unwrap_or(pts[i].p);
        }
        path::auto_handles(pts, closed);
        Ok(json!({"indices": indices, "points": indices.iter().map(|&i| pts[i].p).collect::<Vec<_>>()}))
    })
}

fn handle_schema() -> Value {
    sc::obj(
        target(json!({
            "index": sc::index("Point whose handle to edit."),
            "which": sc::text("in: handle toward the previous point. out: handle toward the next point.", &["in", "out"]),
            "vector": sc::xy("Handle as an offset from the point. Leave out to remove the handle."),
            "independent": sc::flag("Break tangent continuity so the opposite handle stays put and the point becomes a corner.", false),
            "space": sc::space()
        })),
        &["panel", "index", "which"],
    )
}

fn set_handle(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(handle_schema, a)?;
    let which = if text(a, "which") == Some("in") { Which::In } else { Which::Out };
    let (vector, space, index) = (optional_vec2(a, "vector")?, space(a), required_count(a, "index")?);
    edit_panel(s, required_id(a, "panel")?, "Edit Handle", |panel| {
        let local = vector.map(|v| vec_to_local(panel, space, v));
        let (pts, _) = path_of(panel, optional_id(a, "inner"))?;
        check_index(pts.len(), index, "point")?;
        edit::put_handle(pts, index, which, local, flag(a, "independent").unwrap_or(false));
        let p = &pts[index];
        Ok(json!({"index": index, "kind": p.kind, "h_in": p.h_in, "h_out": p.h_out}))
    })
}

fn add_schema() -> Value {
    sc::obj(
        target(json!({
            "segment": sc::index("Segment to add the point on; segment i runs from point i to point i+1. Optional with at."),
            "t": sc::ratio("Curve parameter between 0 and 1 on the segment.", 0.001, 0.999, None),
            "at": sc::xy("Place the point at the spot on the segment nearest to this position."),
            "space": sc::space()
        })),
        &["panel"],
    )
}

fn add_point(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(add_schema, a)?;
    let (t, at, space) = (number(a, "t"), optional_vec2(a, "at")?, space(a));
    if t.is_some() == at.is_some() {
        return Err(bad("give exactly one of t or at"));
    }
    edit_panel(s, required_id(a, "panel")?, "Add Point", |panel| {
        let at = at.map(|q| to_local(panel, space, q));
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        let (segment, t) = match (count(a, "segment"), at) {
            (Some(i), None) => (i, t.unwrap_or(0.5)),
            (seg, Some(q)) => {
                let near = match seg {
                    Some(i) => {
                        check_index(path::seg_count(pts.len(), closed), i, "segment")?;
                        let (t, point, dist) = path::seg(pts, i).nearest(q);
                        path::Near { seg: i, t, point, dist }
                    }
                    None => path::nearest(pts, closed, q).ok_or_else(|| bad("there is no segment"))?,
                };
                (near.seg, near.t.clamp(0.001, 0.999))
            }
            (None, None) => return Err(bad("t needs a segment")),
        };
        check_index(path::seg_count(pts.len(), closed), segment, "segment")?;
        let index = path::split_at(pts, segment, t);
        let point = pts[index].p;
        if optional_id(a, "inner").is_none() {
            notch::on_split(panel, segment, &[t]);
        }
        ffb_pattern::sew::note_split(panel, optional_id(a, "inner"), segment, &[t]);
        Ok(json!({"index": index, "segment": segment, "t": t, "point": point}))
    })
}

fn split_schema() -> Value {
    sc::obj(
        target(json!({
            "segment": sc::index("Segment to split; segment i runs from point i to point i+1."),
            "t": sc::ratio("Split at this curve parameter between 0 and 1. The default is 0.5.", 0.001, 0.999, Some(0.5)),
            "length": sc::mm("Split at this distance along the segment from its start.", 0.001, sc::SIZE, None),
            "count": sc::int("Split into this many pieces of equal length.", 2.0, 64.0, None)
        })),
        &["panel", "segment"],
    )
}

fn split_line(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(split_schema, a)?;
    let given = ["t", "length", "count"].iter().filter(|k| a.get(**k).is_some()).count();
    if given > 1 {
        return Err(bad("give only one of t, length or count"));
    }
    let segment = required_count(a, "segment")?;
    edit_panel(s, required_id(a, "panel")?, "Split Line", |panel| {
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        check_index(path::seg_count(pts.len(), closed), segment, "segment")?;
        let curve = path::seg(pts, segment);
        let total = curve.length();
        let ts: Vec<f64> = if let Some(n) = count(a, "count") {
            if total < 1e-6 {
                return Err(geometry(format!("segment {segment} has no length, so it cannot be split into equal pieces")));
            }
            (1..n).map(|k| curve.t_at_length(total * k as f64 / n as f64)).collect()
        } else if let Some(l) = number(a, "length") {
            if l >= total {
                return Err(bad(format!("length {l:.3} is not shorter than the segment, {total:.3}")));
            }
            vec![curve.t_at_length(l)]
        } else {
            vec![number_or(a, "t", 0.5)]
        };
        let indices = path::split_many(pts, segment, &ts);
        if optional_id(a, "inner").is_none() {
            notch::on_split(panel, segment, &ts);
        }
        ffb_pattern::sew::note_split(panel, optional_id(a, "inner"), segment, &ts);
        Ok(json!({"indices": indices, "segment": segment, "pieces": indices.len() + 1, "length": total}))
    })
}

fn delete_schema() -> Value {
    sc::obj(target(json!({"index": sc::index("Point to remove.")})), &["panel", "index"])
}

fn delete_point(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(delete_schema, a)?;
    let index = required_count(a, "index")?;
    edit_panel(s, required_id(a, "panel")?, "Delete Point", |panel| {
        let old = panel.outline.clone();
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        check_index(pts.len(), index, "point")?;
        if pts.len() <= min_points(closed) {
            return Err(bad(format!("at least {} points must remain", min_points(closed))));
        }
        pts.remove(index);
        path::auto_handles(pts, closed);
        let remaining = pts.len();
        if optional_id(a, "inner").is_none() {
            notch::on_delete(panel, &old, index);
            ffb_pattern::sew::note_reproject(panel, &old);
        }
        Ok(json!({"removed": index, "points": remaining}))
    })
}

fn kind_schema() -> Value {
    sc::obj(
        target(json!({
            "index": sc::index("Point to convert."),
            "kind": sc::text("corner: sharp, handles removed. curve: handles follow the neighbours automatically. smooth: keeps its handles collinear.", &["corner", "curve", "smooth"])
        })),
        &["panel", "index", "kind"],
    )
}

fn set_point_kind(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(kind_schema, a)?;
    let index = required_count(a, "index")?;
    let kind = point_kind(a, "kind")?.unwrap_or_default();
    edit_panel(s, required_id(a, "panel")?, "Point Kind", |panel| {
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        check_index(pts.len(), index, "point")?;
        edit::set_kind(pts, index, kind, closed);
        let p = &pts[index];
        Ok(json!({"index": index, "kind": p.kind, "h_in": p.h_in, "h_out": p.h_out}))
    })
}

fn curvature_schema() -> Value {
    sc::obj(
        target(json!({
            "segment": sc::index("Segment to bend; segment i runs from point i to point i+1."),
            "amount": sc::mm("How far the middle of the segment bulges from the straight chord. Positive bulges to the right of the segment direction, which is outward on a counter-clockwise outline. 0 straightens the segment.", -sc::SIZE, sc::SIZE, None)
        })),
        &["panel", "segment", "amount"],
    )
}

fn curvature(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(curvature_schema, a)?;
    let (segment, amount) = (required_count(a, "segment")?, number_or(a, "amount", 0.0));
    edit_panel(s, required_id(a, "panel")?, "Curvature", |panel| {
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        check_index(path::seg_count(pts.len(), closed), segment, "segment")?;
        edit::curvature(pts, segment, amount);
        let curve = path::seg(pts, segment);
        Ok(json!({"segment": segment, "curved": curve.curved, "length": curve.length(), "chord": len(ffb_pattern::geom::sub(curve.p[3], curve.p[0]))}))
    })
}

fn smooth_schema() -> Value {
    sc::obj(
        target(json!({
            "points": sc::indices("Corner points to round, counted from 0. Corners next to curved segments are skipped."),
            "length": sc::mm("Distance from the corner where the rounding starts along each edge. Limited to half of the shorter neighbouring edge.", 0.1, sc::SIZE, Some(20.0)),
            "curvature": sc::ratio("1 is a circular arc, 0 a straight bevel, above 1 a fuller corner.", 0.0, 1.5, Some(1.0))
        })),
        &["panel", "points"],
    )
}

fn smooth(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(smooth_schema, a)?;
    let indices = index_list(a, "points")?;
    edit_panel(s, required_id(a, "panel")?, "Smooth Corners", |panel| {
        let old = panel.outline.clone();
        let (pts, closed) = path_of(panel, optional_id(a, "inner"))?;
        if let Some(&i) = indices.iter().find(|&&i| i >= pts.len()) {
            return Err(bad(format!("point {i} is out of range, there are {}", pts.len())));
        }
        let rounded = edit::round_corners(pts, closed, &indices, number_or(a, "length", 20.0), number_or(a, "curvature", 1.0));
        let remaining = pts.len();
        if rounded > 0 && optional_id(a, "inner").is_none() {
            notch::reproject(panel, &old);
            ffb_pattern::sew::note_reproject(panel, &old);
        }
        Ok(json!({"rounded": rounded, "skipped": indices.len() - rounded, "points": remaining}))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.move_point",
            mcp: "pattern_move_point",
            group: Group::Pattern,
            label: "Move Point",
            description: "Move one outline or inner-shape point to a position (to) or move one or several points by an offset (delta). Handles travel with the point and curve-kind neighbours re-flow. Use pattern.begin_drag and pattern.end_drag around many calls to make one undo step.",
            keys: &["Z"],
            hints: SET,
            schema: move_schema,
            run: move_point,
        },
        Command {
            id: "pattern.set_handle",
            mcp: "pattern_set_handle",
            group: Group::Pattern,
            label: "Edit Handle",
            description: "Set or remove the in or out Bezier handle of a point as an offset vector. On smooth points the opposite handle keeps its length and turns to stay collinear; with independent it does not.",
            keys: &[],
            hints: SET,
            schema: handle_schema,
            run: set_handle,
        },
        Command {
            id: "pattern.add_point",
            mcp: "pattern_add_point",
            group: Group::Pattern,
            label: "Add Point",
            description: "Insert a point on a segment at curve parameter t or at the spot nearest to a position. The shape does not change; a curved segment is split into two exact curves. Returns the new point index.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: add_point,
        },
        Command {
            id: "pattern.split_line",
            mcp: "pattern_split_line",
            group: Group::Pattern,
            label: "Split Line",
            description: "Split a segment by curve parameter t, by distance from its start in mm, or into equal pieces by count. Returns the indices of the new points.",
            keys: &[],
            hints: WRITE,
            schema: split_schema,
            run: split_line,
        },
        Command {
            id: "pattern.delete_point",
            mcp: "pattern_delete_point",
            group: Group::Pattern,
            label: "Delete Point",
            description: "Remove a point; its two segments merge into one. An outline keeps at least 3 points, an open inner line at least 2.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete_point,
        },
        Command {
            id: "pattern.set_point_kind",
            mcp: "pattern_set_point_kind",
            group: Group::Pattern,
            label: "Point Kind",
            description: "Convert a point to corner (sharp, no handles), curve (automatic handles that follow the neighbours) or smooth (editable collinear handles).",
            keys: &[],
            hints: SET,
            schema: kind_schema,
            run: set_point_kind,
        },
        Command {
            id: "pattern.curvature",
            mcp: "pattern_curvature",
            group: Group::Pattern,
            label: "Curvature",
            description: "Bend a segment into a symmetric curve by the bulge of its middle in mm; 0 straightens it. Positive bulges to the right of the segment direction.",
            keys: &[],
            hints: SET,
            schema: curvature_schema,
            run: curvature,
        },
        Command {
            id: "pattern.smooth",
            mcp: "pattern_smooth",
            group: Group::Pattern,
            label: "Smooth Corners",
            description: "Round corner points with a Bezier arc that starts the given length from each corner along both edges. Returns how many corners were rounded and how many skipped.",
            keys: &[],
            hints: WRITE,
            schema: smooth_schema,
            run: smooth,
        },
    ]
}
