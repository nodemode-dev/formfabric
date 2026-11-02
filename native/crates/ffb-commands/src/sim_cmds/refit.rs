use std::sync::atomic::Ordering;

use ffb_body::resize::{self, Targets};
use ffb_body::{light_copy, measure, skin, Body, BodyChange};
use ffb_sim::Collider;
use serde_json::{json, Value};

use super::adapter;
use super::apply::posed_panels;
use super::build::{ensure, rebuild};
use super::runtime::{lock, publish};
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

const BODY_OBJECT_BASE: u32 = 1_000_000;
const MAX_STEPS: u64 = 400;
const FOLLOW_LIMIT: f32 = 40.0;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "from": {"type": "integer", "minimum": 1, "description": "Body id the garment is draped on. Defaults to the first visible body."},
            "to": {"type": "integer", "minimum": 1, "description": "Body id to move the garment to."},
            "steps": {"type": "integer", "minimum": 1, "maximum": MAX_STEPS, "default": 20, "description": "Increments along the body morph path."},
            "frames": {"type": "integer", "minimum": 1, "maximum": 30, "default": 4, "description": "Solver frames run at each increment."},
            "settle": {"type": "integer", "minimum": 0, "maximum": 600, "default": 40, "description": "Frames run on the target body at the end."},
            "fix_frames": {"type": "integer", "minimum": 0, "maximum": 1000, "default": 150, "description": "Frame budget after the settle: each frame pushes cloth out of the target body and steps with collisions on, until the deepest vertex is under 1 mm inside or the budget is spent. The result reports inside_mm and the frames used."},
            "method": {
                "type": "string",
                "enum": ["auto", "blend", "sweep", "redrape"],
                "default": "auto",
                "description": "blend: bodies of identical topology, per-vertex linear blend from A to B. sweep: other bodies, body A is stepped through a measurement sweep to B's measurements, then the garment settles on B. redrape: no morph, the garment returns to its placed layout and drapes on B. auto picks blend when the topology matches, else sweep."
            }
        },
        "required": ["to"],
        "additionalProperties": false
    })
}

fn find(s: &Session, id: u64) -> Result<usize, CommandError> {
    s.bodies.iter().position(|b| u64::from(b.id) == id).ok_or_else(|| CommandError::new("not_found", format!("no body with id {id}")))
}

fn same_topology(a: &Body, b: &Body) -> bool {
    if a.positions.is_empty() || a.positions.len() != b.positions.len() || a.triangles.len() != b.triangles.len() {
        return false;
    }
    if a.triangles == b.triangles {
        return true;
    }
    let canon = |body: &Body| {
        let mut out: Vec<[u32; 3]> = body
            .triangles
            .iter()
            .map(|t| {
                let mut s = *t;
                s.sort_unstable();
                s
            })
            .collect();
        out.sort_unstable();
        out
    };
    canon(a) == canon(b)
}

fn lerp(a: &[[f32; 3]], b: &[[f32; 3]], t: f32) -> Vec<[f32; 3]> {
    a.iter().zip(b).map(|(p, q)| [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t, p[2] + (q[2] - p[2]) * t]).collect()
}

fn sweep_shape(a: &Body, from: &std::collections::BTreeMap<String, f64>, to: &std::collections::BTreeMap<String, f64>, t: f64) -> Result<Vec<[f32; 3]>, CommandError> {
    let pick = |key: &str| match (from.get(key), to.get(key)) {
        (Some(x), Some(y)) if *x > 1.0 && *y > 1.0 => Some(x + (y - x) * t),
        _ => None,
    };
    let targets = Targets {
        height: pick("height"),
        inseam: pick("inseam"),
        chest: pick("chest"),
        waist: pick("waist"),
        hip: pick("hip"),
        shoulder_width: pick("shoulder_width"),
        arm_length: pick("arm_length"),
    };
    let mut moved = a.clone();
    if targets.any() {
        resize::resize(&mut moved, &targets).map_err(|e| CommandError::new("invalid_geometry", format!("the measurement sweep failed: {e}")))?;
    }
    Ok(skin::posed_positions(&moved))
}

const TIME_BUDGET_SECONDS: f32 = 60.0;

#[cfg(not(target_arch = "wasm32"))]
fn clock() -> std::time::Instant {
    std::time::Instant::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn late(began: &std::time::Instant) -> bool {
    began.elapsed().as_secs_f32() > TIME_BUDGET_SECONDS
}

#[cfg(target_arch = "wasm32")]
fn clock() {}

#[cfg(target_arch = "wasm32")]
fn late(_: &()) -> bool {
    false
}

fn restore_visible(s: &mut Session, old: &[(usize, bool)]) {
    for (index, visible) in old {
        s.bodies[*index].visible = *visible;
    }
}

struct Grid<'a> {
    points: &'a [[f32; 3]],
    cells: std::collections::HashMap<[i32; 3], Vec<u32>>,
}

fn cell(p: [f32; 3]) -> [i32; 3] {
    [(p[0] / FOLLOW_LIMIT).floor() as i32, (p[1] / FOLLOW_LIMIT).floor() as i32, (p[2] / FOLLOW_LIMIT).floor() as i32]
}

