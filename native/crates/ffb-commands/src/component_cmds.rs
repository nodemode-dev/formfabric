mod file;
mod sample;

use ffb_pattern::component::{self, Component, Join, Port, PortRef, DEFAULT_TOLERANCE_MM};
use ffb_pattern::sew::{self, end_valid, next_seam_id, side_length};
use ffb_pattern::{Garment, Panel, Seam, SeamEnd};
use serde_json::{json, Value};

use crate::{library_cmds, Command, CommandError, Edit, Group, Session, DESTROY, READ, WRITE};

pub(crate) use file::{encode_component, insert_from_bytes};

const MAX_PANELS: usize = 500;
const MAX_PORTS: usize = 64;

pub(crate) fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub(crate) fn mutate<F>(s: &mut Session, label: &str, f: F) -> Result<Value, CommandError>
where
    F: FnOnce(&mut Garment) -> Result<Value, CommandError>,
{
    let before = s.garment.clone();
    let mut after = before.clone();
    let mut out = f(&mut after)?;
    let changed = after != before;
    if changed {
        s.garment = after.clone();
        s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
    }
    out["changed"] = json!(changed);
    Ok(out)
}

pub(crate) fn after_change(s: &mut Session, command: &str) {
    if command.starts_with("history.") || command.starts_with("file.") || command.starts_with("snapshot.") || (s.garment.components.is_empty() && s.garment.joins.is_empty()) {
        return;
    }
    let before = (s.garment.components.clone(), s.garment.joins.clone());
    component::prune(&mut s.garment);
    if (s.garment.components.clone(), s.garment.joins.clone()) == before {
        return;
    }
    let after = s.garment.clone();
    let mut old = after.clone();
    old.components = before.0;
    old.joins = before.1;
    s.history.amend_last(Edit::Garment { label: "Component Update".into(), before: Box::new(old), after: Box::new(after) });
}

fn id_of(v: &Value, key: &str) -> Result<u32, CommandError> {
    v.get(key).and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad(format!("{key} must be a positive integer id")))
}

fn text_of(v: &Value, key: &str) -> Result<String, CommandError> {
    let t = v.get(key).and_then(Value::as_str).ok_or_else(|| bad(format!("{key} must be a string")))?;
    ffb_fabric::validate::name(t).map_err(bad)
}

pub(crate) fn parse_ref(v: &Value, what: &str) -> Result<PortRef, CommandError> {
    match v {
        Value::Object(_) => Ok(PortRef { component: id_of(v, "component")?, port: text_of(v, "port")? }),
        Value::String(t) => {
            let (c, p) = t.split_once(':').ok_or_else(|| bad(format!("{what} must be {{component, port}} or \"component:port\"")))?;
            let component = c.trim().parse::<u32>().ok().filter(|x| *x >= 1).ok_or_else(|| bad(format!("{what} has no component id")))?;
            Ok(PortRef { component, port: ffb_fabric::validate::name(p).map_err(bad)? })
        }
        _ => Err(bad(format!("{what} must be {{component, port}} or \"component:port\""))),
    }
}

fn parse_spans(g: &Garment, v: &Value, panels: &[u32], port: &str) -> Result<Vec<SeamEnd>, CommandError> {
    let list = v.as_array().filter(|l| !l.is_empty() && l.len() <= 256).ok_or_else(|| bad(format!("port {port} needs spans: a list of {{panel, segment, t0, t1}}")))?;
    let mut out = Vec::new();
    for item in list {
        let panel = id_of(item, "panel")?;
        if !panels.contains(&panel) {
            return Err(bad(format!("port {port} uses panel {panel}, which is not one of the component's panels")));
        }
        let segment = item.get("segment").and_then(Value::as_u64).ok_or_else(|| bad(format!("port {port}: segment must be a whole number")))? as usize;
        let num = |key: &str, default: f64| -> Result<f64, CommandError> {
            match item.get(key) {
                None | Some(Value::Null) => Ok(default),
                Some(x) => x.as_f64().filter(|x| (0.0..=1.0).contains(x)).ok_or_else(|| bad(format!("port {port}: {key} must be between 0 and 1"))),
            }
        };
        let inner = match item.get("inner") {
            None | Some(Value::Null) => None,
            Some(_) => Some(id_of(item, "inner")?),
        };
        let end = SeamEnd { panel, inner, segment, t0: num("t0", 0.0)?, t1: num("t1", 1.0)? };
        if !end_valid(g, &end) {
            return Err(bad(format!("port {port}: panel {panel} has no segment {segment}")));
        }
        out.push(end);
    }
    Ok(out)
}

