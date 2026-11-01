use ffb_pattern::{Panel, Place3D};
use serde_json::{json, Value};

use super::build::ensure;
use super::drape::rigid_fit;
use super::runtime::lock;
use crate::{no_args, Command, CommandError, Edit, Group, Session, WRITE};

pub(crate) fn posed_panels(s: &mut Session) -> Result<Vec<(usize, Panel, Panel)>, CommandError> {
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
    let mut out = Vec::new();
    for (id, origin, rotation, positions) in fits {
        let Some(index) = s.garment.index_of(id) else { continue };
        if !origin.iter().chain(rotation.iter()).all(|x| x.is_finite()) || !positions.iter().flatten().all(|x| x.is_finite()) {
            return Err(CommandError::new("invalid_geometry", format!("panel {id} has a non-finite draped shape; reset the simulation")));
        }
        let before = s.garment.panels[index].clone();
        let mut after = before.clone();
        after.place3d = Some(Place3D { origin, rotation, bend: 0.0, pose: Some(positions) });
        if after != before {
            out.push((index, before, after));
        }
    }
    Ok(out)
}

fn apply(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    let mut edits = Vec::new();
    let mut panels = Vec::new();
    for (index, before, after) in posed_panels(s)? {
        panels.push(after.id);
        s.garment.panels[index] = after.clone();
        edits.push(Edit::Panel { label: "Apply drape".into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) });
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
        description: "Write the draped state into the document as one undo step: each panel gets the rigid placement (place3d) that best fits its draped vertices, and the draped vertex positions are saved in place3d.pose (kept by save and open). The viewer and the next simulation or reset start from the pose while the mesh vertex count matches. Undo restores the earlier placement.",
        keys: &[],
        hints: WRITE,
        schema: no_args,
        run: apply,
    }]
}
