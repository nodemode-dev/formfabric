use std::collections::HashMap;
use std::path::Path;

use super::{compact, Raw, RawJoint};
use crate::math::*;

type M4 = [[f32; 4]; 4];

fn mul(a: &M4, b: &M4) -> M4 {
    let mut out = [[0.0f32; 4]; 4];
    for (c, col) in out.iter_mut().enumerate() {
        for (r, cell) in col.iter_mut().enumerate() {
            *cell = (0..4).map(|k| a[k][r] * b[c][k]).sum();
        }
    }
    out
}

fn point(m: &M4, p: V3) -> V3 {
    let mut o = [0.0f32; 3];
    for (r, cell) in o.iter_mut().enumerate() {
        *cell = m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r];
    }
    o
}

fn invert_affine(m: &M4) -> Option<M4> {
    let a = [[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]];
    let c = [
        [a[1][1] * a[2][2] - a[1][2] * a[2][1], a[0][2] * a[2][1] - a[0][1] * a[2][2], a[0][1] * a[1][2] - a[0][2] * a[1][1]],
        [a[1][2] * a[2][0] - a[1][0] * a[2][2], a[0][0] * a[2][2] - a[0][2] * a[2][0], a[0][2] * a[1][0] - a[0][0] * a[1][2]],
        [a[1][0] * a[2][1] - a[1][1] * a[2][0], a[0][1] * a[2][0] - a[0][0] * a[2][1], a[0][0] * a[1][1] - a[0][1] * a[1][0]],
    ];
    let det = a[0][0] * c[0][0] + a[0][1] * c[1][0] + a[0][2] * c[2][0];
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = |r: usize, k: usize| c[r][k] / det;
    let t = [m[3][0], m[3][1], m[3][2]];
    let mut out = [[0.0f32; 4]; 4];
    for (col, cells) in out.iter_mut().enumerate().take(3) {
        for (row, cell) in cells.iter_mut().enumerate().take(3) {
            *cell = inv(row, col);
        }
    }
    for (row, cell) in out[3].iter_mut().enumerate().take(3) {
        *cell = -(inv(row, 0) * t[0] + inv(row, 1) * t[1] + inv(row, 2) * t[2]);
    }
    out[3][3] = 1.0;
    Some(out)
}

