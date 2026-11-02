use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use ffb_mesh::layout;
use ffb_sim::PinPoint;
use serde_json::{json, Value};

use super::adapter::Stepper;
use super::assemble::nearest_vertex;
use super::build::ensure;
use super::runtime::{lock, publish};
use crate::{Command, CommandError, Group, Session, WRITE};

const MAX_PINCH_FRAMES: u64 = 120;

#[derive(Debug, Clone)]
pub(crate) struct Pinch {
    pub key: String,
    pub origin: [f32; 3],
    pub members: Vec<(u32, f32, [f32; 3])>,
}

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn missing(message: impl Into<String>) -> CommandError {
    CommandError::new("not_found", message)
}

pub(crate) fn current_positions(s: &mut Session, id: u32) -> Result<Vec<[f32; 3]>, CommandError> {
    let index = s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))?;
    let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
    let n = built.mesh.positions.len();
    {
        let c = lock(&s.sim.core);
        if let (Some(st), Some(sl)) = (c.stepper.as_ref(), c.slices.iter().find(|sl| sl.id == id && sl.count == n)) {
            if let Some(x) = st.positions().get(sl.start..sl.start + n) {
                return Ok(x.to_vec());
            }
        }
    }
    let panel = &s.garment.panels[index];
    Ok(match panel.place3d.as_ref().and_then(|p| p.pose.as_ref()).filter(|p| p.len() == n) {
        Some(pose) => pose.clone(),
        None => layout::positions_3d(&built.mesh, panel).as_chunks::<3>().0.to_vec(),
    })
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[derive(PartialEq)]
struct Item(f32, u32);

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}

fn surface(x: &[[f32; 3]], triangles: &[[u32; 3]], from: u32, radius: f32) -> HashMap<u32, f32> {
    let mut next: HashMap<u32, Vec<u32>> = HashMap::new();
    for t in triangles {
        for k in 0..3 {
            next.entry(t[k]).or_default().push(t[(k + 1) % 3]);
            next.entry(t[(k + 1) % 3]).or_default().push(t[k]);
        }
    }
    let mut best: HashMap<u32, f32> = HashMap::new();
    let mut heap = BinaryHeap::new();
    best.insert(from, 0.0);
    heap.push(Item(0.0, from));
    while let Some(Item(d, v)) = heap.pop() {
        if best.get(&v).is_some_and(|b| *b < d) {
            continue;
        }
        for &w in next.get(&v).map_or(&[][..], |l| &l[..]) {
            let nd = d + dist(x[v as usize], x[w as usize]);
            if nd <= radius && best.get(&w).is_none_or(|b| nd < *b) {
                best.insert(w, nd);
                heap.push(Item(nd, w));
            }
        }
    }
    best
}

fn weight(falloff: &str, t: f32, power: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let w = if falloff == "linear" { 1.0 - t } else { (1.0 - t * t).powi(2) };
    w.powf(2.0f32.powf(power))
}

struct Spec<'a> {
    falloff: &'a str,
    radius: f32,
    power: f32,
    surface: bool,
}

