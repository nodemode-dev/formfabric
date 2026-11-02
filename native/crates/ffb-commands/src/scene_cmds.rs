use ffb_pattern::{Garment, StyleKind};
use serde_json::{json, Value};

use crate::{Command, CommandError, Group, Session, READ};

fn kind_name(kind: StyleKind) -> &'static str {
    match kind {
        StyleKind::Button => "button",
        StyleKind::Buttonhole => "buttonhole",
        StyleKind::Zipper => "zipper",
        StyleKind::Topstitch => "topstitch",
        StyleKind::Puckering => "puckering",
        StyleKind::Trim => "trim",
    }
}

const KINDS: [&str; 5] = ["panel", "seam", "trim", "body", "style"];
const SORTS: [&str; 3] = ["creation", "name_asc", "name_desc"];

struct Row {
    kind: &'static str,
    id: u32,
    name: String,
    extra: Value,
}

fn style_name(g: &Garment, id: u32) -> String {
    g.styles.iter().find(|x| x.id == id).map_or_else(|| format!("Style {id}"), |x| x.name.clone())
}

fn style_kind(g: &Garment, id: u32) -> &'static str {
    g.styles.iter().find(|x| x.id == id).map_or("trim", |x| kind_name(x.kind))
}

fn rows(s: &Session) -> Vec<Row> {
    let g = &s.garment;
    let mut out = Vec::new();
    for p in &g.panels {
        let name = if p.name.trim().is_empty() { format!("Panel {}", p.id) } else { p.name.clone() };
        out.push(Row { kind: "panel", id: p.id, name, extra: json!({"fabric": p.fabric, "layer": p.layer}) });
    }
    for x in &g.seams {
        let a: Vec<u32> = x.a.iter().map(|e| e.panel).collect();
        let b: Vec<u32> = x.b.iter().map(|e| e.panel).collect();
        out.push(Row { kind: "seam", id: x.id, name: format!("Seam {}", x.id), extra: json!({"a": a, "b": b}) });
    }
    for h in &g.hardware {
        out.push(Row { kind: "trim", id: h.id, name: format!("{} {}", style_name(g, h.style), h.id), extra: json!({"part": "hardware", "style": h.style, "style_kind": style_kind(g, h.style)}) });
    }
    for z in &g.zippers {
        out.push(Row { kind: "trim", id: z.id, name: format!("{} {}", style_name(g, z.style), z.id), extra: json!({"part": "zipper", "style": z.style, "style_kind": style_kind(g, z.style)}) });
    }
    for t in &g.stitches {
        out.push(Row { kind: "trim", id: t.id, name: format!("{} {}", style_name(g, t.style), t.id), extra: json!({"part": "stitch", "style": t.style, "style_kind": style_kind(g, t.style)}) });
    }
    for b in &s.bodies {
        out.push(Row { kind: "body", id: b.id, name: b.name.clone(), extra: json!({"visible": b.visible}) });
    }
    for st in &g.styles {
        out.push(Row { kind: "style", id: st.id, name: st.name.clone(), extra: json!({"style_kind": kind_name(st.kind)}) });
    }
    out
}

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kinds": {"type": "array", "items": {"type": "string", "enum": KINDS}, "description": "Only these kinds. Leave out for all."},
            "sort": {"type": "string", "enum": SORTS, "description": "creation (default) keeps the order things were made in; name_asc and name_desc sort by name."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kinds: Vec<&str> = match a.get("kinds").and_then(Value::as_array) {
        Some(list) => list.iter().filter_map(Value::as_str).collect(),
        None => KINDS.to_vec(),
    };
    let sort = a.get("sort").and_then(Value::as_str).unwrap_or("creation");
    let mut all: Vec<(usize, Row)> = rows(s).into_iter().filter(|r| kinds.contains(&r.kind)).enumerate().collect();
    match sort {
        "name_asc" => all.sort_by(|x, y| x.1.name.to_lowercase().cmp(&y.1.name.to_lowercase()).then(x.0.cmp(&y.0))),
        "name_desc" => all.sort_by(|x, y| y.1.name.to_lowercase().cmp(&x.1.name.to_lowercase()).then(x.0.cmp(&y.0))),
        _ => {}
    }
    let counts: Value = KINDS.iter().map(|k| ((*k).to_string(), json!(all.iter().filter(|(_, r)| r.kind == *k).count()))).collect::<serde_json::Map<_, _>>().into();
    let items: Vec<Value> = all
        .into_iter()
        .map(|(order, r)| {
            let mut v = json!({"key": format!("{}:{}:{}", r.kind, r.extra.get("part").and_then(Value::as_str).unwrap_or("-"), r.id), "kind": r.kind, "id": r.id, "name": r.name, "order": order});
            if let (Some(o), Some(e)) = (v.as_object_mut(), r.extra.as_object()) {
                for (k, x) in e {
                    o.insert(k.clone(), x.clone());
                }
            }
            v
        })
        .collect();
    Ok(json!({"count": items.len(), "counts": counts, "items": items}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "scene.tree",
        mcp: "scene_tree",
        group: Group::View,
        label: "Scene Tree",
        description: "List everything in the scene in creation order: panels, seams, trims (hardware, zippers, stitches), bodies and styles, each with a stable key, id, name and kind. Optional kinds filter and sort by creation, name_asc or name_desc. Read only.",
        keys: &[],
        hints: READ,
        schema,
        run,
    }]
}
