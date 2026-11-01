use std::collections::{HashMap, VecDeque};

use glam::DVec3;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::scene::{self, Scene};
use super::store::{compare_obj, edit};
use crate::{Command, CommandError, Group, Session, READ, WRITE};

const MAX_MEASURES: usize = 64;
const MAX_POINTS: usize = 256;
const MAX_CHAINS: usize = 50;
const MAX_LINE_POINTS: usize = 200;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Pt {
    #[serde(skip_serializing_if = "Option::is_none")]
    panel: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    triangle: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    w: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<[f64; 3]>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Measure {
    id: u32,
    kind: String,
    name: String,
    points: Vec<Pt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    panels: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    center: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    radius: Option<f64>,
}

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn v3(p: [f64; 3]) -> DVec3 {
    DVec3::new(p[0], p[1], p[2])
}

fn arr(p: DVec3) -> [f64; 3] {
    [p.x, p.y, p.z]
}

fn stored(g: &ffb_pattern::Garment) -> Vec<Measure> {
    g.viz.compare.get("measures").and_then(|m| serde_json::from_value(m.clone()).ok()).unwrap_or_default()
}

fn store(g: &mut ffb_pattern::Garment, list: &[Measure]) {
    compare_obj(g).insert("measures".into(), json!(list));
}

fn resolve(scene: &Scene, p: &Pt) -> Result<DVec3, CommandError> {
    if let Some(x) = p.position {
        return Ok(v3(x));
    }
    let (Some(panel), Some(tri), Some(w)) = (p.panel, p.triangle, p.w) else { return Err(bad("a point needs panel, triangle and w, or a position")) };
    let found = scene.panels.iter().find(|q| q.id == panel).ok_or_else(|| CommandError::new("not_found", format!("panel {panel} has no mesh")))?;
    let t = found.triangles.get(tri as usize).ok_or_else(|| bad(format!("panel {panel} has no triangle {tri}")))?;
    let mut out = DVec3::ZERO;
    for k in 0..3 {
        let x = found.positions.get(t[k] as usize).ok_or_else(|| bad("triangle vertex is outside the panel"))?;
        out += DVec3::new(f64::from(x[0]), f64::from(x[1]), f64::from(x[2])) * w[k];
    }
    Ok(out)
}

fn locate(s: &mut Session, panel: u32, point: [f64; 2]) -> Result<Pt, CommandError> {
    let index = s.garment.index_of(panel).ok_or_else(|| CommandError::new("not_found", format!("no panel with id {panel}")))?;
    let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
    let m = &built.mesh;
    let mut best: Option<(f64, usize, [f64; 3])> = None;
    for (k, t) in m.triangles.iter().enumerate() {
        let (a, b, c) = (m.positions[t[0] as usize], m.positions[t[1] as usize], m.positions[t[2] as usize]);
        let det = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if det.abs() < 1.0e-12 {
            continue;
        }
        let w1 = ((point[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (point[1] - a[1])) / det;
        let w2 = ((b[0] - a[0]) * (point[1] - a[1]) - (point[0] - a[0]) * (b[1] - a[1])) / det;
        let w = [1.0 - w1 - w2, w1, w2];
        let inside = w.iter().copied().fold(f64::MAX, f64::min);
        if best.is_none_or(|(b, _, _)| inside > b) {
            best = Some((inside, k, w));
        }
    }
    let (_, tri, w) = best.ok_or_else(|| bad("the panel has no triangles"))?;
    Ok(Pt { panel: Some(panel), triangle: Some(tri as u32), w: Some(w), position: None })
}

fn parse_point(s: &mut Session, v: &Value) -> Result<Pt, CommandError> {
    let vec = |key: &str, n: usize| -> Result<Option<Vec<f64>>, CommandError> {
        match v.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(x) => {
                let list: Vec<f64> = x.as_array().map(|l| l.iter().filter_map(Value::as_f64).filter(|f| f.is_finite()).collect()).unwrap_or_default();
                if list.len() == n && x.as_array().is_some_and(|l| l.len() == n) {
                    Ok(Some(list))
                } else {
                    Err(bad(format!("{key} must be {n} finite numbers")))
                }
            }
        }
    };
    if let Some(p) = vec("position", 3)? {
        return Ok(Pt { position: Some([p[0], p[1], p[2]]), ..Pt::default() });
    }
    let panel = v.get("panel").and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).ok_or_else(|| bad("a point needs a panel id, or a position"))?;
    if let Some(p) = vec("point", 2)? {
        return locate(s, panel, [p[0], p[1]]);
    }
    let tri = v.get("triangle").and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).ok_or_else(|| bad("a point needs triangle and w, or point (panel space)"))?;
    let w = vec("w", 3)?.ok_or_else(|| bad("a point needs w, three barycentric weights"))?;
    let sum = w[0] + w[1] + w[2];
    if sum.abs() < 1.0e-9 {
        return Err(bad("w must not sum to zero"));
    }
    Ok(Pt { panel: Some(panel), triangle: Some(tri), w: Some([w[0] / sum, w[1] / sum, w[2] / sum]), position: None })
}

