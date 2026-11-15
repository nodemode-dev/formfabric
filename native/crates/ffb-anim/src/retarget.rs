use std::collections::BTreeMap;

use ffb_body::math::*;
use ffb_body::{skin, Joint};
use serde_json::{json, Value};

use crate::clip::{world, Clip};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootMode {
    Keep,
    InPlace,
    Off,
}

impl RootMode {
    pub fn name(self) -> &'static str {
        match self {
            RootMode::Keep => "keep",
            RootMode::InPlace => "inplace",
            RootMode::Off => "off",
        }
    }

    pub fn parse(s: &str) -> Option<RootMode> {
        match s.trim().to_lowercase().as_str() {
            "keep" | "full" => Some(RootMode::Keep),
            "inplace" | "in_place" | "in-place" => Some(RootMode::InPlace),
            "off" | "none" => Some(RootMode::Off),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub map: BTreeMap<String, String>,
    pub root: RootMode,
    pub scale: Option<f32>,
    pub looped: bool,
    pub align: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { map: BTreeMap::new(), root: RootMode::InPlace, scale: None, looped: false, align: true }
    }
}

impl Options {
    pub fn from_json(v: &Value) -> Result<Options, String> {
        let mut o = Options::default();
        if v.is_null() {
            return Ok(o);
        }
        let m = v.as_object().ok_or("retarget must be an object")?;
        for (k, x) in m {
            match k.as_str() {
                "map" => {
                    let map = x.as_object().ok_or("retarget.map must map source joint names to body joint names")?;
                    for (a, b) in map {
                        o.map.insert(a.clone(), b.as_str().ok_or("retarget.map values must be joint names")?.to_string());
                    }
                }
                "root" => o.root = x.as_str().and_then(RootMode::parse).ok_or("retarget.root must be keep, inplace or off")?,
                "scale" => {
                    if !x.is_null() {
                        let s = x.as_f64().ok_or("retarget.scale must be a number")?;
                        if !(s.is_finite() && s > 0.0 && s < 100.0) {
                            return Err("retarget.scale must be between 0 and 100".into());
                        }
                        o.scale = Some(s as f32);
                    }
                }
                "loop" => o.looped = x.as_bool().ok_or("retarget.loop must be true or false")?,
                "align" => o.align = x.as_bool().ok_or("retarget.align must be true or false")?,
                other => return Err(format!("retarget has no option {other}")),
            }
        }
        Ok(o)
    }

    pub fn to_json(&self) -> Value {
        json!({"map": self.map, "root": self.root.name(), "scale": self.scale, "loop": self.looped, "align": self.align})
    }
}

struct Canon {
    side: char,
    key: &'static str,
}

fn strip_prefix(name: &str) -> String {
    let lower = name.to_lowercase();
    let lower = lower.rsplit(':').next().unwrap_or(&lower).to_string();
    for p in ["bip01 ", "bip001 ", "bip01_", "cc_base_", "mixamorig_", "mixamorig", "def-", "def_", "j_bip_c_", "j_bip_"] {
        if let Some(rest) = lower.strip_prefix(p) {
            return rest.to_string();
        }
    }
    lower
}

fn canon(name: &str) -> Option<Canon> {
    let s = strip_prefix(name);
    let (side, core) = if let Some(r) = s.strip_prefix("left") {
        ('L', r.to_string())
    } else if let Some(r) = s.strip_prefix("right") {
        ('R', r.to_string())
    } else if let Some(r) = s.strip_prefix("l_").or_else(|| s.strip_prefix("l ")).or_else(|| s.strip_prefix("l.")) {
        ('L', r.to_string())
    } else if let Some(r) = s.strip_prefix("r_").or_else(|| s.strip_prefix("r ")).or_else(|| s.strip_prefix("r.")) {
        ('R', r.to_string())
    } else if let Some(r) = s.strip_suffix(".l").or_else(|| s.strip_suffix("_l")).or_else(|| s.strip_suffix("-l")).or_else(|| s.strip_suffix(" l")) {
        ('L', r.to_string())
    } else if let Some(r) = s.strip_suffix(".r").or_else(|| s.strip_suffix("_r")).or_else(|| s.strip_suffix("-r")).or_else(|| s.strip_suffix(" r")) {
        ('R', r.to_string())
    } else {
        ('C', s)
    };
    let core: String = core.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let key = match (side, core.as_str()) {
        ('C', "hips" | "hip" | "pelvis" | "root" | "cog" | "center") => "hips",
        ('C', "spine" | "lowerback" | "spine0") => "spine",
        ('C', "spine1" | "chest") => "spine1",
        ('C', "spine2" | "upperchest" | "spine3" | "upperback") => "spine2",
        ('C', "neck" | "neck1" | "neck01") => "neck",
        ('C', "head") => "head",
        ('L' | 'R', "shoulder" | "clavicle" | "collar" | "shoulder01") => "shoulder",
        ('L' | 'R', "arm" | "upperarm" | "uparm" | "upperarm01") => "arm",
        ('L' | 'R', "forearm" | "lowerarm" | "loarm" | "lowerarm01") => "forearm",
        ('L' | 'R', "hand" | "wrist") => "hand",
        ('L' | 'R', "upleg" | "upperleg" | "thigh" | "femur" | "upperleg01") => "thigh",
        ('L' | 'R', "leg" | "lowerleg" | "calf" | "shin" | "lowerleg01") => "calf",
        ('L' | 'R', "foot" | "ankle") => "foot",
        _ => return None,
    };
    Some(Canon { side, key })
}

