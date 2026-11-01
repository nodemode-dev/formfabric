use ffb_body::place::Rig;
use ffb_body::{light_copy, skin, Body, PlacementPoint, PlacementVolume, VolumeKind};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, READ, WRITE};

fn joint_index(body: &Body, v: &Value) -> Result<Option<u16>, CommandError> {
    let found = match v {
        Value::Null => return Ok(None),
        Value::Number(n) => n.as_u64().filter(|i| (*i as usize) < body.joints.len()).map(|i| i as usize),
        Value::String(name) => body.joints.iter().position(|j| j.name.eq_ignore_ascii_case(name.trim())),
        _ => None,
    };
    found.map(|i| Some(i as u16)).ok_or_else(|| missing(format!("no joint {v} on this body; body.joints lists them")))
}

fn volume_schema() -> Value {
    obj(
        json!({
            "body": body_schema(),
            "volume": selector("Volume id or name. Leave out to create a new volume (name and radius required)."),
            "name": {"type": "string", "description": "Volume name. A name ending _L or _R pairs it with the other side."},
            "kind": {"type": "string", "enum": ["cylinder", "box", "plane"], "description": "Shape of the cross section. Placement wraps cylinders."},
            "joints": {"type": "array", "minItems": 2, "maxItems": 2, "items": {"type": ["integer", "string"]}, "description": "The two joints the volume follows, by name or index. It moves with them when the body is posed."},
            "center": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "Rest centre in body space."},
            "axis": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "description": "Rest axis direction in body space; it is normalised."},
            "height": {"type": "number", "minimum": 1, "maximum": 5000, "unit": "mm", "description": "Length along the axis."},
            "radius": {"type": "array", "items": {"type": "number", "minimum": 1, "maximum": 2000}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Radius sideways (left right) and radius front to back."},
            "scale": {"type": "number", "minimum": 0.1, "maximum": 10, "description": "Multiply the radius by this."},
            "mirror": {"type": "boolean", "default": true, "description": "Apply radius, height, scale and kind to the opposite side volume as well."},
            "remove": {"type": "boolean", "default": false, "description": "Delete the volume and the points on it."}
        }),
        &[],
    )
}

fn apply_shared(v: &mut PlacementVolume, kind: Option<VolumeKind>, height: Option<f64>, radius: Option<[f64; 2]>, scale: Option<f64>) {
    if let Some(k) = kind {
        v.kind = k;
    }
    if let Some(h) = height {
        v.height = h;
    }
    if let Some(r) = radius {
        v.radius = r;
    }
    if let Some(k) = scale {
        v.radius = [v.radius[0] * k, v.radius[1] * k];
    }
}

fn kind_of(a: &Value) -> Result<Option<VolumeKind>, CommandError> {
    Ok(match a.get("kind").and_then(Value::as_str) {
        None => None,
        Some("cylinder") => Some(VolumeKind::Cylinder),
        Some("box") => Some(VolumeKind::Box),
        Some("plane") => Some(VolumeKind::Plane),
        Some(other) => return Err(bad(format!("kind must be cylinder, box or plane, not {other}"))),
    })
}

fn volume_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let kind = kind_of(a)?;
    let height = number(a, "height", 1.0, 5000.0)?;
    let radius = pair(a, "radius", 1.0, 2000.0)?;
    let scale = number(a, "scale", 0.1, 10.0)?;
    let center = triple(a, "center")?;
    let axis = triple(a, "axis")?;
    let name = a.get("name").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty()).map(str::to_string);
    let mirror = a.get("mirror").and_then(Value::as_bool).unwrap_or(true);
    let remove = a.get("remove").and_then(Value::as_bool).unwrap_or(false);
    let selected = a.get("volume").filter(|v| !v.is_null());
    let before = light_copy(&s.bodies[i]);
    let result = edit_volume(&mut s.bodies[i], selected, name, kind, height, radius, scale, center, axis, a, mirror, remove);
    match result {
        Err(e) => {
            s.bodies[i] = restore(std::mem::take(&mut s.bodies[i]), &before);
            Err(e)
        }
        Ok(id) => {
            let changed = light_copy(&s.bodies[i]) != before;
            if changed {
                record_body(s, i, "Placement Volume", before);
            }
            let rig = Rig::new(&s.bodies[i]);
            let out = s.bodies[i].volumes.iter().find(|v| v.id == id).map(|v| volume_json(&rig, v));
            Ok(json!({"volume": out, "removed": remove, "changed": changed, "volumes": s.bodies[i].volumes.len()}))
        }
    }
}

