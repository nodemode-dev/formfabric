use ffb_body::light_copy;
use ffb_body::place::{cross, rotate, scale, sub, quaternion_from_columns, Rig};
use ffb_pattern::Place3D;
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Edit, Group, Session, SET, WRITE};

fn panel_schema() -> Value {
    obj(
        json!({
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id as listed by pattern.get."},
            "point": selector("Placement point id or name, as listed by place.points."),
            "body": body_schema(),
            "offset": {"type": "number", "minimum": -200, "maximum": 1000, "unit": "mm", "description": "Gap between the volume surface and the panel. Defaults to the point's own offset."},
            "orientation": {"type": "number", "minimum": -360, "maximum": 360, "default": 0, "unit": "deg", "description": "Turn the panel about its normal. 0 puts the panel top toward the head or shoulder; 180 turns it upside down and still wraps. Other angles place the panel flat on the tangent plane."},
            "flip": {"type": "boolean", "default": false, "description": "Face the panel front toward the body instead of away from it."},
            "wrap": {"type": "boolean", "description": "Bend the panel around the volume. Defaults to the point's wrap setting."}
        }),
        &["panel", "point"],
    )
}

fn place_panel(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = a.get("panel").and_then(Value::as_u64).ok_or_else(|| bad("panel must be a panel id"))? as u32;
    let index = s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))?;
    let bi = pick(s, a)?;
    let selected = a.get("point").ok_or_else(|| bad("point is required"))?;
    let orientation = number(a, "orientation", -360.0, 360.0)?.unwrap_or(0.0);
    let flip = a.get("flip").and_then(Value::as_bool).unwrap_or(false);
    let panel = s.garment.panels[index].clone();
    let built = s.mesh.ensure(&panel, false).built.map_err(|e| CommandError::new("invalid_geometry", format!("panel {id} cannot be meshed: {e}")))?;
    let mesh = &built.mesh;
    if mesh.positions.is_empty() {
        return Err(CommandError::new("invalid_geometry", format!("panel {id} has no mesh")));
    }
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in &mesh.positions {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let anchor = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, 0.0];
    let width = hi[0] - lo[0];
    let body_before = light_copy(&s.bodies[bi]);
    let mut body = s.bodies[bi].clone();
    let seeded = seed_defaults(&mut body)?;
    let at = point_at(&body, selected).ok_or_else(|| missing(format!("no placement point {selected} on this body; place.points lists them")))?;
    let mut point = body.points[at].clone();
    if let Some(x) = number(a, "offset", -200.0, 1000.0)? {
        point.offset = x;
    }
    let volume = body.volumes.iter().find(|v| v.id == point.volume).ok_or_else(|| missing(format!("point {} refers to a volume that is gone", point.name)))?;
    let frame = Rig::new(&body).point(volume, &point);
    let (volume_name, point_name) = (volume.name.clone(), point.name.clone());
    let wrap = a.get("wrap").and_then(Value::as_bool).unwrap_or(point.wrap);
    let turn = orientation.to_radians();
    let n = frame.normal;
    let x0 = cross(frame.up, n);
    let y0 = frame.up;
    let mix = |u: [f64; 3], v: [f64; 3], c: f64, d: f64| [u[0] * c + v[0] * d, u[1] * c + v[1] * d, u[2] * c + v[2] * d];
    let x1 = mix(x0, y0, turn.cos(), turn.sin());
    let y1 = mix(x0, y0, -turn.sin(), turn.cos());
    let (cx, cz) = if flip { (scale(x1, -1.0), scale(n, -1.0)) } else { (x1, n) };
    let q = quaternion_from_columns(cx, y1, cz);
    let aligned = (orientation / 180.0 - (orientation / 180.0).round()).abs() < 1e-9;
    let radius = if wrap && aligned { frame.bend_radius } else { 0.0 };
    let bend = if flip { -radius } else { radius };
    let anchor_world = rotate(q, anchor);
    let origin = sub(frame.position, anchor_world);
    let place = Place3D { origin, rotation: q, bend, pose: None };
    let wrapped_degrees = if radius > 0.0 { (width / radius).to_degrees() } else { 0.0 };
    let before = s.garment.panels[index].clone();
    let mut after = before.clone();
    after.place3d = Some(place.clone());
    let mut edits = Vec::new();
    if seeded {
        s.bodies[bi].volumes = body.volumes.clone();
        s.bodies[bi].points = body.points.clone();
        edits.push(Edit::Bodies { label: "Place Panel".into(), changes: vec![body_change(s, bi, body_before)] });
    }
    let changed = after != before;
    if changed {
        s.garment.panels[index] = after.clone();
        edits.push(Edit::Panel { label: "Place Panel".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
    }
    match edits.len() {
        0 => {}
        1 => s.record(edits.remove(0)),
        _ => s.record(Edit::Many { label: "Place Panel".into(), edits }),
    }
    let mut out = json!({
        "panel": id,
        "point": point_name,
        "volume": volume_name,
        "place3d": place,
        "position": round3(frame.position),
        "normal": round3(frame.normal),
        "bend_radius": round(bend),
        "wrapped_degrees": round(wrapped_degrees),
        "changed": changed,
        "defaults_created": seeded,
    });
    if wrapped_degrees > 360.0 {
        out["warning"] = json!("the panel is wider than one turn of the volume; it overlaps itself");
    }
    if wrap && !aligned {
        out["note"] = json!("orientation is not a multiple of 180, so the panel is placed flat on the tangent plane");
    }
    Ok(out)
}

fn reset_schema() -> Value {
    obj(
        json!({
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Panel ids to return to the flat layout. Leave out for every panel."},
            "volumes": {"type": "boolean", "default": false, "description": "Also rebuild the default volumes and points of the body from its joints."},
            "body": body_schema()
        }),
        &[],
    )
}

fn reset(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let wanted: Option<Vec<u32>> = match a.get("panels").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => {
            let list = v.as_array().ok_or_else(|| bad("panels must be a list of panel ids"))?;
            let mut ids = Vec::with_capacity(list.len());
            for item in list {
                let id = item.as_u64().ok_or_else(|| bad("panels must hold whole panel ids"))? as u32;
                if s.garment.index_of(id).is_none() {
                    return Err(missing(format!("no panel with id {id}")));
                }
                ids.push(id);
            }
            Some(ids)
        }
    };
    let mut edits = Vec::new();
    let mut cleared = Vec::new();
    if a.get("volumes").and_then(Value::as_bool).unwrap_or(false) {
        let bi = pick(s, a)?;
        let before = light_copy(&s.bodies[bi]);
        let (volumes, points) = ffb_body::place_defaults::default_volumes(&s.bodies[bi]);
        if volumes.is_empty() {
            return Err(CommandError::new("invalid_geometry", "this body has no skeleton with arms, legs and a neck, so no default volumes can be made"));
        }
        s.bodies[bi].volumes = volumes;
        s.bodies[bi].points = points;
        if light_copy(&s.bodies[bi]) != before {
            edits.push(Edit::Bodies { label: "Reset Placement".into(), changes: vec![body_change(s, bi, before)] });
        }
    }
    for index in 0..s.garment.panels.len() {
        let panel = &s.garment.panels[index];
        if panel.place3d.is_none() || wanted.as_ref().is_some_and(|w| !w.contains(&panel.id)) {
            continue;
        }
        let before = panel.clone();
        let mut after = before.clone();
        after.place3d = None;
        cleared.push(after.id);
        s.garment.panels[index] = after.clone();
        edits.push(Edit::Panel { label: "Reset Placement".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
    }
    let changed = !edits.is_empty();
    match edits.len() {
        0 => {}
        1 => s.record(edits.remove(0)),
        _ => s.record(Edit::Many { label: "Reset Placement".into(), edits }),
    }
    Ok(json!({"reset": cleared, "changed": changed}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "place.panel",
            mcp: "place_panel",
            group: Group::Sim,
            label: "Place Panel",
            description: "Put a panel on a Placement Point: the panel centre goes to the point plus the offset, the panel front faces away from the volume (flip faces it inward), its top points up the body, and the panel is bent about the volume axis with a radius that clears the volume so it wraps instead of cutting through. Sets place3d (origin, rotation, bend); one undo step. Run sim.reset afterwards so the drape starts from this shape.",
            keys: &[],
            hints: SET,
            schema: panel_schema,
            run: place_panel,
        },
        Command {
            id: "place.reset",
            mcp: "place_reset",
            group: Group::Sim,
            label: "Reset Placement",
            description: "Clear the 3D placement of the given panels, or of all panels, so they return to the flat layout, and with volumes true rebuild the body's default volumes and points from its joints. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: reset_schema,
            run: reset,
        },
    ]
}