fn tolerance(v: &Value) -> Result<f64, CommandError> {
    match v.get("tolerance") {
        None | Some(Value::Null) => Ok(DEFAULT_TOLERANCE_MM),
        Some(x) => x.as_f64().filter(|t| t.is_finite() && (0.0..=1000.0).contains(t)).ok_or_else(|| bad("tolerance must be a length in mm between 0 and 1000")),
    }
}

fn component_json(g: &Garment, c: &Component) -> Value {
    let ports: Vec<Value> = c
        .ports
        .iter()
        .map(|p| {
            let me = PortRef { component: c.id, port: p.name.clone() };
            json!({
                "name": p.name,
                "spans": p.spans,
                "groups": p.groups,
                "length_mm": component::port_length(g, p),
                "stored_length_mm": p.length_mm,
                "tolerance_mm": p.tolerance_mm,
                "valid": component::port_valid(g, p),
                "join": g.join_of_port(&me).map(|j| j.id)
            })
        })
        .collect();
    json!({"id": c.id, "name": c.name, "panels": c.panels, "ports": ports, "seams": component::internal_seams(g, c)})
}

fn join_json(j: &Join) -> Value {
    json!({"id": j.id, "a": j.a, "b": j.b, "reversed": j.reversed, "seams": j.seams})
}

fn create_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 1, "maxLength": 64, "description": "Component name, for example Sleeve L."},
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "minItems": 1, "description": "Panel ids the author groups into the component. A panel belongs to at most one component."},
            "ports": {"type": "array", "items": {"type": "object"}, "description": "Named edge spans of the component's own panels: [{name, spans: [{panel, segment, t0, t1}], groups?, tolerance}]. tolerance is a length in mm (default 5); t0 and t1 default to the whole segment. groups splits the spans into runs, for example [6, 6] for a front and a back edge chain; a join sews run k of one port to run k of the other when both have the same number of runs."}
        },
        "required": ["name", "panels"],
        "additionalProperties": false
    })
}

fn create_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = text_of(a, "name")?;
    let mut panels: Vec<u32> = Vec::new();
    for v in a.get("panels").and_then(Value::as_array).ok_or_else(|| bad("panels must be an array of panel ids"))? {
        let id = v.as_u64().and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad("panels holds a value that is not a panel id"))?;
        if !panels.contains(&id) {
            panels.push(id);
        }
    }
    if panels.is_empty() || panels.len() > MAX_PANELS {
        return Err(bad(format!("a component groups 1 to {MAX_PANELS} panels")));
    }
    let ports_in = a.get("ports").and_then(Value::as_array).cloned().unwrap_or_default();
    if ports_in.len() > MAX_PORTS {
        return Err(bad(format!("a component has at most {MAX_PORTS} ports")));
    }
    mutate(s, "New Component", |g| {
        for p in &panels {
            if g.index_of(*p).is_none() {
                return Err(missing(format!("no panel with id {p}")));
            }
            if let Some(c) = g.component_of_panel(*p) {
                return Err(bad(format!("panel {p} already belongs to component {} ({})", c.id, c.name)));
            }
        }
        let mut ports: Vec<Port> = Vec::new();
        for item in &ports_in {
            let pname = text_of(item, "name")?;
            if ports.iter().any(|p| p.name == pname) {
                return Err(bad(format!("two ports are named {pname}")));
            }
            let spans = parse_spans(g, item.get("spans").unwrap_or(&Value::Null), &panels, &pname)?;
            let length_mm = side_length(g, &spans).unwrap_or(0.0);
            let groups: Vec<usize> = match item.get("groups") {
                None | Some(Value::Null) => Vec::new(),
                Some(v) => v.as_array().map(|l| l.iter().filter_map(|x| x.as_u64().map(|n| n as usize)).collect::<Vec<_>>()).filter(|l| l.iter().sum::<usize>() == spans.len() && !l.contains(&0)).ok_or_else(|| bad(format!("port {pname}: groups must be counts of spans that add up to the number of spans")))?,
            };
            ports.push(Port { name: pname, spans, groups, length_mm, tolerance_mm: tolerance(item)? });
        }
        let id = component::next_component_id(g);
        let c = Component { id, name, panels: panels.clone(), ports };
        g.components.push(c.clone());
        Ok(json!({"component": component_json(g, &c)}))
    })
}

