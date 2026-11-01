use ffb_pattern::trim::{defaults, geometry, merge, next_style_id};
use ffb_pattern::{Garment, Style, StyleKind};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, READ, SET, WRITE};

const KINDS: [&str; 6] = ["button", "buttonhole", "zipper", "topstitch", "puckering", "trim"];
const MAX_PARAMS: usize = 65_536;

fn range(kind: StyleKind, key: &str) -> Option<(f64, f64)> {
    Some(match (kind, key) {
        (StyleKind::Button, "width") => (0.5, 100.0),
        (StyleKind::Button, "thickness") => (0.1, 50.0),
        (StyleKind::Button, "weight") => (0.0, 500.0),
        (StyleKind::Buttonhole, "length") => (1.0, 100.0),
        (StyleKind::Buttonhole, "z_offset") => (-5.0, 5.0),
        (StyleKind::Zipper, "width") => (1.0, 100.0),
        (StyleKind::Topstitch, "count") => (1.0, 5.0),
        (StyleKind::Topstitch, "spi") => (1.0, 40.0),
        (StyleKind::Topstitch, "thickness") => (0.05, 5.0),
        (StyleKind::Topstitch, "offset") => (-50.0, 50.0),
        (StyleKind::Topstitch, "stiffen") => (1.0, 5.0),
        (StyleKind::Puckering, "density") => (0.0, 100.0),
        (StyleKind::Puckering, "width") => (0.0, 50.0),
        (_, "stiffness") => (0.0, 20.0),
        _ => return None,
    })
}

fn check_params(kind: StyleKind, params: &Value) -> Result<(), CommandError> {
    let map = params.as_object().ok_or_else(|| bad("params must be an object"))?;
    if map.contains_key("mesh") {
        return Err(bad("params.mesh is written by trim.import only"));
    }
    if serde_json::to_string(params).map_or(true, |t| t.len() > MAX_PARAMS) {
        return Err(bad("params are larger than 64 KiB"));
    }
    for (key, value) in map {
        if let Some(x) = value.as_f64() {
            if !x.is_finite() {
                return Err(bad(format!("params.{key} must be finite")));
            }
            if let Some((lo, hi)) = range(kind, key) {
                if !(lo..=hi).contains(&x) {
                    return Err(bad(format!("params.{key} must be between {lo} and {hi}")));
                }
            }
        }
    }
    Ok(())
}

pub fn usage(garment: &Garment, style: u32) -> (usize, usize, usize) {
    (
        garment.hardware.iter().filter(|h| h.style == style).count(),
        garment.zippers.iter().filter(|z| z.style == style).count(),
        garment.stitches.iter().filter(|s| s.style == style).count(),
    )
}

pub fn describe(garment: &Garment, style: &Style) -> Value {
    let (hardware, zippers, stitches) = usage(garment, style.id);
    let mut params = style.params.clone();
    if let Some(g) = geometry(style) {
        params["mesh"] = json!({"vertices": g.positions.len(), "triangles": g.triangles.len()});
    }
    json!({"id": style.id, "kind": kind_name(style.kind), "name": style.name, "params": params, "used_by": {"hardware": hardware, "zippers": zippers, "stitches": stitches}})
}

fn kind_prop() -> Value {
    json!({"type": "string", "enum": KINDS, "description": "button, buttonhole, zipper, topstitch, puckering or trim."})
}

fn params_prop(what: &str) -> Value {
    json!({"type": "object", "description": format!("{what} Known keys are range checked: button width 0.5 to 100 mm, thickness 0.1 to 50 mm, weight 0 to 500 g; buttonhole length 1 to 100 mm, z_offset -5 to 5 mm; zipper width 1 to 100 mm; topstitch count 1 to 5, spi 1 to 40, thickness 0.05 to 5 mm, offset -50 to 50 mm, stiffen 1 to 5; puckering density 0 to 100 percent, width 0 to 50 mm; stiffness 0 to 20 on any kind. Unknown keys are kept as given.")})
}

fn create_schema() -> Value {
    obj(json!({"kind": kind_prop(), "name": {"type": "string", "description": "Style name. Defaults to the kind and the new id."}, "params": params_prop("Style parameters merged over the defaults of the kind.")}), &["kind"])
}

