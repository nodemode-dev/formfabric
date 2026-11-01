use std::collections::{BTreeMap, HashMap};

use crate::math::*;
use crate::skin;
use crate::{Body, Joint};

const BUCKET: f32 = 8.0;

type Chord = ((u32, u32), (u32, u32), [f32; 2], [f32; 2]);

pub struct Loop {
    pub pts: Vec<[f32; 2]>,
    pub area: f32,
}

pub struct Slicer<'a> {
    pos: &'a [V3],
    tris: &'a [[u32; 3]],
    y0: f32,
    y1: f32,
    buckets: Vec<Vec<u32>>,
}

impl<'a> Slicer<'a> {
    pub fn new(pos: &'a [V3], tris: &'a [[u32; 3]]) -> Slicer<'a> {
        let y0 = pos.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let y1 = pos.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        let count = if pos.is_empty() { 0 } else { ((y1 - y0) / BUCKET) as usize + 2 };
        let mut buckets = vec![Vec::new(); count];
        for (i, t) in tris.iter().enumerate() {
            let ys = [pos[t[0] as usize][1], pos[t[1] as usize][1], pos[t[2] as usize][1]];
            let lo = ys.iter().copied().fold(f32::MAX, f32::min);
            let hi = ys.iter().copied().fold(f32::MIN, f32::max);
            let b0 = ((lo - y0) / BUCKET) as usize;
            let b1 = (((hi - y0) / BUCKET) as usize).min(count.saturating_sub(1));
            for bucket in buckets.iter_mut().take(b1 + 1).skip(b0) {
                bucket.push(i as u32);
            }
        }
        Slicer { pos, tris, y0, y1, buckets }
    }

    pub fn range(&self) -> (f32, f32) {
        (self.y0, self.y1)
    }

    pub fn loops(&self, y: f32) -> Vec<Loop> {
        if y < self.y0 || y > self.y1 || self.buckets.is_empty() {
            return Vec::new();
        }
        let b = (((y - self.y0) / BUCKET) as usize).min(self.buckets.len() - 1);
        let mut segs: Vec<Chord> = Vec::new();
        for ti in &self.buckets[b] {
            let t = self.tris[*ti as usize];
            let above = [self.pos[t[0] as usize][1] >= y, self.pos[t[1] as usize][1] >= y, self.pos[t[2] as usize][1] >= y];
            let n = above.iter().filter(|a| **a).count();
            if n == 0 || n == 3 {
                continue;
            }
            let mut hits: Vec<((u32, u32), [f32; 2])> = Vec::with_capacity(2);
            for k in 0..3 {
                let k2 = (k + 1) % 3;
                if above[k] != above[k2] {
                    let (a, b) = (t[k], t[k2]);
                    let (pa, pb) = (self.pos[a as usize], self.pos[b as usize]);
                    let s = (y - pa[1]) / (pb[1] - pa[1]);
                    hits.push(((a.min(b), a.max(b)), [pa[0] + (pb[0] - pa[0]) * s, pa[2] + (pb[2] - pa[2]) * s]));
                }
            }
            if hits.len() == 2 {
                segs.push((hits[0].0, hits[1].0, hits[0].1, hits[1].1));
            }
        }
        let mut by_edge: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
        for (i, s) in segs.iter().enumerate() {
            by_edge.entry(s.0).or_default().push(i);
            by_edge.entry(s.1).or_default().push(i);
        }
        let mut used = vec![false; segs.len()];
        let mut out = Vec::new();
        for start in 0..segs.len() {
            if used[start] {
                continue;
            }
            used[start] = true;
            let mut pts = vec![segs[start].2];
            let mut edge = segs[start].1;
            let mut at = start;
            pts.push(segs[start].3);
            loop {
                let next = by_edge.get(&edge).and_then(|v| v.iter().copied().find(|i| !used[*i] && *i != at));
                let Some(n) = next else { break };
                used[n] = true;
                let s = &segs[n];
                if s.0 == edge {
                    pts.push(s.3);
                    edge = s.1;
                } else {
                    pts.push(s.2);
                    edge = s.0;
                }
                at = n;
            }
            if pts.len() >= 3 {
                let area = polygon_area(&pts).abs();
                out.push(Loop { pts, area });
            }
        }
        out
    }

    pub fn torso_loop(&self, y: f32) -> Option<Loop> {
        self.loops(y).into_iter().max_by(|a, b| a.area.total_cmp(&b.area))
    }

    pub fn girth(&self, y: f32) -> Option<f32> {
        self.torso_loop(y).map(|l| hull_perimeter(&l.pts))
    }
}