type Segment = (DVec3, DVec3);

fn chain(segs: &[Segment]) -> Vec<Vec<DVec3>> {
    let key = |p: DVec3| ((p.x * 50.0).round() as i64, (p.y * 50.0).round() as i64, (p.z * 50.0).round() as i64);
    let mut at: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    for (i, (a, b)) in segs.iter().enumerate() {
        at.entry(key(*a)).or_default().push(i);
        at.entry(key(*b)).or_default().push(i);
    }
    let mut used = vec![false; segs.len()];
    let mut out = Vec::new();
    let next = |used: &mut Vec<bool>, tail: DVec3| -> Option<DVec3> {
        let j = at.get(&key(tail))?.iter().copied().find(|&j| !used[j])?;
        used[j] = true;
        let (a, b) = segs[j];
        Some(if key(a) == key(tail) { b } else { a })
    };
    for i in 0..segs.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        let mut line: VecDeque<DVec3> = VecDeque::from([segs[i].0, segs[i].1]);
        while let Some(p) = next(&mut used, *line.back().expect("two points")) {
            line.push_back(p);
        }
        while let Some(p) = next(&mut used, *line.front().expect("two points")) {
            line.push_front(p);
        }
        out.push(line.into_iter().collect());
    }
    out
}

fn slice(scene: &Scene, origin: DVec3, normal: DVec3, only: &[u32], center: Option<DVec3>, radius: Option<f64>) -> Vec<Vec<DVec3>> {
    let mut lines = Vec::new();
    for panel in scene.panels.iter().filter(|p| only.is_empty() || only.contains(&p.id)) {
        let x: Vec<DVec3> = panel.positions.iter().map(|p| DVec3::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))).collect();
        let d: Vec<f64> = x.iter().map(|p| normal.dot(*p - origin)).collect();
        let mut segs: Vec<Segment> = Vec::new();
        for t in &panel.triangles {
            let mut hits: Vec<DVec3> = Vec::with_capacity(2);
            for k in 0..3 {
                let (a, b) = (t[k] as usize, t[(k + 1) % 3] as usize);
                if (d[a] < 0.0) != (d[b] < 0.0) {
                    hits.push(x[a] + (x[b] - x[a]) * (d[a] / (d[a] - d[b])));
                }
            }
            if hits.len() == 2 {
                let mid = (hits[0] + hits[1]) * 0.5;
                let near = match (center, radius) {
                    (Some(c), Some(r)) => {
                        let v = mid - c;
                        (v - normal * v.dot(normal)).length() <= r
                    }
                    _ => true,
                };
                if near {
                    segs.push((hits[0], hits[1]));
                }
            }
        }
        lines.extend(chain(&segs));
    }
    lines
}

fn length_of(line: &[DVec3]) -> f64 {
    line.windows(2).map(|w| (w[1] - w[0]).length()).sum()
}

fn thin(line: &[DVec3]) -> Vec<[f64; 3]> {
    let step = line.len().div_ceil(MAX_LINE_POINTS).max(1);
    let round = |v: f64| (v * 100.0).round() / 100.0;
    let mut out: Vec<[f64; 3]> = line.iter().step_by(step).map(|p| [round(p.x), round(p.y), round(p.z)]).collect();
    if let (Some(last), Some(end)) = (line.last(), out.last()) {
        if (end[0] - last.x).abs() > 1.0e-6 || (end[1] - last.y).abs() > 1.0e-6 || (end[2] - last.z).abs() > 1.0e-6 {
            out.push([round(last.x), round(last.y), round(last.z)]);
        }
    }
    out
}

