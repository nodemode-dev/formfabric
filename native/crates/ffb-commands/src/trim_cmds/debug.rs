use ffb_pattern::sew::Binding;
use serde_json::{json, Value};

use super::sync::{session_meshes, stiffen_bands, trim_bindings};
use crate::{Command, CommandError, Group, Session, READ};

fn run(s: &mut Session, _a: &Value) -> Result<Value, CommandError> {
    let meshes = session_meshes(s);
    let bound = trim_bindings(&s.garment, &meshes);
    let bands = stiffen_bands(&s.garment, &meshes);
    let (mut pairs, mut edges) = (0, 0);
    let mut per_zipper: Vec<Value> = Vec::new();
    for z in &s.garment.zippers {
        let (mut p, mut e) = (0, 0);
        for b in bound.zippers.iter().filter(|b| matches!(b, Binding::Pair { seam, .. } | Binding::Edge { seam, .. } if *seam == z.id)) {
            match b {
                Binding::Pair { .. } => p += 1,
                Binding::Edge { .. } => e += 1,
            }
        }
        pairs += p;
        edges += e;
        per_zipper.push(json!({"zipper": z.id, "closed": z.closed, "pairs": p, "edges": e}));
    }
    let hardware: Vec<Value> = bound
        .hardware
        .iter()
        .map(|h| json!({"hardware": h.hardware, "kind": format!("{:?}", h.kind).to_lowercase(), "panel": h.at.panel, "vertices": h.at.vertices, "weights": h.at.weights, "width": h.width, "thickness": h.thickness, "off_mesh": h.off_mesh}))
        .collect();
    let fastenings: Vec<Value> = bound.fastenings.iter().map(|f| json!({"button": f.button, "buttonhole": f.buttonhole, "rest_length": f.rest_length, "stiffness": f.stiffness})).collect();
    let bands_out: Vec<Value> = bands
        .iter()
        .map(|(panel, v)| json!({"panel": panel, "triangles": v.len(), "stiffened": v.iter().filter(|x| **x > 1.0).count(), "max": v.iter().copied().fold(1.0f32, f32::max)}))
        .collect();
    let finishes: Vec<Value> = s.garment.finishes.iter().map(|f| json!({"id": f.id, "kind": f.kind, "generated": f.generated, "spans": f.spans.len()})).collect();
    Ok(json!({
        "styles": s.garment.styles.len(),
        "hardware_total": s.garment.hardware.len(),
        "hardware_bound": hardware.len(),
        "hardware": hardware,
        "fastenings": fastenings,
        "zipper_pairs": pairs,
        "zipper_edges": edges,
        "zippers": per_zipper,
        "bands": bands_out,
        "stiffened_triangles": bands.values().map(|v| v.iter().filter(|x| **x > 1.0).count()).sum::<usize>(),
        "finishes": finishes,
        "stitches": s.garment.stitches.len(),
        "ripped": s.garment.ripped
    }))
}

fn schema() -> Value {
    crate::no_args()
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "trim.debug",
        mcp: "trim_debug",
        group: Group::Sew,
        label: "Trim Debug",
        description: "Read only: what the simulation would read from the trims right now. Hardware attached to mesh triangles with barycentric weights, button and buttonhole fastenings, zipper binding counts and the stiffened triangle count per panel.",
        keys: &[],
        hints: READ,
        schema,
        run,
    }]
}
