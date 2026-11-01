use ffb_pattern::geom::{dist, V2};
use ffb_pattern::snap::{self, SnapKind, SnapOptions};
use ffb_pattern::{path, polygon, Panel};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::index_of;
use crate::{Command, CommandError, Group, Session, READ};

const MEASURE_TOL: f64 = 0.02;

fn get_schema() -> Value {
    sc::obj(json!({"panel": sc::panel()}), &[])
}

fn get(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(get_schema, a)?;
    let mut out = serde_json::to_value(&s.garment).map_err(|e| CommandError::new("io", e.to_string()))?;
    if let Some(id) = optional_id(a, "panel") {
        let index = index_of(s, id)?;
        out["panels"] = json!([s.garment.panels[index]]);
    }
    super::derived::annotate(&mut out, &s.garment);
    Ok(out)
}

fn measure_schema() -> Value {
    sc::obj(json!({"panel": sc::panel()}), &[])
}

fn segment_report(points: &[ffb_pattern::Point], closed: bool) -> Vec<Value> {
    path::segments(points, closed)
        .iter()
        .enumerate()
        .map(|(i, s)| json!({"index": i, "length": s.length(), "chord": dist(s.p[0], s.p[3]), "curved": s.curved}))
        .collect()
}

fn panel_report(p: &Panel) -> Value {
    let poly = path::flatten(&p.outline, true, MEASURE_TOL);
    let area = polygon::signed_area(&poly);
    let bounds = path::bounds(&p.outline, true);
    let crossings = polygon::self_intersections(&path::flatten_segments(&p.outline, true, MEASURE_TOL), true);
    let inner: Vec<Value> = p
        .inner
        .iter()
        .map(|n| {
            let area = n.closed.then(|| polygon::signed_area(&path::flatten(&n.points, true, MEASURE_TOL)).abs());
            json!({"id": n.id, "kind": n.kind, "closed": n.closed, "length": path::length(&n.points, n.closed), "area": area, "segments": segment_report(&n.points, n.closed)})
        })
        .collect();
    json!({
        "panel": p.id,
        "name": p.name,
        "perimeter": path::length(&p.outline, true),
        "area": area.abs(),
        "orientation": polygon::orientation(area),
        "centroid": polygon::centroid(&poly),
        "bounds": bounds,
        "world_bounds": p.world_bounds(),
        "width": bounds.map(|b| b[1][0] - b[0][0]),
        "height": bounds.map(|b| b[1][1] - b[0][1]),
        "points": p.outline.len(),
        "self_intersecting": !crossings.is_empty(),
        "crossings": crossings,
        "segments": segment_report(&p.outline, true),
        "inner": inner
    })
}

fn measure(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(measure_schema, a)?;
    match optional_id(a, "panel") {
        Some(id) => Ok(panel_report(&s.garment.panels[index_of(s, id)?])),
        None => Ok(json!({"units": s.garment.units, "panels": s.garment.panels.iter().map(panel_report).collect::<Vec<_>>()})),
    }
}

fn select_schema() -> Value {
    sc::obj(
        json!({
            "at": sc::xy("World position to hit-test."),
            "box": {"type": "array", "items": {"type": "number", "minimum": -sc::COORD, "maximum": sc::COORD}, "minItems": 4, "maxItems": 4, "unit": "mm", "description": "World rectangle [x0, y0, x1, y1] to select inside."},
            "tolerance": sc::mm("Pick distance for points and segments under at.", 0.1, 1000.0, Some(5.0)),
            "touch": sc::flag("With box, also select panels the box only touches.", false)
        }),
        &[],
    )
}

fn hit_point(panel: &Panel, inner: Option<u32>, index: usize, distance: f64) -> Value {
    json!({"panel": panel.id, "inner": inner, "index": index, "distance": distance})
}

fn pick_at(s: &Session, q: V2, tol: f64) -> Value {
    let (mut panels, mut points, mut segments) = (Vec::new(), Vec::new(), Vec::new());
    for panel in s.garment.panels.iter().rev() {
        let place = panel.placement();
        let ql = place.to_local(q);
        let poly = path::flatten(&panel.outline, true, MEASURE_TOL);
        let edge = path::nearest(&panel.outline, true, ql).map_or(f64::MAX, |n| n.dist);
        if polygon::contains(&poly, ql) || edge <= tol {
            panels.push(panel);
        }
        let paths = std::iter::once((None, &panel.outline, true)).chain(panel.inner.iter().map(|n| (Some(n.id), &n.points, n.closed)));
        for (inner, pts, closed) in paths {
            for (i, p) in pts.iter().enumerate() {
                let d = dist(p.p, ql);
                if d <= tol {
                    points.push(hit_point(panel, inner, i, d));
                }
            }
            if let Some(n) = path::nearest(pts, closed, ql).filter(|n| n.dist <= tol) {
                segments.push(json!({"panel": panel.id, "inner": inner, "index": n.seg, "t": n.t, "distance": n.dist}));
            }
        }
    }
    panels.sort_by_key(|p| std::cmp::Reverse(p.layer));
    let by_distance = |list: &mut Vec<Value>| list.sort_by(|x, y| x["distance"].as_f64().unwrap_or(0.0).total_cmp(&y["distance"].as_f64().unwrap_or(0.0)));
    by_distance(&mut points);
    by_distance(&mut segments);
    json!({"panels": panels.iter().map(|p| p.id).collect::<Vec<_>>(), "points": points, "segments": segments})
}