fn plane_of(scene: &Scene, m: &Measure) -> Result<(DVec3, DVec3), CommandError> {
    if let Some(h) = m.height {
        return Ok((DVec3::new(0.0, h, 0.0), DVec3::Y));
    }
    let p: Vec<DVec3> = m.points.iter().map(|q| resolve(scene, q)).collect::<Result<_, _>>()?;
    if p.len() != 3 {
        return Err(bad("a circumference needs a height or three plane points"));
    }
    let n = (p[1] - p[0]).cross(p[2] - p[0]);
    if n.length() < 1.0e-9 {
        return Err(bad("the three plane points are collinear"));
    }
    Ok((p[0], n.normalize()))
}

fn evaluate(scene: &Scene, m: &Measure) -> Result<Value, CommandError> {
    let mut out = json!({"id": m.id, "kind": m.kind, "name": m.name, "unit": "mm"});
    match m.kind.as_str() {
        "length" | "distance" => {
            let p: Vec<DVec3> = m.points.iter().map(|q| resolve(scene, q)).collect::<Result<_, _>>()?;
            out["value"] = json!(length_of(&p));
            out["polyline"] = json!(p.iter().map(|q| arr(*q)).collect::<Vec<_>>());
        }
        _ => {
            let (origin, normal) = plane_of(scene, m)?;
            let lines = slice(scene, origin, normal, &m.panels, m.center.map(v3), m.radius);
            let total: f64 = lines.iter().map(|l| length_of(l)).sum();
            let closed = lines.iter().filter(|l| l.len() > 2 && (l[0] - l[l.len() - 1]).length() < 0.05).count();
            out["value"] = json!(total);
            out["arcs"] = json!(lines.len());
            out["closed_loops"] = json!(closed);
            out["polylines"] = json!(lines.iter().take(MAX_CHAINS).map(|l| thin(l)).collect::<Vec<_>>());
            out["plane"] = json!({"point": arr(origin), "normal": arr(normal)});
        }
    }
    Ok(out)
}

fn rounded(mut v: Value) -> Value {
    if let Some(x) = v.get("value").and_then(Value::as_f64) {
        v["value"] = json!((x * 100.0).round() / 100.0);
    }
    v
}