impl<'a> Grid<'a> {
    fn new(points: &'a [[f32; 3]]) -> Grid<'a> {
        let mut cells: std::collections::HashMap<[i32; 3], Vec<u32>> = std::collections::HashMap::new();
        for (i, p) in points.iter().enumerate() {
            cells.entry(cell(*p)).or_default().push(i as u32);
        }
        Grid { points, cells }
    }
}

fn nearest(grid: &Grid, p: [f32; 3]) -> Option<usize> {
    let c = cell(p);
    let mut best: Option<(usize, f32)> = None;
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let Some(list) = grid.cells.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) else { continue };
                for &i in list {
                    let q = grid.points[i as usize];
                    let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2);
                    if best.is_none_or(|b| d < b.1) {
                        best = Some((i as usize, d));
                    }
                }
            }
        }
    }
    best.filter(|b| b.1.sqrt() <= FOLLOW_LIMIT).map(|b| b.0)
}

fn flat(points: &[[f32; 3]]) -> Vec<f32> {
    points.iter().flat_map(|p| p.iter().copied()).collect()
}

fn scene(s: &Session, skip: &[u32], moving: Collider) -> Vec<Collider> {
    let others: Vec<Body> = s.bodies.iter().filter(|b| !skip.contains(&b.id)).cloned().collect();
    let mut all = adapter::body_colliders(&s.garment, &others);
    all.push(moving);
    all.push(adapter::floor());
    all
}

