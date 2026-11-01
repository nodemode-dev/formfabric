use ffb_pattern::sew::{next_seam_id, side_length};
use ffb_pattern::{path, Garment, Seam, SeamEnd};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Edit, Group, Session, WRITE};

const BUDGET: u64 = 4_000_000;

struct Edge {
    panel: u32,
    segment: usize,
    length: f64,
    notches: Vec<f64>,
    tag: Option<String>,
    name: String,
    mid: Option<[f64; 3]>,
}

fn middle(panel: &ffb_pattern::Panel, segment: usize) -> Option<[f64; 3]> {
    let place = panel.place3d.as_ref()?;
    let local = path::seg(&panel.outline, segment).at(0.5);
    let r = ffb_mesh::layout::rotate(ffb_mesh::layout::quaternion(place.rotation), [local[0], local[1], 0.0]);
    Some([place.origin[0] + r[0], place.origin[1] + r[1], place.origin[2] + r[2]])
}

#[derive(Clone, Copy)]
struct Link {
    other: usize,
    weight: f64,
    cost: f64,
    reversed: bool,
}

fn whole(panel: u32, segment: usize) -> SeamEnd {
    SeamEnd { panel, inner: None, segment, t0: 0.0, t1: 1.0 }
}

fn covered(garment: &Garment, panel: u32, segment: usize) -> bool {
    garment.seams.iter().flat_map(|s| s.a.iter().chain(s.b.iter())).any(|e| e.panel == panel && e.inner.is_none() && e.segment == segment && (e.t1 - e.t0).abs() > 1e-6)
}

fn tags_of(a: &Value) -> Result<Vec<(u32, usize, String)>, CommandError> {
    let Some(list) = a.get("tags").filter(|v| !v.is_null()) else { return Ok(Vec::new()) };
    let list = list.as_array().ok_or_else(|| bad("tags must be a list of {panel, segment, tag}"))?;
    list.iter()
        .map(|t| {
            let panel = t.get("panel").and_then(Value::as_u64).ok_or_else(|| bad("a tag needs panel"))? as u32;
            let segment = t.get("segment").and_then(Value::as_u64).ok_or_else(|| bad("a tag needs segment"))? as usize;
            let tag = t.get("tag").and_then(Value::as_str).filter(|x| !x.trim().is_empty()).ok_or_else(|| bad("a tag needs tag text"))?;
            Ok((panel, segment, tag.trim().to_lowercase()))
        })
        .collect()
}

fn edges_of(garment: &Garment, scope: Option<&[u32]>, min_length: f64, tags: &[(u32, usize, String)]) -> Vec<Edge> {
    let mut out = Vec::new();
    for panel in &garment.panels {
        if scope.is_some_and(|s| !s.contains(&panel.id)) {
            continue;
        }
        for segment in 0..path::seg_count(panel.outline.len(), true) {
            if covered(garment, panel.id, segment) {
                continue;
            }
            let Some(length) = side_length(garment, &[whole(panel.id, segment)]) else { continue };
            if length < min_length {
                continue;
            }
            let mut notches: Vec<f64> = panel.notches.iter().filter(|n| n.segment == segment).map(|n| n.t).collect();
            notches.sort_by(f64::total_cmp);
            let tag = tags.iter().find(|t| t.0 == panel.id && t.1 == segment).map(|t| t.2.clone());
            out.push(Edge { panel: panel.id, segment, length, notches, tag, name: panel.name.to_lowercase(), mid: middle(panel, segment) });
        }
    }
    out
}

fn position_gap(a: &[f64], b: &[f64], reversed: bool) -> f64 {
    let total: f64 = a.iter().enumerate().map(|(k, t)| (t - if reversed { 1.0 - b[b.len() - 1 - k] } else { b[k] }).abs()).sum();
    total / a.len().max(1) as f64
}

fn complementary(a: &str, b: &str) -> bool {
    (a.contains("front") && b.contains("back")) || (a.contains("back") && b.contains("front"))
}

