use ffb_pattern::sew::{self, next_seam_id};
use ffb_pattern::trim::{self, build_strip, crease_points, next_finish_id, StripKind};
use ffb_pattern::{EdgeFinish, EdgeKind, Garment, Inner, InnerRole, Point, Seam, SeamEnd, SeamFold};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, WRITE};

const KINDS: [&str; 9] = ["piping", "binding", "elastic", "shirring", "tape", "bond", "press", "crease", "skive"];

fn add_schema() -> Value {
    obj(
        json!({
            "kind": {"type": "string", "enum": KINDS, "description": "piping and binding generate a strip panel sewn along the spans; elastic and shirring generate a narrow elastic strip (shirring when the spans lie on inner lines); tape, bond and press stiffen a band along the spans; skive softens a band along the spans (the inverse of bond); crease adds a fold line between the ends of the spans."},
            "spans": spans("Spans of one or more panels the finish follows, in walking order."),
            "width": num("Piping 1 to 15 mm (default 6), binding 1 to 50 mm per side (default 12), elastic 1 to 50 mm (default 8), tape, bond and press: band half width on each side, 1 to 100 mm (default 10).", 1.0, 100.0, Some("mm")),
            "ratio": num("Elastic and shirring: length of the elastic at rest over its drawn length, 0.2 to 2 (default 0.8; 0.7 for shirring). Below 1 gathers the edge.", 0.2, 2.0, None),
            "strength": num("Elastic: seam tension, default 1. Tape default 3, bond default 5, press default 1.5: stiffness multiplier of the band. Crease: fold strength 0 to 20, default 1.", 0.0, 20.0, None),
            "fabric": ident("Fabric id of the strip; default is the fabric of the first span panel."),
            "percent": num("Skive only: how far the band is shaved down, 0 to 100 percent (default 50). Bending stiffness and thickness of the band are scaled by one minus the percent (floor 5 percent for bending, 10 percent for thickness).", 0.0, 100.0, Some("percent")),
            "angle": num("Crease only: fold angle, 180 flat, lower folds to the front, higher to the back. Default 120.", 0.0, 360.0, Some("deg"))
        }),
        &["kind", "spans"],
    )
}

fn strip_kind(kind: &str) -> Option<StripKind> {
    match kind {
        "piping" => Some(StripKind::Piping),
        "binding" => Some(StripKind::Binding),
        "elastic" | "shirring" => Some(StripKind::Elastic),
        _ => None,
    }
}

fn width_range(kind: &str) -> (f64, f64, f64) {
    match kind {
        "piping" => (1.0, 15.0, 6.0),
        "binding" => (1.0, 50.0, 12.0),
        "elastic" => (1.0, 50.0, 8.0),
        "shirring" => (1.0, 50.0, 6.0),
        _ => (1.0, 100.0, 10.0),
    }
}

fn edge_kind(kind: &str) -> EdgeKind {
    match kind {
        "piping" => EdgeKind::Piping,
        "binding" => EdgeKind::Binding,
        "elastic" | "shirring" => EdgeKind::Elastic,
        "tape" => EdgeKind::Tape,
        "bond" => EdgeKind::Bond,
        "skive" => EdgeKind::Skive,
        _ => EdgeKind::Press,
    }
}

fn describe(garment: &Garment, f: &EdgeFinish) -> Value {
    let mut out = serde_json::to_value(f).unwrap_or(Value::Null);
    out["length"] = json!(sew::side_length(garment, &f.spans));
    out["seams"] = json!(garment.seams.iter().filter(|s| s.a.iter().chain(s.b.iter()).any(|e| f.generated.contains(&e.panel))).map(|s| s.id).collect::<Vec<_>>());
    out
}

