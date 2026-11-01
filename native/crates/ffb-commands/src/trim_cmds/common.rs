use ffb_pattern::geom::V2;
use ffb_pattern::{path, polygon, Garment, Panel, SeamEnd, Style, StyleKind};
use serde_json::{json, Value};

use crate::{CommandError, Edit, Session};

pub const COORD: f64 = 100_000.0;
pub const BIG: f64 = 4_294_967_295.0;

pub fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub fn geometry(message: impl Into<String>) -> CommandError {
    CommandError::new("invalid_geometry", message)
}

pub fn number(a: &Value, key: &str) -> Option<f64> {
    a.get(key).and_then(Value::as_f64).filter(|x| x.is_finite())
}

pub fn number_or(a: &Value, key: &str, default: f64) -> f64 {
    number(a, key).unwrap_or(default)
}

pub fn count(a: &Value, key: &str) -> Option<usize> {
    a.get(key).and_then(Value::as_f64).filter(|x| *x >= 0.0).map(|x| x as usize)
}

pub fn id(a: &Value, key: &str) -> Option<u32> {
    let whole = a.get(key).and_then(|v| v.as_u64().or_else(|| v.as_f64().filter(|x| x.fract() == 0.0 && *x >= 0.0).map(|x| x as u64)));
    whole.and_then(|x| u32::try_from(x).ok())
}

pub fn required_id(a: &Value, key: &str) -> Result<u32, CommandError> {
    id(a, key).ok_or_else(|| bad(format!("{key} is required")))
}

pub fn flag(a: &Value, key: &str) -> Option<bool> {
    a.get(key).and_then(Value::as_bool)
}

pub fn text<'a>(a: &'a Value, key: &str) -> Option<&'a str> {
    a.get(key).and_then(Value::as_str)
}

pub fn clean_text(a: &Value, key: &str) -> Result<Option<String>, CommandError> {
    let Some(t) = text(a, key) else { return Ok(None) };
    if t.chars().any(|c| c.is_control()) || t.len() > 200 {
        return Err(bad(format!("{key} must be plain text of at most 200 characters")));
    }
    Ok(Some(t.to_string()))
}

pub fn id_list(a: &Value, key: &str) -> Result<Vec<u32>, CommandError> {
    let list = a.get(key).and_then(Value::as_array).ok_or_else(|| bad(format!("{key} must be a list of ids")))?;
    let mut out: Vec<u32> = Vec::new();
    for v in list {
        let value = v.as_f64().filter(|x| x.fract() == 0.0 && *x >= 1.0).ok_or_else(|| bad(format!("{key} must hold whole ids")))? as u32;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    if out.is_empty() {
        return Err(bad(format!("{key} must not be empty")));
    }
    Ok(out)
}

pub fn vec2(v: &Value, name: &str) -> Result<V2, CommandError> {
    let list = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad(format!("{name} must be [x, y]")))?;
    match [list[0].as_f64(), list[1].as_f64()] {
        [Some(x), Some(y)] if x.is_finite() && y.is_finite() => Ok([x, y]),
        _ => Err(bad(format!("{name} must hold two finite numbers"))),
    }
}

pub fn optional_vec2(a: &Value, key: &str) -> Result<Option<V2>, CommandError> {
    a.get(key).filter(|v| !v.is_null()).map(|v| vec2(v, key)).transpose()
}

pub fn int(description: &str, lo: f64, hi: f64) -> Value {
    json!({"type": "integer", "minimum": lo, "maximum": hi, "description": description})
}

pub fn num(description: &str, lo: f64, hi: f64, unit: Option<&str>) -> Value {
    let mut v = json!({"type": "number", "minimum": lo, "maximum": hi, "description": description});
    if let Some(u) = unit {
        v["unit"] = json!(u);
    }
    v
}

pub fn ident(description: &str) -> Value {
    int(description, 1.0, BIG)
}

pub fn xy(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "number", "minimum": -COORD, "maximum": COORD}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": description})
}

pub fn obj(props: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

pub fn space() -> Value {
    json!({"type": "string", "enum": ["world", "local"], "default": "world", "description": "world: coordinates on the pattern plane. local: coordinates inside the panel before its position, rotation and mirror apply."})
}

pub fn span() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": ident("Panel id as listed by pattern.get."),
            "inner": ident("Inner line id inside the panel. Leave out for the outline."),
            "segment": int("Segment of the outline or inner line; segment i runs from point i to point i+1.", 0.0, 100_000.0),
            "t0": num("Start of the span as a curve parameter on the segment. Default 0.", 0.0, 1.0, None),
            "t1": num("End of the span as a curve parameter on the segment. Default 1. Below t0 walks the span against the line direction.", 0.0, 1.0, None)
        },
        "required": ["panel", "segment"],
        "additionalProperties": false
    })
}

