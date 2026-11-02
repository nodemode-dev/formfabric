use std::collections::BTreeMap;

use ffb_body::math::*;

use crate::clip::{Clip, ClipJoint, MAX_FRAMES, MAX_JOINTS};

pub const MAX_BYTES: u64 = 64 << 20;

fn quat(q: &ufbx::Quat) -> Q {
    qnorm([q.x as f32, q.y as f32, q.z as f32, q.w as f32])
}

fn vec3(v: &ufbx::Vec3) -> V3 {
    [v.x as f32, v.y as f32, v.z as f32]
}

pub fn parse(path: &str, fps: f32, name: String) -> Result<Clip, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("could not read {path}: {e}"))?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return Err(format!("{path} is not a regular file under {} MB", MAX_BYTES >> 20));
    }
    let opts = ufbx::LoadOpts {
        ignore_geometry: true,
        temp_allocator: ufbx::AllocatorOpts { memory_limit: 1 << 30, ..Default::default() },
        result_allocator: ufbx::AllocatorOpts { memory_limit: 1 << 30, ..Default::default() },
        target_axes: ufbx::CoordinateAxes { right: ufbx::CoordinateAxis::PositiveX, up: ufbx::CoordinateAxis::PositiveY, front: ufbx::CoordinateAxis::PositiveZ },
        target_unit_meters: 0.001,
        ..Default::default()
    };
    let scene = ufbx::load_file(path, opts).map_err(|e| format!("could not read the FBX file: {}", e.description))?;
    let mut list: Vec<usize> = Vec::new();
    for (i, node) in scene.nodes.iter().enumerate() {
        if node.bone.is_some() && !node.is_root {
            list.push(i);
        }
    }
    if list.is_empty() {
        return Err("the FBX file has no skeleton bones to read a motion from".into());
    }
    if list.len() > MAX_JOINTS {
        return Err(format!("the FBX motion drives more than {MAX_JOINTS} joints"));
    }
    let mut index_of: BTreeMap<u32, usize> = BTreeMap::new();
    for (k, i) in list.iter().enumerate() {
        index_of.insert(scene.nodes[*i].element.element_id, k);
    }
    let (t0, t1) = (scene.anim.time_begin, scene.anim.time_end);
    if t1.is_nan() || t0.is_nan() || t1 <= t0 || t1 - t0 > 3600.0 {
        return Err("the FBX file has no animation of up to one hour".into());
    }
    let fps = if fps.is_finite() && (1.0..=240.0).contains(&fps) { fps } else { 30.0 };
    let frames = (((t1 - t0) * f64::from(fps)).round() as usize + 1).clamp(1, MAX_FRAMES);
    let mut joints = Vec::with_capacity(list.len());
    for i in &list {
        let node = &scene.nodes[*i];
        let mut parent = None;
        let mut up = node.parent.as_ref();
        while let Some(p) = up {
            if let Some(k) = index_of.get(&p.element.element_id) {
                parent = Some(*k);
                break;
            }
            up = p.parent.as_ref();
        }
        let local = &node.local_transform;
        joints.push(ClipJoint { name: node.element.name.to_string(), parent, rest_pos: vec3(&local.translation), rest_rot: quat(&local.rotation) });
    }
    let mut rot = Vec::with_capacity(frames * list.len());
    let mut pos = Vec::with_capacity(frames * list.len());
    for f in 0..frames {
        let time = t0 + f as f64 / f64::from(fps);
        for i in &list {
            let tr = ufbx::evaluate_transform(&scene.anim, &scene.nodes[*i], time);
            rot.push(quat(&tr.rotation));
            pos.push(vec3(&tr.translation));
        }
    }
    let clip = Clip { name, fps, joints, frames, rot, pos }.order_parents_first();
    clip.check()?;
    Ok(clip)
}
