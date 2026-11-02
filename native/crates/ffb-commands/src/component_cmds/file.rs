use std::collections::BTreeMap;

use ffb_pattern::component::{self, Component, Port, PortRef};
use ffb_pattern::sew::{end_valid, next_seam_id};
use ffb_pattern::{Garment, Panel, Seam, SeamEnd};
use serde_json::{json, Value};

use super::{bad, component_json, join_json, make_join, mutate, parse_ref};
use crate::{CommandError, Session};

const FORMAT: &str = "ffcomponent";
const MAX_PANELS: usize = 500;
const MAX_SEAMS: usize = 5000;
const MAX_PORTS: usize = 64;

pub(crate) struct ComponentFile {
    pub name: String,
    pub panels: Vec<Panel>,
    pub seams: Vec<Seam>,
    pub ports: Vec<Port>,
}

fn broken(message: impl Into<String>) -> CommandError {
    CommandError::new("open_failed", message)
}

pub(crate) fn encode_component(g: &Garment, id: u32) -> Result<Vec<u8>, CommandError> {
    let c = g.component(id).ok_or_else(|| CommandError::new("not_found", format!("no component with id {id}")))?;
    let panels: Vec<Panel> = g
        .panels
        .iter()
        .filter(|p| c.panels.contains(&p.id))
        .map(|p| {
            let mut p = p.clone();
            p.fabric = None;
            p.prints.clear();
            if p.link.as_ref().is_some_and(|l| !c.panels.contains(&l.source)) {
                p.link = None;
            }
            p
        })
        .collect();
    let seams: Vec<Seam> = g.seams.iter().filter(|s| component::internal_seams(g, c).contains(&s.id)).cloned().collect();
    let mut ports = c.ports.clone();
    for p in ports.iter_mut() {
        if !component::port_valid(g, p) {
            return Err(CommandError::new("stale_port", format!("port {} points at an edge that no longer exists", p.name)));
        }
        p.length_mm = component::port_length(g, p).unwrap_or(0.0);
    }
    let body = json!({"format": FORMAT, "schema": 1, "app_version": ffb_core::ENGINE_VERSION, "name": c.name, "panels": panels, "seams": seams, "ports": ports});
    serde_json::to_vec(&body).map_err(|e| CommandError::new("save_failed", e.to_string()))
}

pub(crate) fn decode(bytes: &[u8]) -> Result<ComponentFile, CommandError> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| broken(format!("not a component file: {e}")))?;
    if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(broken("not a component file"));
    }
    if v.get("schema").and_then(Value::as_u64).unwrap_or(0) != 1 {
        return Err(broken("this component file has a schema this engine does not read"));
    }
    let name = ffb_fabric::validate::name(v.get("name").and_then(Value::as_str).unwrap_or("Component")).map_err(broken)?;
    let get = |key: &str| v.get(key).cloned().unwrap_or(json!([]));
    let panels: Vec<Panel> = serde_json::from_value(get("panels")).map_err(|e| broken(format!("the panels are invalid: {e}")))?;
    let seams: Vec<Seam> = serde_json::from_value(get("seams")).map_err(|e| broken(format!("the seams are invalid: {e}")))?;
    let ports: Vec<Port> = serde_json::from_value(get("ports")).map_err(|e| broken(format!("the ports are invalid: {e}")))?;
    if panels.is_empty() || panels.len() > MAX_PANELS || seams.len() > MAX_SEAMS || ports.len() > MAX_PORTS {
        return Err(broken("the component holds no panels or too many panels, seams or ports"));
    }
    let tmp = Garment { panels: panels.clone(), ..Garment::default() };
    let ids: Vec<u32> = panels.iter().map(|p| p.id).collect();
    for (i, id) in ids.iter().enumerate() {
        if ids[..i].contains(id) {
            return Err(broken(format!("panel id {id} appears twice")));
        }
    }
    for seam in &seams {
        if seam.a.iter().chain(seam.b.iter()).any(|e| !ids.contains(&e.panel) || !end_valid(&tmp, e)) {
            return Err(broken("a seam points at a panel edge that is not in the file"));
        }
    }
    for port in &ports {
        if port.spans.iter().any(|e| !ids.contains(&e.panel) || !end_valid(&tmp, e)) || port.spans.is_empty() {
            return Err(broken(format!("port {} points at a panel edge that is not in the file", port.name)));
        }
        if ffb_fabric::validate::name(&port.name).is_err() || !port.tolerance_mm.is_finite() || port.tolerance_mm < 0.0 {
            return Err(broken("a port has an invalid name or tolerance"));
        }
        if ports.iter().filter(|p| p.name == port.name).count() > 1 {
            return Err(broken(format!("two ports are named {}", port.name)));
        }
    }
    Ok(ComponentFile { name, panels, seams, ports })
}