fn create(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kind = kind_of(text(a, "kind").unwrap_or(""))?;
    let params = a.get("params").cloned().unwrap_or_else(|| json!({}));
    check_params(kind, &params)?;
    let mut merged = defaults(kind);
    merge(&mut merged, &params);
    let id = next_style_id(&s.garment);
    let name = clean_text(a, "name")?.unwrap_or_else(|| format!("{} {id}", kind_name(kind)));
    let before = s.garment.clone();
    let style = Style { id, kind, name, params: merged };
    s.garment.styles.push(style.clone());
    record(s, "Create Style", before);
    Ok(describe(&s.garment, &style))
}

fn update_schema() -> Value {
    obj(json!({"style": ident("Style id as listed by style.list."), "name": {"type": "string", "description": "New name."}, "params": params_prop("Keys to set; null removes a key.")}), &["style"])
}

fn update(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = required_id(a, "style")?;
    let index = s.garment.styles.iter().position(|x| x.id == id).ok_or_else(|| missing(format!("no style with id {id}")))?;
    let kind = s.garment.styles[index].kind;
    let patch = a.get("params").cloned();
    if patch.as_ref().is_some_and(|p| p.get("mesh").is_some()) {
        return Err(bad("params.mesh is written by trim.import only"));
    }
    if let Some(p) = &patch {
        let checked: Value = p.as_object().map_or(json!({}), |m| Value::Object(m.iter().filter(|(_, v)| !v.is_null()).map(|(k, v)| (k.clone(), v.clone())).collect()));
        check_params(kind, &checked)?;
    }
    let name = clean_text(a, "name")?;
    let before = s.garment.clone();
    let style = &mut s.garment.styles[index];
    if let Some(n) = name {
        style.name = n;
    }
    if let Some(p) = &patch {
        merge(&mut style.params, p);
    }
    if s.garment == before {
        return Ok(json!({"changed": false, "style": describe(&s.garment, &s.garment.styles[index])}));
    }
    record(s, "Update Style", before);
    Ok(json!({"changed": true, "style": describe(&s.garment, &s.garment.styles[index])}))
}

fn delete_schema() -> Value {
    obj(json!({"style": ident("Style id as listed by style.list.")}), &["style"])
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = required_id(a, "style")?;
    let index = s.garment.styles.iter().position(|x| x.id == id).ok_or_else(|| missing(format!("no style with id {id}")))?;
    let (hardware, zippers, stitches) = usage(&s.garment, id);
    if hardware + zippers + stitches > 0 {
        return Err(bad(format!("style {id} is used by {hardware} hardware, {zippers} zippers and {stitches} stitches; delete or reassign them first")));
    }
    let before = s.garment.clone();
    s.garment.styles.remove(index);
    record(s, "Delete Style", before);
    Ok(json!({"changed": true, "removed": id, "remaining": s.garment.styles.len()}))
}

fn list_schema() -> Value {
    obj(json!({"kind": kind_prop()}), &[])
}

fn list(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kind = text(a, "kind").map(kind_of).transpose()?;
    let styles: Vec<Value> = s.garment.styles.iter().filter(|x| kind.is_none_or(|k| k == x.kind)).map(|x| describe(&s.garment, x)).collect();
    Ok(json!({"count": styles.len(), "styles": styles}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "style.create",
            mcp: "style_create",
            group: Group::Pattern,
            label: "Create Style",
            description: "Create a named style for buttons, buttonholes, zippers, topstitch, puckering or imported trims. Parameters merge over the defaults of the kind and are range checked. Returns the style with its id.",
            keys: &[],
            hints: WRITE,
            schema: create_schema,
            run: create,
        },
        Command {
            id: "style.update",
            mcp: "style_update",
            group: Group::Pattern,
            label: "Update Style",
            description: "Rename a style or change parameters (merged; null removes a key). Everything that uses the style follows. The kind and an imported trim mesh cannot change.",
            keys: &[],
            hints: SET,
            schema: update_schema,
            run: update,
        },
        Command {
            id: "style.delete",
            mcp: "style_delete",
            group: Group::Pattern,
            label: "Delete Style",
            description: "Remove a style. Refused while any hardware, zipper or stitch uses it.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
        Command {
            id: "style.list",
            mcp: "style_list",
            group: Group::Pattern,
            label: "List Styles",
            description: "List styles, optionally of one kind, with their parameters and how many hardware, zippers and stitches use each.",
            keys: &[],
            hints: READ,
            schema: list_schema,
            run: list,
        },
    ]
}
