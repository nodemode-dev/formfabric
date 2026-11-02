use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::geom::{dist, lerp, signed_area};
use crate::types::{QuadMesh, V2};

pub const MAX_POINTS: usize = 50_000;
pub const MAX_DIVISION: usize = 32;
const SAME: f64 = 1e-6;
const MIN_AREA: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopoError {
    pub code: &'static str,
    pub message: String,
}

fn err<T>(code: &'static str, message: impl Into<String>) -> Result<T, TopoError> {
    Err(TopoError { code, message: message.into() })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Replace,
    Duplicate,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelTopo {
    pub points: Vec<V2>,
    pub faces: Vec<Vec<u32>>,
    pub lines: Vec<Vec<u32>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Open {
    pub panel: u32,
    pub points: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Topology {
    pub panels: BTreeMap<u32, PanelTopo>,
    pub open: Option<Open>,
    pub remeshed: BTreeMap<u32, Mode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Division {
    pub edges_cut: usize,
    pub new_points: usize,
    pub faces_before: usize,
    pub faces_after: usize,
}

fn key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn edge_pos(face: &[u32], e: (u32, u32)) -> Option<usize> {
    let n = face.len();
    (0..n).find(|i| key(face[*i], face[(*i + 1) % n]) == e)
}

fn crosses(a: V2, b: V2, c: V2, d: V2) -> bool {
    let side = |p: V2, q: V2, r: V2| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    let (d1, d2, d3, d4) = (side(c, d, a), side(c, d, b), side(a, b, c), side(a, b, d));
    ((d1 > 0.0) != (d2 > 0.0)) && ((d3 > 0.0) != (d4 > 0.0)) && d1 != 0.0 && d2 != 0.0 && d3 != 0.0 && d4 != 0.0
}

fn valid_face(points: &[V2], ids: &[u32]) -> Result<f64, TopoError> {
    let p: Vec<V2> = ids.iter().map(|i| points[*i as usize]).collect();
    let area = signed_area(&p);
    if area.abs() < MIN_AREA {
        return err("invalid_geometry", "the shape has no area");
    }
    if p.len() == 4 && (crosses(p[0], p[1], p[2], p[3]) || crosses(p[1], p[2], p[3], p[0])) {
        return err("invalid_geometry", "the four points cross each other; place them around the face in order");
    }
    Ok(area)
}

fn face_key(face: &[u32]) -> Vec<u32> {
    let mut k = face.to_vec();
    k.sort_unstable();
    k
}

fn dedupe_cycle(ids: &mut Vec<u32>) {
    ids.dedup();
    while ids.len() > 1 && ids.first() == ids.last() {
        ids.pop();
    }
}

fn along(div: &BTreeMap<(u32, u32), Vec<u32>>, u: u32, w: u32) -> Vec<u32> {
    let mids = div.get(&key(u, w)).cloned().unwrap_or_default();
    let mut v = vec![u];
    if u < w {
        v.extend(mids);
    } else {
        v.extend(mids.into_iter().rev());
    }
    v.push(w);
    v
}

impl PanelTopo {
    fn referenced(&self, idx: u32) -> bool {
        self.faces.iter().any(|f| f.contains(&idx)) || self.lines.iter().any(|l| l.contains(&idx))
    }

    pub fn find(&self, pos: V2) -> Option<u32> {
        self.points.iter().position(|p| dist(*p, pos) <= SAME).map(|i| i as u32)
    }
}

impl Topology {
    pub fn sanitize(&mut self) -> usize {
        let mut dropped = 0;
        for panel in self.panels.values_mut() {
            let n = panel.points.len();
            let ok = |ids: &Vec<u32>, p: &PanelTopo| ids.iter().all(|i| (*i as usize) < n && p.points[*i as usize].iter().all(|c| c.is_finite()));
            let snapshot = panel.clone();
            let (f0, l0) = (panel.faces.len(), panel.lines.len());
            panel.faces.retain(|f| (f.len() == 3 || f.len() == 4) && ok(f, &snapshot));
            panel.lines.retain(|l| l.len() >= 2 && ok(l, &snapshot));
            dropped += f0 - panel.faces.len() + l0 - panel.lines.len();
        }
        if let Some(open) = &self.open {
            let valid = self.panels.get(&open.panel).is_some_and(|p| open.points.iter().all(|i| (*i as usize) < p.points.len()));
            if !valid {
                self.open = None;
                dropped += 1;
            }
        }
        dropped
    }

    pub fn is_empty(&self) -> bool {
        self.panels.values().all(|p| p.points.is_empty()) && self.open.is_none() && self.remeshed.is_empty()
    }

    pub fn add_point(&mut self, panel: u32, pos: V2) -> Result<(u32, bool), TopoError> {
        if !pos[0].is_finite() || !pos[1].is_finite() {
            return err("bad_args", "the position must be finite numbers");
        }
        if let Some(open) = &self.open {
            if open.panel != panel {
                return err("invalid_state", format!("a shape is open on panel {}; close, finish or cancel it before drawing on panel {panel}", open.panel));
            }
        }
        let existing = self.panels.get(&panel).and_then(|t| t.find(pos));
        if let (Some(idx), Some(open)) = (existing, &self.open) {
            if open.points.first() == Some(&idx) && open.points.len() >= 3 {
                return err("invalid_state", "that is the first point of the open shape; use topology.close to make the face");
            }
            if open.points.contains(&idx) {
                return err("invalid_state", "that point is already in the open shape");
            }
        }
        let t = self.panels.entry(panel).or_default();
        let (idx, created) = match existing {
            Some(i) => (i, false),
            None => {
                if t.points.len() >= MAX_POINTS {
                    return err("invalid_state", "this panel has the maximum number of topology points");
                }
                t.points.push(pos);
                (t.points.len() as u32 - 1, true)
            }
        };
        self.open.get_or_insert_with(|| Open { panel, points: Vec::new() }).points.push(idx);
        Ok((idx, created))
    }

    pub fn close(&mut self) -> Result<(u32, usize), TopoError> {
        let Some(open) = self.open.clone() else { return err("invalid_state", "there is no open shape to close") };
        let n = open.points.len();
        if n != 3 && n != 4 {
            return err("invalid_state", format!("a closed loop becomes a face only with 3 or 4 points; the open shape has {n}"));
        }
        let t = self.panels.get_mut(&open.panel).ok_or(TopoError { code: "not_found", message: "the open shape has no panel".into() })?;
        let mut ids = open.points.clone();
        if valid_face(&t.points, &ids)? < 0.0 {
            ids.reverse();
        }
        let k = face_key(&ids);
        if t.faces.iter().any(|f| face_key(f) == k) {
            return err("invalid_state", "that face already exists");
        }
        t.faces.push(ids);
        self.open = None;
        Ok((open.panel, t.faces.len() - 1))
    }

    pub fn finish(&mut self) -> Result<(u32, usize), TopoError> {
        let Some(open) = self.open.clone() else { return err("invalid_state", "there is no open shape to finish") };
        if open.points.len() < 2 {
            return err("invalid_state", "an open shape needs at least 2 points; use topology.cancel to drop it");
        }
        let t = self.panels.get_mut(&open.panel).ok_or(TopoError { code: "not_found", message: "the open shape has no panel".into() })?;
        t.lines.push(open.points.clone());
        self.open = None;
        Ok((open.panel, open.points.len()))
    }

    pub fn delete_last(&mut self) -> Result<(u32, usize), TopoError> {
        let Some(open) = self.open.as_mut() else { return err("invalid_state", "there is no open shape; nothing to remove") };
        open.points.pop();
        let panel = open.panel;
        let left = open.points.len();
        if left == 0 {
            self.open = None;
        }
        self.compact(panel);
        Ok((panel, left))
    }

    pub fn cancel(&mut self) -> Option<u32> {
        let open = self.open.take()?;
        self.compact(open.panel);
        Some(open.panel)
    }

    pub fn compact(&mut self, panel: u32) {
        let open_ids: Vec<u32> = self.open.as_ref().filter(|o| o.panel == panel).map(|o| o.points.clone()).unwrap_or_default();
        let Some(t) = self.panels.get_mut(&panel) else { return };
        let n = t.points.len();
        let mut new_of = vec![u32::MAX; n];
        let mut points = Vec::with_capacity(n);
        for (i, slot) in new_of.iter_mut().enumerate() {
            if t.referenced(i as u32) || open_ids.contains(&(i as u32)) {
                *slot = points.len() as u32;
                points.push(t.points[i]);
            }
        }
        if points.len() == n {
            return;
        }
        for f in t.faces.iter_mut().chain(t.lines.iter_mut()) {
            for i in f.iter_mut() {
                *i = new_of[*i as usize];
            }
        }
        t.points = points;
        if let Some(o) = self.open.as_mut().filter(|o| o.panel == panel) {
            for i in o.points.iter_mut() {
                *i = new_of[*i as usize];
            }
        }
        if t.points.is_empty() && t.faces.is_empty() && t.lines.is_empty() {
            self.panels.remove(&panel);
        }
    }

    fn collapse(&mut self, panel: u32, rep: &[u32], pos: &[V2]) {
        let Some(t) = self.panels.get_mut(&panel) else { return };
        let n = t.points.len();
        let mut new_of = vec![u32::MAX; n];
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            if rep[i] == i as u32 {
                new_of[i] = points.len() as u32;
                points.push(pos[i]);
            }
        }
        let map = |i: u32| new_of[rep[i as usize] as usize];
        let mut faces: Vec<Vec<u32>> = Vec::new();
        let mut seen: BTreeSet<Vec<u32>> = BTreeSet::new();
        for f in &t.faces {
            let mut ids: Vec<u32> = f.iter().map(|i| map(*i)).collect();
            dedupe_cycle(&mut ids);
            if ids.len() < 3 || face_key(&ids).windows(2).any(|w| w[0] == w[1]) {
                continue;
            }
            let area = signed_area(&ids.iter().map(|i| points[*i as usize]).collect::<Vec<_>>());
            if area.abs() < MIN_AREA {
                continue;
            }
            if area < 0.0 {
                ids.reverse();
            }
            if seen.insert(face_key(&ids)) {
                faces.push(ids);
            }
        }
        let mut lines: Vec<Vec<u32>> = Vec::new();
        for l in &t.lines {
            let mut ids: Vec<u32> = l.iter().map(|i| map(*i)).collect();
            ids.dedup();
            if ids.len() >= 2 {
                lines.push(ids);
            }
        }
        t.points = points;
        t.faces = faces;
        t.lines = lines;
        if let Some(o) = self.open.as_mut().filter(|o| o.panel == panel) {
            let mut ids: Vec<u32> = o.points.iter().map(|i| map(*i)).collect();
            ids.dedup();
            o.points = ids;
            if o.points.is_empty() {
                self.open = None;
            }
        }
    }

    pub fn merge(&mut self, panel: u32, a: u32, b: u32) -> Result<(), TopoError> {
        let t = self.panels.get(&panel).ok_or(TopoError { code: "not_found", message: format!("panel {panel} has no topology") })?;
        let n = t.points.len() as u32;
        if a >= n || b >= n {
            return err("not_found", format!("points must be 0 to {}", n.saturating_sub(1)));
        }
        if a == b {
            return err("bad_args", "a and b are the same point");
        }
        let (keep, drop) = (a.min(b), a.max(b));
        let mut rep: Vec<u32> = (0..n).collect();
        rep[drop as usize] = keep;
        let mut pos = t.points.clone();
        pos[keep as usize] = lerp(t.points[a as usize], t.points[b as usize], 0.5);
        self.collapse(panel, &rep, &pos);
        Ok(())
    }

    pub fn merge_all(&mut self, panel: u32, distance: f64) -> usize {
        let Some(t) = self.panels.get(&panel) else { return 0 };
        let n = t.points.len();
        let mut parent: Vec<u32> = (0..n as u32).collect();
        fn root(parent: &mut [u32], mut i: u32) -> u32 {
            while parent[i as usize] != i {
                parent[i as usize] = parent[parent[i as usize] as usize];
                i = parent[i as usize];
            }
            i
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|a, b| t.points[*a][0].total_cmp(&t.points[*b][0]));
        for (oi, &i) in order.iter().enumerate() {
            for &j in &order[oi + 1..] {
                if t.points[j][0] - t.points[i][0] > distance {
                    break;
                }
                if dist(t.points[i], t.points[j]) <= distance {
                    let (ri, rj) = (root(&mut parent, i as u32), root(&mut parent, j as u32));
                    if ri != rj {
                        parent[ri.max(rj) as usize] = ri.min(rj);
                    }
                }
            }
        }
        let rep: Vec<u32> = (0..n as u32).map(|i| root(&mut parent, i)).collect();
        let merged = rep.iter().enumerate().filter(|(i, r)| **r != *i as u32).count();
        if merged == 0 {
            return 0;
        }
        let mut sum = vec![[0.0f64; 2]; n];
        let mut cnt = vec![0usize; n];
        for (i, r) in rep.iter().enumerate() {
            let r = *r as usize;
            sum[r][0] += t.points[i][0];
            sum[r][1] += t.points[i][1];
            cnt[r] += 1;
        }
        let pos: Vec<V2> = (0..n).map(|i| if cnt[i] > 0 { [sum[i][0] / cnt[i] as f64, sum[i][1] / cnt[i] as f64] } else { t.points[i] }).collect();
        self.collapse(panel, &rep, &pos);
        merged
    }

    pub fn delete_face(&mut self, panel: u32, index: usize) -> Result<(), TopoError> {
        let t = self.panels.get_mut(&panel).ok_or(TopoError { code: "not_found", message: format!("panel {panel} has no topology") })?;
        if index >= t.faces.len() {
            return err("not_found", format!("the panel has {} faces", t.faces.len()));
        }
        t.faces.remove(index);
        self.compact(panel);
        Ok(())
    }

    pub fn delete_point(&mut self, panel: u32, index: u32) -> Result<(), TopoError> {
        if self.open.as_ref().is_some_and(|o| o.panel == panel) {
            return err("invalid_state", "finish or cancel the open shape first");
        }
        let t = self.panels.get_mut(&panel).ok_or(TopoError { code: "not_found", message: format!("panel {panel} has no topology") })?;
        if index as usize >= t.points.len() {
            return err("not_found", format!("the panel has {} points", t.points.len()));
        }
        t.faces.retain(|f| !f.contains(&index));
        let mut lines: Vec<Vec<u32>> = Vec::new();
        for l in &t.lines {
            for part in l.split(|i| *i == index) {
                if part.len() >= 2 {
                    lines.push(part.to_vec());
                }
            }
        }
        t.lines = lines;
        let drop = index;
        let n = t.points.len();
        t.points.remove(drop as usize);
        let fix = |i: &mut u32| {
            if *i > drop {
                *i -= 1;
            }
        };
        for f in t.faces.iter_mut().chain(t.lines.iter_mut()) {
            f.iter_mut().for_each(fix);
        }
        debug_assert_eq!(t.points.len() + 1, n);
        self.compact(panel);
        Ok(())
    }

    pub fn clear(&mut self, panel: u32) -> bool {
        let had = self.panels.remove(&panel).is_some() | self.remeshed.remove(&panel).is_some();
        if self.open.as_ref().is_some_and(|o| o.panel == panel) {
            self.open = None;
        }
        had
    }

    pub fn divide(&mut self, panel: u32, a: u32, b: u32, count: usize) -> Result<Division, TopoError> {
        if self.open.as_ref().is_some_and(|o| o.panel == panel) {
            return err("invalid_state", "finish or cancel the open shape before dividing");
        }
        if !(2..=MAX_DIVISION).contains(&count) {
            return err("bad_args", format!("count must be 2 to {MAX_DIVISION}"));
        }
        let mut t = self.panels.get(&panel).cloned().ok_or(TopoError { code: "not_found", message: format!("panel {panel} has no topology") })?;
        let n = t.points.len() as u32;
        if a >= n || b >= n || a == b {
            return err("bad_args", "edge needs two different points of the panel");
        }
        let start = key(a, b);
        let in_faces = t.faces.iter().any(|f| edge_pos(f, start).is_some());
        let in_lines = t.lines.iter().any(|l| l.windows(2).any(|w| key(w[0], w[1]) == start));
        if !in_faces && !in_lines {
            return err("not_found", "no face or line has that edge");
        }
        let mut cut = vec![start];
        let mut seen: BTreeSet<(u32, u32)> = BTreeSet::from([start]);
        let mut queue: VecDeque<(u32, u32)> = VecDeque::from([start]);
        let mut assigned: BTreeMap<usize, usize> = BTreeMap::new();
        while let Some(e) = queue.pop_front() {
            for (fi, f) in t.faces.iter().enumerate() {
                let Some(pos) = edge_pos(f, e) else { continue };
                let slot = if f.len() == 4 { pos % 2 } else { pos };
                match assigned.get(&fi) {
                    Some(p) if *p != slot => return err("invalid_state", "the division loop would cross itself or reach a triangle from two sides"),
                    Some(_) => {}
                    None => {
                        assigned.insert(fi, slot);
                        if f.len() == 4 {
                            let opp = key(f[(pos + 2) % 4], f[(pos + 3) % 4]);
                            if seen.insert(opp) {
                                cut.push(opp);
                                queue.push_back(opp);
                            }
                        }
                    }
                }
            }
        }
        if t.points.len() + cut.len() * (count - 1) > MAX_POINTS {
            return err("invalid_state", "the division would exceed the point limit");
        }
        let mut div: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
        for e in &cut {
            let (lo, hi) = (t.points[e.0 as usize], t.points[e.1 as usize]);
            let mut ids = Vec::with_capacity(count - 1);
            for k in 1..count {
                t.points.push(lerp(lo, hi, k as f64 / count as f64));
                ids.push(t.points.len() as u32 - 1);
            }
            div.insert(*e, ids);
        }
        let faces_before = t.faces.len();
        let mut faces: Vec<Vec<u32>> = Vec::new();
        for (fi, f) in t.faces.iter().enumerate() {
            let Some(&slot) = assigned.get(&fi) else {
                faces.push(f.clone());
                continue;
            };
            if f.len() == 4 {
                let v = [f[slot], f[(slot + 1) % 4], f[(slot + 2) % 4], f[(slot + 3) % 4]];
                let (p1, p2) = (along(&div, v[0], v[1]), along(&div, v[3], v[2]));
                for k in 0..count {
                    faces.push(vec![p1[k], p1[k + 1], p2[k + 1], p2[k]]);
                }
            } else {
                let (u, w, x) = (f[slot], f[(slot + 1) % 3], f[(slot + 2) % 3]);
                let p = along(&div, u, w);
                for k in 0..count {
                    faces.push(vec![p[k], p[k + 1], x]);
                }
            }
        }
        let mut lines: Vec<Vec<u32>> = Vec::new();
        for l in &t.lines {
            let mut out = vec![l[0]];
            for w in l.windows(2) {
                if div.contains_key(&key(w[0], w[1])) {
                    out.extend(along(&div, w[0], w[1]).into_iter().skip(1));
                } else {
                    out.push(w[1]);
                }
            }
            lines.push(out);
        }
        let new_points = cut.len() * (count - 1);
        let faces_after = faces.len();
        t.faces = faces;
        t.lines = lines;
        self.panels.insert(panel, t);
        Ok(Division { edges_cut: cut.len(), new_points, faces_before, faces_after })
    }

    pub fn to_quad_mesh(&self, panel: u32) -> Result<QuadMesh, TopoError> {
        let Some(t) = self.panels.get(&panel).filter(|t| !t.faces.is_empty()) else { return err("invalid_state", format!("panel {panel} has no topology faces")) };
        let mut new_of: BTreeMap<u32, u32> = BTreeMap::new();
        let mut positions: Vec<V2> = Vec::new();
        let mut quads: Vec<[u32; 4]> = Vec::new();
        let mut tris: Vec<[u32; 3]> = Vec::new();
        for f in &t.faces {
            let ids: Vec<u32> = f
                .iter()
                .map(|i| {
                    *new_of.entry(*i).or_insert_with(|| {
                        positions.push(t.points[*i as usize]);
                        positions.len() as u32 - 1
                    })
                })
                .collect();
            match ids.len() {
                4 => quads.push([ids[0], ids[1], ids[2], ids[3]]),
                3 => tris.push([ids[0], ids[1], ids[2]]),
                _ => {}
            }
        }
        let mut total = 0.0;
        let mut count = 0usize;
        for f in quads.iter().map(|q| q.as_slice()).chain(tris.iter().map(|q| q.as_slice())) {
            for k in 0..f.len() {
                total += dist(positions[f[k] as usize], positions[f[(k + 1) % f.len()] as usize]);
                count += 1;
            }
        }
        Ok(QuadMesh { panel, positions, quads, tris, rim: Vec::new(), edge: if count > 0 { total / count as f64 } else { 0.0 }, report: Default::default() })
    }
}
