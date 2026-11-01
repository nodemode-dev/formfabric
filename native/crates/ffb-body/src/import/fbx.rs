use super::{compact, Raw, RawJoint};
use crate::math::*;

fn point(m: &ufbx::Matrix, p: &ufbx::Vec3) -> V3 {
    [
        (m.m00 * p.x + m.m01 * p.y + m.m02 * p.z + m.m03) as f32,
        (m.m10 * p.x + m.m11 * p.y + m.m12 * p.z + m.m13) as f32,
        (m.m20 * p.x + m.m21 * p.y + m.m22 * p.z + m.m23) as f32,
    ]
}

fn rotation(m: &ufbx::Matrix) -> Q {
    let c0 = normalize([m.m00 as f32, m.m10 as f32, m.m20 as f32]);
    let c1 = normalize([m.m01 as f32, m.m11 as f32, m.m21 as f32]);
    let c2 = normalize([m.m02 as f32, m.m12 as f32, m.m22 as f32]);
    qfrom_columns(c0, c1, c2)
}

pub fn read(path: &str) -> Result<Raw, String> {
    let opts = ufbx::LoadOpts {
        ignore_animation: true,
        target_axes: ufbx::CoordinateAxes { right: ufbx::CoordinateAxis::PositiveX, up: ufbx::CoordinateAxis::PositiveY, front: ufbx::CoordinateAxis::PositiveZ },
        target_unit_meters: 0.001,
        ..Default::default()
    };
    let scene = ufbx::load_file(path, opts).map_err(|e| format!("could not read the FBX file: {}", e.description))?;
    let mut raw = Raw { unit_mm: Some(1.0), ..Raw::default() };
    let skinned_mesh = scene.nodes.iter().find_map(|n| n.mesh.as_ref().filter(|m| !m.skin_deformers.is_empty()));
    let deformer = skinned_mesh.and_then(|m| m.skin_deformers.first());
    let mut bone_ids: Vec<u32> = Vec::new();
    if let Some(d) = deformer {
        for c in d.clusters.iter() {
            bone_ids.push(c.bone_node.as_ref().map(|n| n.element.element_id).unwrap_or(u32::MAX));
        }
        for (k, c) in d.clusters.iter().enumerate() {
            let Some(node) = c.bone_node.as_ref() else {
                raw.joints.push(RawJoint { name: format!("joint_{k}"), parent: None, pos: [0.0; 3], rot: IDENTITY });
                continue;
            };
            let mut parent = None;
            let mut up = node.parent.as_ref();
            while let Some(p) = up {
                if let Some(j) = bone_ids.iter().position(|id| *id == p.element.element_id) {
                    parent = Some(j as u16);
                    break;
                }
                up = p.parent.as_ref();
            }
            let m = &c.bind_to_world;
            raw.joints.push(RawJoint { name: node.element.name.to_string(), parent, pos: [m.m03 as f32, m.m13 as f32, m.m23 as f32], rot: rotation(m) });
        }
    }
    let mut weights: Vec<[(u16, f32); 4]> = Vec::new();
    let mut any = false;
    let mut buf: Vec<u32> = Vec::new();
    for node in scene.nodes.iter() {
        let Some(mesh) = node.mesh.as_ref() else { continue };
        let base = raw.positions.len() as u32;
        let skin = deformer.filter(|d| mesh.skin_deformers.iter().any(|x| x.element.element_id == d.element.element_id));
        let to_world = match skin.and_then(|d| d.clusters.first()) {
            Some(c) => c.geometry_to_world,
            None => node.geometry_to_world,
        };
        for v in mesh.vertices.iter() {
            raw.positions.push(point(&to_world, v));
        }
        for face in mesh.faces.iter() {
            let n = ufbx::triangulate_face_vec(&mut buf, mesh, *face);
            for t in 0..n as usize {
                let ix = |k: usize| mesh.vertex_indices[buf[t * 3 + k] as usize] + base;
                raw.triangles.push([ix(0), ix(1), ix(2)]);
            }
        }
        let mut slot = vec![[(0u16, 0.0f32); 4]; mesh.vertices.len()];
        if let Some(d) = skin {
            for (vi, sv) in d.vertices.iter().enumerate().take(slot.len()) {
                let mut list: Vec<(u16, f32)> = Vec::new();
                for w in d.weights.iter().skip(sv.weight_begin as usize).take(sv.num_weights as usize) {
                    list.push((w.cluster_index as u16, w.weight as f32));
                }
                list.sort_by(|a, b| b.1.total_cmp(&a.1));
                for (k, w) in list.into_iter().take(4).enumerate() {
                    slot[vi][k] = w;
                    any = true;
                }
            }
        }
        weights.extend(slot);
    }
    if raw.positions.is_empty() || raw.triangles.is_empty() {
        return Err("the FBX file has no triangle meshes".into());
    }
    if any && !raw.joints.is_empty() {
        raw.weights = weights;
    }
    let mut positions = std::mem::take(&mut raw.positions);
    let mut tris = std::mem::take(&mut raw.triangles);
    compact(&mut positions, &mut tris, &mut raw.weights);
    raw.positions = positions;
    raw.triangles = tris;
    Ok(raw)
}
