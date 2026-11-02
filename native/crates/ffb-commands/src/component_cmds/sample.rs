use ffb_pattern::component::{self, Component, Join, Port, PortRef};
use ffb_pattern::sew::side_length;
use ffb_pattern::{Garment, Panel, Point, SeamEnd};
use serde_json::{json, Value};

use super::{bad, component_json};
use crate::CommandError;

const SAMPLE_TOLERANCE_MM: f64 = 10.0;
const BINDING_WIDTH_MM: f64 = 30.0;

fn panel_named(g: &Garment, name: &str) -> Result<u32, CommandError> {
    g.panels.iter().find(|p| p.name == name).map(|p| p.id).ok_or_else(|| CommandError::new("not_found", format!("no panel named {name}; load the sample with sample.load tshirt first")))
}

fn side_in(ends: &[SeamEnd], panels: &[u32]) -> bool {
    !ends.is_empty() && ends.iter().all(|e| panels.contains(&e.panel))
}

pub(super) fn tshirt(g: &mut Garment) -> Result<Value, CommandError> {
    if !g.components.is_empty() {
        return Err(bad("the garment already has components; remove them first"));
    }
    let (front, back) = (panel_named(g, "T-Shirt Front")?, panel_named(g, "T-Shirt Back")?);
    let sleeves = [(panel_named(g, "T-Shirt Sleeve L")?, "Sleeve L", "sleeve-l"), (panel_named(g, "T-Shirt Sleeve R")?, "Sleeve R", "sleeve-r")];
    let torso_id = component::next_component_id(g);
    let mut torso = Component { id: torso_id, name: "Torso".into(), panels: vec![front, back], ports: Vec::new() };
    let mut parts: Vec<Component> = Vec::new();
    let mut joins: Vec<Join> = Vec::new();
    for (k, (panel, name, slug)) in sleeves.iter().enumerate() {
        let id = torso_id + 1 + k as u32;
        let mut sleeve = Component { id, name: (*name).into(), panels: vec![*panel], ports: Vec::new() };
        let (mut mine, mut theirs, mut seams, mut reversed) = (Vec::new(), Vec::new(), Vec::new(), true);
        let (mut mine_runs, mut their_runs): (Vec<usize>, Vec<usize>) = (Vec::new(), Vec::new());
        for seam in &g.seams {
            let (sleeve_side, torso_side) = if side_in(&seam.a, &sleeve.panels) && side_in(&seam.b, &torso.panels) {
                (&seam.a, &seam.b)
            } else if side_in(&seam.b, &sleeve.panels) && side_in(&seam.a, &torso.panels) {
                (&seam.b, &seam.a)
            } else {
                continue;
            };
            if seams.is_empty() {
                reversed = seam.reversed;
            }
            mine.extend(sleeve_side.iter().cloned());
            theirs.extend(torso_side.iter().cloned());
            mine_runs.push(sleeve_side.len());
            their_runs.push(torso_side.len());
            seams.push(seam.id);
        }
        if seams.is_empty() {
            return Err(bad(format!("{name} is not sewn to the torso")));
        }
        let torso_port = format!("armhole-{slug}");
        sleeve.ports.push(Port { name: "armhole".into(), length_mm: side_length(g, &mine).unwrap_or(0.0), spans: mine, groups: mine_runs, tolerance_mm: SAMPLE_TOLERANCE_MM });
        torso.ports.push(Port { name: torso_port.clone(), length_mm: side_length(g, &theirs).unwrap_or(0.0), spans: theirs, groups: their_runs, tolerance_mm: SAMPLE_TOLERANCE_MM });
        joins.push(Join { id: k as u32 + 1, a: PortRef { component: id, port: "armhole".into() }, b: PortRef { component: torso_id, port: torso_port }, reversed, seams });
        parts.push(sleeve);
    }
    g.components.push(torso);
    g.components.extend(parts);
    g.joins.extend(joins);
    Ok(json!({"components": g.components.iter().map(|c| component_json(g, c)).collect::<Vec<_>>(), "joins": g.joins.len()}))
}

fn strip(id: u32, name: String, length: f64, width: f64, pd: f64, origin: [f64; 2]) -> Panel {
    let corner = |x: f64, y: f64| Point { p: [x, y], ..Point::default() };
    Panel { id, name, outline: vec![corner(0.0, 0.0), corner(length, 0.0), corner(length, width), corner(0.0, width)], position: origin, particle_distance: pd, ..Panel::default() }
}

pub(super) fn sleeveless(g: &mut Garment, like: u32, port: &str) -> Result<Value, CommandError> {
    let reference = g.component(like).and_then(|c| c.ports.iter().find(|p| p.name == port)).cloned().ok_or_else(|| CommandError::new("not_found", format!("component {like} has no port {port}")))?;
    let lengths: Vec<f64> = reference.runs().iter().map(|run| side_length(g, run).ok_or_else(|| CommandError::new("stale_port", format!("port {port} points at an edge that no longer exists")))).collect::<Result<_, _>>()?;
    let pd = g.panels.iter().find(|p| reference.spans.first().is_some_and(|e| e.panel == p.id)).map_or(20.0, |p| p.particle_distance);
    let right = g.panels.iter().flat_map(|p| p.outline.iter().map(move |pt| pt.p[0] + p.position[0])).fold(0.0f64, f64::max);
    let first = g.next_id();
    let mut ids = Vec::new();
    let mut spans = Vec::new();
    for (i, len) in lengths.iter().enumerate() {
        let id = first + i as u32;
        let y = i as f64 * (BINDING_WIDTH_MM + 40.0);
        g.panels.push(strip(id, format!("Armhole Binding {}", i + 1), *len, BINDING_WIDTH_MM, pd, [right + 120.0, y]));
        ids.push(id);
        spans.push(SeamEnd { panel: id, inner: None, segment: 0, t0: 0.0, t1: 1.0 });
    }
    let id = component::next_component_id(g);
    let ports = vec![Port { name: port.into(), length_mm: lengths.iter().sum(), groups: vec![1; spans.len()], spans, tolerance_mm: reference.tolerance_mm }];
    g.components.push(Component { id, name: "Sleeveless Armhole".into(), panels: ids, ports });
    let c = g.component(id).cloned().unwrap_or_default();
    Ok(json!({"component": component_json(g, &c)}))
}
