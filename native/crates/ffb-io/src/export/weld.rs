use std::collections::HashMap;

use super::{vertex_normals, Group, Object, Scene, V3};

#[derive(Debug, Clone, PartialEq)]
pub struct Weld {
    pub map: Vec<u32>,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Report {
    pub vertices: usize,
    pub points: usize,
    pub dropped_faces: usize,
}

impl Weld {
    pub fn from_keys(keys: &[u32]) -> Weld {
        let mut ids: HashMap<u32, u32> = HashMap::new();
        let mut map = Vec::with_capacity(keys.len());
        for k in keys {
            let next = ids.len() as u32;
            map.push(*ids.entry(*k).or_insert(next));
        }
        Weld { map, count: ids.len() }
    }

    pub fn reduce(&self, values: &[V3]) -> Vec<V3> {
        let mut sum = vec![[0.0f32; 3]; self.count];
        let mut hits = vec![0u32; self.count];
        for (v, p) in values.iter().zip(&self.map) {
            let s = &mut sum[*p as usize];
            s[0] += v[0];
            s[1] += v[1];
            s[2] += v[2];
            hits[*p as usize] += 1;
        }
        sum.iter().zip(&hits).map(|(s, h)| if *h > 1 { let k = 1.0 / *h as f32; [s[0] * k, s[1] * k, s[2] * k] } else { *s }).collect()
    }
}

fn runs(materials: &[usize], offset: usize, out: &mut Vec<Group>) {
    let mut at = 0;
    while at < materials.len() {
        let m = materials[at];
        let mut end = at + 1;
        while end < materials.len() && materials[end] == m {
            end += 1;
        }
        out.push(Group { material: m, first: offset + at, count: end - at });
        at = end;
    }
}

pub fn apply(scene: &Scene, weld: &Weld) -> Result<(Scene, Report), String> {
    let vertices = scene.vertex_count();
    if weld.map.len() != vertices {
        return Err(format!("the weld map has {} entries for {} vertices", weld.map.len(), vertices));
    }
    let raw: Vec<V3> = scene.objects.iter().flat_map(|o| o.positions.iter().copied()).collect();
    let uvs: Vec<[f32; 2]> = scene.objects.iter().flat_map(|o| o.uvs.iter().copied()).collect();
    let points = weld.reduce(&raw);
    let (mut tris, mut tri_uv, mut tri_mat) = (Vec::new(), Vec::new(), Vec::new());
    let (mut quads, mut quad_uv, mut quad_mat): (Vec<[u32; 4]>, Vec<u32>, Vec<usize>) = (Vec::new(), Vec::new(), Vec::new());
    let mut dropped = 0usize;
    let mut base = 0usize;
    for obj in &scene.objects {
        let (t, q) = (obj.indices.len() / 3, obj.quads.len());
        let mut mat = vec![obj.groups.first().map_or(0, |g| g.material); t + q];
        for g in &obj.groups {
            mat.iter_mut().take((g.first + g.count).min(t + q)).skip(g.first).for_each(|x| *x = g.material);
        }
        let m = |i: u32| weld.map[base + i as usize];
        let u = |i: u32| (base as u32) + i;
        for (f, c) in obj.indices.as_chunks::<3>().0.iter().enumerate() {
            let p = [m(c[0]), m(c[1]), m(c[2])];
            if p[0] == p[1] || p[1] == p[2] || p[0] == p[2] {
                dropped += 1;
                continue;
            }
            tris.extend(p);
            tri_uv.extend([u(c[0]), u(c[1]), u(c[2])]);
            tri_mat.push(mat[f]);
        }
        for (k, c) in obj.quads.iter().enumerate() {
            let p = [m(c[0]), m(c[1]), m(c[2]), m(c[3])];
            let uu = [u(c[0]), u(c[1]), u(c[2]), u(c[3])];
            if p[0] != p[1] && p[1] != p[2] && p[2] != p[3] && p[3] != p[0] && p[0] != p[2] && p[1] != p[3] {
                quads.push(p);
                quad_uv.extend(uu);
                quad_mat.push(mat[t + k]);
                continue;
            }
            let keep: Vec<usize> = (0..4).filter(|j| p[*j] != p[(*j + 1) % 4]).collect();
            if keep.len() == 3 && p[keep[0]] != p[keep[2]] && p[keep[0]] != p[keep[1]] && p[keep[1]] != p[keep[2]] {
                tris.extend(keep.iter().map(|j| p[*j]));
                tri_uv.extend(keep.iter().map(|j| uu[*j]));
                tri_mat.push(mat[t + k]);
            } else {
                dropped += 1;
            }
        }
        base += obj.positions.len();
    }
    if tris.is_empty() && quads.is_empty() {
        return Err("welding removed every face".into());
    }
    let mut groups = Vec::new();
    runs(&tri_mat, 0, &mut groups);
    runs(&quad_mat, tri_mat.len(), &mut groups);
    tri_uv.extend(quad_uv);
    let mut all = tris.clone();
    for q in &quads {
        all.extend_from_slice(&[q[0], q[1], q[2], q[0], q[2], q[3]]);
    }
    let normals = vertex_normals(&points, &all);
    let object = Object { name: "garment".into(), positions: points, normals, uvs, indices: tris, quads, uv_ix: tri_uv, groups };
    let report = Report { vertices, points: object.positions.len(), dropped_faces: dropped };
    Ok((Scene { objects: vec![object], materials: scene.materials.clone() }, report))
}

pub fn single_index(obj: &Object, use_uv: bool) -> (Object, Vec<u32>) {
    if obj.uv_ix.is_empty() {
        return (obj.clone(), (0..obj.positions.len() as u32).collect());
    }
    let mut seen: HashMap<(u32, u32), u32> = HashMap::new();
    let mut out = Object { name: obj.name.clone(), groups: obj.groups.clone(), ..Object::default() };
    let mut origin = Vec::new();
    for (c, p) in obj.indices.iter().enumerate() {
        let uv = if use_uv { obj.uv_ix[c] } else { 0 };
        let next = out.positions.len() as u32;
        let id = *seen.entry((*p, uv)).or_insert_with(|| {
            out.positions.push(obj.positions[*p as usize]);
            out.normals.push(obj.normals[*p as usize]);
            out.uvs.push(if use_uv { obj.uvs[uv as usize] } else { [0.0, 0.0] });
            origin.push(*p);
            next
        });
        out.indices.push(id);
    }
    (out, origin)
}