fn list_run(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let g = &s.garment;
    Ok(json!({"components": g.components.iter().map(|c| component_json(g, c)).collect::<Vec<_>>(), "joins": g.joins.iter().map(join_json).collect::<Vec<_>>()}))
}

fn save_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "minimum": 1, "description": "Component id."},
            "path": {"type": "string", "description": "Absolute .ffcomp path inside the component folder of a library folder. Leave out to save into the first library folder under the component's name."},
            "name": {"type": "string", "maxLength": 64, "description": "Library name. Defaults to the component name."},
            "tags": {"type": "array", "items": {"type": "string"}, "maxItems": 16},
            "thumbnail": {"type": "string", "description": "Optional PNG as base64, at most 1 MiB."}
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn save_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a, "id")?;
    let c = s.garment.component(id).ok_or_else(|| missing(format!("no component with id {id}")))?;
    let name = match a.get("name") {
        Some(_) => text_of(a, "name")?,
        None => c.name.clone(),
    };
    let bytes = encode_component(&s.garment, id)?;
    let thumb = match a.get("thumbnail").and_then(Value::as_str) {
        Some(t) => Some(crate::b64::png_from_base64(t).map_err(bad)?),
        None => None,
    };
    let tags = match a.get("tags") {
        Some(_) => {
            let list: Vec<String> = a["tags"].as_array().ok_or_else(|| bad("tags must be an array of strings"))?.iter().filter_map(|t| t.as_str().map(|x| x.trim().to_string())).filter(|t| !t.is_empty() && t.chars().count() <= 40 && !t.chars().any(char::is_control)).collect();
            list
        }
        None => Vec::new(),
    };
    let item = library_cmds::write_component(&name, tags, &bytes, thumb, a.get("path").and_then(Value::as_str))?;
    Ok(json!({"item": item, "bytes": bytes.len()}))
}

fn insert_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "from": {"type": "string", "minLength": 1, "description": "Library component id (component/sleeve-l) or an absolute .ffcomp path inside a library folder."},
            "joins": {"type": "array", "items": {"type": "object"}, "description": "Joins to make now: [{port, to: {component, port}, reversed, ease}], where port is a port of the inserted component and to a port of a component in the project."}
        },
        "required": ["from"],
        "additionalProperties": false
    })
}

fn insert_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let reference = a.get("from").and_then(Value::as_str).ok_or_else(|| bad("from must be a library id or path"))?;
    let bytes = library_cmds::read_component(reference)?;
    insert_from_bytes(s, &bytes, a.get("joins").unwrap_or(&Value::Null))
}

pub(crate) fn make_join(g: &mut Garment, a: PortRef, b: PortRef, reversed: bool, ease: bool) -> Result<Join, CommandError> {
    if a.component == b.component {
        return Err(bad("a join connects two different components"));
    }
    let pa = g.port(&a).cloned().ok_or_else(|| missing(format!("component {} has no port {}", a.component, a.port)))?;
    let pb = g.port(&b).cloned().ok_or_else(|| missing(format!("component {} has no port {}", b.component, b.port)))?;
    for r in [&a, &b] {
        if let Some(j) = g.join_of_port(r) {
            return Err(bad(format!("port {} of component {} is already joined (join {}); remove that join or component first", r.port, r.component, j.id)));
        }
    }
    if !component::port_valid(g, &pa) || !component::port_valid(g, &pb) {
        return Err(CommandError::new("stale_port", "a port points at an edge that no longer exists; recreate the component's ports"));
    }
    let la = component::port_length(g, &pa).unwrap_or(0.0);
    let lb = component::port_length(g, &pb).unwrap_or(0.0);
    let tol = pa.tolerance_mm.max(pb.tolerance_mm);
    let delta = (la - lb).abs();
    let eased = delta > tol;
    if eased && !ease {
        return Err(CommandError::new(
            "port_mismatch",
            format!("port {} is {la:.1} mm and port {} is {lb:.1} mm: {delta:.1} mm apart, more than the {tol:.1} mm tolerance; pass ease true to sew them with a seam ratio, or choose another component", a.port, b.port),
        ));
    }
    let (runs_a, runs_b) = (pa.runs(), pb.runs());
    let pairs: Vec<(Vec<SeamEnd>, Vec<SeamEnd>)> = if runs_a.len() == runs_b.len() {
        runs_a.iter().zip(runs_b.iter()).map(|(x, y)| (x.to_vec(), y.to_vec())).collect()
    } else {
        vec![(pa.spans.clone(), pb.spans.clone())]
    };
    let mut ids = Vec::new();
    for (ea, eb) in pairs {
        let ratio = if eased {
            let (x, y) = (side_length(g, &ea).unwrap_or(0.0), side_length(g, &eb).unwrap_or(0.0));
            if x > 1e-9 { (y / x).clamp(0.05, 20.0) } else { 1.0 }
        } else {
            1.0
        };
        let id = next_seam_id(g);
        g.seams.push(Seam { id, a: ea, b: eb, reversed, ratio, ..Seam::default() });
        ids.push(id);
    }
    let join = Join { id: component::next_join_id(g), a, b, reversed, seams: ids };
    g.joins.push(join.clone());
    Ok(join)
}