fn link(a: &Edge, b: &Edge, max_cost: f64) -> Option<(f64, bool)> {
    if a.panel == b.panel {
        return None;
    }
    if let (Some(x), Some(y)) = (&a.tag, &b.tag) {
        return (x == y).then_some((0.0, true));
    }
    if a.tag.is_some() != b.tag.is_some() {
        return None;
    }
    if a.notches.is_empty() || a.notches.len() != b.notches.len() {
        return None;
    }
    let longer = a.length.max(b.length).max(1e-9);
    let length_cost = (a.length - b.length).abs() / longer;
    let forward = position_gap(&a.notches, &b.notches, false);
    let backward = position_gap(&a.notches, &b.notches, true);
    let reversed = backward <= forward;
    let mut cost = length_cost + forward.min(backward);
    if complementary(&a.name, &b.name) {
        cost -= 0.05;
    }
    if let (Some(p), Some(q)) = (a.mid, b.mid) {
        let gap = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        cost += 0.1 * (gap / longer).min(1.0);
    }
    let cost = cost.max(0.0);
    (cost < max_cost).then_some((cost, reversed))
}

struct Search<'a> {
    links: &'a [Vec<Link>],
    nodes: Vec<usize>,
    used: Vec<bool>,
    half: Vec<f64>,
    remaining: f64,
    best: f64,
    best_pairs: Vec<(usize, Link, usize)>,
    current: Vec<(usize, Link, usize)>,
    steps: u64,
    exact: bool,
}

impl Search<'_> {
    fn go(&mut self, from: usize, gained: f64) {
        self.steps += 1;
        if self.steps > BUDGET {
            self.exact = false;
            return;
        }
        if gained > self.best {
            self.best = gained;
            self.best_pairs = self.current.clone();
        }
        let Some(&node) = self.nodes[from..].iter().find(|n| !self.used[**n]) else { return };
        if gained + self.remaining <= self.best + 1e-12 {
            return;
        }
        let position = self.nodes.iter().position(|n| *n == node).unwrap_or(from);
        self.used[node] = true;
        self.remaining -= self.half[node];
        let mut options: Vec<Link> = self.links[node].iter().copied().filter(|l| !self.used[l.other]).collect();
        options.sort_by(|x, y| y.weight.total_cmp(&x.weight));
        for l in options {
            self.used[l.other] = true;
            let before = self.remaining;
            self.remaining -= self.half[l.other];
            self.current.push((node, l, l.other));
            self.go(position + 1, gained + l.weight);
            self.current.pop();
            self.remaining = before;
            self.used[l.other] = false;
        }
        self.go(position + 1, gained);
        self.remaining += self.half[node];
        self.used[node] = false;
    }
}

fn solve(edges: &[Edge], max_cost: f64) -> (Vec<(usize, Link, usize)>, bool) {
    let n = edges.len();
    let mut links: Vec<Vec<Link>> = vec![Vec::new(); n];
    for i in 0..n {
        for j in (i + 1)..n {
            if let Some((cost, reversed)) = link(&edges[i], &edges[j], max_cost) {
                let weight = max_cost - cost;
                links[i].push(Link { other: j, weight, cost, reversed });
                links[j].push(Link { other: i, weight, cost, reversed });
            }
        }
    }
    let mut seen = vec![false; n];
    let mut chosen = Vec::new();
    let mut exact = true;
    for start in 0..n {
        if seen[start] || links[start].is_empty() {
            continue;
        }
        let mut nodes = vec![start];
        seen[start] = true;
        let mut k = 0;
        while k < nodes.len() {
            for l in &links[nodes[k]] {
                if !seen[l.other] {
                    seen[l.other] = true;
                    nodes.push(l.other);
                }
            }
            k += 1;
        }
        nodes.sort_unstable();
        let mut half = vec![0.0; n];
        for &v in &nodes {
            half[v] = links[v].iter().map(|l| l.weight).fold(0.0, f64::max) * 0.5;
        }
        let remaining = nodes.iter().map(|v| half[*v]).sum();
        let mut search = Search { links: &links, nodes, used: vec![false; n], half, remaining, best: 0.0, best_pairs: Vec::new(), current: Vec::new(), steps: 0, exact: true };
        search.go(0, 0.0);
        exact &= search.exact;
        chosen.extend(search.best_pairs);
    }
    (chosen, exact)
}

