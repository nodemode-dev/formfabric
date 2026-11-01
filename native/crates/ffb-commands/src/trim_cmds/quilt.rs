use ffb_pattern::sew::next_seam_id;
use ffb_pattern::{path, InnerRole, Panel, Place3D, Seam, SeamEnd, SeamFold};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, WRITE};

fn schema() -> Value {
    obj(
        json!({
            "panel": ident("Panel to quilt; it keeps the outer layer."),
            "lines": {"type": "array", "minItems": 1, "maxItems": 64, "items": {"type": "integer", "minimum": 1}, "description": "Inner line ids of the panel that become quilting lines. Shape and reference lines turn into flat fold lines."},
            "pressure": num("Pressure of the outer layer in pascals; the lining gets the opposite value so the cells puff.", -5000.0, 5000.0, Some("Pa")),
            "gap": num("Distance of the lining behind the outer layer in the 3D placement.", 0.0, 100.0, Some("mm")),
            "edge": {"type": "boolean", "default": true, "description": "Also sew the two outlines together so the cells hold the filling."}
        }),
        &["panel", "lines"],
    )
}

fn along(panel: &Panel, other: &Panel, line: u32, mapped: u32) -> Option<(Vec<SeamEnd>, Vec<SeamEnd>)> {
    let (pts, closed) = panel.path(Some(line))?;
    let n = path::seg_count(pts.len(), closed);
    let end = |p: &Panel, inner: Option<u32>, k: usize| SeamEnd { panel: p.id, inner, segment: k, t0: 0.0, t1: 1.0 };
    Some(((0..n).map(|k| end(panel, Some(line), k)).collect(), (0..n).map(|k| end(other, Some(mapped), k)).collect()))
}

fn make(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel_id = required_id(a, "panel")?;
    let lines = id_list(a, "lines")?;
    let source = panel_of(&s.garment, panel_id)?;
    if source.link.is_some() {
        return Err(bad("a linked panel cannot be quilted; quilt its source"));
    }
    for line in &lines {
        let inner = source.inner.iter().find(|i| i.id == *line).ok_or_else(|| missing(format!("panel {panel_id} has no inner line {line}")))?;
        if inner.role == InnerRole::Dart {
            return Err(bad(format!("inner line {line} is a dart")));
        }
    }
    let pressure = number_or(a, "pressure", if source.pressure.abs() > 0.0 { source.pressure } else { 100.0 });
    let gap = number_or(a, "gap", 2.0);
    let base = s.garment.next_id();
    let mut lining = Panel {
        id: base,
        name: format!("{} lining", source.name),
        outline: source.outline.clone(),
        position: source.position,
        rotation: source.rotation,
        mirror: source.mirror,
        grain: source.grain,
        particle_distance: source.particle_distance,
        layer: source.layer + 1,
        seam_allowance: source.seam_allowance.clone(),
        grain_line: source.grain_line,
        fabric: source.fabric,
        pressure: -pressure,
        tags: vec!["lining".into(), "quilt".into()],
        ..Panel::default()
    };
    if let Some(p3) = &source.place3d {
        let q = p3.rotation;
        let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
        let q = if n < 1e-12 { [0.0, 0.0, 0.0, 1.0] } else { q.map(|x| x / n) };
        let back = ffb_mesh::layout::rotate(q, [0.0, 0.0, -gap]);
        lining.place3d = Some(Place3D { origin: [p3.origin[0] + back[0], p3.origin[1] + back[1], p3.origin[2] + back[2]], rotation: p3.rotation, bend: p3.bend, pose: None });
    }
    let mut mapped: Vec<(u32, u32)> = Vec::new();
    for (k, line) in lines.iter().enumerate() {
        let mut copy = source.inner.iter().find(|i| i.id == *line).cloned().ok_or_else(|| missing("inner line vanished"))?;
        copy.id = base + 1 + k as u32;
        copy.role = InnerRole::Fold;
        copy.fold_angle = 180.0;
        copy.fold_strength = 0.0;
        mapped.push((*line, copy.id));
        lining.inner.push(copy);
    }
    let before = s.garment.clone();
    let mut seams: Vec<Seam> = Vec::new();
    let mut next = next_seam_id(&s.garment);
    let outer_index = s.garment.index_of(panel_id).ok_or_else(|| missing("panel vanished"))?;
    for (line, copy) in &mapped {
        if let Some((ea, eb)) = along(&s.garment.panels[outer_index], &lining, *line, *copy) {
            seams.push(Seam { id: next, a: ea, b: eb, reversed: false, fold: SeamFold::Flat, ..Seam::default() });
            next += 1;
        }
    }
    if flag(a, "edge").unwrap_or(true) {
        let n = path::seg_count(lining.outline.len(), true);
        let ring = |p: &Panel| (0..n).map(|k| SeamEnd { panel: p.id, inner: None, segment: k, t0: 0.0, t1: 1.0 }).collect::<Vec<_>>();
        seams.push(Seam { id: next, a: ring(&s.garment.panels[outer_index]), b: ring(&lining), reversed: false, fold: SeamFold::Flat, ..Seam::default() });
    }
    let seam_ids: Vec<u32> = seams.iter().map(|x| x.id).collect();
    let lining_id = lining.id;
    let outer = &mut s.garment.panels[outer_index];
    for inner in outer.inner.iter_mut().filter(|i| lines.contains(&i.id)) {
        inner.role = InnerRole::Fold;
        inner.fold_angle = 180.0;
        inner.fold_strength = 0.0;
    }
    outer.pressure = pressure;
    s.garment.panels.push(lining);
    s.garment.seams.extend(seams);
    record(s, "Make Quilt", before);
    Ok(json!({"panel": panel_id, "lining": lining_id, "lines": mapped.iter().map(|(a, b)| json!({"outer": a, "lining": b})).collect::<Vec<_>>(), "seams": seam_ids, "pressure": pressure}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "quilt.make",
        mcp: "quilt_make",
        group: Group::Sew,
        label: "Make Quilt",
        description: "Quilt a panel: clone it as a lining layer behind it with the opposite pressure, copy the chosen inner lines onto the lining as flat fold lines and sew each outer line to its copy; the outlines are sewn together too unless edge is false. One undo step.",
        keys: &[],
        hints: WRITE,
        schema,
        run: make,
    }]
}