fn join_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "a_port": {"type": ["object", "string"], "description": "{component, port} or \"component:port\"."},
            "b_port": {"type": ["object", "string"], "description": "{component, port} or \"component:port\"."},
            "reversed": {"type": "boolean", "default": true, "description": "Pair the start of side a with the end of side b (right for two counter-clockwise outlines meeting along an edge)."},
            "ease": {"type": "boolean", "default": false, "description": "Allow ports whose lengths differ by more than the tolerance; the seam gets ratio = b length / a length."}
        },
        "required": ["a_port", "b_port"],
        "additionalProperties": false
    })
}

fn join_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let pa = parse_ref(a.get("a_port").unwrap_or(&Value::Null), "a_port")?;
    let pb = parse_ref(a.get("b_port").unwrap_or(&Value::Null), "b_port")?;
    let reversed = a.get("reversed").and_then(Value::as_bool).unwrap_or(true);
    let ease = a.get("ease").and_then(Value::as_bool).unwrap_or(false);
    mutate(s, "Join Components", |g| {
        let j = make_join(g, pa, pb, reversed, ease)?;
        Ok(json!({"join": join_json(&j)}))
    })
}

fn remove_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "minimum": 1, "description": "Component id."},
            "delete_panels": {"type": "boolean", "default": false, "description": "Also delete the component's panels and the seams on them. Without it only the grouping and its joins are removed and the panels and seams stay."}
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn drop_component(g: &mut Garment, id: u32, with_panels: bool) -> Result<(Component, Vec<Join>, usize), CommandError> {
    let index = g.components.iter().position(|c| c.id == id).ok_or_else(|| missing(format!("no component with id {id}")))?;
    let old = g.components.remove(index);
    let joins: Vec<Join> = g.joins.iter().filter(|j| j.a.component == id || j.b.component == id).cloned().collect();
    g.joins.retain(|j| j.a.component != id && j.b.component != id);
    if with_panels {
        let seams: Vec<u32> = joins.iter().flat_map(|j| j.seams.iter().copied()).collect();
        g.seams.retain(|x| !seams.contains(&x.id));
        g.panels.retain(|p| !old.panels.contains(&p.id));
        g.pins.retain(|p| !old.panels.contains(&p.panel));
        g.style_lines.retain(|l| !old.panels.contains(&l.panel));
        sew::prune(g);
        ffb_pattern::trim::prune(g);
    }
    Ok((old, joins, index))
}

fn remove_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a, "id")?;
    let with_panels = a.get("delete_panels").and_then(Value::as_bool).unwrap_or(false);
    mutate(s, "Remove Component", |g| {
        let (old, joins, _) = drop_component(g, id, with_panels)?;
        Ok(json!({"removed": id, "name": old.name, "joins_removed": joins.iter().map(|j| j.id).collect::<Vec<_>>(), "panels_deleted": with_panels}))
    })
}

struct Report {
    value: Value,
    compatible: bool,
    missing: bool,
}

