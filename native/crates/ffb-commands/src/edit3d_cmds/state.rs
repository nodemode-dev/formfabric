use ffb_pattern::PanelState;
use serde_json::{json, Value};

use super::common::{bad, id_list, panel_index, ranged};
use crate::{Command, CommandError, Edit, Group, Session, SET};

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "minItems": 1, "maxItems": 256, "description": "Panel ids as listed by pattern.get."},
            "frozen": {"type": "boolean", "description": "Freeze: the panel is held as pins at its current positions and still collides."},
            "disabled": {"type": "boolean", "description": "Disable: the panel is left out of the simulation together with its seams."},
            "stiffen": {"type": "number", "minimum": 0.01, "maximum": 1000, "description": "Multiplier on stretch, shear and bending. 1 is none."},
            "hold_shape": {"type": "boolean", "description": "Hold Shape: a weak spring pulls each vertex back to the shape it has when the simulation is built."},
            "shrink_weft": {"type": "number", "minimum": 0.3, "maximum": 2, "description": "Weft shrinkage factor. 1 is none."},
            "shrink_warp": {"type": "number", "minimum": 0.3, "maximum": 2, "description": "Warp shrinkage factor. 1 is none."}
        },
        "required": ["panels"],
        "additionalProperties": false
    })
}

fn state_json(id: u32, st: &PanelState) -> Value {
    json!({
        "panel": id, "frozen": st.frozen, "disabled": st.disabled, "stiffen": st.stiffen,
        "hold_shape": st.hold_shape, "shrink_weft": st.shrink_weft, "shrink_warp": st.shrink_warp
    })
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ids = id_list(a, "panels")?.filter(|l| !l.is_empty()).ok_or_else(|| bad("panels must list at least one panel id"))?;
    let mut indices = Vec::new();
    for id in &ids {
        indices.push(panel_index(s, *id)?);
    }
    let stiffen = ranged(a, "stiffen", 0.01, 1000.0)?;
    let weft = ranged(a, "shrink_weft", 0.3, 2.0)?;
    let warp = ranged(a, "shrink_warp", 0.3, 2.0)?;
    let flag = |key: &str| a.get(key).filter(|v| !v.is_null()).map(|v| v.as_bool().ok_or_else(|| bad(format!("{key} must be true or false")))).transpose();
    let (frozen, disabled, hold) = (flag("frozen")?, flag("disabled")?, flag("hold_shape")?);
    let mut edits = Vec::new();
    for index in indices {
        let before = s.garment.panels[index].clone();
        let mut after = before.clone();
        let st = &mut after.state;
        if let Some(x) = frozen {
            st.frozen = x;
        }
        if let Some(x) = disabled {
            st.disabled = x;
        }
        if let Some(x) = hold {
            st.hold_shape = x;
        }
        if let Some(x) = stiffen {
            st.stiffen = x;
        }
        if let Some(x) = weft {
            st.shrink_weft = x;
        }
        if let Some(x) = warp {
            st.shrink_warp = x;
        }
        if after != before {
            s.garment.panels[index] = after.clone();
            edits.push(Edit::Panel { label: "Panel state".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
        }
    }
    let changed = !edits.is_empty();
    if changed {
        s.record(Edit::Many { label: "Panel state".into(), edits });
    }
    let states: Vec<Value> = ids.iter().filter_map(|id| s.garment.index_of(*id)).map(|i| &s.garment.panels[i]).map(|p| state_json(p.id, &p.state)).collect();
    Ok(json!({"changed": changed, "panels": states}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "panel.state",
        mcp: "panel_state",
        group: Group::Pattern,
        label: "Panel state",
        description: "Set the simulation state of panels: freeze, disable, stiffen (strengthen), hold shape (solidify) and shrinkage. One undo step. The simulation rebuilds with the new state and keeps the other panels where they are.",
        keys: &[],
        hints: SET,
        schema,
        run,
    }]
}
