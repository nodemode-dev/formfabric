use ffb_pattern::Place3D;
use serde_json::{json, Value};

use super::build::ensure;
use super::drape::{placement_signature, rigid_fit};
use super::runtime::lock;
use crate::{no_args, Command, CommandError, Edit, Group, Session, WRITE};

fn apply(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    ensure(s)?;
    let fits = {
        let c = lock(&s.sim.core);
        let Some(stepper) = c.stepper.as_ref() else { return Err(CommandError::new("not_found", "nothing has been simulated")) };
        let x = stepper.positions();
        let mut out = Vec::new();
        for sl in &c.slices {
            let range = sl.start..sl.start + sl.count;
            if range.end > x.len() {
                continue;
            }
            let base = sl.start as u32;
            let tris: Vec<[u32; 3]> = c
                .base
                .triangles
                .iter()
                .filter(|t| t[0] >= base && (t[0] as usize) < range.end)
                .map(|t| [t[0] - base, t[1] - base, t[2] - base])
                .collect();
            let positions = x[range.clone()].to_vec();
            if let Some((origin, rotation)) = rigid_fit(&c.base.rest[range], &positions, &tris) {
                out.push((sl.id, origin, rotation, positions));
            }
        }
        out
    };
    if fits.is_empty() {
        return Err(CommandError::new("not_found", "no panel has a draped shape"));
    }
    let mut edits = Vec::new();
    let mut panels = Vec::new();
    for (id, origin, rotation, positions) in fits {
        let Some(index) = s.garment.index_of(id) else { continue };
        let before = s.garment.panels[index].clone();
        let mut after = before.clone();
        let bend = before.place3d.as_ref().map_or(0.0, |p| p.bend);
        after.place3d = Some(Place3D { origin, rotation, bend });
        let sig = placement_signature(&after);
        s.sim.drape.put(id, sig, positions);
        if after != before {
            s.garment.panels[index] = after.clone();
            edits.push(Edit::Panel { label: "Apply drape".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
            panels.push(id);
        }
    }
    let changed = !edits.is_empty();
    if changed {
        s.record(Edit::Many { label: "Apply drape".into(), edits });
    }
    Ok(json!({"panels": panels, "changed": changed}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "sim.apply",
        mcp: "sim_apply",
        group: Group::Sim,
        label: "Apply drape",
        description: "Write the draped state into the document as one undo step: each panel gets the rigid placement (place3d) that best fits its draped vertices, and the draped vertex positions are kept in a cache keyed by that placement so the viewer and the next simulation start from the drape. Undo restores the earlier placement.",
        keys: &[],
        hints: WRITE,
        schema: no_args,
        run: apply,
    }]
}