fn compare(g: &Garment, old: &Component, newer: &file::ComponentFile) -> Report {
    let joined: Vec<&str> = g.joins.iter().flat_map(|j| [&j.a, &j.b]).filter(|r| r.component == old.id).map(|r| r.port.as_str()).collect();
    let tmp = Garment { panels: newer.panels.clone(), ..Garment::default() };
    let mut rows = Vec::new();
    let mut compatible = true;
    let mut missing_port = false;
    for port in &old.ports {
        let required = joined.contains(&port.name.as_str());
        let expected = component::port_length(g, port).unwrap_or(port.length_mm);
        let found = newer.ports.iter().find(|p| p.name == port.name);
        let actual = found.and_then(|p| component::port_length(&tmp, p));
        let tol = port.tolerance_mm.max(found.map_or(0.0, |p| p.tolerance_mm));
        let delta = actual.map(|x| x - expected);
        let ok = found.is_some() && found.is_some_and(|p| component::port_valid(&tmp, p)) && delta.is_some_and(|d| d.abs() <= tol);
        if required && !ok {
            compatible = false;
        }
        if required && found.is_none() {
            missing_port = true;
        }
        rows.push(json!({"port": port.name, "joined": required, "found": found.is_some(), "expected_mm": expected, "actual_mm": actual, "delta_mm": delta, "tolerance_mm": tol, "ok": ok}));
    }
    let extra: Vec<&str> = newer.ports.iter().filter(|p| !old.ports.iter().any(|o| o.name == p.name)).map(|p| p.name.as_str()).collect();
    Report { value: json!({"compatible": compatible, "ports": rows, "extra_ports": extra, "basis": "port names and lengths only"}), compatible, missing: missing_port }
}

fn check_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "minimum": 1, "description": "Project component that would be replaced."},
            "with": {"type": "string", "minLength": 1, "description": "Library component id or an absolute .ffcomp path inside a library folder."}
        },
        "required": ["id", "with"],
        "additionalProperties": false
    })
}

fn check_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a, "id")?;
    let old = s.garment.component(id).cloned().ok_or_else(|| missing(format!("no component with id {id}")))?;
    let newer = file::decode(&library_cmds::read_component(a.get("with").and_then(Value::as_str).ok_or_else(|| bad("with must be a library id or path"))?)?)?;
    let mut report = compare(&s.garment, &old, &newer).value;
    report["id"] = json!(id);
    report["with"] = json!(newer.name);
    Ok(report)
}

fn swap_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "integer", "minimum": 1, "description": "Project component to replace."},
            "with": {"type": "string", "minLength": 1, "description": "Library component id or an absolute .ffcomp path inside a library folder."},
            "ease": {"type": "boolean", "default": false, "description": "Accept ports whose length differs by more than the tolerance and sew them with a seam ratio. A missing port name always refuses the swap. Panels are never resized."}
        },
        "required": ["id", "with"],
        "additionalProperties": false
    })
}

fn swap_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_of(a, "id")?;
    let ease = a.get("ease").and_then(Value::as_bool).unwrap_or(false);
    let newer = file::decode(&library_cmds::read_component(a.get("with").and_then(Value::as_str).ok_or_else(|| bad("with must be a library id or path"))?)?)?;
    mutate(s, "Swap Component", |g| {
        let old = g.component(id).cloned().ok_or_else(|| missing(format!("no component with id {id}")))?;
        let report = compare(g, &old, &newer);
        if report.missing || (!report.compatible && !ease) {
            return Err(CommandError::new("port_mismatch", format!("{} cannot replace {}: {}", newer.name, old.name, report.value)));
        }
        let offset = {
            let olds: Vec<&Panel> = g.panels.iter().filter(|p| old.panels.contains(&p.id)).collect();
            let news: Vec<&Panel> = newer.panels.iter().collect();
            match (file::centre(&olds), file::centre(&news)) {
                (Some(o), Some(n)) => [o[0] - n[0], o[1] - n[1]],
                _ => [0.0, 0.0],
            }
        };
        let (_, old_joins, index) = drop_component(g, id, true)?;
        let new_id = file::instantiate(g, &newer, offset)?;
        let at = g.components.len() - 1;
        let moved = g.components.remove(at);
        g.components.insert(index.min(g.components.len()), moved);
        let mut made = Vec::new();
        for j in old_joins {
            let (mine_is_a, peer) = if j.a.component == id { (true, j.b.clone()) } else { (false, j.a.clone()) };
            let mine_port = if mine_is_a { &j.a.port } else { &j.b.port };
            let mine = PortRef { component: new_id, port: mine_port.clone() };
            let (x, y) = if mine_is_a { (mine, peer) } else { (peer, mine) };
            made.push(join_json(&make_join(g, x, y, j.reversed, ease)?));
        }
        let c = g.component(new_id).cloned().unwrap_or_default();
        Ok(json!({"replaced": id, "component": component_json(g, &c), "joins": made, "check": report.value}))
    })
}

