use std::collections::BTreeMap;

use crate::math::*;
use crate::{Body, Joint};

#[derive(Debug, Clone, Default)]
pub struct Frames {
    pub pos: Vec<V3>,
    pub rot: Vec<Q>,
}

pub fn order(joints: &[Joint]) -> Vec<usize> {
    let n = joints.len();
    let mut done = vec![false; n];
    let mut out = Vec::with_capacity(n);
    for _ in 0..=n {
        if out.len() == n {
            break;
        }
        for i in 0..n {
            if done[i] {
                continue;
            }
            let ready = match joints[i].parent {
                None => true,
                Some(p) => (p as usize) >= n || (p as usize) == i || done[p as usize],
            };
            if ready {
                done[i] = true;
                out.push(i);
            }
        }
    }
    for (i, d) in done.iter().enumerate() {
        if !d {
            out.push(i);
        }
    }
    out
}

fn parent_of(joints: &[Joint], i: usize) -> Option<usize> {
    joints[i].parent.map(|p| p as usize).filter(|p| *p < joints.len() && *p != i)
}

pub fn frames(joints: &[Joint], pose: Option<&BTreeMap<String, Q>>) -> Frames {
    let n = joints.len();
    let mut f = Frames { pos: vec![[0.0; 3]; n], rot: vec![IDENTITY; n] };
    for i in order(joints) {
        let j = &joints[i];
        let local = qnorm(j.rest_rotation);
        let delta = pose.and_then(|p| p.get(&j.name)).map(|q| qnorm(*q)).unwrap_or(IDENTITY);
        match parent_of(joints, i) {
            Some(p) => {
                f.pos[i] = add(f.pos[p], qrot(f.rot[p], j.rest));
                f.rot[i] = qmul(f.rot[p], qmul(local, delta));
            }
            None => {
                f.pos[i] = j.rest;
                f.rot[i] = qmul(local, delta);
            }
        }
    }
    f
}

pub fn set_from_world(joints: &mut [Joint], pos: &[V3], rot: &[Q]) {
    for i in 0..joints.len() {
        match parent_of(joints, i) {
            Some(p) => {
                let inv = qconj(qnorm(rot[p]));
                joints[i].rest = qrot(inv, sub(pos[i], pos[p]));
                joints[i].rest_rotation = qnorm(qmul(inv, rot[i]));
            }
            None => {
                joints[i].rest = pos[i];
                joints[i].rest_rotation = qnorm(rot[i]);
            }
        }
    }
}

pub fn bind_frames(body: &Body) -> Frames {
    frames(&body.joints, None)
}

pub fn pose_is_rest(body: &Body) -> bool {
    body.pose.values().all(|q| {
        let q = qnorm(*q);
        (q[3].abs() - 1.0).abs() < 1e-6
    })
}

pub fn posed_positions(body: &Body) -> Vec<V3> {
    if body.joints.is_empty() || body.weights.len() != body.positions.len() || pose_is_rest(body) {
        return body.positions.clone();
    }
    let bind = frames(&body.joints, None);
    let posed = frames(&body.joints, Some(&body.pose));
    let delta: Vec<Q> = (0..body.joints.len()).map(|i| qmul(posed.rot[i], qconj(bind.rot[i]))).collect();
    let mut out = Vec::with_capacity(body.positions.len());
    for (p, w) in body.positions.iter().zip(&body.weights) {
        let mut acc = [0.0f32; 3];
        let mut total = 0.0f32;
        for (j, weight) in w {
            let j = *j as usize;
            if *weight <= 0.0 || j >= delta.len() {
                continue;
            }
            let moved = add(posed.pos[j], qrot(delta[j], sub(*p, bind.pos[j])));
            acc = add(acc, scale(moved, *weight));
            total += *weight;
        }
        out.push(if total > 1e-6 { scale(acc, 1.0 / total) } else { *p });
    }
    out
}

pub fn compute_normals(positions: &[V3], triangles: &[[u32; 3]]) -> Vec<V3> {
    let mut n = vec![[0.0f32; 3]; positions.len()];
    for t in triangles {
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }
        let f = cross(sub(positions[b], positions[a]), sub(positions[c], positions[a]));
        for i in [a, b, c] {
            n[i] = add(n[i], f);
        }
    }
    for v in n.iter_mut() {
        *v = normalize(*v);
    }
    n
}

pub fn posed_normals(body: &Body, posed: &[V3]) -> Vec<V3> {
    compute_normals(posed, &body.triangles)
}

pub struct Bone {
    pub joint: usize,
    pub a: V3,
    pub b: V3,
}

pub fn hierarchy_bones(joints: &[Joint], pos: &[V3]) -> Vec<Bone> {
    let mut bones = Vec::new();
    for i in 0..joints.len() {
        let mut has_child = false;
        for j in 0..joints.len() {
            if parent_of(joints, j) == Some(i) {
                has_child = true;
                bones.push(Bone { joint: i, a: pos[i], b: pos[j] });
            }
        }
        if !has_child {
            bones.push(Bone { joint: i, a: pos[i], b: pos[i] });
        }
    }
    bones
}

fn segment_distance(p: V3, a: V3, b: V3) -> f32 {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 > 1e-9 { (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    length(sub(p, add(a, scale(ab, t))))
}

pub fn compute_weights(positions: &[V3], joint_count: usize, bones: &[Bone], sigma: f32) -> Vec<[(u16, f32); 4]> {
    let mut out = Vec::with_capacity(positions.len());
    let mut best = vec![f32::MAX; joint_count];
    for p in positions {
        best.iter_mut().for_each(|d| *d = f32::MAX);
        for b in bones {
            let d = segment_distance(*p, b.a, b.b);
            if d < best[b.joint] {
                best[b.joint] = d;
            }
        }
        let dmin = best.iter().copied().fold(f32::MAX, f32::min);
        let mut cand: Vec<(u16, f32)> = Vec::new();
        for (j, d) in best.iter().enumerate() {
            if *d == f32::MAX {
                continue;
            }
            let x = (*d - dmin) / sigma;
            let w = (-x * x).exp();
            if w > 1e-3 {
                cand.push((j as u16, w));
            }
        }
        cand.sort_by(|a, b| b.1.total_cmp(&a.1));
        cand.truncate(4);
        let total: f32 = cand.iter().map(|c| c.1).sum();
        let mut slot = [(0u16, 0.0f32); 4];
        if total > 0.0 {
            for (k, c) in cand.iter().enumerate() {
                slot[k] = (c.0, c.1 / total);
            }
        }
        out.push(slot);
    }
    out
}

pub fn normalize_weights(weights: &mut [[(u16, f32); 4]], joint_count: usize) {
    for w in weights.iter_mut() {
        for s in w.iter_mut() {
            if (s.0 as usize) >= joint_count || !s.1.is_finite() || s.1 < 0.0 {
                s.1 = 0.0;
            }
        }
        w.sort_by(|a, b| b.1.total_cmp(&a.1));
        let total: f32 = w.iter().map(|s| s.1).sum();
        if total > 0.0 {
            for s in w.iter_mut() {
                s.1 /= total;
            }
        }
    }
}
