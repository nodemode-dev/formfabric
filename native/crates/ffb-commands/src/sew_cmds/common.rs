use ffb_pattern::sew::{lengths_of, side_length};
use ffb_pattern::{Garment, Seam, SeamEnd, SeamFold, SeamSide};
use serde_json::{json, Value};

use crate::{CommandError, Edit, Session};

pub const COORD: f64 = 100_000.0;

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
    a.get(key).and_then(Value::as_f64)
}

pub fn number_or(a: &Value, key: &str, default: f64) -> f64 {
    number(a, key).unwrap_or(default)
}

pub fn count(a: &Value, key: &str) -> Option<usize> {
    a.get(key).and_then(Value::as_f64).map(|x| x as usize)
}

pub fn id(a: &Value, key: &str) -> Option<u32> {
    count(a, key).map(|x| x as u32)
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

pub fn vec2(v: &Value, name: &str) -> Result<[f64; 2], CommandError> {
    let list = v.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad(format!("{name} must be [x, y]")))?;
    match [list[0].as_f64(), list[1].as_f64()] {
        [Some(x), Some(y)] if x.is_finite() && y.is_finite() => Ok([x, y]),
        _ => Err(bad(format!("{name} must hold two finite numbers"))),
    }
}

pub fn optional_vec2(a: &Value, key: &str) -> Result<Option<[f64; 2]>, CommandError> {
    a.get(key).filter(|v| !v.is_null()).map(|v| vec2(v, key)).transpose()
}

pub fn optional_vec3(a: &Value, key: &str) -> Result<Option<[f64; 3]>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() == 3).ok_or_else(|| bad(format!("{key} must be [x, y, z]")))?;
    let n: Vec<f64> = list.iter().filter_map(Value::as_f64).filter(|x| x.is_finite()).collect();
    if n.len() != 3 {
        return Err(bad(format!("{key} must hold three finite numbers")));
    }
    Ok(Some([n[0], n[1], n[2]]))
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

pub fn panel_id(description: &str) -> Value {
    int(description, 1.0, 4_294_967_295.0)
}

pub fn xy(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "number", "minimum": -COORD, "maximum": COORD}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": description})
}

pub fn xyz(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "number", "minimum": -COORD, "maximum": COORD}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": description})
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
            "panel": panel_id("Panel id as listed by pattern.get."),
            "inner": int("Inner line id inside the panel. Leave out for the outline.", 1.0, 4_294_967_295.0),
            "segment": int("Segment of the outline or inner line; segment i runs from point i to point i+1.", 0.0, 100_000.0),
            "t0": num("Start of the span as a curve parameter on the segment. Default 0.", 0.0, 1.0, None),
            "t1": num("End of the span as a curve parameter on the segment. Default 1. Below t0 walks the span against the line direction.", 0.0, 1.0, None)
        },
        "required": ["panel", "segment"],
        "additionalProperties": false
    })
}

pub fn spans(description: &str) -> Value {
    json!({"type": "array", "minItems": 1, "items": span(), "description": description})
}

pub fn seam_fields(props: &mut Value, with_defaults: bool) {
    let mut reversed = json!({"type": "boolean", "description": "Pair the start of side a with the end of side b. Outlines drawn counter-clockwise that meet along a shared edge run in opposite directions, so the usual seam is reversed."});
    let mut fold = json!({"type": "string", "enum": ["turned", "flat", "custom"], "description": "turned: folded back 180 degrees, two plies, angle and strength locked. flat: butt joint, panels stay in their plane. custom: use fold_angle and fold_strength."});
    let mut tension = num("Stiffness of the seam pull relative to the cloth; 1 is normal, above 1 pulls harder, 0 leaves the seam slack.", 0.0, 20.0, None);
    let mut ratio = num("Intended arc length of side b over side a. Above 1 gathers b onto a.", 0.05, 20.0, None);
    if with_defaults {
        reversed["default"] = json!(true);
        fold["default"] = json!("turned");
        tension["default"] = json!(1.0);
        ratio["default"] = json!(1.0);
    }
    props["reversed"] = reversed;
    props["fold"] = fold;
    props["fold_angle"] = num("Fold angle for custom seams: 180 flat, lower folds to the front, higher to the back.", 0.0, 360.0, Some("deg"));
    props["fold_strength"] = num("How firmly a custom seam holds its fold angle, 0 for none.", 0.0, 20.0, None);
    props["ratio"] = ratio;
    props["tension"] = tension;
    props["side"] = json!({"type": "string", "enum": ["auto", "front", "back"], "description": "Which face of the fabric of side a the other side attaches to. auto picks it from the 3D placement and the panel normal; sew.list reports the result as side_resolved."});
}