fn members(stepper: &Stepper, solid: usize, triangles: &[[u32; 3]], pick: u32, spec: &Spec, pinned: &[bool]) -> Vec<(u32, f32, [f32; 3])> {
    let x = stepper.positions();
    let mut out = Vec::new();
    let Spec { falloff, radius, power, surface: along } = *spec;
    if along {
        for (v, d) in surface(x, triangles, pick, radius) {
            if (v as usize) < solid && !pinned[v as usize] {
                out.push((v, weight(falloff, d / radius, power), x[v as usize]));
            }
        }
    } else {
        let origin = x[pick as usize];
        for (v, p) in x.iter().enumerate().take(solid) {
            let d = dist(origin, *p);
            if d <= radius && !pinned[v] {
                out.push((v as u32, weight(falloff, d / radius, power), *p));
            }
        }
    }
    out.retain(|m| m.1 > 1.0e-4);
    out.sort_by_key(|m| m.0);
    out
}

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "panel": {"type": "integer", "minimum": 1, "description": "Panel id. With vertex, the index is in that panel's mesh; with point, the nearest vertex is taken."},
            "vertex": {"type": "integer", "minimum": 0, "description": "Vertex index in the panel mesh when panel is given, else an index into the whole cloth."},
            "point": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "unit": "mm", "description": "Panel-local point (needs panel)."},
            "target": {"type": ["array", "null"], "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "unit": "mm", "description": "Where the picked vertex is pulled to in 3D. Neighbours follow by the falloff weight. Null or missing releases the pinch."},
            "falloff": {"type": "string", "enum": ["bump", "linear"], "default": "bump"},
            "radius": {"type": "number", "minimum": 1, "maximum": 500, "default": 50, "unit": "mm", "description": "Falloff distance."},
            "power": {"type": "number", "minimum": -2, "maximum": 2, "default": 0, "description": "Falloff power: above 0 narrows the weights, below 0 widens them."},
            "distance": {"type": "string", "enum": ["straight", "surface"], "default": "straight", "description": "straight: every panel within the radius. surface: along the picked panel's mesh only."},
            "frames": {"type": "integer", "minimum": 0, "maximum": MAX_PINCH_FRAMES, "default": 0, "description": "Frames to step after setting the pinch (use with a stopped simulation)."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn triple(a: &Value, key: &str) -> Result<Option<[f32; 3]>, CommandError> {
    let Some(v) = a.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let list = v.as_array().filter(|l| l.len() == 3).ok_or_else(|| bad(format!("{key} must hold 3 numbers")))?;
    let mut out = [0.0f32; 3];
    for (slot, item) in out.iter_mut().zip(list) {
        *slot = item.as_f64().filter(|x| x.is_finite() && x.abs() <= 1e7).ok_or_else(|| bad(format!("{key} must hold 3 finite numbers")))? as f32;
    }
    Ok(Some(out))
}

fn pinch(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let target = triple(a, "target")?;
    let falloff = a.get("falloff").and_then(Value::as_str).unwrap_or("bump");
    if !["bump", "linear"].contains(&falloff) {
        return Err(bad("falloff must be bump or linear"));
    }
    let radius = a.get("radius").map_or(Some(50.0), Value::as_f64).filter(|r| (1.0..=500.0).contains(r)).ok_or_else(|| bad("radius must be from 1 to 500"))? as f32;
    let power = a.get("power").map_or(Some(0.0), Value::as_f64).filter(|p| (-2.0..=2.0).contains(p)).ok_or_else(|| bad("power must be from -2 to 2"))? as f32;
    let surface_distance = match a.get("distance").and_then(Value::as_str).unwrap_or("straight") {
        "straight" => false,
        "surface" => true,
        _ => return Err(bad("distance must be straight or surface")),
    };
    let frames = a.get("frames").and_then(Value::as_u64).unwrap_or(0);
    if frames > MAX_PINCH_FRAMES {
        return Err(bad(format!("frames must be from 0 to {MAX_PINCH_FRAMES}")));
    }
    if target.is_none() {
        let mut c = lock(&s.sim.core);
        let had = c.pinch.take().is_some();
        let pins = c.pins.clone();
        if let Some(st) = c.stepper.as_mut() {
            st.set_pins(pins);
        }
        return Ok(json!({"pinched": false, "released": had}));
    }
    let panel = match a.get("panel").filter(|v| !v.is_null()) {
        Some(v) => Some(v.as_u64().and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad("panel must be a positive integer id"))?),
        None => None,
    };
    ensure(s)?;
    let local = match (a.get("vertex").and_then(Value::as_u64), a.get("point")) {
        (Some(v), None) => Some(v as usize),
        (None, Some(p)) if !p.is_null() => {
            let id = panel.ok_or_else(|| bad("point needs panel"))?;
            let list = p.as_array().filter(|l| l.len() == 2).ok_or_else(|| bad("point must hold 2 numbers"))?;
            let pt = [list[0].as_f64().ok_or_else(|| bad("point must hold numbers"))?, list[1].as_f64().ok_or_else(|| bad("point must hold numbers"))?];
            let index = s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))?;
            let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
            Some(nearest_vertex(&built.mesh, pt).ok_or_else(|| missing("the panel mesh is empty"))?)
        }
        (None, _) => None,
        _ => return Err(bad("give at most one of vertex and point")),
    };
    let mut c = lock(&s.sim.core);
    c.sink = s.sink.clone();
    let total = c.solid;
    let pick = match (local, panel) {
        (Some(v), Some(id)) => {
            let slice = c.slices.iter().find(|sl| sl.id == id).ok_or_else(|| missing(format!("panel {id} is not in the simulation")))?;
            if v >= slice.count {
                return Err(bad(format!("panel {id} has {} vertices", slice.count)));
            }
            slice.start + v
        }
        (Some(v), None) => v,
        (None, _) => match c.pinch.as_ref() {
            Some(p) => p.members.iter().max_by(|a, b| a.1.total_cmp(&b.1)).map_or(0, |m| m.0 as usize),
            None => return Err(bad("give vertex or point")),
        },
    };
    if pick >= total {
        return Err(bad(format!("the cloth has {total} vertices")));
    }
    let key = format!("{pick}|{falloff}|{radius}|{power}|{surface_distance}");
    let fresh = c.pinch.as_ref().is_none_or(|p| p.key != key);
    if fresh {
        let mut pinned = vec![false; total.max(c.base.positions.len())];
        for p in &c.pins {
            if let Some(slot) = pinned.get_mut(p.vertex as usize) {
                *slot = true;
            }
        }
        let triangles = c.base.triangles.clone();
        let Some(st) = c.stepper.as_ref() else { return Err(missing("nothing has been simulated")) };
        let list = members(st, total, &triangles, pick as u32, &Spec { falloff, radius, power, surface: surface_distance }, &pinned);
        let origin = st.positions()[pick];
        c.pinch = Some(Pinch { key, origin, members: list });
    }
    let Some(active) = c.pinch.clone() else { return Err(missing("nothing to pinch")) };
    let goal = target.unwrap_or(active.origin);
    let delta = [goal[0] - active.origin[0], goal[1] - active.origin[1], goal[2] - active.origin[2]];
    let mut pins = c.pins.clone();
    pins.extend(active.members.iter().map(|(v, w, p)| PinPoint { vertex: *v, target: [p[0] + delta[0] * w, p[1] + delta[1] * w, p[2] + delta[2] * w] }));
    if let Some(st) = c.stepper.as_mut() {
        st.set_pins(pins);
    }
    let count = active.members.len();
    let snapshot = if frames > 0 {
        c.step_frames(frames as u32);
        Some((c.snapshot(), c.sink.clone()))
    } else {
        None
    };
    drop(c);
    if let Some((snapshot, sink)) = snapshot {
        publish(&sink, &snapshot);
    }
    Ok(json!({"pinched": true, "vertex": pick, "vertices": count, "radius": radius, "falloff": falloff, "power": power}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "sim.pinch",
        mcp: "sim_pinch",
        group: Group::Sim,
        label: "Pinch",
        description: "Pull a picked vertex to a 3D target while the simulation runs, with the vertices around it following by a falloff weight (bump or linear, radius 1 to 500 mm, power -2 to 2, straight or surface distance). Send again with a new target to drag; a null target releases. Not saved and not an undo step.",
        keys: &["Shift+C"],
        hints: WRITE,
        schema,
        run: pinch,
    }]
}
