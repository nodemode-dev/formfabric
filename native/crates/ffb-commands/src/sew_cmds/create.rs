use ffb_pattern::sew::next_seam_id;
use ffb_pattern::{Seam, SeamEnd};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, WRITE};

pub fn check_ranges(a: &Value) -> Result<(), CommandError> {
    let within = |key: &str, lo: f64, hi: f64| match number(a, key) {
        Some(x) if !(lo..=hi).contains(&x) => Err(bad(format!("{key} must be between {lo} and {hi}"))),
        _ => Ok(()),
    };
    within("ratio", 0.05, 20.0)?;
    within("tension", 0.0, 20.0)?;
    within("fold_angle", 0.0, 360.0)?;
    within("fold_strength", 0.0, 20.0)
}

fn single(prefix: &str, a: &Value, full: bool) -> Result<Value, CommandError> {
    let key = |name: &str| format!("{name}_{prefix}");
    let mut v = json!({"panel": required_id(a, &key("panel"))?, "segment": count(a, &key("segment")).ok_or_else(|| bad(format!("{} is required", key("segment"))))?});
    if let Some(i) = id(a, &key("inner")) {
        v["inner"] = json!(i);
    }
    if !full {
        v["t0"] = json!(number(a, &key("t0")).ok_or_else(|| bad(format!("{} is required", key("t0"))))?);
        v["t1"] = json!(number(a, &key("t1")).ok_or_else(|| bad(format!("{} is required", key("t1"))))?);
    }
    Ok(v)
}

fn make(s: &mut Session, label: &str, ends_a: Vec<SeamEnd>, ends_b: Vec<SeamEnd>, a: &Value) -> Result<Value, CommandError> {
    check_ranges(a)?;
    if ends_a.iter().any(|x| ends_b.iter().any(|y| same_place(std::slice::from_ref(x), std::slice::from_ref(y)))) {
        return Err(bad("a span cannot be sewn to itself"));
    }
    let mut seam = Seam { id: next_seam_id(&s.garment), a: ends_a, b: ends_b, reversed: true, ..Seam::default() };
    apply_fields(&mut seam, a)?;
    let before = s.garment.clone();
    let out = describe(&s.garment, &seam);
    s.garment.seams.push(seam);
    record(s, label, before);
    Ok(out)
}

fn pair_props() -> Value {
    json!({
        "panel_a": panel_id("Panel of side a."),
        "inner_a": int("Inner line of side a. Leave out for the outline.", 1.0, 4_294_967_295.0),
        "segment_a": int("Segment of side a; segment i runs from point i to point i+1.", 0.0, 100_000.0),
        "panel_b": panel_id("Panel of side b."),
        "inner_b": int("Inner line of side b. Leave out for the outline.", 1.0, 4_294_967_295.0),
        "segment_b": int("Segment of side b.", 0.0, 100_000.0)
    })
}

fn segment_schema() -> Value {
    let mut props = pair_props();
    seam_fields(&mut props, true);
    obj(props, &["panel_a", "segment_a", "panel_b", "segment_b"])
}

fn segment(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ea = parse_span(&s.garment, &single("a", a, true)?)?;
    let eb = parse_span(&s.garment, &single("b", a, true)?)?;
    make(s, "Segment Seam", vec![ea], vec![eb], a)
}

fn free_schema() -> Value {
    let mut props = pair_props();
    props["t0_a"] = num("Start of the span on side a as a curve parameter on its segment.", 0.0, 1.0, None);
    props["t1_a"] = num("End of the span on side a.", 0.0, 1.0, None);
    props["t0_b"] = num("Start of the span on side b.", 0.0, 1.0, None);
    props["t1_b"] = num("End of the span on side b.", 0.0, 1.0, None);
    seam_fields(&mut props, true);
    obj(props, &["panel_a", "segment_a", "t0_a", "t1_a", "panel_b", "segment_b", "t0_b", "t1_b"])
}

fn free(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ea = parse_span(&s.garment, &single("a", a, false)?)?;
    let eb = parse_span(&s.garment, &single("b", a, false)?)?;
    make(s, "Free Seam", vec![ea], vec![eb], a)
}

fn multi_schema() -> Value {
    let mut props = json!({"a": spans("Spans that make side a, in walking order."), "b": spans("Spans that make side b, in walking order.")});
    seam_fields(&mut props, true);
    obj(props, &["a", "b"])
}

fn multi(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let ea = span_list(&s.garment, a, "a")?;
    let eb = span_list(&s.garment, a, "b")?;
    make(s, "Multi Seam", ea, eb, a)
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "sew.segment",
            mcp: "sew_segment",
            group: Group::Sew,
            label: "Segment Seam",
            description: "Sew one whole segment to another. reversed (default true) pairs the start of side a with the end of side b, which is right for counter-clockwise outlines meeting along a shared edge. With ratio 1 and full segments both segments get the same number of mesh vertices. Returns the seam with both side lengths and the mismatch in mm.",
            keys: &[],
            hints: WRITE,
            schema: segment_schema,
            run: segment,
        },
        Command {
            id: "sew.free",
            mcp: "sew_free",
            group: Group::Sew,
            label: "Free Seam",
            description: "Sew a span of one segment (t0 to t1 as curve parameters) to a span of another, so a seam can start and end anywhere on an outline or inner line. Sides are matched by arc length fraction.",
            keys: &[],
            hints: WRITE,
            schema: free_schema,
            run: free,
        },
        Command {
            id: "sew.multi",
            mcp: "sew_multi",
            group: Group::Sew,
            label: "Multi Seam",
            description: "Sew a list of spans (M) to another list of spans (N) as one seam, for example a sleeve cap to an armhole made of front and back pieces. Spans are listed in walking order and matched by total arc length fraction.",
            keys: &[],
            hints: WRITE,
            schema: multi_schema,
            run: multi,
        },
    ]
}
