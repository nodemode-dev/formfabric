use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ffb_body::math::*;
use gltf::animation::{Interpolation, Property, util::ReadOutputs};

use crate::clip::{Clip, ClipJoint, MAX_FRAMES, MAX_JOINTS};
use crate::interp::{hermite, slerp};

pub const MAX_BUFFER: usize = 512 << 20;

#[derive(Debug, Clone)]
pub struct Options {
    pub fps: f32,
    pub animation: Option<String>,
    pub units_mm: f32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Step,
    Linear,
    Cubic,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub times: Vec<f32>,
    pub values: Vec<[f32; 4]>,
    pub mode: Mode,
    pub rotation: bool,
}

impl Track {
    pub fn duration(&self) -> f32 {
        self.times.last().copied().unwrap_or(0.0)
    }

    pub fn sample(&self, t: f32) -> [f32; 4] {
        let n = self.times.len();
        if n == 0 {
            return if self.rotation { IDENTITY } else { [0.0; 4] };
        }
        let stride = if self.mode == Mode::Cubic { 3 } else { 1 };
        let value = |k: usize| self.values[k * stride + usize::from(stride == 3)];
        if n == 1 || t <= self.times[0] {
            return value(0);
        }
        if t >= self.times[n - 1] {
            return value(n - 1);
        }
        let hi = self.times.partition_point(|x| *x <= t).clamp(1, n - 1);
        let lo = hi - 1;
        let (t0, t1) = (self.times[lo], self.times[hi]);
        let dt = t1 - t0;
        if dt <= 1e-9 {
            return value(hi);
        }
        let s = (t - t0) / dt;
        match self.mode {
            Mode::Step => value(lo),
            Mode::Linear => {
                if self.rotation {
                    slerp(value(lo), value(hi), s)
                } else {
                    let (a, b) = (value(lo), value(hi));
                    [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, a[2] + (b[2] - a[2]) * s, 0.0]
                }
            }
            Mode::Cubic => {
                let (v0, v1) = (value(lo), value(hi));
                let out_tangent = self.values[lo * 3 + 2];
                let in_tangent = self.values[hi * 3];
                let mut r = [0.0f32; 4];
                for c in 0..4 {
                    r[c] = hermite(f64::from(v0[c]), f64::from(out_tangent[c]), f64::from(v1[c]), f64::from(in_tangent[c]), f64::from(s), f64::from(dt)) as f32;
                }
                if self.rotation {
                    qnorm(r)
                } else {
                    r[3] = 0.0;
                    r
                }
            }
        }
    }
}

fn base64(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return Err("bad base64 in a glTF data URI".into()),
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Ok(out)
}

fn external(dir: &Path, uri: &str) -> Result<Vec<u8>, String> {
    let name = uri.replace("%20", " ");
    let refuse = || format!("the glTF buffer URI {uri} is not a plain relative path inside the motion folder");
    if name.is_empty() || name.contains(':') || name.contains('\\') || name.contains('\0') || name.starts_with('/') {
        return Err(refuse());
    }
    let relative = Path::new(&name);
    if relative.components().any(|c| !matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir)) {
        return Err(refuse());
    }
    let root = dir.canonicalize().map_err(|e| format!("could not resolve the motion folder: {e}"))?;
    let full = root.join(relative).canonicalize().map_err(|e| format!("could not read buffer {name}: {e}"))?;
    if !full.starts_with(&root) {
        return Err(refuse());
    }
    let meta = std::fs::metadata(&full).map_err(|e| format!("could not read buffer {name}: {e}"))?;
    if !meta.is_file() || meta.len() as usize > MAX_BUFFER {
        return Err(format!("the glTF buffer {name} is not a regular file under {} MB", MAX_BUFFER >> 20));
    }
    std::fs::read(&full).map_err(|e| format!("could not read buffer {name}: {e}"))
}

fn quat_from(v: [f32; 4]) -> Q {
    qnorm(v)
}

pub fn animation_names(bytes: &[u8]) -> Result<Vec<String>, String> {
    let g = gltf::Gltf::from_slice(bytes).map_err(|e| format!("could not read the glTF file: {e}"))?;
    Ok(g.document.animations().map(|a| a.name().unwrap_or("").to_string()).collect())
}