fn restore(mut body: Body, before: &Body) -> Body {
    body.volumes = before.volumes.clone();
    body.points = before.points.clone();
    body
}

#[allow(clippy::too_many_arguments)]
fn edit_volume(
    body: &mut Body,
    selected: Option<&Value>,
    name: Option<String>,
    kind: Option<VolumeKind>,
    height: Option<f64>,
    radius: Option<[f64; 2]>,
    scale: Option<f64>,
    center: Option<[f64; 3]>,
    axis: Option<[f64; 3]>,
    a: &Value,
    mirror: bool,
    remove: bool,
) -> Result<u32, CommandError> {
    let joints = match a.get("joints").and_then(Value::as_array) {
        Some(list) => Some([joint_index(body, &list[0])?, joint_index(body, &list[1])?]),
        None => None,
    };
    let Some(selected) = selected else {
        let name = name.ok_or_else(|| bad("a new volume needs a name"))?;
        let radius = radius.ok_or_else(|| bad("a new volume needs a radius"))?;
        let (center, axis, height) = match (center, axis, height, joints) {
            (Some(c), Some(x), Some(h), _) => (c, x, h),
            (_, _, _, Some([Some(j0), Some(j1)])) => {
                let bind = skin::bind_frames(body);
                let p0 = bind.pos[usize::from(j0)];
                let p1 = bind.pos[usize::from(j1)];
                let (p0, p1) = ([f64::from(p0[0]), f64::from(p0[1]), f64::from(p0[2])], [f64::from(p1[0]), f64::from(p1[1]), f64::from(p1[2])]);
                let span = ffb_body::place::sub(p1, p0);
                (ffb_body::place::scale(ffb_body::place::add(p0, p1), 0.5), ffb_body::place::unit(span), ffb_body::place::length(span))
            }
            _ => return Err(bad("a new volume needs center, axis and height, or two joints")),
        };
        let id = next_volume_id(body);
        let mut v = PlacementVolume { id, name, kind: kind.unwrap_or_default(), joints: joints.unwrap_or([None, None]), center, axis, radius, height };
        apply_shared(&mut v, None, None, None, scale);
        body.volumes.push(v);
        return Ok(id);
    };
    seed_defaults(body)?;
    let at = volume_at(body, selected).ok_or_else(|| missing(format!("no volume {selected} on this body; place.points lists them")))?;
    let id = body.volumes[at].id;
    let partner = if mirror { volume_partner(body, at) } else { None };
    if remove {
        let gone: Vec<u32> = std::iter::once(id).chain(partner.map(|p| body.volumes[p].id)).collect();
        body.volumes.retain(|v| !gone.contains(&v.id));
        body.points.retain(|p| !gone.contains(&p.volume));
        return Ok(id);
    }
    if let Some(p) = partner {
        apply_shared(&mut body.volumes[p], kind, height, radius, scale);
    }
    let v = &mut body.volumes[at];
    apply_shared(v, kind, height, radius, scale);
    if let Some(n) = name {
        v.name = n;
    }
    if let Some(j) = joints {
        v.joints = j;
    }
    if let Some(c) = center {
        v.center = c;
    }
    if let Some(x) = axis {
        v.axis = ffb_body::place::unit(x);
    }
    Ok(id)
}