fn sample_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "enum": ["tshirt", "sleeveless"], "description": "tshirt groups the drafted sample T-shirt (sample.load tshirt) into Torso, Sleeve L and Sleeve R with an armhole port and a join each. sleeveless adds a no-sleeve variant: two armhole binding strips as one component with an armhole port the lengths of another component's port."},
            "like": {"type": "integer", "minimum": 1, "description": "sleeveless only: the component whose port sets the binding lengths (a sleeve)."},
            "port": {"type": "string", "default": "armhole", "description": "sleeveless only: the port name to match."}
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

fn sample_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    match a.get("name").and_then(Value::as_str) {
        Some("tshirt") => mutate(s, "Group T-Shirt Components", sample::tshirt),
        Some("sleeveless") => {
            let like = id_of(a, "like")?;
            let port = a.get("port").and_then(Value::as_str).unwrap_or("armhole").to_string();
            mutate(s, "Add Sleeveless Variant", |g| sample::sleeveless(g, like, &port))
        }
        _ => Err(bad("name must be tshirt or sleeveless")),
    }
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "component.create",
            mcp: "component_create",
            group: Group::Pattern,
            label: "New Component",
            description: "Group panels into a component: the author lists the panels, and names ports as edge spans of those panels with a length tolerance in mm. A component owns its own panels and their real seams; a panel belongs to at most one component. Nothing is inferred from panel shape.",
            keys: &[],
            hints: WRITE,
            schema: create_schema,
            run: create_run,
        },
        Command {
            id: "component.list",
            mcp: "component_list",
            group: Group::Pattern,
            label: "Components",
            description: "List components with their panels, ports (live length against the stored length, tolerance, whether the edge still exists, the join it is in) and internal seams, and the joins with the real seams they made.",
            keys: &[],
            hints: READ,
            schema: crate::no_args,
            run: list_run,
        },
        Command {
            id: "component.save",
            mcp: "component_save",
            group: Group::File,
            label: "Save Component to Library",
            description: "Write a component to the library as a .ffcomp file: its panels, their internal seams and its ports, nothing else (no fabrics, trims or body). Path must be inside the component folder of a library folder; leave it out to use the first library folder.",
            keys: &[],
            hints: crate::EXTERNAL,
            schema: save_schema,
            run: save_run,
        },
        Command {
            id: "component.insert",
            mcp: "component_insert",
            group: Group::Pattern,
            label: "Insert Component",
            description: "Insert a library component: its panels get new ids and are placed beside the garment, its internal seams and ports are recreated, and the given joins become ordinary seams between real panel edges. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: insert_schema,
            run: insert_run,
        },
        Command {
            id: "component.swap",
            mcp: "component_swap",
            group: Group::Pattern,
            label: "Swap Component",
            description: "Replace a component with a library component: remove the old panels, insert the new ones centred where the old were, and recreate every join by port name. Refused when a joined port name is missing or its length differs by more than the tolerance (unless ease is true, which sews with a seam ratio). Panels are never graded or resized. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: swap_schema,
            run: swap_run,
        },
        Command {
            id: "component.join",
            mcp: "component_join",
            group: Group::Pattern,
            label: "Join Components",
            description: "Join a port of one component to a port of another: one seam per pair of spans (or one multi-span seam when the span counts differ) between the real panel edges, recorded as a join. Refused when the port lengths differ by more than the tolerance unless ease is true.",
            keys: &[],
            hints: WRITE,
            schema: join_schema,
            run: join_run,
        },
        Command {
            id: "component.check",
            mcp: "component_check",
            group: Group::Pattern,
            label: "Check Component Swap",
            description: "Compare a library component with a project component by port names and lengths only: for every joined port of the project component, whether the other has a port of that name and its length is within the tolerance. Changes nothing.",
            keys: &[],
            hints: READ,
            schema: check_schema,
            run: check_run,
        },
        Command {
            id: "component.remove",
            mcp: "component_remove",
            group: Group::Pattern,
            label: "Remove Component",
            description: "Remove a component and its joins. The panels and seams stay unless delete_panels is true.",
            keys: &[],
            hints: DESTROY,
            schema: remove_schema,
            run: remove_run,
        },
        Command {
            id: "component.sample",
            mcp: "component_sample",
            group: Group::Pattern,
            label: "Sample Components",
            description: "Build the sample modular T-shirt: tshirt groups the drafted sample into components and joins; sleeveless adds a no-sleeve armhole binding component for swapping.",
            keys: &[],
            hints: WRITE,
            schema: sample_schema,
            run: sample_run,
        },
    ]
}