pub fn parse(bytes: &[u8], dir: Option<&Path>, options: &Options) -> Result<Clip, String> {
    let g = gltf::Gltf::from_slice(bytes).map_err(|e| format!("could not read the glTF file: {e}"))?;
    let mut buffers: Vec<Vec<u8>> = Vec::new();
    let mut total = 0usize;
    for b in g.document.buffers() {
        let data = match b.source() {
            gltf::buffer::Source::Bin => g.blob.clone().ok_or("the GLB has no binary chunk")?,
            gltf::buffer::Source::Uri(uri) => {
                if let Some(rest) = uri.strip_prefix("data:") {
                    let (_, payload) = rest.split_once(',').ok_or("bad data URI")?;
                    if payload.len() > MAX_BUFFER {
                        return Err("a glTF data URI is too large".into());
                    }
                    base64(payload)?
                } else {
                    external(dir.ok_or("a .gltf with external buffers needs a file path")?, uri)?
                }
            }
        };
        total += data.len();
        if total > MAX_BUFFER * 2 {
            return Err("the glTF buffers are larger than the import limit".into());
        }
        buffers.push(data);
    }
    let animations: Vec<_> = g.document.animations().collect();
    if animations.is_empty() {
        return Err("the glTF file has no animation".into());
    }
    let animation = match &options.animation {
        None => &animations[0],
        Some(want) => {
            let by_index = want.parse::<usize>().ok().and_then(|i| animations.get(i));
            by_index.or_else(|| animations.iter().find(|a| a.name() == Some(want.as_str()))).ok_or_else(|| format!("the glTF file has no animation named {want}"))?
        }
    };
    let mut tracks: BTreeMap<(usize, u8), Track> = BTreeMap::new();
    for channel in animation.channels() {
        let target = channel.target();
        let kind = match target.property() {
            Property::Translation => 0u8,
            Property::Rotation => 1,
            Property::Scale => 2,
            Property::MorphTargetWeights => continue,
        };
        let reader = channel.reader(|b| buffers.get(b.index()).map(|d| &d[..]));
        let Some(inputs) = reader.read_inputs() else { continue };
        let Some(outputs) = reader.read_outputs() else { continue };
        let times: Vec<f32> = inputs.collect();
        let values: Vec<[f32; 4]> = match outputs {
            ReadOutputs::Translations(it) => it.map(|v| [v[0], v[1], v[2], 0.0]).collect(),
            ReadOutputs::Scales(it) => it.map(|v| [v[0], v[1], v[2], 0.0]).collect(),
            ReadOutputs::Rotations(r) => r.into_f32().collect(),
            ReadOutputs::MorphTargetWeights(_) => continue,
        };
        let mode = match channel.sampler().interpolation() {
            Interpolation::Linear => Mode::Linear,
            Interpolation::Step => Mode::Step,
            Interpolation::CubicSpline => Mode::Cubic,
        };
        let want = if mode == Mode::Cubic { times.len() * 3 } else { times.len() };
        if times.is_empty() || values.len() != want || times.iter().any(|t| !t.is_finite()) || values.iter().any(|v| v.iter().any(|x| !x.is_finite())) {
            return Err("a glTF animation channel has inconsistent or non-finite data".into());
        }
        tracks.insert((target.node().index(), kind), Track { times, values, mode, rotation: kind == 1 });
    }
    if tracks.is_empty() {
        return Err("the glTF animation has no translation, rotation or scale channel".into());
    }
    let node_count = g.document.nodes().len();
    let mut parent: Vec<Option<usize>> = vec![None; node_count];
    for n in g.document.nodes() {
        for c in n.children() {
            parent[c.index()] = Some(n.index());
        }
    }
    let mut keep: BTreeSet<usize> = BTreeSet::new();
    for skin in g.document.skins() {
        for j in skin.joints() {
            keep.insert(j.index());
        }
    }
    if keep.is_empty() {
        for (node, _) in tracks.keys() {
            keep.insert(*node);
        }
    }
    let animated: BTreeSet<usize> = tracks.keys().map(|(n, _)| *n).collect();
    if !keep.iter().any(|n| animated.contains(n)) {
        keep.extend(animated.iter().copied());
    }
    let mut closure: BTreeSet<usize> = keep.clone();
    for n in keep.iter() {
        let mut p = parent[*n];
        while let Some(x) = p {
            if closure.contains(&x) {
                break;
            }
            if animated.contains(&x) {
                closure.insert(x);
            }
            p = parent[x];
        }
    }
    if closure.len() > MAX_JOINTS {
        return Err(format!("the glTF animation drives more than {MAX_JOINTS} joints"));
    }
    let nodes: Vec<gltf::Node> = g.document.nodes().collect();
    let in_set = |i: usize| closure.contains(&i);
    let joint_parent = |i: usize| -> Option<usize> {
        let mut p = parent[i];
        while let Some(x) = p {
            if in_set(x) {
                return Some(x);
            }
            p = parent[x];
        }
        None
    };
    let mut ancestor_rot: BTreeMap<usize, (Q, f32, V3)> = BTreeMap::new();
    for i in closure.iter() {
        if joint_parent(*i).is_some() {
            continue;
        }
        let mut chain = Vec::new();
        let mut p = parent[*i];
        while let Some(x) = p {
            chain.push(x);
            p = parent[x];
        }
        let (mut rot, mut sc, mut tr) = (IDENTITY, 1.0f32, [0.0f32; 3]);
        for x in chain.iter().rev() {
            let (t, r, s) = nodes[*x].transform().decomposed();
            let us = (s[0].abs() + s[1].abs() + s[2].abs()) / 3.0;
            tr = add(tr, qrot(rot, scale(t, sc)));
            rot = qnorm(qmul(rot, quat_from(r)));
            sc *= us;
        }
        ancestor_rot.insert(*i, (rot, sc, tr));
    }
    let duration = tracks.values().map(Track::duration).fold(0.0f32, f32::max);
    if !(duration.is_finite() && duration >= 0.0) || duration > 3600.0 {
        return Err("the glTF animation is longer than one hour".into());
    }
    let fps = if options.fps.is_finite() && options.fps >= 1.0 && options.fps <= 240.0 { options.fps } else { 30.0 };
    let frames = ((duration * fps).round() as usize + 1).clamp(1, MAX_FRAMES);
    let order: Vec<usize> = closure.iter().copied().collect();
    let mut index = BTreeMap::new();
    for (k, n) in order.iter().enumerate() {
        index.insert(*n, k);
    }
    let unit = if options.units_mm.is_finite() && options.units_mm > 0.0 { options.units_mm } else { 1000.0 };
    let mut joints = Vec::with_capacity(order.len());
    let mut frame_of: Vec<(Q, f32, V3, bool)> = Vec::new();
    for n in &order {
        let (t, r, _) = nodes[*n].transform().decomposed();
        let jp = joint_parent(*n).map(|p| index[&p]);
        let mut top = *n;
        while let Some(p) = joint_parent(top) {
            top = p;
        }
        let (arot, asc, atr) = ancestor_rot.get(&top).copied().unwrap_or((IDENTITY, 1.0, [0.0; 3]));
        let is_root = jp.is_none();
        frame_of.push((arot, asc, scale(atr, unit), is_root));
        let (rp, rr) = rest_transform(frame_of[frame_of.len() - 1], scale(t, unit), quat_from(r));
        joints.push(ClipJoint { name: nodes[*n].name().map(str::to_string).unwrap_or_else(|| format!("node_{n}")), parent: jp, rest_pos: rp, rest_rot: rr });
    }
    let nj = order.len();
    let mut rot = Vec::with_capacity(frames * nj);
    let mut pos = Vec::with_capacity(frames * nj);
    for f in 0..frames {
        let time = f as f32 / fps;
        for (k, n) in order.iter().enumerate() {
            let (t0, r0, _) = nodes[*n].transform().decomposed();
            let t = tracks.get(&(*n, 0)).map_or(t0, |tr| {
                let v = tr.sample(time);
                [v[0], v[1], v[2]]
            });
            let r = tracks.get(&(*n, 1)).map_or(quat_from(r0), |tr| quat_from(tr.sample(time)));
            let (p, q) = rest_transform(frame_of[k], scale(t, unit), r);
            rot.push(q);
            pos.push(p);
        }
    }
    let clip = Clip { name: options.name.clone(), fps, joints, frames, rot, pos }.order_parents_first();
    clip.check()?;
    Ok(clip)
}

fn rest_transform(frame: (Q, f32, V3, bool), p: V3, q: Q) -> (V3, Q) {
    let (arot, asc, atr, is_root) = frame;
    if is_root {
        (add(atr, qrot(arot, scale(p, asc))), qnorm(qmul(arot, q)))
    } else {
        (scale(p, asc), q)
    }
}
