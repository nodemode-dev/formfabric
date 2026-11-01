use crate::math::V3;
use crate::measure::{self, Side};
use crate::place::{add, cross, dot, length, scale, sub, unit, V};
use crate::skin;
use crate::{Body, Joint, PlacementPoint, PlacementVolume, VolumeKind};

struct LegChain {
    hip: usize,
    knee: usize,
    ankle: usize,
    side: Side,
}

fn v64(a: V3) -> V {
    [f64::from(a[0]), f64::from(a[1]), f64::from(a[2])]
}

fn parent_of(joints: &[Joint], i: usize) -> Option<usize> {
    joints[i].parent.map(usize::from).filter(|p| *p < joints.len() && *p != i)
}

fn depth(joints: &[Joint], i: usize) -> usize {
    let mut d = 0;
    let mut at = i;
    while let Some(p) = parent_of(joints, at) {
        d += 1;
        at = p;
        if d > joints.len() {
            break;
        }
    }
    d
}

fn ancestors(joints: &[Joint], i: usize) -> Vec<usize> {
    let mut out = vec![i];
    let mut at = i;
    while let Some(p) = parent_of(joints, at) {
        out.push(p);
        at = p;
        if out.len() > joints.len() {
            break;
        }
    }
    out
}

fn common_ancestor(joints: &[Joint], a: usize, b: usize) -> Option<usize> {
    let other = ancestors(joints, b);
    ancestors(joints, a).into_iter().find(|x| other.contains(x))
}

fn has(name: &str, tokens: &[&str]) -> bool {
    let n = name.to_lowercase();
    tokens.iter().any(|t| n.contains(t))
}

fn leg_chains(joints: &[Joint], pos: &[V3]) -> Vec<LegChain> {
    let mut out: Vec<LegChain> = Vec::new();
    for (i, j) in joints.iter().enumerate() {
        if !has(&j.name, &["foot", "ankle"]) || has(&j.name, &["toe", "ball"]) {
            continue;
        }
        let mut chain = Vec::new();
        let mut at = i;
        while let Some(p) = parent_of(joints, at) {
            if !has(&joints[p].name, &["leg", "thigh", "shin", "calf", "knee", "hip"]) || has(&joints[p].name, &["pelvis"]) {
                break;
            }
            chain.push(p);
            at = p;
        }
        if chain.len() < 2 {
            continue;
        }
        let hip = chain[chain.len() - 1];
        let knee = chain.iter().rev().copied().find(|k| has(&joints[*k].name, &["lower", "shin", "calf", "knee"])).unwrap_or(chain[chain.len() / 2]);
        if knee == hip {
            continue;
        }
        let side = measure::side_of(&j.name, pos[i][0]);
        if out.iter().any(|c| c.side == side) {
            continue;
        }
        out.push(LegChain { hip, knee, ankle: i, side });
    }
    out
}

fn neck_joint(joints: &[Joint]) -> Option<usize> {
    joints.iter().enumerate().filter(|(_, j)| has(&j.name, &["neck"])).min_by_key(|(i, _)| depth(joints, *i)).map(|(i, _)| i)
}

fn head_joint(joints: &[Joint]) -> Option<usize> {
    joints.iter().position(|j| j.name.eq_ignore_ascii_case("head")).or_else(|| joints.iter().position(|j| has(&j.name, &["head"]) && !has(&j.name, &["top", "end"])))
}

struct Maker {
    volumes: Vec<PlacementVolume>,
    points: Vec<PlacementPoint>,
}

impl Maker {
    fn volume(&mut self, name: &str, joints: [usize; 2], start: V, end: V, radius: f64, ratio: f64) -> u32 {
        let id = self.volumes.len() as u32 + 1;
        let span = sub(end, start);
        self.volumes.push(PlacementVolume {
            id,
            name: name.into(),
            kind: VolumeKind::Cylinder,
            joints: [Some(joints[0] as u16), Some(joints[1] as u16)],
            center: scale(add(start, end), 0.5),
            axis: unit(span),
            radius: [radius * ratio, radius],
            height: length(span),
        });
        id
    }

    fn point(&mut self, name: &str, volume: u32, around: f64, along: f64, offset: f64, mirror_of: Option<u32>) -> u32 {
        let id = self.points.len() as u32 + 1;
        self.points.push(PlacementPoint { id, name: name.into(), volume, around, along, offset, wrap: true, mirror_of });
        id
    }

    fn pair(&mut self, names: [&str; 2], volumes: [u32; 2], around: f64, along: f64, offset: f64) {
        let l = self.point(names[0], volumes[0], around, along, offset, None);
        self.point(names[1], volumes[1], (360.0 - around) % 360.0, along, offset, Some(l));
    }
}

