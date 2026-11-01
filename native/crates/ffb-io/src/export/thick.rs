use std::collections::HashMap;

use super::{normalize, vertex_normals, Group, Object, SrcObject, Thickness, V3};

#[derive(Debug, Clone)]
pub struct Extrusion {
    indices: Vec<u32>,
    count: usize,
}

impl Extrusion {
    pub fn positions(&self, front: &[V3], amount: f32) -> Vec<V3> {
        let normals = vertex_normals(front, &self.indices);
        let mut out = Vec::with_capacity(front.len() * 2);
        out.extend_from_slice(front);
        for (p, n) in front.iter().zip(&normals) {
            let n = normalize(*n);
            out.push([p[0] - n[0] * amount, p[1] - n[1] * amount, p[2] - n[2] * amount]);
        }
        debug_assert_eq!(out.len(), self.count * 2);
        out
    }
}

pub fn border_edges(indices: &[u32]) -> Vec<(u32, u32)> {
    let mut seen: HashMap<(u32, u32), (u32, u32, u32)> = HashMap::new();
    for t in indices.chunks_exact(3) {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let key = (a.min(b), a.max(b));
            let e = seen.entry(key).or_insert((a, b, 0));
            e.2 += 1;
        }
    }
    let mut out: Vec<(u32, u32)> = seen.into_values().filter(|e| e.2 == 1).map(|e| (e.0, e.1)).collect();
    out.sort_unstable();
    out
}

pub fn extrude(o: &SrcObject, t: &Thickness) -> (Object, Extrusion) {
    let n = o.positions.len();
    let ext = Extrusion { indices: o.indices.clone(), count: n };
    let positions = ext.positions(&o.positions, t.amount);
    let mut uvs = o.uvs.clone();
    uvs.extend_from_slice(&o.uvs);
    let mut indices = o.indices.clone();
    let front_tris = indices.len() / 3;
    for tri in o.indices.chunks_exact(3) {
        indices.extend_from_slice(&[tri[0] + n as u32, tri[2] + n as u32, tri[1] + n as u32]);
    }
    let mut groups = vec![Group { material: o.front, first: 0, count: front_tris }, Group { material: o.back, first: front_tris, count: front_tris }];
    if t.sides {
        let first = indices.len() / 3;
        for (a, b) in border_edges(&o.indices) {
            let (a2, b2) = (a + n as u32, b + n as u32);
            indices.extend_from_slice(&[a, a2, b2, a, b2, b]);
        }
        let count = indices.len() / 3 - first;
        if count > 0 {
            groups.push(Group { material: o.side, first, count });
        }
    }
    let normals = vertex_normals(&positions, &indices);
    (Object { name: o.name.clone(), positions, normals, uvs, indices, quads: Vec::new(), groups }, ext)
}
