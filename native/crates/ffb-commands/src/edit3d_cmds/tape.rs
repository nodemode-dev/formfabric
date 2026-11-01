use ffb_body::skin;
use ffb_pattern::{Tack, TackEnd};
use serde_json::{json, Value};

use super::common::{bad, id_arg, missing, numbers, panel_index, ranged};
use crate::sim_cmds::current_positions;
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1},
            "point": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Panel-local point; the nearest mesh vertex is held. Give point or vertex."},
            "vertex": {"type": "integer", "minimum": 0, "description": "Vertex index in the panel mesh."},
            "body_point": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "World point to hold the cloth point to. Leave out to take the nearest point of the body surface."},
            "body": {"type": "integer", "minimum": 1, "description": "Body id for the nearest-point search. Defaults to the first visible body."},
            "length": {"type": "number", "minimum": 0, "maximum": 1000, "default": 0, "unit": "mm", "description": "Rest length of the tack."}
        },
        "required": ["panel"],
        "additionalProperties": false
    })
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "panel")?;
    let index = panel_index(s, id)?;
    let length = ranged(a, "length", 0.0, 1000.0)?.unwrap_or(0.0);
    let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
    let positions2 = built.mesh.positions.clone();
    let local = match (numbers::<2>(a, "point")?, a.get("vertex").and_then(Value::as_u64)) {
        (Some(p), None) => positions2
            .iter()
            .enumerate()
            .min_by(|x, y| ((x.1[0] - p[0]).powi(2) + (x.1[1] - p[1]).powi(2)).total_cmp(&((y.1[0] - p[0]).powi(2) + (y.1[1] - p[1]).powi(2))))
            .map(|m| m.0)
            .ok_or_else(|| missing("the panel mesh is empty"))?,
        (None, Some(v)) => {
            let v = v as usize;
            if v >= positions2.len() {
                return Err(bad(format!("the panel mesh has {} vertices", positions2.len())));
            }
            v
        }
        _ => return Err(bad("give exactly one of point and vertex")),
    };
    let anchor = match numbers::<3>(a, "body_point")? {
        Some(p) => p,
        None => {
            let wanted = match a.get("body").filter(|v| !v.is_null()) {
                Some(_) => Some(id_arg(a, "body")?),
                None => None,
            };
            let at = current_positions(s, id)?[local];
            let body = s
                .bodies
                .iter()
                .find(|b| !b.triangles.is_empty() && wanted.map_or(b.visible, |w| b.id == w))
                .ok_or_else(|| missing("there is no body to tape to; use body.load_default"))?;
            let posed = skin::posed_positions(body);
            let near = posed
                .iter()
                .min_by(|x, y| {
                    let d = |p: &[f32; 3]| (p[0] - at[0]).powi(2) + (p[1] - at[1]).powi(2) + (p[2] - at[2]).powi(2);
                    d(x).total_cmp(&d(y))
                })
                .ok_or_else(|| missing("the body has no mesh"))?;
            near.map(f64::from)
        }
    };
    let before = Box::new(s.garment.clone());
    let tack = Tack {
        id: ffb_pattern::sew::next_tack_id(&s.garment),
        a: TackEnd { panel: Some(id), point: positions2[local], body_point: None },
        b: TackEnd { panel: None, point: [0.0, 0.0], body_point: Some(anchor) },
        length,
    };
    let tack_id = tack.id;
    s.garment.tacks.push(tack);
    let after = Box::new(s.garment.clone());
    s.record(Edit::Garment { label: "Body tape".into(), before, after });
    Ok(json!({"tack": tack_id, "panel": id, "vertex": local, "body_point": anchor, "length": length}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "body.tape",
        mcp: "body_tape",
        group: Group::Sim,
        label: "Body tape",
        description: "Hold a panel point to a point of the body during simulation: a tack between the cloth vertex and a body point, one rest length, one stiffness. Without body_point the nearest body surface point is used. One undo step; remove it with tack.delete.",
        keys: &["Shift+K"],
        hints: WRITE,
        schema,
        run,
    }]
}
