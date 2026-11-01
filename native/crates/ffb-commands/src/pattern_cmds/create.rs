use ffb_pattern::geom::{sub, V2};
use ffb_pattern::{edit, path, polygon, shapes, Inner, InnerKind, Panel, Point};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::edit_panel;
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

const OUTLINE_TOL: f64 = 0.05;
const MIN_AREA: f64 = 1.0;

fn add_panel(s: &mut Session, label: &str, a: &Value, mut pts: Vec<Point>) -> Result<Value, CommandError> {
    path::auto_handles(&mut pts, true);
    let area = polygon::signed_area(&path::flatten(&pts, true, OUTLINE_TOL));
    if area.abs() < MIN_AREA {
        return Err(geometry("the outline encloses no area"));
    }
    if let Some((i, j)) = polygon::self_intersections(&path::flatten_segments(&pts, true, OUTLINE_TOL), true).first() {
        return Err(geometry(format!("the outline crosses itself between segments {i} and {j}")));
    }
    let reversed = area < 0.0;
    if reversed {
        path::reverse(&mut pts);
    }
    let bounds = path::bounds(&pts, true).ok_or_else(|| geometry("the outline is empty"))?;
    let origin = bounds[0];
    for p in pts.iter_mut() {
        p.p = sub(p.p, origin);
    }
    let id = s.garment.next_id();
    let panel = Panel {
        id,
        name: name(a, "name")?.unwrap_or_else(|| format!("Panel {id}")),
        outline: pts,
        position: origin,
        particle_distance: number_or(a, "particle_distance", 20.0),
        ..Panel::default()
    };
    let index = s.garment.panels.len();
    let report = json!({
        "id": id, "index": index, "name": panel.name, "position": origin, "reversed": reversed,
        "width": bounds[1][0] - origin[0], "height": bounds[1][1] - origin[1],
        "area": area.abs(), "perimeter": path::length(&panel.outline, true)
    });
    s.garment.panels.push(panel.clone());
    s.record(Edit::Panel { label: label.into(), index, before: None, after: Some(Box::new(panel)) });
    Ok(report)
}

fn panel_props(extra: Value) -> Value {
    let mut props = json!({
        "name": sc::name("Panel name. Defaults to Panel and the new id."),
        "particle_distance": sc::mm("Simulation particle spacing for this panel.", 0.8, 700.0, Some(20.0))
    });
    for (k, v) in extra.as_object().into_iter().flatten() {
        props[k] = v.clone();
    }
    props
}

fn draw_schema() -> Value {
    sc::obj(panel_props(json!({"points": sc::points("Outline vertices in world mm, at least 3, in drawing order. A point is [x, y] or an object with p, kind, h_in and h_out. The outline is stored counter-clockwise.", 3)})), &["points"])
}

fn draw_panel(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(draw_schema, a)?;
    let pts = point_list(a, "points", 3)?;
    add_panel(s, "Draw Panel", a, pts)
}

fn rectangle_schema() -> Value {
    sc::obj(
        panel_props(json!({
            "x": sc::coord("Left edge.", Some(0.0)),
            "y": sc::coord("Bottom edge.", Some(0.0)),
            "width": sc::size("Width along x."),
            "height": sc::size("Height along y."),
            "corner_radius": sc::mm("Round all four corners by this tangent length; 0 keeps them sharp. Limited to half of the shorter side.", 0.0, sc::SIZE, Some(0.0))
        })),
        &["width", "height"],
    )
}

fn rectangle(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(rectangle_schema, a)?;
    let mut pts = shapes::rectangle(number_or(a, "x", 0.0), number_or(a, "y", 0.0), number_or(a, "width", 0.0), number_or(a, "height", 0.0));
    let radius = number_or(a, "corner_radius", 0.0);
    if radius > 0.0 {
        edit::round_corners(&mut pts, true, &[0, 1, 2, 3], radius, 1.0);
    }
    add_panel(s, "Rectangle", a, pts)
}

fn ellipse_props() -> Value {
    json!({
        "cx": sc::coord("Centre x.", Some(0.0)),
        "cy": sc::coord("Centre y.", Some(0.0)),
        "rx": sc::size("Radius along x."),
        "ry": sc::size("Radius along y. Equal to rx gives a circle."),
        "segments": sc::int("Number of curve points around the ellipse. 4 already gives a close fit.", 4.0, 64.0, Some(4.0))
    })
}

fn ellipse_schema() -> Value {
    sc::obj(panel_props(ellipse_props()), &["rx", "ry"])
}

fn ellipse_points(spec: &Value) -> Vec<Point> {
    shapes::ellipse(number_or(spec, "cx", 0.0), number_or(spec, "cy", 0.0), number_or(spec, "rx", 1.0), number_or(spec, "ry", 1.0), count(spec, "segments").unwrap_or(4))
}

fn ellipse(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(ellipse_schema, a)?;
    add_panel(s, "Ellipse", a, ellipse_points(a))
}

fn fold_props(mut props: Value) -> Value {
    sc::fold_fields(&mut props);
    props
}

fn inner_shape_schema() -> Value {
    sc::obj(
        fold_props(json!({
            "panel": sc::panel(),
            "kind": sc::text("Shape to add. rectangle uses rect, ellipse uses ellipse, the others use points.", &["line", "polyline", "rectangle", "ellipse", "polygon"]),
            "points": sc::points("Vertices: exactly 2 for line, at least 2 for polyline, at least 3 for polygon.", 2),
            "rect": sc::obj(json!({
                "x": sc::coord("Left edge.", Some(0.0)),
                "y": sc::coord("Bottom edge.", Some(0.0)),
                "width": sc::size("Width along x."),
                "height": sc::size("Height along y.")
            }), &["width", "height"]),
            "ellipse": sc::obj(ellipse_props(), &["rx", "ry"]),
            "closed": sc::flag("Close a polyline into a loop. Polygons, rectangles and ellipses are always closed.", false),
            "space": sc::space()
        })),
        &["panel", "kind"],
    )
}

