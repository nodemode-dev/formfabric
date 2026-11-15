mod draft;
mod fit;
mod info;

use ffb_pattern::Place3D;
use serde_json::{json, Value};

use crate::{dispatch, Command, CommandError, Edit, Group, Session, DESTROY, READ};

use draft::{Draft, SpanRef};

const NAMES: [(&str, &str, &str); 3] = [
    ("tshirt", "Basic T-Shirt", "Front, back and two set-in sleeves drafted from the body bust, hip, shoulder width, arm length and neck."),
    ("skirt", "A-Line Skirt", "Front and back panels with a waistband, drafted from the body waist and hip."),
    ("dress", "Sleeveless Dress", "Front and back dress panels with scooped armholes and neckline, fitted at the bust and flaring from the hip."),
];

fn list_schema() -> Value {
    crate::no_args()
}

fn list(_: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let items: Vec<Value> = NAMES.iter().map(|(n, l, d)| json!({"name": n, "label": l, "description": d})).collect();
    Ok(json!({"samples": items}))
}

fn load_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "enum": ["tshirt", "skirt", "dress"], "description": "Sample garment, as listed by sample.list."},
            "body": {"type": ["integer", "string"], "description": "Body id in the project, or the name of a library body to load (MPFB2 Female or MPFB2 Male). Defaults to the first body, or MPFB2 Female when the project has none."},
            "force": {"type": "boolean", "default": false, "description": "Replace a project that has unsaved edits. Without it the call fails with the code unsaved_changes so the caller can ask the user to confirm."}
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn no_skeleton() -> CommandError {
    CommandError::new("invalid_geometry", "this body has no skeleton with arms and a neck, so a sample cannot be drafted on it")
}