fn outward_around(v: &PlacementVolume, sign: f64) -> f64 {
    let axis = unit(v.axis);
    let reference = if axis[2].abs() > 0.9 { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
    let front = unit(sub(reference, scale(axis, dot(reference, axis))));
    let side = cross(axis, front);
    let want = [sign, 0.0, 0.0];
    let mut flat = sub(want, scale(axis, dot(want, axis)));
    if length(flat) < 0.3 {
        flat = [0.0, 1.0, 0.0];
    }
    let flat = unit(flat);
    dot(flat, side).atan2(dot(flat, front)).to_degrees().rem_euclid(360.0)
}

pub fn default_volumes(body: &Body) -> (Vec<PlacementVolume>, Vec<PlacementPoint>) {
    let bind = skin::bind_frames(body);
    let pos: Vec<V> = bind.pos.iter().map(|p| v64(*p)).collect();
    let arms = measure::arm_chains(&body.joints, &bind.pos);
    let legs = leg_chains(&body.joints, &bind.pos);
    let mut m = Maker { volumes: Vec::new(), points: Vec::new() };
    let left_leg = legs.iter().find(|c| c.side == Side::Left);
    let right_leg = legs.iter().find(|c| c.side == Side::Right);
    let neck = neck_joint(&body.joints);
    let torso = match (left_leg, right_leg, neck) {
        (Some(l), Some(r), Some(n)) => common_ancestor(&body.joints, l.hip, r.hip).map(|root| (root, scale(add(pos[l.hip], pos[r.hip]), 0.5), n)),
        _ => None,
    };
    let Some((pelvis, hips, neck)) = torso else { return (Vec::new(), Vec::new()) };
    let span = length(sub(pos[neck], hips)).max(1.0);
    let torso_id = m.volume("torso", [pelvis, neck], hips, pos[neck], span * 0.24, 175.0 / 127.0);
    let neck_id = head_joint(&body.joints).map(|h| m.volume("neck", [neck, h], pos[neck], pos[h], span * 0.095, 1.0));
    let mut upper = [None, None];
    let mut lower = [None, None];
    for c in &arms {
        let k = usize::from(c.side == Side::Right);
        upper[k] = Some(m.volume(if k == 0 { "upper_arm_L" } else { "upper_arm_R" }, [c.shoulder, c.elbow], pos[c.shoulder], pos[c.elbow], span * 0.095, 1.0));
        lower[k] = Some(m.volume(if k == 0 { "lower_arm_L" } else { "lower_arm_R" }, [c.elbow, c.wrist], pos[c.elbow], pos[c.wrist], span * 0.075, 1.0));
    }
    let mut thigh = [None, None];
    let mut shin = [None, None];
    for c in &legs {
        let k = usize::from(c.side == Side::Right);
        thigh[k] = Some(m.volume(if k == 0 { "thigh_L" } else { "thigh_R" }, [c.hip, c.knee], pos[c.hip], pos[c.knee], span * 0.17, 1.0));
        shin[k] = Some(m.volume(if k == 0 { "shin_L" } else { "shin_R" }, [c.knee, c.ankle], pos[c.knee], pos[c.ankle], span * 0.11, 1.0));
    }
    m.point("front", torso_id, 0.0, 0.55, 15.0, None);
    m.point("back", torso_id, 180.0, 0.55, 15.0, None);
    m.pair(["left", "right"], [torso_id, torso_id], 90.0, 0.55, 15.0);
    m.point("skirt_front", torso_id, 0.0, 0.12, 15.0, None);
    m.point("skirt_back", torso_id, 180.0, 0.12, 15.0, None);
    if let Some(n) = neck_id {
        m.point("neck_front", n, 0.0, 0.5, 8.0, None);
        m.point("neck_back", n, 180.0, 0.5, 8.0, None);
    }
    if let (Some(ul), Some(ur)) = (upper[0], upper[1]) {
        let around = outward_around(&m.volumes[ul as usize - 1], 1.0);
        m.pair(["sleeve_L", "sleeve_R"], [ul, ur], around, 0.5, 10.0);
    }
    if let (Some(ll), Some(lr)) = (lower[0], lower[1]) {
        let around = outward_around(&m.volumes[ll as usize - 1], 1.0);
        m.pair(["cuff_L", "cuff_R"], [ll, lr], around, 0.5, 10.0);
    }
    if let (Some(tl), Some(tr)) = (thigh[0], thigh[1]) {
        m.pair(["leg_front_L", "leg_front_R"], [tl, tr], 0.0, 0.5, 10.0);
        m.pair(["leg_back_L", "leg_back_R"], [tl, tr], 180.0, 0.5, 10.0);
    }
    if let (Some(sl), Some(sr)) = (shin[0], shin[1]) {
        m.pair(["calf_front_L", "calf_front_R"], [sl, sr], 0.0, 0.5, 10.0);
        m.pair(["calf_back_L", "calf_back_R"], [sl, sr], 180.0, 0.5, 10.0);
    }
    (m.volumes, m.points)
}