fn bounds<'a>(panels: impl Iterator<Item = &'a Panel>) -> Option<[f64; 4]> {
    let mut b: Option<[f64; 4]> = None;
    for panel in panels {
        for pt in &panel.outline {
            let (x, y) = (pt.p[0] + panel.position[0], pt.p[1] + panel.position[1]);
            b = Some(match b {
                None => [x, y, x, y],
                Some(r) => [r[0].min(x), r[1].min(y), r[2].max(x), r[3].max(y)],
            });
        }
    }
    b
}

pub(crate) fn centre(panels: &[&Panel]) -> Option<[f64; 2]> {
    bounds(panels.iter().copied()).map(|b| [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0])
}

pub(crate) fn instantiate(g: &mut Garment, file: &ComponentFile, offset: [f64; 2]) -> Result<u32, CommandError> {
    let mut next = g.next_id();
    let mut panel_map: BTreeMap<u32, u32> = BTreeMap::new();
    let mut inner_map: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let mut made: Vec<Panel> = Vec::new();
    for orig in &file.panels {
        let mut p = orig.clone();
        p.id = next;
        panel_map.insert(orig.id, next);
        next += 1;
        for inner in p.inner.iter_mut() {
            inner_map.insert((orig.id, inner.id), next);
            inner.id = next;
            next += 1;
        }
        p.position = [p.position[0] + offset[0], p.position[1] + offset[1]];
        p.fabric = None;
        p.prints.clear();
        if let Some(l) = p.link.as_mut() {
            match panel_map.get(&l.source) {
                Some(n) => l.source = *n,
                None => p.link = None,
            }
        }
        made.push(p);
    }
    let end = |e: &SeamEnd| -> SeamEnd { SeamEnd { panel: panel_map[&e.panel], inner: e.inner.map(|i| inner_map.get(&(e.panel, i)).copied().unwrap_or(i)), ..e.clone() } };
    let new_panels: Vec<u32> = made.iter().map(|p| p.id).collect();
    g.panels.extend(made);
    for seam in &file.seams {
        let id = next_seam_id(g);
        g.seams.push(Seam { id, a: seam.a.iter().map(end).collect(), b: seam.b.iter().map(end).collect(), ..seam.clone() });
    }
    let ports: Vec<Port> = file.ports.iter().map(|p| Port { name: p.name.clone(), spans: p.spans.iter().map(end).collect(), groups: p.groups.clone(), length_mm: p.length_mm, tolerance_mm: p.tolerance_mm }).collect();
    let id = component::next_component_id(g);
    let mut c = Component { id, name: file.name.clone(), panels: new_panels, ports };
    for p in c.ports.iter_mut() {
        p.length_mm = component::port_length(g, p).unwrap_or(p.length_mm);
    }
    g.components.push(c);
    Ok(id)
}

pub(crate) fn insert_from_bytes(s: &mut Session, bytes: &[u8], joins: &Value) -> Result<Value, CommandError> {
    let file = decode(bytes)?;
    let wanted: Vec<Value> = match joins {
        Value::Null => Vec::new(),
        Value::Array(l) => l.clone(),
        _ => return Err(bad("joins must be an array")),
    };
    mutate(s, "Insert Component", |g| {
        let offset = match (bounds(g.panels.iter()), bounds(file.panels.iter())) {
            (Some(have), Some(new)) => [have[2] + 80.0 - new[0], have[1] - new[1]],
            _ => [0.0, 0.0],
        };
        let id = instantiate(g, &file, offset)?;
        let mut made = Vec::new();
        for item in &wanted {
            let port = item.get("port").and_then(Value::as_str).ok_or_else(|| bad("each join needs port, the port of the inserted component"))?;
            let to = parse_ref(item.get("to").unwrap_or(&Value::Null), "to")?;
            let reversed = item.get("reversed").and_then(Value::as_bool).unwrap_or(true);
            let ease = item.get("ease").and_then(Value::as_bool).unwrap_or(false);
            let me = PortRef { component: id, port: ffb_fabric::validate::name(port).map_err(bad)? };
            made.push(join_json(&make_join(g, me, to, reversed, ease)?));
        }
        let c = g.component(id).cloned().unwrap_or_default();
        Ok(json!({"component": component_json(g, &c), "joins": made}))
    })
}
