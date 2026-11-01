use std::collections::BTreeMap;

use ffb_pattern::sew::{lengths_of, Binding};
use serde_json::{json, Value};

use super::common::*;
use super::sync::session_bindings;
use crate::{Command, CommandError, Group, Session, READ};

fn list_schema() -> Value {
    obj(json!({"seam": panel_id("Only this seam.")}), &[])
}

fn list(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let only = id(a, "seam");
    if let Some(i) = only {
        seam_index(s, i)?;
    }
    let seams: Vec<Value> = s.garment.seams.iter().filter(|x| only.is_none_or(|i| i == x.id)).map(|x| describe(&s.garment, x)).collect();
    let tacks = serde_json::to_value(&s.garment.tacks).unwrap_or(Value::Null);
    Ok(json!({"count": seams.len(), "seams": seams, "tacks": tacks}))
}

fn check_schema() -> Value {
    obj(json!({"tolerance": num("Largest allowed difference between side b and ratio times side a.", 0.0, 100_000.0, Some("mm"))}), &[])
}

fn check(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let tolerance = number_or(a, "tolerance", 1.0);
    if tolerance < 0.0 {
        return Err(bad("tolerance must not be negative"));
    }
    let mut off = Vec::new();
    for seam in &s.garment.seams {
        match lengths_of(&s.garment, seam) {
            Some(l) if l.mismatch.abs() > tolerance => {
                off.push(json!({"id": seam.id, "length_a": l.a, "length_b": l.b, "ratio": seam.ratio, "expected_b": l.a * seam.ratio, "mismatch": l.mismatch}));
            }
            Some(_) => {}
            None => off.push(json!({"id": seam.id, "error": "a side refers to a segment that no longer exists"})),
        }
    }
    Ok(json!({"tolerance": tolerance, "checked": s.garment.seams.len(), "ok": off.is_empty(), "mismatched": off}))
}

fn bindings_schema() -> Value {
    obj(json!({"seam": panel_id("Only this seam.")}), &[])
}

fn bindings(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let only = id(a, "seam");
    if let Some(i) = only {
        seam_index(s, i)?;
    }
    let all = session_bindings(s);
    let mut per: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
    for b in &all {
        let (seam, edge) = match b {
            Binding::Pair { seam, .. } => (*seam, false),
            Binding::Edge { seam, .. } => (*seam, true),
        };
        let entry = per.entry(seam).or_default();
        if edge {
            entry.1 += 1;
        } else {
            entry.0 += 1;
        }
    }
    let rows: Vec<Value> = per.iter().filter(|(k, _)| only.is_none_or(|i| i == **k)).map(|(k, v)| json!({"seam": k, "pairs": v.0, "edges": v.1})).collect();
    let (pairs, edges) = rows.iter().fold((0, 0), |acc, r| (acc.0 + r["pairs"].as_u64().unwrap_or(0), acc.1 + r["edges"].as_u64().unwrap_or(0)));
    Ok(json!({"pairs": pairs, "edges": edges, "seams": rows}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "sew.list",
            mcp: "sew_list",
            group: Group::Sew,
            label: "Seam List",
            description: "Every seam with the length of each side in mm and the mismatch (side b minus ratio times side a), plus the tacks.",
            keys: &[],
            hints: READ,
            schema: list_schema,
            run: list,
        },
        Command {
            id: "sew.check",
            mcp: "sew_check",
            group: Group::Sew,
            label: "Seam Check",
            description: "List the seams whose side lengths differ by more than a tolerance (default 1 mm), measured as side b minus ratio times side a.",
            keys: &[],
            hints: READ,
            schema: check_schema,
            run: check,
        },
        Command {
            id: "sew.bindings",
            mcp: "sew_bindings",
            group: Group::Sew,
            label: "Seam Bindings",
            description: "Count the solver bindings the seams produce on the current meshes: vertex pairs and vertex-to-edge bindings, per seam.",
            keys: &[],
            hints: READ,
            schema: bindings_schema,
            run: bindings,
        },
    ]
}