fn mpfb_name(c: &Canon) -> Option<String> {
    let side = |base: &str| format!("{base}.{}", c.side);
    Some(match c.key {
        "hips" => "root".to_string(),
        "spine" => "spine05".to_string(),
        "spine1" => "spine03".to_string(),
        "spine2" => "spine01".to_string(),
        "neck" => "neck01".to_string(),
        "head" => "head".to_string(),
        "shoulder" => side("clavicle"),
        "arm" => side("upperarm01"),
        "forearm" => side("lowerarm01"),
        "hand" => side("wrist"),
        "thigh" => side("upperleg01"),
        "calf" => side("lowerleg01"),
        "foot" => side("foot"),
        _ => return None,
    })
}

pub fn build_map(clip: &Clip, joints: &[Joint], overrides: &BTreeMap<String, String>) -> Vec<(usize, usize)> {
    let mut by_name: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, j) in joints.iter().enumerate() {
        by_name.insert(j.name.as_str(), i);
    }
    let mut by_canon: BTreeMap<(char, &'static str), usize> = BTreeMap::new();
    for (i, j) in joints.iter().enumerate() {
        if let Some(c) = canon(&j.name) {
            by_canon.entry((c.side, c.key)).or_insert(i);
        }
    }
    let mut used = vec![false; joints.len()];
    let mut out: Vec<(usize, usize)> = Vec::new();
    let take = |s: usize, t: usize, out: &mut Vec<(usize, usize)>, used: &mut Vec<bool>| {
        if !used[t] {
            used[t] = true;
            out.push((s, t));
        }
    };
    for (s, cj) in clip.joints.iter().enumerate() {
        if let Some(target) = overrides.get(&cj.name) {
            if let Some(t) = by_name.get(target.as_str()) {
                take(s, *t, &mut out, &mut used);
            }
        }
    }
    let done = |out: &Vec<(usize, usize)>, s: usize| out.iter().any(|p| p.0 == s);
    for (s, cj) in clip.joints.iter().enumerate() {
        if done(&out, s) {
            continue;
        }
        if let Some(t) = by_name.get(cj.name.as_str()) {
            take(s, *t, &mut out, &mut used);
        }
    }
    for (s, cj) in clip.joints.iter().enumerate() {
        if done(&out, s) {
            continue;
        }
        let Some(c) = canon(&cj.name) else { continue };
        let named = mpfb_name(&c).and_then(|n| by_name.get(n.as_str()).copied());
        let t = named.or_else(|| by_canon.get(&(c.side, c.key)).copied());
        if let Some(t) = t {
            take(s, t, &mut out, &mut used);
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct Retargeter {
    pairs: Vec<(usize, usize)>,
    z: Vec<Q>,
    src_of: Vec<Option<usize>>,
    tgt_rest_rot: Vec<Q>,
    tgt_local: Vec<Q>,
    parent: Vec<Option<usize>>,
    order: Vec<usize>,
    names: Vec<String>,
    hips: Option<usize>,
    ratio: f32,
    root: RootMode,
    looped: bool,
    src_rest_rot: Vec<Q>,
    first: Option<V3>,
}

#[derive(Debug, Clone, Default)]
pub struct Posed {
    pub rotations: BTreeMap<String, Q>,
    pub offset: V3,
}

fn direction(from: V3, to: V3) -> Option<V3> {
    let d = sub(to, from);
    let l = length(d);
    (l > 1e-4).then(|| scale(d, 1.0 / l))
}

impl Retargeter {
    pub fn new(clip: &Clip, joints: &[Joint], options: &Options) -> Result<Retargeter, String> {
        if joints.is_empty() {
            return Err("this body has no skeleton to receive a motion".into());
        }
        let pairs = build_map(clip, joints, &options.map);
        if pairs.len() < 4 {
            let names: Vec<&str> = clip.joints.iter().take(12).map(|j| j.name.as_str()).collect();
            return Err(format!("only {} motion joints map onto this body; the motion has {} joints, for example {}. Give retarget.map from motion joint names to body joint names", pairs.len(), clip.joints.len(), names.join(", ")));
        }
        let n = joints.len();
        let rest = clip.rest();
        let (src_wp, src_wr) = world(&clip.joints, &rest.rot, &rest.pos);
        let bind = skin::bind_frames(&ffb_body::Body { joints: joints.to_vec(), ..ffb_body::Body::default() });
        let mut src_of = vec![None; n];
        let mut tgt_of = vec![None; clip.joints.len()];
        for (s, t) in &pairs {
            src_of[*t] = Some(*s);
            tgt_of[*s] = Some(*t);
        }
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); clip.joints.len()];
        for (i, j) in clip.joints.iter().enumerate() {
            if let Some(p) = j.parent {
                children[p].push(i);
            }
        }
        fn first_mapped(s: usize, children: &[Vec<usize>], tgt_of: &[Option<usize>], out: &mut Vec<usize>) {
            for c in &children[s] {
                if tgt_of[*c].is_some() {
                    out.push(*c);
                } else {
                    first_mapped(*c, children, tgt_of, out);
                }
            }
        }
        let mut z = vec![IDENTITY; n];
        let mut z_done = vec![false; n];
        let tparent: Vec<Option<usize>> = joints.iter().map(|j| j.parent.map(|p| p as usize).filter(|p| *p < n)).collect();
        let order = skin::order(joints);
        if options.align {
            for t in &order {
                let Some(s) = src_of[*t] else { continue };
                let mut kids = Vec::new();
                first_mapped(s, &children, &tgt_of, &mut kids);
                if kids.len() == 1 {
                    let c = kids[0];
                    let tc = tgt_of[c].unwrap_or(*t);
                    if let (Some(ds), Some(dt)) = (direction(src_wp[s], src_wp[c]), direction(bind.pos[*t], bind.pos[tc])) {
                        z[*t] = qarc(dt, ds);
                        z_done[*t] = true;
                    }
                } else if kids.is_empty() {
                    let mut p = tparent[*t];
                    let mut steps = 0;
                    while let Some(x) = p {
                        if src_of[x].is_some() && z_done[x] {
                            z[*t] = z[x];
                            z_done[*t] = true;
                            break;
                        }
                        p = tparent[x];
                        steps += 1;
                        if steps > n {
                            break;
                        }
                    }
                }
            }
        }
        let root_t = (0..n).find(|i| tparent[*i].is_none() && src_of[*i].is_some());
        let hips = root_t.and_then(|t| src_of[t]).or_else(|| pairs.iter().find(|(s, _)| clip.joints[*s].parent.is_none()).map(|p| p.0));
        let src_h = hips.map_or(0.0, |h| src_wp[h][1]);
        let tgt_h = hips.and_then(|h| tgt_of[h]).map_or(0.0, |t| bind.pos[t][1]);
        let ratio = options.scale.unwrap_or(if src_h > 1.0 && tgt_h > 1.0 { tgt_h / src_h } else { 1.0 });
        let first = hips.map(|h| {
            let s0 = clip.sample(0.0, false);
            world(&clip.joints, &s0.rot, &s0.pos).0[h]
        });
        Ok(Retargeter {
            pairs,
            z,
            src_of,
            tgt_rest_rot: bind.rot.clone(),
            tgt_local: joints.iter().map(|j| qnorm(j.rest_rotation)).collect(),
            parent: tparent,
            order,
            names: joints.iter().map(|j| j.name.clone()).collect(),
            hips,
            ratio,
            root: options.root,
            looped: options.looped,
            src_rest_rot: src_wr,
            first,
        })
    }

    pub fn mapping<'a>(&'a self, clip: &'a Clip) -> Vec<(&'a str, &'a str)> {
        self.pairs.iter().map(|(s, t)| (clip.joints[*s].name.as_str(), self.names[*t].as_str())).collect()
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    pub fn pose_at(&self, clip: &Clip, time: f32) -> Posed {
        let sample = clip.sample(time, self.looped);
        let (wp, wr) = world(&clip.joints, &sample.rot, &sample.pos);
        let n = self.names.len();
        let mut pw = vec![IDENTITY; n];
        let mut out = Posed::default();
        for t in &self.order {
            let parent_rot = self.parent[*t].map_or(IDENTITY, |p| pw[p]);
            let rest_world_local = qmul(parent_rot, self.tgt_local[*t]);
            match self.src_of[*t] {
                Some(s) => {
                    let delta_world = qmul(wr[s], qconj(self.src_rest_rot[s]));
                    let target = qnorm(qmul(qmul(delta_world, self.z[*t]), self.tgt_rest_rot[*t]));
                    let local = qnorm(qmul(qconj(rest_world_local), target));
                    pw[*t] = target;
                    out.rotations.insert(self.names[*t].clone(), local);
                }
                None => pw[*t] = rest_world_local,
            }
        }
        if let (Some(h), Some(first)) = (self.hips, self.first) {
            let d = scale(sub(wp[h], first), self.ratio);
            out.offset = match self.root {
                RootMode::Keep => d,
                RootMode::InPlace => [0.0, d[1], 0.0],
                RootMode::Off => [0.0; 3],
            };
        }
        out
    }
}

pub fn blend(from: &BTreeMap<String, Q>, to: &Posed, from_offset: V3, t: f32) -> Posed {
    let mut out = Posed::default();
    let ident = IDENTITY;
    for (k, q) in &to.rotations {
        let a = from.get(k).copied().unwrap_or(ident);
        out.rotations.insert(k.clone(), crate::interp::slerp(a, *q, t));
    }
    out.offset = lerp(from_offset, to.offset, t);
    out
}