fn add_schema() -> Value {
    let point = json!({"type": "object", "description": "A surface point: {panel, triangle, w: [w0, w1, w2]} barycentric on a mesh triangle, {panel, point: [x, y]} in panel space, or {position: [x, y, z]} in world mm."});
    json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": ["length", "circumference", "distance"]},
            "name": {"type": "string", "maxLength": 80},
            "points": {"type": "array", "items": point, "description": "length: two or more points of a polyline; distance: two points; circumference: three points that define the slice plane (instead of height)."},
            "height": {"type": "number", "description": "circumference: slice with the horizontal plane y = height (mm)."},
            "panels": {"type": "array", "items": {"type": "integer"}, "description": "circumference: only slice these panels."},
            "center": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3, "description": "circumference: keep only the part of the slice within radius of this point (separates a torso from sleeves)."},
            "radius": {"type": "number", "minimum": 0}
        },
        "required": ["kind"],
        "additionalProperties": false
    })
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let kind = crate::args::string(a, "kind")?.to_string();
    let name = a.get("name").and_then(Value::as_str).unwrap_or("").trim().to_string();
    crate::args::clean_name("name", &name)?;
    let raw = a.get("points").and_then(Value::as_array).cloned().unwrap_or_default();
    if raw.len() > MAX_POINTS {
        return Err(bad(format!("at most {MAX_POINTS} points")));
    }
    let mut points = Vec::new();
    for v in &raw {
        points.push(parse_point(s, v)?);
    }
    let height = match a.get("height") {
        None => None,
        Some(v) => Some(v.as_f64().filter(|h| h.is_finite()).ok_or_else(|| bad("height must be a number"))?),
    };
    match kind.as_str() {
        "length" if points.len() < 2 => return Err(bad("a length needs at least two points")),
        "distance" if points.len() != 2 => return Err(bad("a distance needs exactly two points")),
        "circumference" if height.is_none() && points.len() != 3 => return Err(bad("a circumference needs a height or three plane points")),
        "circumference" if height.is_some() && !points.is_empty() => return Err(bad("give either a height or three plane points")),
        _ => {}
    }
    let panels: Vec<u32> = a.get("panels").and_then(Value::as_array).map(|l| l.iter().filter_map(Value::as_u64).filter_map(|x| u32::try_from(x).ok()).collect()).unwrap_or_default();
    let center = match a.get("center").and_then(Value::as_array) {
        None => None,
        Some(l) => {
            let v: Vec<f64> = l.iter().filter_map(Value::as_f64).filter(|x| x.is_finite()).collect();
            if v.len() != 3 || l.len() != 3 {
                return Err(bad("center must be three numbers"));
            }
            Some([v[0], v[1], v[2]])
        }
    };
    let radius = match a.get("radius") {
        None => None,
        Some(v) => Some(v.as_f64().filter(|r| r.is_finite() && *r >= 0.0).ok_or_else(|| bad("radius must be a non-negative number"))?),
    };
    if center.is_some() != radius.is_some() {
        return Err(bad("center and radius go together"));
    }
    let scene = scene::build(s, 0)?;
    let id = stored(&s.garment).iter().map(|m| m.id).max().unwrap_or(0) + 1;
    let measure = Measure { id, kind, name: if name.is_empty() { format!("Measure {id}") } else { name }, points, height, panels, center, radius };
    let shown = evaluate(&scene, &measure)?;
    edit(s, "Add Measure", |g| {
        let mut list = stored(g);
        if list.len() >= MAX_MEASURES {
            return Err(bad(format!("at most {MAX_MEASURES} measures")));
        }
        list.push(measure);
        store(g, &list);
        Ok(rounded(shown))
    })
}

fn remove_schema() -> Value {
    json!({"type": "object", "properties": {"id": {"type": "integer", "minimum": 1}}, "required": ["id"], "additionalProperties": false})
}

fn remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = a.get("id").and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).ok_or_else(|| bad("id must be a measure id"))?;
    edit(s, "Remove Measure", |g| {
        let mut list = stored(g);
        let at = list.iter().position(|m| m.id == id).ok_or_else(|| CommandError::new("not_found", format!("no measure with id {id}")))?;
        list.remove(at);
        store(g, &list);
        Ok(json!({"removed": id}))
    })
}

pub(crate) fn live(s: &mut Session) -> Result<Vec<Value>, CommandError> {
    let list = stored(&s.garment);
    if list.is_empty() {
        return Ok(Vec::new());
    }
    let scene = scene::build(s, 0)?;
    Ok(list
        .iter()
        .map(|m| match evaluate(&scene, m) {
            Ok(v) => rounded(v),
            Err(e) => json!({"id": m.id, "kind": m.kind, "name": m.name, "error": e.message}),
        })
        .collect())
}

fn list(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let measures = live(s)?;
    Ok(json!({"count": measures.len(), "measures": measures}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "measure3d.add",
            mcp: "measure3d_add",
            group: Group::View,
            label: "Add 3D Measure",
            description: "Add a garment measure read from the current draped positions: length (polyline through surface points), circumference (slice of the draped garment by a horizontal height or by a plane through three points, summed over every panel it cuts; filter with panels or center and radius) or distance (straight line between two points). Definitions are kept in Garment.viz.compare under measures. An undo step.",
            keys: &[],
            hints: WRITE,
            schema: add_schema,
            run: add,
        },
        Command {
            id: "measure3d.remove",
            mcp: "measure3d_remove",
            group: Group::View,
            label: "Remove 3D Measure",
            description: "Remove one garment measure by id. An undo step.",
            keys: &[],
            hints: WRITE,
            schema: remove_schema,
            run: remove,
        },
        Command {
            id: "measure3d.list",
            mcp: "measure3d_list",
            group: Group::View,
            label: "3D Measures",
            description: "List every garment measure with its live value in mm from the current positions, and the polylines to draw.",
            keys: &[],
            hints: READ,
            schema: crate::no_args,
            run: list,
        },
    ]
}
