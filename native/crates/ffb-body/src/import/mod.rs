mod gltf_reader;
mod obj;

#[cfg(not(target_arch = "wasm32"))]
mod fbx;

use std::path::Path;

use crate::math::*;
use crate::skin;
use crate::{Body, Joint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Units {
    Auto,
    Mm,
    Cm,
    M,
    Inch,
    Foot,
}

impl Units {
    pub fn parse(s: &str) -> Option<Units> {
        match s.trim().to_lowercase().as_str() {
            "auto" => Some(Units::Auto),
            "mm" => Some(Units::Mm),
            "cm" => Some(Units::Cm),
            "m" => Some(Units::M),
            "in" | "inch" => Some(Units::Inch),
            "ft" | "foot" => Some(Units::Foot),
            _ => None,
        }
    }

    fn factor(self) -> Option<f32> {
        match self {
            Units::Auto => None,
            Units::Mm => Some(1.0),
            Units::Cm => Some(10.0),
            Units::M => Some(1000.0),
            Units::Inch => Some(25.4),
            Units::Foot => Some(304.8),
        }
    }
}

pub fn up_rotation(up: &str) -> Option<Q> {
    let axis = match up.trim().to_lowercase().as_str() {
        "y" | "+y" => return Some(IDENTITY),
        "-y" => [0.0, -1.0, 0.0],
        "z" | "+z" => [0.0, 0.0, 1.0],
        "-z" => [0.0, 0.0, -1.0],
        "x" | "+x" => [1.0, 0.0, 0.0],
        "-x" => [-1.0, 0.0, 0.0],
        _ => return None,
    };
    Some(qarc(axis, [0.0, 1.0, 0.0]))
}

#[derive(Debug, Clone)]
pub struct Options {
    pub units: Units,
    pub up: Q,
    pub ground: bool,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RawJoint {
    pub name: String,
    pub parent: Option<u16>,
    pub pos: V3,
    pub rot: Q,
}

#[derive(Debug, Clone, Default)]
pub struct Raw {
    pub positions: Vec<V3>,
    pub triangles: Vec<[u32; 3]>,
    pub joints: Vec<RawJoint>,
    pub weights: Vec<[(u16, f32); 4]>,
    pub unit_mm: Option<f32>,
    pub label: Option<String>,
}

pub fn extensions() -> &'static [&'static str] {
    &["obj", "glb", "gltf", "fbx"]
}

pub fn read(path: &str, options: &Options) -> Result<Body, String> {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let bytes = std::fs::read(path).map_err(|e| format!("could not read {path}: {e}"))?;
    let raw = match ext.as_str() {
        "obj" => obj::read(&bytes)?,
        "glb" | "gltf" => gltf_reader::read(&bytes, Path::new(path).parent())?,
        "fbx" => read_fbx(path, &bytes)?,
        other => return Err(format!("unsupported body format .{other}; Import Body reads .obj, .glb, .gltf and .fbx (Alembic is deferred)")),
    };
    let stem = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("Body").to_string();
    finish(raw, options, stem)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_fbx(path: &str, _bytes: &[u8]) -> Result<Raw, String> {
    fbx::read(path)
}

#[cfg(target_arch = "wasm32")]
fn read_fbx(_path: &str, _bytes: &[u8]) -> Result<Raw, String> {
    Err("FBX import is not available in the browser build".into())
}

pub fn finish(mut raw: Raw, options: &Options, stem: String) -> Result<Body, String> {
    if raw.positions.is_empty() || raw.triangles.is_empty() {
        return Err("the file holds no triangle mesh".into());
    }
    let rot = qnorm(options.up);
    for p in raw.positions.iter_mut() {
        *p = qrot(rot, *p);
    }
    for j in raw.joints.iter_mut() {
        j.pos = qrot(rot, j.pos);
        j.rot = qmul(rot, qnorm(j.rot));
    }
    let height = {
        let lo = raw.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let hi = raw.positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        hi - lo
    };
    if !height.is_finite() || height <= 0.0 {
        return Err("the mesh has no height; check the up axis".into());
    }
    let factor = match options.units.factor().or(raw.unit_mm) {
        Some(f) => f,
        None => {
            if height < 4.0 {
                1000.0
            } else if height < 400.0 {
                10.0
            } else {
                1.0
            }
        }
    };
    for p in raw.positions.iter_mut() {
        *p = scale(*p, factor);
    }
    for j in raw.joints.iter_mut() {
        j.pos = scale(j.pos, factor);
    }
    if options.ground {
        let lo = raw.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        for p in raw.positions.iter_mut() {
            p[1] -= lo;
        }
        for j in raw.joints.iter_mut() {
            j.pos[1] -= lo;
        }
    }
    let n = raw.positions.len() as u32;
    raw.triangles.retain(|t| t[0] < n && t[1] < n && t[2] < n && t[0] != t[1] && t[1] != t[2] && t[0] != t[2]);
    if raw.triangles.is_empty() {
        return Err("the file holds no usable triangles".into());
    }
    let final_height = {
        let lo = raw.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let hi = raw.positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        hi - lo
    };
    if !(300.0..=4000.0).contains(&final_height) {
        return Err(format!("the body is {final_height:.0} mm tall after conversion; set units (mm, cm, m, in, ft) and the up axis so it reads between 300 and 4000 mm"));
    }
    let joint_count = raw.joints.len();
    let mut joints: Vec<Joint> = raw.joints.iter().map(|j| Joint { name: j.name.clone(), parent: j.parent, rest: [0.0; 3], rest_rotation: IDENTITY }).collect();
    let jpos: Vec<V3> = raw.joints.iter().map(|j| j.pos).collect();
    let jrot: Vec<Q> = raw.joints.iter().map(|j| qnorm(j.rot)).collect();
    skin::set_from_world(&mut joints, &jpos, &jrot);
    let mut weights = std::mem::take(&mut raw.weights);
    if joint_count > 0 {
        if weights.len() != raw.positions.len() {
            let bones = skin::hierarchy_bones(&joints, &jpos);
            weights = skin::compute_weights(&raw.positions, joint_count, &bones, 20.0);
        } else {
            skin::normalize_weights(&mut weights, joint_count);
            let bones = skin::hierarchy_bones(&joints, &jpos);
            let fallback = skin::compute_weights(&raw.positions, joint_count, &bones, 20.0);
            for (w, f) in weights.iter_mut().zip(fallback) {
                if w.iter().map(|s| s.1).sum::<f32>() < 0.5 {
                    *w = f;
                }
            }
        }
    } else {
        weights.clear();
    }
    let normals = skin::compute_normals(&raw.positions, &raw.triangles);
    let mut body = Body {
        name: options.name.clone().or(raw.label).unwrap_or(stem),
        positions: raw.positions,
        normals,
        triangles: raw.triangles,
        joints,
        weights,
        skin_offset: 3.0,
        friction: 0.3,
        visible: true,
        ..Body::default()
    };
    body.measurements = crate::measure::measure(&body);
    Ok(body)
}

pub fn compact(positions: &mut Vec<V3>, triangles: &mut [[u32; 3]], weights: &mut Vec<[(u16, f32); 4]>) {
    let mut map = vec![u32::MAX; positions.len()];
    let mut next = 0u32;
    for t in triangles.iter() {
        for v in t {
            let slot = &mut map[*v as usize];
            if *slot == u32::MAX {
                *slot = next;
                next += 1;
            }
        }
    }
    let mut np = vec![[0.0; 3]; next as usize];
    let has_w = weights.len() == positions.len();
    let mut nw = vec![[(0u16, 0.0f32); 4]; if has_w { next as usize } else { 0 }];
    for (old, new) in map.iter().enumerate() {
        if *new != u32::MAX {
            np[*new as usize] = positions[old];
            if has_w {
                nw[*new as usize] = weights[old];
            }
        }
    }
    for t in triangles.iter_mut() {
        for v in t.iter_mut() {
            *v = map[*v as usize];
        }
    }
    *positions = np;
    *weights = nw;
}
