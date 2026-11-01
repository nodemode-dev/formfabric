#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use std::collections::BTreeMap;

use ffb_anim::clip::Clip;
use ffb_anim::interp;
use ffb_anim::retarget::{self, Options, Retargeter};
use ffb_anim::Key;
use ffb_body::math::{lerp, smoothstep, Q};
use ffb_body::resize::{self, Targets};
use ffb_body::{pose, skin, Body};
use ffb_pattern::anim::Anim;
use serde_json::Value;

pub(crate) const MEASUREMENTS: [&str; 8] = ["height", "inseam", "chest", "bust", "waist", "hip", "shoulder_width", "arm_length"];

pub(crate) struct MotionRig {
    pub start: u32,
    pub lead_in: u32,
    pub clip: Clip,
    pub rt: Retargeter,
}

pub(crate) struct MorphRig {
    pub to: usize,
    pub start: u32,
    pub frames: u32,
}

pub(crate) struct BodyRig {
    pub index: usize,
    pub base_pose: BTreeMap<String, Q>,
    pub motions: Vec<MotionRig>,
    pub pose_keys: Vec<Key>,
    pub measure_keys: Vec<(String, Vec<Key>)>,
    pub morph: Option<MorphRig>,
    pub pristine: Body,
    pub measured: Vec<(String, f64)>,
}

pub(crate) struct Evaluated {
    pub pose: BTreeMap<String, Q>,
    pub offset: [f32; 3],
    pub morph: f32,
}

pub(crate) fn motion_entry(id: u32) -> String {
    format!("motion/{id}.bin")
}

pub(crate) fn cache_entry(id: u32) -> String {
    format!("cache/{id}.bin")
}

pub(crate) fn track_keys<'a>(anim: &'a Anim, target: &str) -> &'a [Key] {
    anim.tracks.iter().find(|t| t.target == target).map_or(&[], |t| &t.keys)
}

pub(crate) fn build_rigs(anim: &Anim, bodies: &[Body], unknown: &[(String, Vec<u8>)], warnings: &mut Vec<String>) -> Vec<BodyRig> {
    let mut rigs = Vec::new();
    for (index, body) in bodies.iter().enumerate() {
        let mut motions = Vec::new();
        let mut morph = None;
        for m in anim.motions.iter().filter(|m| m.body == body.id) {
            if let Some(to) = m.source.strip_prefix("morph:") {
                let to_id: u32 = to.parse().unwrap_or(0);
                match bodies.iter().position(|b| b.id == to_id) {
                    Some(to) => morph = Some(MorphRig { to, start: m.start, frames: m.frames.max(1) }),
                    None => warnings.push(format!("morph {} targets a body that no longer exists", m.id)),
                }
                continue;
            }
            let Some((_, data)) = unknown.iter().find(|(n, _)| *n == motion_entry(m.id)) else {
                warnings.push(format!("motion {} has no stored data and was skipped", m.id));
                continue;
            };
            let clip = match Clip::decode(data) {
                Ok(c) => c,
                Err(e) => {
                    warnings.push(format!("motion {} could not be read: {e}", m.id));
                    continue;
                }
            };
            let options = Options::from_json(&m.retarget).unwrap_or_default();
            match Retargeter::new(&clip, &body.joints, &options) {
                Ok(rt) => motions.push(MotionRig { start: m.start, lead_in: m.lead_in, clip, rt }),
                Err(e) => warnings.push(format!("motion {} does not fit body {}: {e}", m.id, body.id)),
            }
        }
        motions.sort_by_key(|m| m.start);
        let prefix = format!("body.{}.", body.id);
        let pose_keys = track_keys(anim, &format!("{prefix}pose")).to_vec();
        let mut measure_keys = Vec::new();
        for name in MEASUREMENTS {
            let keys = track_keys(anim, &format!("{prefix}{name}"));
            if !keys.is_empty() {
                measure_keys.push((name.to_string(), keys.to_vec()));
            }
        }
        if motions.is_empty() && pose_keys.is_empty() && morph.is_none() && measure_keys.is_empty() {
            continue;
        }
        rigs.push(BodyRig { index, base_pose: body.pose.clone(), motions, pose_keys, measure_keys, morph, pristine: body.clone(), measured: Vec::new() });
    }
    rigs
}

fn merge(into: &mut BTreeMap<String, Q>, from: BTreeMap<String, Q>) {
    for (k, v) in from {
        into.insert(k, v);
    }
}

impl BodyRig {
    pub fn evaluate(&self, anim: &Anim, frame: f64) -> Evaluated {
        let mut pose = self.base_pose.clone();
        let mut offset = [0.0f32; 3];
        let active = self.motions.iter().rposition(|m| f64::from(m.start) <= frame);
        if let Some(k) = active {
            let m = &self.motions[k];
            let local = (frame - f64::from(m.start)).max(0.0);
            let lead = f64::from(m.lead_in);
            if local < lead {
                let first = m.rt.pose_at(&m.clip, 0.0);
                let w = smoothstep((local / lead) as f32);
                let blended = retarget::blend(&self.base_pose, &first, [0.0; 3], w);
                merge(&mut pose, blended.rotations);
                offset = blended.offset;
            } else {
                let seconds = ((local - lead) / f64::from(anim.fps.max(1.0))) as f32;
                let posed = m.rt.pose_at(&m.clip, seconds);
                merge(&mut pose, posed.rotations);
                offset = posed.offset;
            }
        }
        if let Some(v) = interp::evaluate(&self.pose_keys, frame) {
            match v {
                Value::String(name) => {
                    if let Ok(named) = pose::named(&self.pristine, &name) {
                        if name.trim().eq_ignore_ascii_case("rest") || name.trim().eq_ignore_ascii_case("bind") {
                            pose.clear();
                        }
                        merge(&mut pose, named);
                    }
                }
                other => merge(&mut pose, interp::quat_pose(&other)),
            }
        }
        let morph = self.morph.as_ref().map_or(0.0, |m| ((frame - f64::from(m.start)) / f64::from(m.frames)).clamp(0.0, 1.0) as f32);
        Evaluated { pose, offset, morph }
    }

    pub fn measurement_targets(&self, frame: f64) -> Vec<(String, f64)> {
        self.measure_keys.iter().filter_map(|(n, k)| interp::evaluate_number(k, frame).map(|v| (n.clone(), v))).collect()
    }
}

pub(crate) fn resized(base: &Body, targets: &[(String, f64)]) -> Option<Body> {
    let get = |n: &str| targets.iter().find(|(k, _)| k == n).map(|(_, v)| *v);
    let t = Targets { height: get("height"), inseam: get("inseam"), chest: get("chest").or_else(|| get("bust")), waist: get("waist"), hip: get("hip"), shoulder_width: get("shoulder_width"), arm_length: get("arm_length") };
    if !t.any() {
        return None;
    }
    let mut body = base.clone();
    resize::resize(&mut body, &t).ok()?;
    Some(body)
}

pub(crate) fn posed(body: &mut Body, e: &Evaluated) -> Vec<[f32; 3]> {
    body.pose = e.pose.clone();
    let mut p = skin::posed_positions(body);
    if e.offset != [0.0; 3] {
        for v in p.iter_mut() {
            *v = [v[0] + e.offset[0], v[1] + e.offset[1], v[2] + e.offset[2]];
        }
    }
    p
}

pub(crate) fn blend_positions(a: &[[f32; 3]], b: &[[f32; 3]], t: f32) -> Vec<[f32; 3]> {
    a.iter().zip(b).map(|(x, y)| lerp(*x, *y, t)).collect()
}