fn schema() -> Value {
    obj(
        json!({
            "apply": {"type": "boolean", "default": false, "description": "Create the proposed seams as one undo step. Without it the command only returns proposals."},
            "panels": {"type": "array", "items": {"type": "integer", "minimum": 1}, "description": "Only consider the outlines of these panels. Leave out for all panels."},
            "tags": {"type": "array", "description": "Join tags for this call: segments given the same tag are proposed as a seam, whatever their notches. Nothing is stored.", "items": obj(json!({"panel": {"type": "integer", "minimum": 1}, "segment": {"type": "integer", "minimum": 0}, "tag": {"type": "string"}}), &["panel", "segment", "tag"])},
            "max_cost": {"type": "number", "minimum": 0.01, "maximum": 5, "default": 0.35, "description": "Largest cost of a proposed pair: relative length difference plus mean notch position difference."},
            "min_length": {"type": "number", "minimum": 0, "maximum": 10000, "default": 10, "unit": "mm", "description": "Ignore outline segments shorter than this."}
        }),
        &[],
    )
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let apply = a.get("apply").and_then(Value::as_bool).unwrap_or(false);
    let max_cost = number(a, "max_cost", 0.01, 5.0)?.unwrap_or(0.35);
    let min_length = number(a, "min_length", 0.0, 10_000.0)?.unwrap_or(10.0);
    let scope: Option<Vec<u32>> = match a.get("panels").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => {
            let list = v.as_array().ok_or_else(|| bad("panels must be a list of panel ids"))?;
            let mut ids = Vec::new();
            for item in list {
                let id = item.as_u64().ok_or_else(|| bad("panels must hold whole panel ids"))? as u32;
                if s.garment.index_of(id).is_none() {
                    return Err(missing(format!("no panel with id {id}")));
                }
                ids.push(id);
            }
            Some(ids)
        }
    };
    let tags = tags_of(a)?;
    let edges = edges_of(&s.garment, scope.as_deref(), min_length, &tags);
    let (mut pairs, exact) = solve(&edges, max_cost);
    pairs.sort_by_key(|(i, _, j)| (edges[*i].panel.min(edges[*j].panel), edges[*i].segment));
    let mut matched = vec![false; edges.len()];
    let mut proposals = Vec::new();
    let mut seams = Vec::new();
    for (next, (i, l, j)) in (next_seam_id(&s.garment)..).zip(pairs.iter()) {
        matched[*i] = true;
        matched[*j] = true;
        let (x, y) = if (edges[*i].panel, edges[*i].segment) <= (edges[*j].panel, edges[*j].segment) { (&edges[*i], &edges[*j]) } else { (&edges[*j], &edges[*i]) };
        proposals.push(json!({
            "a": {"panel": x.panel, "segment": x.segment, "length": round(x.length), "notches": x.notches.len()},
            "b": {"panel": y.panel, "segment": y.segment, "length": round(y.length), "notches": y.notches.len()},
            "reversed": l.reversed,
            "cost": round(l.cost),
            "mismatch": round(y.length - x.length),
        }));
        seams.push(Seam { id: next, a: vec![whole(x.panel, x.segment)], b: vec![whole(y.panel, y.segment)], reversed: l.reversed, ..Seam::default() });
    }
    let open: Vec<Value> = edges.iter().enumerate().filter(|(k, _)| !matched[*k]).map(|(_, e)| json!({"panel": e.panel, "segment": e.segment, "length": round(e.length), "notches": e.notches.len()})).collect();
    let created: Vec<u32> = seams.iter().map(|x| x.id).collect();
    if apply && !seams.is_empty() {
        let before = s.garment.clone();
        s.garment.seams.extend(seams);
        let after = s.garment.clone();
        s.record(Edit::Garment { label: "Auto Seam".into(), before: Box::new(before), after: Box::new(after) });
    }
    Ok(json!({
        "proposals": proposals,
        "count": proposals.len(),
        "applied": apply && !created.is_empty(),
        "seams": if apply { created } else { Vec::new() },
        "open_edges": open,
        "optimal": exact,
    }))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "sew.auto",
        mcp: "sew_auto",
        group: Group::Sew,
        label: "Auto Seam",
        description: "Propose seams for the garment from the notches on the panels' own outline segments. Every open outline segment with notches is a candidate; segments pair only when their notch counts are equal, and all pairs are chosen together as one minimum-cost assignment where the cost is the relative length difference plus the mean difference of the notch positions (segments given the same tag in this call pair at zero cost). When both panels are placed in 3D, a small term for the distance between the segment midpoints settles ties between otherwise equal candidates; panels placed by hand or not placed work the same. Panels are never resized: a length mismatch is reported and left to the seam ratio. Works on panels placed by hand or not placed at all. Returns the proposals and the open segments; with apply the seams are created as one undo step and can be edited like any other seam.",
        keys: &[],
        hints: WRITE,
        schema,
        run,
    }]
}