fn pick_box(s: &Session, b: [f64; 4], touch: bool) -> Value {
    let (lo, hi) = ([b[0].min(b[2]), b[1].min(b[3])], [b[0].max(b[2]), b[1].max(b[3])]);
    let inside = |p: V2| p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1];
    let (mut panels, mut points, mut segments) = (Vec::new(), Vec::new(), Vec::new());
    for panel in &s.garment.panels {
        let place = panel.placement();
        if let Some(wb) = panel.world_bounds() {
            let whole = inside(wb[0]) && inside(wb[1]);
            let overlap = wb[0][0] <= hi[0] && wb[1][0] >= lo[0] && wb[0][1] <= hi[1] && wb[1][1] >= lo[1];
            if whole || (touch && overlap) {
                panels.push(panel.id);
            }
        }
        let paths = std::iter::once((None, &panel.outline, true)).chain(panel.inner.iter().map(|n| (Some(n.id), &n.points, n.closed)));
        for (inner, pts, closed) in paths {
            let hits: Vec<bool> = pts.iter().map(|p| inside(place.to_world(p.p))).collect();
            for (i, _) in hits.iter().enumerate().filter(|(_, h)| **h) {
                points.push(json!({"panel": panel.id, "inner": inner, "index": i}));
            }
            for i in 0..path::seg_count(pts.len(), closed) {
                if hits[i] && hits[(i + 1) % pts.len()] {
                    segments.push(json!({"panel": panel.id, "inner": inner, "index": i}));
                }
            }
        }
    }
    json!({"panels": panels, "points": points, "segments": segments})
}

fn select(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(select_schema, a)?;
    match (optional_vec2(a, "at")?, a.get("box")) {
        (Some(q), None) => Ok(pick_at(s, q, number_or(a, "tolerance", 5.0))),
        (None, Some(b)) => {
            let v: Vec<f64> = b.as_array().map(|l| l.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
            let b: [f64; 4] = v.try_into().map_err(|_| bad("box must be [x0, y0, x1, y1]"))?;
            Ok(pick_box(s, b, flag(a, "touch").unwrap_or(false)))
        }
        _ => Err(bad("give exactly one of at or box")),
    }
}

fn snap_schema() -> Value {
    sc::obj(
        json!({
            "point": sc::xy("World position to snap."),
            "targets": {
                "type": "array",
                "items": {"type": "string", "enum": ["grid", "point", "segment", "midpoint", "inner"]},
                "minItems": 1,
                "description": "Snap targets to try, in priority order point, midpoint, inner, segment, grid. Defaults to all. inner covers inner shape points, midpoints and lines."
            },
            "tolerance": sc::mm("Largest distance a target may be from the point.", 0.01, 1000.0, Some(5.0)),
            "grid": sc::mm("Grid spacing. 0 turns the grid target off.", 0.0, 1000.0, Some(10.0)),
            "exclude": sc::ids("Panel ids to ignore, such as the panel being dragged.")
        }),
        &["point"],
    )
}

fn snap_cmd(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(snap_schema, a)?;
    let q = vec2(a.get("point").ok_or_else(|| bad("point is required"))?, "point")?;
    let kinds = match a.get("targets").and_then(Value::as_array) {
        Some(list) => list.iter().filter_map(Value::as_str).map(|t| SnapKind::parse(t).ok_or_else(|| bad(format!("unknown target {t}")))).collect::<Result<Vec<_>, _>>()?,
        None => SnapKind::ORDER.to_vec(),
    };
    let exclude = if a.get("exclude").is_some() { id_list(a, "exclude")? } else { Vec::new() };
    let options = SnapOptions { kinds, tolerance: number_or(a, "tolerance", 5.0), grid: number_or(a, "grid", 10.0), exclude };
    Ok(match snap::snap(&s.garment, q, &options) {
        Some(h) => json!({"snapped": true, "point": h.point, "kind": h.kind.name(), "distance": h.distance, "panel": h.panel, "inner": h.inner, "index": h.index}),
        None => json!({"snapped": false, "point": q, "kind": null, "distance": 0.0, "panel": null, "inner": null, "index": null}),
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "pattern.get",
            mcp: "pattern_get",
            group: Group::Pattern,
            label: "Pattern",
            description: "Return the garment document: units, every panel with outline points, handles, inner shapes and placement, and the selection. Pass panel to return only that panel.",
            keys: &[],
            hints: READ,
            schema: get_schema,
            run: get,
        },
        Command {
            id: "pattern.select",
            mcp: "pattern_select",
            group: Group::Pattern,
            label: "Select",
            description: "Hit-test without changing the document. With at, return the panels under the position (top layer first) and the points and segments within tolerance, nearest first. With box, return panels fully inside (or touched when touch is true) and the points and segments inside the rectangle.",
            keys: &[],
            hints: READ,
            schema: select_schema,
            run: select,
        },
        Command {
            id: "pattern.measure",
            mcp: "pattern_measure",
            group: Group::Pattern,
            label: "Measure",
            description: "Measure a panel: segment lengths and chords, perimeter, area, orientation, centroid, bounds, self-intersections and the same for each inner shape. Without panel it reports every panel. All values are in mm and mm squared.",
            keys: &[],
            hints: READ,
            schema: measure_schema,
            run: measure,
        },
        Command {
            id: "pattern.snap",
            mcp: "pattern_snap",
            group: Group::Pattern,
            label: "Snap",
            description: "Find where a world position snaps to: an outline point, a segment midpoint, an inner shape, the nearest spot on a segment, or the grid, within a tolerance in mm. Does not change the document. Returns the snapped position and the kind of target.",
            keys: &[],
            hints: READ,
            schema: snap_schema,
            run: snap_cmd,
        },
    ]
}