pub fn fold_of(a: &Value) -> Result<Option<SeamFold>, CommandError> {
    Ok(match text(a, "fold") {
        None => None,
        Some("turned") => Some(SeamFold::Turned),
        Some("flat") => Some(SeamFold::Flat),
        Some("custom") => Some(SeamFold::Custom),
        Some(other) => return Err(bad(format!("fold must be turned, flat or custom, not {other}"))),
    })
}

pub fn side_of(a: &Value) -> Result<Option<SeamSide>, CommandError> {
    Ok(match text(a, "side") {
        None => None,
        Some("auto") => Some(SeamSide::Auto),
        Some("front") => Some(SeamSide::Front),
        Some("back") => Some(SeamSide::Back),
        Some(other) => return Err(bad(format!("side must be auto, front or back, not {other}"))),
    })
}

pub fn apply_fields(seam: &mut Seam, a: &Value) -> Result<(), CommandError> {
    if let Some(v) = flag(a, "reversed") {
        seam.reversed = v;
    }
    if let Some(v) = fold_of(a)? {
        seam.fold = v;
    }
    if let Some(v) = side_of(a)? {
        seam.side = v;
    }
    seam.fold_angle = number_or(a, "fold_angle", seam.fold_angle);
    seam.fold_strength = number_or(a, "fold_strength", seam.fold_strength);
    seam.ratio = number_or(a, "ratio", seam.ratio);
    seam.tension = number_or(a, "tension", seam.tension);
    if seam.fold == SeamFold::Turned {
        seam.fold_angle = 180.0;
        seam.fold_strength = 0.0;
    }
    Ok(())
}

pub fn parse_span(garment: &Garment, v: &Value) -> Result<SeamEnd, CommandError> {
    let panel = required_id(v, "panel")?;
    let segment = count(v, "segment").ok_or_else(|| bad("a span needs segment"))?;
    let inner = id(v, "inner");
    let (t0, t1) = (number_or(v, "t0", 0.0), number_or(v, "t1", 1.0));
    if !(0.0..=1.0).contains(&t0) || !(0.0..=1.0).contains(&t1) {
        return Err(bad("t0 and t1 must be between 0 and 1"));
    }
    if (t1 - t0).abs() < 1e-6 {
        return Err(bad("a span needs a length, t0 and t1 must differ"));
    }
    let end = SeamEnd { panel, inner, segment, t0, t1 };
    check_end(garment, &end)?;
    Ok(end)
}

pub fn check_end(garment: &Garment, end: &SeamEnd) -> Result<(), CommandError> {
    let p = garment.panels.iter().find(|p| p.id == end.panel).ok_or_else(|| missing(format!("no panel with id {}", end.panel)))?;
    let (pts, closed) = p.path(end.inner).ok_or_else(|| missing(format!("panel {} has no inner line {}", p.id, end.inner.unwrap_or(0))))?;
    let n = ffb_pattern::path::seg_count(pts.len(), closed);
    if end.segment >= n {
        return Err(bad(format!("segment {} is out of range, panel {} has {n}", end.segment, p.id)));
    }
    Ok(())
}

pub fn span_list(garment: &Garment, a: &Value, key: &str) -> Result<Vec<SeamEnd>, CommandError> {
    let list = a.get(key).and_then(Value::as_array).filter(|l| !l.is_empty()).ok_or_else(|| bad(format!("{key} must be a list of spans")))?;
    list.iter().map(|v| parse_span(garment, v)).collect()
}

pub fn same_place(a: &[SeamEnd], b: &[SeamEnd]) -> bool {
    a.len() == 1 && b.len() == 1 && a[0].panel == b[0].panel && a[0].inner == b[0].inner && a[0].segment == b[0].segment && a[0].t0.min(a[0].t1) < b[0].t0.max(b[0].t1) && b[0].t0.min(b[0].t1) < a[0].t0.max(a[0].t1)
}

pub fn describe(garment: &Garment, seam: &Seam) -> Value {
    let mut out = serde_json::to_value(seam).unwrap_or(Value::Null);
    let lengths = lengths_of(garment, seam);
    out["length_a"] = json!(side_length(garment, &seam.a));
    out["length_b"] = json!(side_length(garment, &seam.b));
    out["mismatch"] = json!(lengths.map(|l| l.mismatch));
    out["expected_b"] = json!(side_length(garment, &seam.a).map(|l| l * seam.ratio));
    out["side_resolved"] = json!(ffb_pattern::sew::resolve_side(garment, seam));
    out
}

pub fn record(s: &mut Session, label: &str, before: Garment) {
    let after = s.garment.clone();
    s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
}

pub fn seam_index(s: &Session, seam: u32) -> Result<usize, CommandError> {
    s.garment.seams.iter().position(|x| x.id == seam).ok_or_else(|| missing(format!("no seam with id {seam}")))
}
