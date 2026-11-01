use ffb_body::{skin, Body};
use ffb_pattern::Garment;
use ffb_sim::Collider;
use serde_json::{json, Value};

use super::common::{bad, id_arg, missing, ranged};
use super::fx;
use super::shell::{offset_surface, Shell};
use crate::{Command, CommandError, Edit, Group, Session, READ, SET};

const MAX_OFFSET: f64 = 100.0;

#[allow(dead_code)]
pub(crate) fn fit_shell_collider(garment: &Garment, body: &Body) -> Collider {
    match fx::shell_of(garment, body.id).filter(|s| s.1 && s.0 > 1e-6) {
        Some((offset, _)) => {
            let shell = offset_surface(&skin::posed_positions(body), &body.triangles, offset as f32);
            Collider::Mesh { positions: shell.positions, triangles: body.triangles.clone() }
        }
        None => ffb_body::collider(body),
    }
}

#[allow(dead_code)]
pub(crate) fn fit_shell_active(garment: &Garment, body_id: u32) -> bool {
    fx::shell_of(garment, body_id).is_some_and(|s| s.1 && s.0 > 1e-6)
}

fn schema_set() -> Value {
    json!({
        "type": "object",
        "properties": {
            "body": {"type": "integer", "minimum": 1, "description": "Body id as listed by body.list. Defaults to the first visible body."},
            "offset": {"type": "number", "minimum": 0, "maximum": MAX_OFFSET, "unit": "mm", "description": "Distance of the shell from the body surface, along smoothed normals. Default 10. This is the fitting ease: garments rest on the shell instead of on the skin."},
            "enabled": {"type": "boolean", "description": "Use the shell as the collider in place of the body. Default true the first time."}
        },
        "additionalProperties": false
    })
}

fn schema_get() -> Value {
    json!({
        "type": "object",
        "properties": {
            "body": {"type": "integer", "minimum": 1, "description": "Body id. Defaults to the first visible body."},
            "mesh": {"type": "boolean", "default": false, "description": "Also return the shell vertex positions and triangles (world mm)."}
        },
        "additionalProperties": false
    })
}

fn pick_body(s: &Session, a: &Value) -> Result<usize, CommandError> {
    match a.get("body").filter(|v| !v.is_null()) {
        Some(_) => {
            let id = id_arg(a, "body")?;
            s.bodies.iter().position(|b| b.id == id).ok_or_else(|| missing(format!("no body with id {id}")))
        }
        None => s.bodies.iter().position(|b| b.visible && !b.triangles.is_empty()).or_else(|| s.bodies.iter().position(|b| !b.triangles.is_empty())).ok_or_else(|| missing("there is no body; use body.load_default")),
    }
}

fn report(garment: &Garment, body: &Body, with_mesh: bool) -> Value {
    let (offset, enabled) = fx::shell_of(garment, body.id).unwrap_or((10.0, false));
    let posed = skin::posed_positions(body);
    let shell: Shell = offset_surface(&posed, &body.triangles, offset as f32);
    let (mut lo, mut hi, mut sum) = (f32::INFINITY, 0.0f32, 0.0f64);
    for d in &shell.distances {
        lo = lo.min(*d);
        hi = hi.max(*d);
        sum += f64::from(*d);
    }
    let n = shell.distances.len().max(1) as f64;
    let mut out = json!({
        "body": body.id, "offset": offset, "enabled": enabled, "vertices": posed.len(),
        "realised_min": lo, "realised_max": hi, "realised_mean": sum / n, "clamped_vertices": shell.clamped
    });
    if with_mesh {
        if let Some(o) = out.as_object_mut() {
            o.insert("positions".into(), json!(shell.positions));
            o.insert("triangles".into(), json!(body.triangles));
        }
    }
    out
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let at = pick_body(s, a)?;
    let id = s.bodies[at].id;
    let previous = fx::shell_of(&s.garment, id);
    let offset = ranged(a, "offset", 0.0, MAX_OFFSET)?;
    let enabled = a.get("enabled").filter(|v| !v.is_null()).map(|v| v.as_bool().ok_or_else(|| bad("enabled must be true or false"))).transpose()?;
    let next = (offset.or(previous.map(|p| p.0)).unwrap_or(10.0), enabled.or(previous.map(|p| p.1)).unwrap_or(true));
    let changed = previous != Some(next);
    if changed {
        let before = Box::new(s.garment.clone());
        fx::put(&mut s.garment, "fit_shell", &id.to_string(), Some(json!({"offset": next.0, "enabled": next.1})));
        let after = Box::new(s.garment.clone());
        s.record(Edit::Garment { label: "Fit shell".into(), before, after });
    }
    let mut out = report(&s.garment, &s.bodies[at], false);
    if let Some(o) = out.as_object_mut() {
        o.insert("changed".into(), json!(changed));
    }
    Ok(out)
}

fn get(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let at = pick_body(s, a)?;
    let with_mesh = a.get("mesh").and_then(Value::as_bool).unwrap_or(false);
    Ok(report(&s.garment, &s.bodies[at], with_mesh))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "fitshell.set",
            mcp: "fitshell_set",
            group: Group::Sim,
            label: "Fit shell",
            description: "Turn on a fit shell for a body: a smooth surface offset outward from the body along smoothed normals (offset in mm, reduced where the surface is concave so it cannot fold into itself) that replaces the body as the collider, so garments drape at a fitting ease. Takes effect when the simulation is next built. One undo step.",
            keys: &[],
            hints: SET,
            schema: schema_set,
            run: set,
        },
        Command {
            id: "fitshell.get",
            mcp: "fitshell_get",
            group: Group::Sim,
            label: "Fit shell state",
            description: "Report the fit shell of a body: offset, enabled, and the realised offset distances (minimum, mean, maximum and how many vertices were reduced to keep the shell free of self intersection). With mesh true, also the shell positions and triangles.",
            keys: &[],
            hints: READ,
            schema: schema_get,
            run: get,
        },
    ]
}