fn rotation_of(m: &M4) -> Q {
    let col = |c: usize| normalize([m[c][0], m[c][1], m[c][2]]);
    qfrom_columns(col(0), col(1), col(2))
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

fn load_buffers(g: &gltf::Gltf, dir: Option<&Path>) -> Result<Vec<Vec<u8>>, String> {
    let mut out = Vec::new();
    for b in g.document.buffers() {
        let data = match b.source() {
            gltf::buffer::Source::Bin => g.blob.clone().ok_or("the GLB has no binary chunk")?,
            gltf::buffer::Source::Uri(uri) => {
                if let Some(rest) = uri.strip_prefix("data:") {
                    let (_, payload) = rest.split_once(',').ok_or("bad data URI")?;
                    base64(payload)?
                } else {
                    let dir = dir.ok_or("a .gltf with external buffers needs a file path")?;
                    let name = uri.replace("%20", " ");
                    std::fs::read(dir.join(&name)).map_err(|e| format!("could not read buffer {name}: {e}"))?
                }
            }
        };
        out.push(data);
    }
    Ok(out)
}

pub fn read(bytes: &[u8], dir: Option<&Path>) -> Result<Raw, String> {
    let g = gltf::Gltf::from_slice(bytes).map_err(|e| format!("not a valid glTF file: {e}"))?;
    let buffers = load_buffers(&g, dir)?;
    let doc = &g.document;
    let count = doc.nodes().len();
    let mut parent: Vec<Option<usize>> = vec![None; count];
    for n in doc.nodes() {
        for c in n.children() {
            parent[c.index()] = Some(n.index());
        }
    }
    let local: Vec<M4> = doc.nodes().map(|n| n.transform().matrix()).collect();
    let mut global: Vec<Option<M4>> = vec![None; count];
    fn world(i: usize, parent: &[Option<usize>], local: &[M4], global: &mut [Option<M4>], depth: usize) -> M4 {
        if let Some(m) = global[i] {
            return m;
        }
        let m = match parent[i] {
            Some(p) if depth < 512 => mul(&world(p, parent, local, global, depth + 1), &local[i]),
            _ => local[i],
        };
        global[i] = Some(m);
        m
    }
    let world_of: Vec<M4> = (0..count).map(|i| world(i, &parent, &local, &mut global, 0)).collect();

    let skin = doc.nodes().filter(|n| n.mesh().is_some()).find_map(|n| n.skin());
    let mut raw = Raw { unit_mm: Some(1000.0), ..Raw::default() };
    let mut joint_of_node: HashMap<usize, usize> = HashMap::new();
    if let Some(skin) = &skin {
        let nodes: Vec<usize> = skin.joints().map(|n| n.index()).collect();
        for (k, n) in nodes.iter().enumerate() {
            joint_of_node.insert(*n, k);
        }
        let ibm: Option<Vec<M4>> = skin.reader(|b| buffers.get(b.index()).map(|v| v.as_slice())).read_inverse_bind_matrices().map(|it| it.collect());
        for (k, n) in nodes.iter().enumerate() {
            let bind = match ibm.as_ref().and_then(|v| v.get(k)).and_then(invert_affine) {
                Some(m) => m,
                None => world_of[*n],
            };
            let mut up = parent[*n];
            let mut nearest = None;
            while let Some(p) = up {
                if let Some(j) = joint_of_node.get(&p) {
                    nearest = Some(*j as u16);
                    break;
                }
                if nodes.contains(&p) {
                    nearest = nodes.iter().position(|x| *x == p).map(|j| j as u16);
                    break;
                }
                up = parent[p];
            }
            let name = doc.nodes().nth(*n).and_then(|node| node.name().map(str::to_string)).unwrap_or_else(|| format!("joint_{k}"));
            raw.joints.push(RawJoint { name, parent: nearest, pos: [bind[3][0], bind[3][1], bind[3][2]], rot: rotation_of(&bind) });
        }
    }
    let skin_index = skin.as_ref().map(|s| s.index());
    let mut weights: Vec<[(u16, f32); 4]> = Vec::new();
    let mut any_weights = false;
    for node in doc.nodes() {
        let Some(mesh) = node.mesh() else { continue };
        let skinned = node.skin().map(|s| s.index()) == skin_index && skin_index.is_some();
        for prim in mesh.primitives() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let reader = prim.reader(|b| buffers.get(b.index()).map(|v| v.as_slice()));
            let Some(pos) = reader.read_positions() else { continue };
            let base = raw.positions.len() as u32;
            let m = world_of[node.index()];
            let pts: Vec<V3> = pos.map(|p| if skinned { p } else { point(&m, p) }).collect();
            let n = pts.len();
            raw.positions.extend(pts);
            match reader.read_indices() {
                Some(ix) => {
                    let ix: Vec<u32> = ix.into_u32().collect();
                    for t in ix.chunks_exact(3) {
                        raw.triangles.push([base + t[0], base + t[1], base + t[2]]);
                    }
                }
                None => {
                    for t in 0..(n / 3) as u32 {
                        raw.triangles.push([base + 3 * t, base + 3 * t + 1, base + 3 * t + 2]);
                    }
                }
            }
            let mut slot = vec![[(0u16, 0.0f32); 4]; n];
            if skinned {
                if let (Some(j), Some(w)) = (reader.read_joints(0), reader.read_weights(0)) {
                    for (k, (j, w)) in j.into_u16().zip(w.into_f32()).enumerate().take(n) {
                        slot[k] = [(j[0], w[0]), (j[1], w[1]), (j[2], w[2]), (j[3], w[3])];
                        any_weights = true;
                    }
                }
            }
            weights.extend(slot);
        }
    }
    if raw.positions.is_empty() {
        return Err("the glTF file has no triangle meshes".into());
    }
    if any_weights && !raw.joints.is_empty() {
        raw.weights = weights;
    }
    let mut positions = std::mem::take(&mut raw.positions);
    let mut tris = std::mem::take(&mut raw.triangles);
    compact(&mut positions, &mut tris, &mut raw.weights);
    raw.positions = positions;
    raw.triangles = tris;
    Ok(raw)
}