pub fn spans(description: &str) -> Value {
    json!({"type": "array", "minItems": 1, "maxItems": 256, "items": span(), "description": description})
}

fn parse_span(garment: &Garment, v: &Value) -> Result<SeamEnd, CommandError> {
    let panel = required_id(v, "panel")?;
    let segment = count(v, "segment").ok_or_else(|| bad("a span needs segment"))?;
    let (t0, t1) = (number_or(v, "t0", 0.0), number_or(v, "t1", 1.0));
    if !(0.0..=1.0).contains(&t0) || !(0.0..=1.0).contains(&t1) {
        return Err(bad("t0 and t1 must be between 0 and 1"));
    }
    if (t1 - t0).abs() < 1e-6 {
        return Err(bad("a span needs a length, t0 and t1 must differ"));
    }
    let end = SeamEnd { panel, inner: id(v, "inner"), segment, t0, t1 };
    let p = garment.panels.iter().find(|p| p.id == end.panel).ok_or_else(|| missing(format!("no panel with id {}", end.panel)))?;
    let (pts, closed) = p.path(end.inner).ok_or_else(|| missing(format!("panel {} has no inner line {}", p.id, end.inner.unwrap_or(0))))?;
    let n = path::seg_count(pts.len(), closed);
    if end.segment >= n {
        return Err(bad(format!("segment {} is out of range, panel {} has {n}", end.segment, p.id)));
    }
    Ok(end)
}

pub fn parse_spans(garment: &Garment, a: &Value, key: &str) -> Result<Vec<SeamEnd>, CommandError> {
    let list = a.get(key).and_then(Value::as_array).filter(|l| !l.is_empty() && l.len() <= 256).ok_or_else(|| bad(format!("{key} must be a list of 1 to 256 spans")))?;
    list.iter().map(|v| parse_span(garment, v)).collect()
}

pub fn record(s: &mut Session, label: &str, before: Garment) {
    let after = s.garment.clone();
    s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
}

pub fn panel_of(garment: &Garment, id: u32) -> Result<&Panel, CommandError> {
    garment.panels.iter().find(|p| p.id == id).ok_or_else(|| missing(format!("no panel with id {id}")))
}

pub fn local_point(panel: &Panel, point: V2, world: bool) -> V2 {
    if world {
        panel.placement().to_local(point)
    } else {
        point
    }
}

pub fn inside(panel: &Panel, local: V2) -> bool {
    let outline = path::flatten(&panel.outline, true, 0.05);
    if outline.len() < 3 {
        return false;
    }
    polygon::contains(&outline, local) || path::nearest(&panel.outline, true, local).is_some_and(|n| n.dist < 0.5)
}

pub fn style_of<'a>(garment: &'a Garment, style: u32, kinds: &[StyleKind]) -> Result<&'a Style, CommandError> {
    let found = garment.styles.iter().find(|x| x.id == style).ok_or_else(|| missing(format!("no style with id {style}")))?;
    if !kinds.contains(&found.kind) {
        return Err(bad(format!("style {style} is a {:?} style; this command needs {}", found.kind, kinds.iter().map(|k| format!("{k:?}").to_lowercase()).collect::<Vec<_>>().join(" or "))));
    }
    Ok(found)
}

pub fn kind_name(kind: StyleKind) -> &'static str {
    match kind {
        StyleKind::Button => "button",
        StyleKind::Buttonhole => "buttonhole",
        StyleKind::Zipper => "zipper",
        StyleKind::Topstitch => "topstitch",
        StyleKind::Puckering => "puckering",
        StyleKind::Trim => "trim",
    }
}

pub fn kind_of(name: &str) -> Result<StyleKind, CommandError> {
    Ok(match name {
        "button" => StyleKind::Button,
        "buttonhole" => StyleKind::Buttonhole,
        "zipper" => StyleKind::Zipper,
        "topstitch" => StyleKind::Topstitch,
        "puckering" => StyleKind::Puckering,
        "trim" => StyleKind::Trim,
        other => return Err(bad(format!("kind must be button, buttonhole, zipper, topstitch, puckering or trim, not {other}"))),
    })
}