fn build_inner(a: &Value) -> Result<(InnerKind, Vec<Point>, bool), CommandError> {
    let kind = text(a, "kind").unwrap_or("line");
    let spec = |key: &str| a.get(key).ok_or_else(|| bad(format!("{kind} needs {key}")));
    Ok(match kind {
        "rectangle" => {
            let r = spec("rect")?;
            (InnerKind::Rectangle, shapes::rectangle(number_or(r, "x", 0.0), number_or(r, "y", 0.0), number_or(r, "width", 1.0), number_or(r, "height", 1.0)), true)
        }
        "ellipse" => (InnerKind::Ellipse, ellipse_points(spec("ellipse")?), true),
        "polygon" => (InnerKind::Polygon, point_list(a, "points", 3)?, true),
        "polyline" => (InnerKind::Polyline, point_list(a, "points", 2)?, flag(a, "closed").unwrap_or(false)),
        _ => {
            let pts = point_list(a, "points", 2)?;
            if pts.len() != 2 {
                return Err(bad("a line takes exactly 2 points; use polyline for more"));
            }
            (InnerKind::Line, pts, false)
        }
    })
}

fn add_inner(s: &mut Session, a: &Value, label: &str, kind: InnerKind, mut pts: Vec<Point>, closed: bool) -> Result<Value, CommandError> {
    let panel_id = required_id(a, "panel")?;
    let id = s.garment.next_id();
    let space = space(a);
    let (angle, strength) = (number_or(a, "fold_angle", 180.0), number_or(a, "fold_strength", 0.0));
    let out = edit_panel(s, panel_id, label, |panel| {
        for p in pts.iter_mut() {
            p.p = to_local(panel, space, p.p);
            p.h_in = p.h_in.map(|h| vec_to_local(panel, space, h));
            p.h_out = p.h_out.map(|h| vec_to_local(panel, space, h));
        }
        path::auto_handles(&mut pts, closed);
        let outline = path::flatten(&panel.outline, true, OUTLINE_TOL);
        let inside = |q: V2| polygon::contains(&outline, q) || path::nearest(&panel.outline, true, q).is_some_and(|n| n.dist <= 0.01);
        if !path::flatten(&pts, closed, 0.2).into_iter().all(inside) {
            return Err(geometry("the inner shape must lie inside the panel outline"));
        }
        panel.inner.push(Inner { id, kind, points: pts, closed, fold_angle: angle, fold_strength: strength, ..Inner::default() });
        Ok(json!({"panel": panel_id, "id": id, "kind": format!("{kind:?}").to_lowercase(), "index": panel.inner.len() - 1}))
    })?;
    Ok(out)
}

fn inner_shape(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(inner_shape_schema, a)?;
    let (kind, pts, closed) = build_inner(a)?;
    add_inner(s, a, "Inner Shape", kind, pts, closed)
}

fn inner_line_schema() -> Value {
    sc::obj(
        fold_props(json!({
            "panel": sc::panel(),
            "points": sc::points("Vertices of the inner line, at least 2. Two points make a line, more make a polyline.", 2),
            "space": sc::space()
        })),
        &["panel", "points"],
    )
}

fn inner_line(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(inner_line_schema, a)?;
    let pts = point_list(a, "points", 2)?;
    let kind = if pts.len() == 2 { InnerKind::Line } else { InnerKind::Polyline };
    add_inner(s, a, "Inner Line", kind, pts, false)
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.draw_panel",
            mcp: "pattern_draw_panel",
            group: Group::Pattern,
            label: "Draw Panel",
            description: "Create a pattern panel from outline points given in world mm. Points may carry curve kinds and Bezier handles. The outline is stored counter-clockwise, placed at its lower left corner, and rejected when it has no area or crosses itself. Returns the new panel id.",
            keys: &[],
            hints: WRITE,
            schema: draw_schema,
            run: draw_panel,
        },
        Command {
            id: "pattern.rectangle",
            mcp: "pattern_rectangle",
            group: Group::Pattern,
            label: "Rectangle",
            description: "Create a rectangular panel from its lower left corner, width and height in mm, with an optional corner radius. Returns the new panel id.",
            keys: &[],
            hints: WRITE,
            schema: rectangle_schema,
            run: rectangle,
        },
        Command {
            id: "pattern.ellipse",
            mcp: "pattern_ellipse",
            group: Group::Pattern,
            label: "Ellipse",
            description: "Create an elliptical panel from its centre and radii in mm as 4 or more smooth curve points with exact Bezier handles. Equal radii make a circle. Returns the new panel id.",
            keys: &[],
            hints: WRITE,
            schema: ellipse_schema,
            run: ellipse,
        },
        Command {
            id: "pattern.inner_shape",
            mcp: "pattern_inner_shape",
            group: Group::Pattern,
            label: "Inner Shape",
            description: "Add an inner line, polyline, polygon, rectangle or ellipse to a panel. The shape must lie inside the outline. Rectangles take rect, ellipses take ellipse, the others take points. Optional fold angle and strength apply to open inner lines.",
            keys: &[],
            hints: WRITE,
            schema: inner_shape_schema,
            run: inner_shape,
        },
        Command {
            id: "pattern.inner_line",
            mcp: "pattern_inner_line",
            group: Group::Pattern,
            label: "Inner Line",
            description: "Add an open inner line through two or more points inside a panel, for folds, darts guides or sewing references. Returns the inner shape id.",
            keys: &[],
            hints: WRITE,
            schema: inner_line_schema,
            run: inner_line,
        },
    ]
}
