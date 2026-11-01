use ffb_body::{math, measure, skin};
use serde_json::{json, Value};

use super::ops::{id_schema, pick, summary};
use crate::{Command, CommandError, Group, Session, READ};

fn list(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let bodies: Vec<Value> = s.bodies.iter().map(summary).collect();
    let library: Vec<Value> = std::iter::once(json!({"name": ffb_body::FORM_NAME, "license": ffb_body::FORM_LICENSE, "built_in": true}))
        .chain(super::library::entries().into_iter().map(|e| json!({"name": e.name, "license": e.license, "path": e.path.to_string_lossy(), "built_in": false})))
        .collect();
    Ok(json!({"bodies": bodies, "library": library, "poses": ffb_body::pose::NAMES}))
}

fn measure_cmd(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let body = &mut s.bodies[i];
    body.measurements = measure::measure(body);
    Ok(json!({"id": body.id, "name": body.name, "unit": "mm", "measurements": body.measurements}))
}

fn joints(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let i = pick(s, a)?;
    let body = &s.bodies[i];
    let bind = skin::bind_frames(body);
    let posed = skin::frames(&body.joints, Some(&body.pose));
    let list: Vec<Value> = body
        .joints
        .iter()
        .enumerate()
        .map(|(k, j)| {
            let q = body.pose.get(&j.name).map(|q| math::qnorm(*q)).unwrap_or(math::IDENTITY);
            json!({"index": k, "name": j.name, "parent": j.parent, "rest": j.rest, "position": bind.pos[k], "posed_position": posed.pos[k], "pose": q})
        })
        .collect();
    Ok(json!({"id": body.id, "count": list.len(), "joints": list}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "body.list",
            mcp: "body_list",
            group: Group::Body,
            label: "Bodies",
            description: "List the bodies in the project and the body library (formfabric's Basic Form plus the files under assets/bodies), and the named poses.",
            keys: &[],
            hints: READ,
            schema: crate::no_args,
            run: list,
        },
        Command {
            id: "body.measure",
            mcp: "body_measure",
            group: Group::Body,
            label: "Measure Body",
            description: "Standard measurements in millimetres computed from the posed mesh: height, chest (also bust), waist, hip (widest girth below the waist), inseam, shoulder width and arm length. Girths are convex hull perimeters of horizontal slices.",
            keys: &[],
            hints: READ,
            schema: id_schema,
            run: measure_cmd,
        },
        Command {
            id: "body.joints",
            mcp: "body_joints",
            group: Group::Body,
            label: "Skeleton",
            description: "The skeleton of a body: each joint with parent, local rest offset, rest and posed world positions and its pose rotation as a quaternion [x, y, z, w].",
            keys: &[],
            hints: READ,
            schema: id_schema,
            run: joints,
        },
    ]
}