fn point_schema() -> Value {
    obj(
        json!({
            "body": body_schema(),
            "point": selector("Point id or name. Leave out to create a new point (name and volume required)."),
            "name": {"type": "string", "description": "Point name."},
            "volume": selector("Volume the point sits on, by id or name."),
            "around": {"type": "number", "minimum": -720, "maximum": 720, "unit": "deg", "description": "Angle around the volume axis: 0 front, 90 left, 180 back, 270 right."},
            "along": {"type": "number", "minimum": -1, "maximum": 2, "description": "Position along the axis from the first joint end (0) to the second (1)."},
            "offset": {"type": "number", "minimum": -200, "maximum": 1000, "unit": "mm", "description": "Gap between the volume surface and a panel placed here."},
            "wrap": {"type": "boolean", "description": "Whether a panel placed here bends around the volume."},
            "mirror": {"type": "boolean", "default": true, "description": "Apply the change to the mirrored partner point as well."},
            "remove": {"type": "boolean", "default": false, "description": "Delete the point."}
        }),
        &[],
    )
}

fn point_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let before = light_copy(&s.bodies[i]);
    let result = edit_point(&mut s.bodies[i], a);
    match result {
        Err(e) => {
            s.bodies[i] = restore(std::mem::take(&mut s.bodies[i]), &before);
            Err(e)
        }
        Ok((id, removed)) => {
            let changed = light_copy(&s.bodies[i]) != before;
            if changed {
                record_body(s, i, "Placement Point", before);
            }
            let rig = Rig::new(&s.bodies[i]);
            let out = s.bodies[i].points.iter().find(|p| p.id == id).map(|p| point_json(&rig, &s.bodies[i], p));
            Ok(json!({"point": out, "removed": removed, "changed": changed, "points": s.bodies[i].points.len()}))
        }
    }
}

fn edit_point(body: &mut Body, a: &Value) -> Result<(u32, bool), CommandError> {
    let name = a.get("name").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty()).map(str::to_string);
    let around = number(a, "around", -720.0, 720.0)?.map(|x| x.rem_euclid(360.0));
    let along = number(a, "along", -1.0, 2.0)?;
    let offset = number(a, "offset", -200.0, 1000.0)?;
    let wrap = a.get("wrap").and_then(Value::as_bool);
    let mirror = a.get("mirror").and_then(Value::as_bool).unwrap_or(true);
    let remove = a.get("remove").and_then(Value::as_bool).unwrap_or(false);
    let volume = match a.get("volume").filter(|v| !v.is_null()) {
        Some(v) => Some(volume_at(body, v).map(|k| body.volumes[k].id).ok_or_else(|| missing(format!("no volume {v} on this body; place.points lists them")))?),
        None => None,
    };
    let Some(selected) = a.get("point").filter(|v| !v.is_null()) else {
        let name = name.ok_or_else(|| bad("a new point needs a name"))?;
        let volume = volume.ok_or_else(|| bad("a new point needs a volume"))?;
        let id = next_point_id(body);
        body.points.push(PlacementPoint { id, name, volume, around: around.unwrap_or(0.0), along: along.unwrap_or(0.5), offset: offset.unwrap_or(10.0), wrap: wrap.unwrap_or(true), mirror_of: None });
        return Ok((id, false));
    };
    seed_defaults(body)?;
    let at = point_at(body, selected).ok_or_else(|| missing(format!("no point {selected} on this body; place.points lists them")))?;
    let id = body.points[at].id;
    let partner = if mirror { point_partner(body, at) } else { None };
    if remove {
        let gone: Vec<u32> = std::iter::once(id).chain(partner.map(|p| body.points[p].id)).collect();
        body.points.retain(|p| !gone.contains(&p.id));
        for p in body.points.iter_mut() {
            if p.mirror_of.is_some_and(|m| gone.contains(&m)) {
                p.mirror_of = None;
            }
        }
        return Ok((id, true));
    }
    if let Some(p) = partner {
        let twin = &mut body.points[p];
        if let Some(x) = around {
            twin.around = (360.0 - x) % 360.0;
        }
        if let Some(x) = along {
            twin.along = x;
        }
        if let Some(x) = offset {
            twin.offset = x;
        }
        if let Some(x) = wrap {
            twin.wrap = x;
        }
    }
    let point = &mut body.points[at];
    if let Some(n) = name {
        point.name = n;
    }
    if let Some(v) = volume {
        point.volume = v;
    }
    if let Some(x) = around {
        point.around = x;
    }
    if let Some(x) = along {
        point.along = x;
    }
    if let Some(x) = offset {
        point.offset = x;
    }
    if let Some(x) = wrap {
        point.wrap = x;
    }
    Ok((id, false))
}