fn crease(s: &mut Session, a: &Value, ends: &[SeamEnd]) -> Result<Value, CommandError> {
    let (panel, line) = crease_points(&s.garment, ends).map_err(geometry)?;
    let id = s.garment.next_id();
    let angle = number_or(a, "angle", 120.0);
    let strength = number_or(a, "strength", 1.0);
    let before = s.garment.clone();
    let p = s.garment.panels.iter_mut().find(|p| p.id == panel).ok_or_else(|| missing(format!("no panel with id {panel}")))?;
    let points = line.iter().map(|q| Point { p: *q, ..Point::default() }).collect();
    p.inner.push(Inner { id, points, role: InnerRole::Fold, fold_angle: angle, fold_strength: strength, ..Inner::default() });
    record(s, "Add Crease", before);
    Ok(json!({"kind": "crease", "panel": panel, "inner": id, "fold_angle": angle, "fold_strength": strength}))
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = text(a, "kind").unwrap_or("");
    let ends = parse_spans(&s.garment, a, "spans")?;
    if name == "crease" {
        return crease(s, a, &ends);
    }
    let (lo, hi, default) = width_range(name);
    let width = number_or(a, "width", default);
    if !(lo..=hi).contains(&width) {
        return Err(bad(format!("width for {name} must be between {lo} and {hi} mm")));
    }
    let ratio = number_or(a, "ratio", if name == "shirring" { 0.7 } else { 0.8 });
    let strength = if name == "skive" {
        let p = number_or(a, "percent", 50.0);
        if !(0.0..=100.0).contains(&p) {
            return Err(bad("percent must be between 0 and 100"));
        }
        Some(p)
    } else {
        number(a, "strength")
    };
    let fabric = id(a, "fabric");
    if let Some(f) = fabric {
        if !s.garment.fabrics.iter().any(|x| x.id == f) {
            return Err(missing(format!("no fabric with id {f}")));
        }
    }
    let id = next_finish_id(&s.garment);
    let before = s.garment.clone();
    let mut finish = EdgeFinish { id, kind: edge_kind(name), spans: ends.clone(), width, ratio: if strip_kind(name) == Some(StripKind::Elastic) { ratio } else { 1.0 }, strength: strength.unwrap_or(0.0), fabric, generated: Vec::new() };
    if let Some(kind) = strip_kind(name) {
        let strip = build_strip(&s.garment, kind, id, &ends, width, ratio, fabric).map_err(geometry)?;
        let tension = if kind == StripKind::Elastic { strength.unwrap_or(1.0) } else { 1.0 };
        let seam = Seam { id: next_seam_id(&s.garment), a: strip.edge, b: ends, reversed: true, fold: SeamFold::Turned, tension, ..Seam::default() };
        finish.generated.push(strip.panel.id);
        s.garment.panels.push(strip.panel);
        s.garment.seams.push(seam);
    }
    s.garment.finishes.push(finish.clone());
    let out = describe(&s.garment, &finish);
    record(s, "Add Edge Finish", before);
    Ok(out)
}

fn set_schema() -> Value {
    obj(
        json!({
            "finish": ident("Finish id as returned by finish.add."),
            "width": num("New width, in the range of the finish kind (see finish.add).", 1.0, 100.0, Some("mm")),
            "ratio": num("Elastic only: rest length over drawn length, 0.2 to 2.", 0.2, 2.0, None),
            "strength": num("Elastic seam tension, or stiffness multiplier of tape, bond and press, 0 to 20.", 0.0, 20.0, None),
            "percent": num("Skive only: percent shaved down, 0 to 100.", 0.0, 100.0, Some("percent")),
            "fabric": ident("Piping, binding and elastic only: fabric id of the strip.")
        }),
        &["finish"],
    )
}