fn refit(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let to_id = a.get("to").and_then(Value::as_u64).ok_or_else(|| bad("to must be a body id"))?;
    let from_id = match a.get("from").filter(|v| !v.is_null()) {
        Some(v) => v.as_u64().ok_or_else(|| bad("from must be a body id"))?,
        None => u64::from(s.bodies.iter().find(|b| b.visible && b.id != to_id as u32).ok_or_else(|| CommandError::new("not_found", "there is no visible body to refit from"))?.id),
    };
    let steps = a.get("steps").and_then(Value::as_u64).unwrap_or(20);
    let frames = a.get("frames").and_then(Value::as_u64).unwrap_or(4);
    let settle = a.get("settle").and_then(Value::as_u64).unwrap_or(40);
    if !(1..=MAX_STEPS).contains(&steps) || !(1..=30).contains(&frames) || settle > 600 {
        return Err(bad("steps must be 1 to 400, frames 1 to 30, settle 0 to 600"));
    }
    if from_id == to_id {
        return Err(bad("from and to are the same body"));
    }
    let (ia, ib) = (find(s, from_id)?, find(s, to_id)?);
    let (body_a, body_b) = (s.bodies[ia].clone(), s.bodies[ib].clone());
    if body_a.triangles.is_empty() || body_b.triangles.is_empty() {
        return Err(bad("both bodies need a mesh"));
    }
    let wanted = a.get("method").and_then(Value::as_str).unwrap_or("auto");
    let method = match wanted {
        "auto" if same_topology(&body_a, &body_b) => "blend",
        "auto" => "sweep",
        "blend" if same_topology(&body_a, &body_b) => "blend",
        "blend" => return Err(bad("blend needs two bodies of identical topology; use sweep or redrape")),
        "sweep" | "redrape" => wanted,
        _ => return Err(bad("method must be auto, blend, sweep or redrape")),
    };
    ensure(s)?;
    s.sim.running.store(false, Ordering::Release);
    s.sim.generation.fetch_add(1, Ordering::AcqRel);
    let before = s.garment.clone();
    let skip = [body_a.id, body_b.id];
    let posed_a = skin::posed_positions(&body_a);
    let posed_b = skin::posed_positions(&body_b);
    let core = s.sim.core.clone();
    let sink = s.sink.clone();
    let grid = Grid::new(&posed_a);
    let began = clock();
    let follow: Vec<(u32, [f32; 3], Option<usize>)> = {
        let mut c = lock(&core);
        c.sink = sink.clone();
        c.pinch = None;
        let first = c.base.positions.len() - c.extras.anchors.len();
        c.extras.anchors.iter().enumerate().map(|(k, an)| ((first + k) as u32, an.point, nearest(&grid, an.point))).collect()
    };
    let (m_from, m_to) = if method == "sweep" { (measure::measure(&body_a), measure::measure(&body_b)) } else { Default::default() };
    let mut last_shape = posed_a.clone();
    if method != "redrape" {
        for k in 1..=steps {
            if late(&began) {
                break;
            }
            let t = k as f64 / steps as f64;
            let shape = if method == "blend" { lerp(&posed_a, &posed_b, t as f32) } else { sweep_shape(&body_a, &m_from, &m_to, t)? };
            let colliders = scene(s, &skip, adapter::mesh_collider(shape.clone(), &body_a.triangles));
            let snapshot = {
                let mut c = lock(&core);
                for (vertex, point, near) in &follow {
                    let Some(i) = near else { continue };
                    let moved = [point[0] + shape[*i][0] - posed_a[*i][0], point[1] + shape[*i][1] - posed_a[*i][1], point[2] + shape[*i][2] - posed_a[*i][2]];
                    if let Some(pin) = c.pins.iter_mut().find(|p| p.vertex == *vertex) {
                        pin.target = moved;
                    }
                }
                let pins = c.pins.clone();
                if let Some(st) = c.stepper.as_mut() {
                    st.set_colliders(colliders);
                    st.set_pins(pins);
                    st.push_out(0.0);
                }
                c.step_frames(frames as u32);
                c.snapshot()
            };
            publish(&sink, &snapshot);
            if let Some(sink) = &sink {
                sink.positions(BODY_OBJECT_BASE + body_a.id, &flat(&shape));
            }
            last_shape = shape;
        }
    }
    let mut changes = Vec::new();
    let old_visible = [(ia, s.bodies[ia].visible), (ib, s.bodies[ib].visible)];
    for (index, visible) in [(ia, false), (ib, true)] {
        let old = light_copy(&s.bodies[index]);
        s.bodies[index].visible = visible;
        if old.visible != visible {
            let after = light_copy(&s.bodies[index]);
            changes.push(BodyChange { id: s.bodies[index].id, index, before: Some(Box::new(old)), after: Some(Box::new(after)), light: true });
        }
    }
    if method == "redrape" {
        if let Err(e) = rebuild(s) {
            restore_visible(s, &old_visible);
            return Err(e);
        }
        lock(&core).reset();
    } else {
        let colliders = scene(s, &skip, adapter::mesh_collider(posed_b.clone(), &body_b.triangles));
        let mut c = lock(&core);
        if let Some(st) = c.stepper.as_mut() {
            st.set_colliders(colliders);
            st.push_out(1.0);
        }
    }
    let snapshot = {
        let mut c = lock(&core);
        c.step_frames(settle as u32);
        c.snapshot()
    };
    publish(&sink, &snapshot);
    let budget = a.get("fix_frames").and_then(Value::as_u64).unwrap_or(150).min(1000);
    let mut fixed = 0;
    let inside = {
        let mut c = lock(&core);
        loop {
            let depth = c.stepper.as_ref().map_or(0.0, |st| st.max_penetration());
            if depth < 1.0 || fixed >= budget {
                break depth;
            }
            if let Some(st) = c.stepper.as_mut() {
                st.push_out(1.0);
            }
            c.step_frames(1);
            fixed += 1;
        }
    };
    let snapshot = lock(&core).snapshot();
    publish(&sink, &snapshot);
    let panels = if method == "redrape" {
        Vec::new()
    } else {
        match posed_panels(s) {
            Ok(p) => p,
            Err(e) => {
                restore_visible(s, &old_visible);
                return Err(e);
            }
        }
    };
    let mut moved_panels = Vec::new();
    for (index, _, after) in panels {
        moved_panels.push(after.id);
        s.garment.panels[index] = after;
    }
    if method != "redrape" {
        for tack in s.garment.tacks.iter_mut() {
            for end in [&mut tack.a, &mut tack.b] {
                let Some(p) = end.body_point else { continue };
                let q = [p[0] as f32, p[1] as f32, p[2] as f32];
                if let Some(i) = nearest(&grid, q) {
                    end.body_point = Some([f64::from(q[0] + last_shape[i][0] - posed_a[i][0]), f64::from(q[1] + last_shape[i][1] - posed_a[i][1]), f64::from(q[2] + last_shape[i][2] - posed_a[i][2])]);
                }
            }
        }
    }
    let after = s.garment.clone();
    let tapes = s.garment.tacks.iter().filter(|t| t.a.body_point.is_some() || t.b.body_point.is_some()).count();
    let mut edits = Vec::new();
    if after != before {
        edits.push(Edit::Garment { label: "Refit".into(), before: Box::new(before), after: Box::new(after) });
    }
    if !changes.is_empty() {
        edits.push(Edit::Bodies { label: "Refit".into(), changes });
    }
    if !edits.is_empty() {
        s.record(Edit::Many { label: "Refit".into(), edits });
    }
    Ok(json!({
        "method": method,
        "from": from_id,
        "to": to_id,
        "steps": if method == "redrape" { 0 } else { steps },
        "frames_per_step": frames,
        "settle": settle,
        "fix_frames": fixed,
        "inside_mm": inside,
        "panels": moved_panels,
        "body_tapes": tapes,
        "note": if method == "redrape" && tapes > 0 { "body tapes keep their old body points; set them again on the new body" } else { "" }
    }))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "refit",
        mcp: "refit",
        group: Group::Sim,
        label: "Refit",
        description: "Move the draped garment from body A to body B without opening a seam. The body is morphed from A to B in steps while the solver runs, so the garment follows only by simulation and contact: no reference garment, no correspondence map between the bodies, no stored fit data, no pattern change. Bodies of identical topology blend per vertex; other bodies are stepped through a measurement sweep of body A and then the garment settles on B, or use method redrape to drape from the placed layout on B. Ends by hiding A, showing B and saving the draped shape (one undo step).",
        keys: &[],
        hints: WRITE,
        schema,
        run: refit,
    }]
}