fn points_schema() -> Value {
    obj(json!({"body": body_schema()}), &[])
}

fn points(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let mut body = s.bodies[i].clone();
    let stored = !body.volumes.is_empty();
    seed_defaults(&mut body)?;
    let rig = Rig::new(&body);
    let volumes: Vec<Value> = body.volumes.iter().map(|v| volume_json(&rig, v)).collect();
    let list: Vec<Value> = body.points.iter().map(|p| point_json(&rig, &body, p)).collect();
    let posed: Vec<&String> = body.pose.keys().collect();
    Ok(json!({"body": body.id, "stored": stored, "posed_joints": posed, "volumes": volumes, "points": list}))
}

fn defaults_schema() -> Value {
    obj(json!({"body": body_schema()}), &[])
}

fn defaults(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let before = light_copy(&s.bodies[i]);
    let (volumes, points) = ffb_body::place_defaults::default_volumes(&s.bodies[i]);
    if volumes.is_empty() {
        return Err(CommandError::new("invalid_geometry", "this body has no skeleton with arms, legs and a neck, so no default volumes can be made; add them with place.volume_set"));
    }
    s.bodies[i].volumes = volumes;
    s.bodies[i].points = points;
    let changed = light_copy(&s.bodies[i]) != before;
    if changed {
        record_body(s, i, "Default Placement Volumes", before);
    }
    Ok(json!({"body": s.bodies[i].id, "volumes": s.bodies[i].volumes.len(), "points": s.bodies[i].points.len(), "changed": changed}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "place.volume_set",
            mcp: "place_volume_set",
            group: Group::Body,
            label: "Placement Volume",
            description: "Create, edit or remove a Placement Volume of a body: a cylinder stretched between two joints with a sideways and a front to back radius. Volumes are authored data: a body without any gets the default set from its joints the first time you edit one. Radius, height, scale and kind follow to the mirrored side (_L and _R names) unless mirror is false. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: volume_schema,
            run: volume_set,
        },
        Command {
            id: "place.point_set",
            mcp: "place_point_set",
            group: Group::Body,
            label: "Placement Point",
            description: "Create, edit or remove a Placement Point: a named spot on a volume given by an angle around the axis, a fraction along it, an offset from the surface and whether panels wrap. Points are stored in volume coordinates and follow the volume when the body is posed. Mirrored partners change together unless mirror is false. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: point_schema,
            run: point_set,
        },
        Command {
            id: "place.points",
            mcp: "place_points",
            group: Group::Body,
            label: "Placement Points",
            description: "List the Placement Volumes and Points of a body with their world position, outward normal and up direction for the current pose. A body without stored volumes shows the default set that would be created, with stored false.",
            keys: &[],
            hints: READ,
            schema: points_schema,
            run: points,
        },
        Command {
            id: "place.defaults",
            mcp: "place_defaults",
            group: Group::Body,
            label: "Default Placement Volumes",
            description: "Replace the volumes and points of a body with the default set built from its joints: torso, neck, upper and lower arms, thighs and shins, with points front, back, left, right, skirt, neck, sleeves, cuffs and trouser legs in mirrored pairs. The volumes are sized by a fixed profile from the joint distances, not from the mesh. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: defaults_schema,
            run: defaults,
        },
    ]
}