fn kind_text(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Piping => "piping",
        EdgeKind::Binding => "binding",
        EdgeKind::Elastic => "elastic",
        EdgeKind::Tape => "tape",
        EdgeKind::Bond => "bond",
        EdgeKind::Press => "press",
        EdgeKind::Skive => "skive",
    }
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let target = required_id(a, "finish")?;
    let index = s.garment.finishes.iter().position(|f| f.id == target).ok_or_else(|| missing(format!("no finish with id {target}")))?;
    let old = s.garment.finishes[index].clone();
    let name = kind_text(old.kind);
    let strip = strip_kind(name);
    let (lo, hi, _) = width_range(name);
    let width = number_or(a, "width", old.width);
    if !(lo..=hi).contains(&width) {
        return Err(bad(format!("width for {name} must be between {lo} and {hi} mm")));
    }
    if a.get("ratio").is_some() && strip != Some(StripKind::Elastic) {
        return Err(bad("ratio applies to elastic only"));
    }
    if a.get("fabric").is_some() && strip.is_none() {
        return Err(bad("fabric applies to piping, binding and elastic only"));
    }
    let ratio = number_or(a, "ratio", old.ratio);
    if a.get("percent").is_some() && old.kind != EdgeKind::Skive {
        return Err(bad("percent applies to skive only"));
    }
    let skive = old.kind == EdgeKind::Skive;
    if skive && a.get("strength").is_some() {
        return Err(bad("a skive takes percent, not strength"));
    }
    let strength = if skive { number_or(a, "percent", old.strength) } else { number_or(a, "strength", old.strength) };
    if !(0.0..=if skive { 100.0 } else { 20.0 }).contains(&strength) || (strip == Some(StripKind::Elastic) && !(0.2..=2.0).contains(&ratio)) {
        return Err(bad("strength must be between 0 and 20 and ratio between 0.2 and 2"));
    }
    let fabric = if a.get("fabric").is_some() { id(a, "fabric") } else { old.fabric };
    if let Some(f) = fabric {
        if !s.garment.fabrics.iter().any(|x| x.id == f) {
            return Err(missing(format!("no fabric with id {f}")));
        }
    }
    let mut next = EdgeFinish { width, ratio, strength, fabric, ..old.clone() };
    if next == old {
        return Ok(json!({"changed": false, "finish": describe(&s.garment, &old)}));
    }
    let before = s.garment.clone();
    if let Some(kind) = strip {
        s.garment.panels.retain(|p| !old.generated.contains(&p.id));
        sew::prune(&mut s.garment);
        let made = build_strip(&s.garment, kind, old.id, &old.spans, width, ratio, fabric).map_err(geometry);
        let made = match made {
            Ok(m) => m,
            Err(e) => {
                s.garment = before;
                return Err(e);
            }
        };
        let tension = if kind == StripKind::Elastic { if a.get("strength").is_some() { strength } else if old.strength > 0.0 { old.strength } else { 1.0 } } else { 1.0 };
        let seam = Seam { id: next_seam_id(&s.garment), a: made.edge, b: old.spans.clone(), reversed: true, fold: SeamFold::Turned, tension, ..Seam::default() };
        next.generated = vec![made.panel.id];
        s.garment.panels.push(made.panel);
        s.garment.seams.push(seam);
    }
    s.garment.finishes[index] = next.clone();
    let out = describe(&s.garment, &next);
    record(s, "Set Edge Finish", before);
    Ok(json!({"changed": true, "finish": out}))
}

fn delete_schema() -> Value {
    obj(json!({"finish": ident("Finish id as returned by finish.add."), "finishes": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 1}, "description": "Several finish ids."}}), &[])
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let mut ids = if a.get("finishes").is_some() { id_list(a, "finishes")? } else { Vec::new() };
    ids.extend(id(a, "finish"));
    if ids.is_empty() {
        return Err(bad("give finish or finishes"));
    }
    if let Some(gone) = ids.iter().find(|i| !s.garment.finishes.iter().any(|f| f.id == **i)) {
        return Err(missing(format!("no finish with id {gone}")));
    }
    let before = s.garment.clone();
    let doomed: Vec<u32> = s.garment.finishes.iter().filter(|f| ids.contains(&f.id)).flat_map(|f| f.generated.iter().copied()).collect();
    s.garment.finishes.retain(|f| !ids.contains(&f.id));
    let (panels, seams) = (s.garment.panels.len(), s.garment.seams.len());
    s.garment.panels.retain(|p| !doomed.contains(&p.id));
    sew::prune(&mut s.garment);
    trim::prune(&mut s.garment);
    let out = json!({"changed": true, "removed": ids.len(), "panels_removed": panels - s.garment.panels.len(), "seams_removed": seams - s.garment.seams.len(), "remaining": s.garment.finishes.len()});
    record(s, "Delete Edge Finish", before);
    Ok(out)
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "finish.add",
            mcp: "finish_add",
            group: Group::Sew,
            label: "Add Edge Finish",
            description: "Add an edge finish along spans. piping and binding create a strip panel of the given width, sewn to the spans by a real seam, and record it in generated. elastic and shirring create an elastic strip that shrinks to ratio and pulls the edge in. tape, bond and press stiffen a band of the given half width along the spans. crease adds a fold line from the start of the first span to the end of the last. One undo step.",
            keys: &["Shift+E"],
            hints: WRITE,
            schema: add_schema,
            run: add,
        },
        Command {
            id: "finish.set",
            mcp: "finish_set",
            group: Group::Sew,
            label: "Set Edge Finish",
            description: "Change the width, ratio (elastic), strength or fabric (piping, binding, elastic) of an edge finish. Strip finishes regenerate their strip panel and seam, so panel and seam ids change. One undo step.",
            keys: &[],
            hints: WRITE,
            schema: set_schema,
            run: set,
        },
        Command {
            id: "finish.delete",
            mcp: "finish_delete",
            group: Group::Sew,
            label: "Delete Edge Finish",
            description: "Remove edge finishes together with the strip panels they generated and every seam that touches those panels, in one undo step.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
    ]
}
