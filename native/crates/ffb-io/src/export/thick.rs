use std::collections::{HashMap, HashSet};

use super::{normalize, vertex_normals, Group, Object, SrcObject, Thickness, V3};

pub struct Weave<'a> {
    pub groups: &'a [u32],
    pub shared: &'a HashSet<(u32, u32)>,
}

fn fan(indices: &[u32], quads: &[[u32; 4]]) -> Vec<u32> {
    let mut out = indices.to_vec();
    for q in quads {
        out.extend_from_slice(&[q[0], q[1], q[2], q[0], q[2], q[3]]);
    }
    out
}

pub fn shared_edges(objects: &[SrcObject], groups: &[u32]) -> HashSet<(u32, u32)> {
    let mut counts: HashMap<(u32, u32), u32> = HashMap::new();
    let mut base = 0usize;
    for o in objects {
        let g = |i: u32| groups[base + i as usize];
        let mut edge = |a: u32, b: u32| {
            let (x, y) = (g(a), g(b));
            if x != y {
                *counts.entry((x.min(y), x.max(y))).or_insert(0) += 1;
            }
        };
        for t in o.indices.as_chunks::<3>().0 {
            for k in 0..3 {
                edge(t[k], t[(k + 1) % 3]);
            }
        }
        for q in &o.quads {
            for k in 0..4 {
                edge(q[k], q[(k + 1) % 4]);
            }
        }
        base += o.positions.len();
    }
    counts.into_iter().filter(|(_, c)| *c >= 2).map(|(k, _)| k).collect()
}

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
    border_edges_mixed(indices, &[])
}

pub fn border_edges_mixed(indices: &[u32], quads: &[[u32; 4]]) -> Vec<(u32, u32)> {
    let mut seen: HashMap<(u32, u32), (u32, u32, u32)> = HashMap::new();
    let mut count = |a: u32, b: u32| {
        let e = seen.entry((a.min(b), a.max(b))).or_insert((a, b, 0));
        e.2 += 1;
    };
    for t in indices.as_chunks::<3>().0 {
        for k in 0..3 {
            count(t[k], t[(k + 1) % 3]);
        }
    }
    for q in quads {
        for k in 0..4 {
            count(q[k], q[(k + 1) % 4]);
        }
    }
    let mut out: Vec<(u32, u32)> = seen.into_values().filter(|e| e.2 == 1).map(|e| (e.0, e.1)).collect();
    out.sort_unstable();
    out
}

pub fn extrude(o: &SrcObject, t: &Thickness, weave: Option<&Weave>) -> (Object, Extrusion) {
    let n = o.positions.len();
    let shift = n as u32;
    let ext = Extrusion { indices: fan(&o.indices, &o.quads), count: n };
    let positions = ext.positions(&o.positions, t.amount);
    let mut uvs = o.uvs.clone();
    uvs.extend_from_slice(&o.uvs);
    let mut indices = o.indices.clone();
    let front_tris = indices.len() / 3;
    for tri in o.indices.as_chunks::<3>().0 {
        indices.extend_from_slice(&[tri[0] + shift, tri[2] + shift, tri[1] + shift]);
    }
    let mut quads = o.quads.clone();
    for q in &o.quads {
        quads.push([q[0] + shift, q[3] + shift, q[2] + shift, q[1] + shift]);
    }
    let front_quads = o.quads.len();
    let mut walls: Vec<(u32, u32)> = Vec::new();
    if t.sides {
        walls = border_edges_mixed(&o.indices, &o.quads);
        if let Some(w) = weave {
            walls.retain(|(a, b)| {
                let (x, y) = (w.groups[*a as usize], w.groups[*b as usize]);
                x != y && !w.shared.contains(&(x.min(y), x.max(y)))
            });
        }
    }
    let wall_tris = if front_quads == 0 { 2 * walls.len() } else { 0 };
    let wall_quads = if front_quads == 0 { 0 } else { walls.len() };
    for (a, b) in &walls {
        let (a2, b2) = (a + shift, b + shift);
        if front_quads == 0 {
            indices.extend_from_slice(&[*a, a2, b2, *a, b2, *b]);
        } else {
            quads.push([*a, a2, b2, *b]);
        }
    }
    let tri_total = indices.len() / 3;
    let mut groups = vec![Group { material: o.front, first: 0, count: front_tris }, Group { material: o.back, first: front_tris, count: front_tris }];
    if wall_tris > 0 {
        groups.push(Group { material: o.side, first: 2 * front_tris, count: wall_tris });
    }
    if front_quads > 0 {
        groups.push(Group { material: o.front, first: tri_total, count: front_quads });
        groups.push(Group { material: o.back, first: tri_total + front_quads, count: front_quads });
        if wall_quads > 0 {
            groups.push(Group { material: o.side, first: tri_total + 2 * front_quads, count: wall_quads });
        }
    }
    groups.retain(|g| g.count > 0);
    let normals = vertex_normals(&positions, &fan(&indices, &quads));
    (Object { name: o.name.clone(), positions, normals, uvs, indices, quads, uv_ix: Vec::new(), groups }, ext)
}