fn pick_body(s: &mut Session, a: &Value) -> Result<usize, CommandError> {
    let kept = |s: &mut Session, i: usize| -> Result<usize, CommandError> {
        if info::Info::new(&s.bodies[i]).is_none() {
            return Err(no_skeleton());
        }
        let bodies = std::mem::take(&mut s.bodies);
        s.reset(crate::fabric_cmds::new_garment());
        s.bodies = bodies;
        Ok(i)
    };
    let fresh = |s: &mut Session, name: &str| -> Result<usize, CommandError> {
        let had = s.bodies.len();
        dispatch(s, "body.load_default", &json!({"name": name}))?;
        if s.bodies.get(had).is_none_or(|b| info::Info::new(b).is_none()) {
            return Err(no_skeleton());
        }
        let mut body = s.bodies.split_off(had);
        dispatch(s, "file.new", &json!({}))?;
        body.iter_mut().for_each(|b| b.id = 1);
        s.bodies = body;
        Ok(0)
    };
    match a.get("body").filter(|v| !v.is_null()) {
        Some(Value::Number(n)) => {
            let id = n.as_u64().ok_or_else(|| bad("body must be a whole body id"))?;
            let i = s.bodies.iter().position(|b| u64::from(b.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no body with id {id}")))?;
            kept(s, i)
        }
        Some(Value::String(name)) => match s.bodies.iter().position(|b| &b.name == name) {
            Some(i) => kept(s, i),
            None => fresh(s, name),
        },
        _ => {
            if s.bodies.is_empty() {
                fresh(s, "MPFB2 Female")
            } else {
                kept(s, 0)
            }
        }
    }
}

fn span_values(ids: &[u32], d: &Draft, refs: &[SpanRef]) -> Vec<Value> {
    let mut out = Vec::new();
    for r in refs {
        let Some((first, count)) = d.panels[r.panel].run(&r.run) else { continue };
        match r.t {
            Some((t0, t1)) => out.push(json!({"panel": ids[r.panel], "segment": first, "t0": t0, "t1": t1})),
            None => {
                for i in first..first + count {
                    out.push(json!({"panel": ids[r.panel], "segment": i, "t0": 0.0, "t1": 1.0}));
                }
            }
        }
    }
    out
}

fn load(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = a.get("name").and_then(Value::as_str).ok_or_else(|| bad("name is required"))?;
    if !NAMES.iter().any(|n| n.0 == name) {
        return Err(bad(format!("no sample named {name}; sample.list shows them")));
    }
    let force = a.get("force").and_then(Value::as_bool).unwrap_or(false);
    if !force && !s.garment.panels.is_empty() && s.history.can_undo() {
        return Err(CommandError::new("unsaved_changes", "the open project has edits that a sample would replace together with its undo history; save it first, or call again with force true to replace it"));
    }
    let bi = pick_body(s, a)?;
    let info = info::Info::new(&s.bodies[bi]).ok_or_else(|| CommandError::new("invalid_geometry", "this body has no skeleton with arms and a neck, so a sample cannot be drafted on it"))?;
    let draft = match name {
        "tshirt" => draft::tshirt(&info),
        "skirt" => draft::skirt(&info),
        _ => draft::dress(&info),
    };
    let mut ids: Vec<u32> = Vec::new();
    let mut origins: Vec<[f64; 2]> = Vec::new();
    for p in &draft.panels {
        let r = dispatch(s, "pattern.draw_panel", &json!({"points": p.json_points(), "name": p.name, "particle_distance": p.pd}))?;
        ids.push(r["id"].as_u64().unwrap_or(0) as u32);
        let pos = r["position"].as_array().map(|v| [v[0].as_f64().unwrap_or(0.0), v[1].as_f64().unwrap_or(0.0)]).unwrap_or([0.0; 2]);
        origins.push(pos);
    }
    for (pi, p) in draft.panels.iter().enumerate() {
        for (run, t) in &p.notches {
            if let Some((first, count)) = p.run(run) {
                let seg = if *t >= 1.0 { first + count - 1 } else { first };
                let tt = if *t >= 1.0 { 1.0 } else { *t };
                dispatch(s, "pattern.notch_add", &json!({"panel": ids[pi], "segment": seg, "t": tt}))?;
            }
        }
    }
    let mut seams = Vec::new();
    for sd in &draft.seams {
        let (sa, sb) = (span_values(&ids, &draft, &sd.a), span_values(&ids, &draft, &sd.b));
        let r = dispatch(s, "sew.multi", &json!({"a": sa, "b": sb, "reversed": true}))?;
        seams.push(json!({"name": sd.name, "id": r["id"], "mismatch": r["mismatch"]}));
    }
    let fitter = fit::Fitter::new(&info, &draft);
    let before = s.garment.clone();
    let mut counts = Vec::new();
    for (pi, id) in ids.iter().enumerate() {
        let index = s.garment.index_of(*id).ok_or_else(|| CommandError::new("not_found", format!("panel {id} vanished")))?;
        let panel = s.garment.panels[index].clone();
        let built = s.mesh.ensure(&panel, false).built.map_err(|e| CommandError::new("invalid_geometry", format!("panel {id} cannot be meshed: {e}")))?;
        let o = origins[pi];
        let pose: Vec<[f32; 3]> = built
            .mesh
            .positions
            .iter()
            .map(|p| {
                let v = fitter.point(pi, [p[0] + o[0], p[1] + o[1]]);
                [v[0] as f32, v[1] as f32, v[2] as f32]
            })
            .collect();
        counts.push(json!({"id": id, "name": panel.name, "vertices": pose.len()}));
        s.garment.panels[index].place3d = Some(Place3D { origin: [0.0; 3], rotation: [0.0, 0.0, 0.0, 1.0], bend: 0.0, pose: Some(pose) });
    }
    let after = s.garment.clone();
    s.record(Edit::Garment { label: "Place Sample".into(), before: Box::new(before), after: Box::new(after) });
    s.history = Default::default();
    let cuffs: Vec<Value> = (0..draft.panels.len()).filter_map(|pi| fitter.cuff_axial(pi)).map(|c| json!(c)).collect();
    let marks = json!({"body": info.name, "floor": info.y_floor, "crotch": info.y_crotch, "chest": info.y_chest, "waist": info.y_waist, "hip": info.y_hip, "armpit": info.y_armpit, "neck": info.y_neck, "knee": info.y_knee, "shoulder_top": info.y_sp});
    Ok(json!({"history_cleared": true, "sample": name, "title": draft.title, "body": s.bodies[bi].id, "body_name": s.bodies[bi].name, "panels": counts, "seams": seams, "draft": draft.report, "landmarks": marks, "cuff_axial": cuffs}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "sample.list",
            mcp: "sample_list",
            group: Group::File,
            label: "Samples",
            description: "List the sample garments: a basic T-shirt, an A-line skirt with waistband and a sleeveless dress. Each is drafted from the body measurements.",
            keys: &[],
            hints: READ,
            schema: list_schema,
            run: list,
        },
        Command {
            id: "sample.load",
            mcp: "sample_load",
            group: Group::File,
            label: "Open Sample",
            description: "Replace the garment with a sample drafted on the given or current body: the pattern panels are drawn from the body measurements, sewn, and placed around the body, ready for sim.reset and sim.run. Keeps the body and discards the previous garment and the whole undo history (the result says history_cleared). Fails with the code unsaved_changes when the open project has edits, unless force is true.",
            keys: &[],
            hints: DESTROY,
            schema: load_schema,
            run: load,
        },
    ]
}