pub fn polygon_area(pts: &[[f32; 2]]) -> f32 {
    let mut a = 0.0f64;
    for i in 0..pts.len() {
        let p = pts[i];
        let q = pts[(i + 1) % pts.len()];
        a += f64::from(p[0]) * f64::from(q[1]) - f64::from(q[0]) * f64::from(p[1]);
    }
    (a * 0.5) as f32
}

pub fn centroid(pts: &[[f32; 2]]) -> [f32; 2] {
    let n = pts.len().max(1) as f32;
    [pts.iter().map(|p| p[0]).sum::<f32>() / n, pts.iter().map(|p| p[1]).sum::<f32>() / n]
}

pub fn hull_perimeter(pts: &[[f32; 2]]) -> f32 {
    let mut p: Vec<[f64; 2]> = pts.iter().map(|q| [f64::from(q[0]), f64::from(q[1])]).collect();
    p.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    p.dedup();
    if p.len() < 3 {
        return 0.0;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    let mut hull: Vec<[f64; 2]> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        let iter: Box<dyn Iterator<Item = &[f64; 2]>> = if pass == 0 { Box::new(p.iter()) } else { Box::new(p.iter().rev()) };
        for q in iter {
            while hull.len() >= start + 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], *q) <= 0.0 {
                hull.pop();
            }
            hull.push(*q);
        }
        hull.pop();
    }
    let mut total = 0.0;
    for i in 0..hull.len() {
        let a = hull[i];
        let b = hull[(i + 1) % hull.len()];
        total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    }
    total as f32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub fn side_of(name: &str, x: f32) -> Side {
    let n = name.to_lowercase();
    let tokens: Vec<&str> = n.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()).collect();
    let left = n.contains("left") || tokens.contains(&"l");
    let right = n.contains("right") || tokens.contains(&"r");
    match (left, right) {
        (true, false) => Side::Left,
        (false, true) => Side::Right,
        _ => {
            if x >= 0.0 {
                Side::Left
            } else {
                Side::Right
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArmChain {
    pub shoulder: usize,
    pub elbow: usize,
    pub wrist: usize,
    pub side: Side,
}

fn is_finger(name: &str) -> bool {
    let n = name.to_lowercase();
    ["thumb", "index", "middle", "ring", "pinky", "finger", "little"].iter().any(|k| n.contains(k))
}

pub fn arm_chains(joints: &[Joint], pos: &[V3]) -> Vec<ArmChain> {
    let mut out: Vec<ArmChain> = Vec::new();
    let parent = |i: usize| joints[i].parent.map(|p| p as usize).filter(|p| *p < joints.len() && *p != i);
    for (i, j) in joints.iter().enumerate() {
        let n = j.name.to_lowercase();
        if is_finger(&n) || !(n.contains("wrist") || n.contains("hand")) {
            continue;
        }
        let name_of = |k: usize| joints[k].name.to_lowercase();
        let fore = |k: usize| {
            let s = name_of(k);
            ["forearm", "lowerarm", "lower_arm", "fore_arm", "elbow"].iter().any(|t| s.contains(t))
        };
        let upper = |k: usize| {
            let s = name_of(k);
            let stem = s.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '_' || c == 'l' || c == 'r').to_string();
            s.contains("upperarm") || s.contains("upper_arm") || (stem.ends_with("arm") && !fore(k))
        };
        let Some(mut elbow) = parent(i) else { continue };
        if name_of(elbow).contains("wrist") {
            let Some(p) = parent(elbow) else { continue };
            elbow = p;
        }
        while let Some(p) = parent(elbow).filter(|p| fore(*p)) {
            elbow = p;
        }
        let Some(mut shoulder) = parent(elbow) else { continue };
        while let Some(p) = parent(shoulder).filter(|p| upper(*p)) {
            if !upper(shoulder) {
                break;
            }
            shoulder = p;
        }
        let side = side_of(&j.name, pos[i][0]);
        if out.iter().any(|c| c.side == side) {
            continue;
        }
        out.push(ArmChain { shoulder, elbow, wrist: i, side });
    }
    out
}

pub struct Landmarks {
    pub floor: f32,
    pub top: f32,
    pub crotch: f32,
    pub waist: f32,
    pub hip: f32,
    pub chest: f32,
    pub shoulder: f32,
    pub center: [f32; 2],
}

fn scan(slicer: &Slicer, from: f32, to: f32, step: f32, want_max: bool) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    let mut y = from;
    while y <= to {
        if let Some(g) = slicer.girth(y) {
            let better = match best {
                None => true,
                Some((_, b)) => (want_max && g > b) || (!want_max && g < b),
            };
            if better {
                best = Some((y, g));
            }
        }
        y += step;
    }
    let (cy, _) = best?;
    let mut refined = best;
    let mut y = (cy - step).max(from);
    while y <= (cy + step).min(to) {
        if let Some(g) = slicer.girth(y) {
            let better = refined.is_none_or(|(_, b)| (want_max && g > b) || (!want_max && g < b));
            if better {
                refined = Some((y, g));
            }
        }
        y += step / 5.0;
    }
    refined
}

pub fn crotch_height(slicer: &Slicer) -> f32 {
    let (y0, y1) = slicer.range();
    let h = y1 - y0;
    let mut y = y0 + 0.30 * h;
    let end = y0 + 0.62 * h;
    while y < end {
        let mut areas: Vec<f32> = slicer.loops(y).iter().map(|l| l.area).collect();
        areas.sort_by(|a, b| b.total_cmp(a));
        if let Some(first) = areas.first() {
            let second = areas.get(1).copied().unwrap_or(0.0);
            if *first > 1.0 && *first > 1.7 * second && y > y0 + 0.36 * h {
                return y;
            }
        }
        y += 3.0;
    }
    y0 + 0.47 * h
}

pub fn landmarks(slicer: &Slicer, shoulder_y: Option<f32>) -> Landmarks {
    let (floor, top) = slicer.range();
    let h = top - floor;
    let crotch = crotch_height(slicer);
    let shoulder = shoulder_y.unwrap_or(floor + 0.818 * h);
    let span = (shoulder - crotch).max(1.0);
    let waist = scan(slicer, crotch + 0.25 * span, crotch + 0.55 * span, 6.0, false).map(|r| r.0).unwrap_or(crotch + 0.4 * span);
    let hip = scan(slicer, crotch + 0.02 * span, waist, 6.0, true).map(|r| r.0).unwrap_or(crotch + 0.15 * span);
    let chest = scan(slicer, waist + 0.2 * span, crotch + 0.78 * span, 6.0, true).map(|r| r.0).unwrap_or(crotch + 0.7 * span);
    let center = slicer.torso_loop(waist).map(|l| centroid(&l.pts)).unwrap_or([0.0, 0.0]);
    Landmarks { floor, top, crotch, waist, hip, chest, shoulder, center }
}

pub fn shoulder_height(joints: &[Joint], pos: &[V3]) -> Option<f32> {
    let chains = arm_chains(joints, pos);
    if chains.is_empty() {
        return None;
    }
    Some(chains.iter().map(|c| pos[c.shoulder][1]).sum::<f32>() / chains.len() as f32)
}

pub fn measure_geometry(pos: &[V3], tris: &[[u32; 3]], joints: &[Joint], frames: &skin::Frames) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    if pos.is_empty() || tris.is_empty() {
        return out;
    }
    let slicer = Slicer::new(pos, tris);
    let lm = landmarks(&slicer, shoulder_height(joints, &frames.pos));
    let put = |out: &mut BTreeMap<String, f64>, k: &str, v: f64| {
        out.insert(k.to_string(), (v * 10.0).round() / 10.0);
    };
    put(&mut out, "height", f64::from(lm.top - lm.floor));
    put(&mut out, "inseam", f64::from(lm.crotch - lm.floor));
    if let Some(g) = slicer.girth(lm.chest) {
        put(&mut out, "chest", f64::from(g));
        put(&mut out, "bust", f64::from(g));
    }
    if let Some(g) = slicer.girth(lm.waist) {
        put(&mut out, "waist", f64::from(g));
    }
    if let Some(g) = slicer.girth(lm.hip) {
        put(&mut out, "hip", f64::from(g));
    }
    let chains = arm_chains(joints, &frames.pos);
    let left = chains.iter().find(|c| c.side == Side::Left);
    let right = chains.iter().find(|c| c.side == Side::Right);
    match (left, right) {
        (Some(l), Some(r)) => put(&mut out, "shoulder_width", f64::from(length(sub(frames.pos[l.shoulder], frames.pos[r.shoulder])))),
        _ => {
            if let Some(l) = slicer.torso_loop(lm.shoulder - 0.04 * (lm.top - lm.floor)) {
                let lo = l.pts.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
                let hi = l.pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
                put(&mut out, "shoulder_width", f64::from(hi - lo));
            }
        }
    }
    if !chains.is_empty() {
        let total: f32 = chains.iter().map(|c| length(sub(frames.pos[c.elbow], frames.pos[c.shoulder])) + length(sub(frames.pos[c.wrist], frames.pos[c.elbow]))).sum();
        put(&mut out, "arm_length", f64::from(total / chains.len() as f32));
    }
    out
}

pub fn measure(body: &Body) -> BTreeMap<String, f64> {
    let posed = skin::posed_positions(body);
    let frames = skin::frames(&body.joints, Some(&body.pose));
    measure_geometry(&posed, &body.triangles, &body.joints, &frames)
}
