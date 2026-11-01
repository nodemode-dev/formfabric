use std::collections::HashMap;

use ffb_sim::Collider;
use glam::DVec3;

#[derive(Debug, Clone, Default)]
pub struct Packed {
    pub col: Vec<[u32; 4]>,
    pub cidx: Vec<u32>,
    pub count: u32,
}

pub const MAX_GRID: usize = 96;

fn v3(p: [f32; 3]) -> DVec3 {
    DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64)
}

fn f(x: f64) -> u32 {
    (x as f32).to_bits()
}

fn vec4(v: DVec3, w: f64) -> [u32; 4] {
    [f(v.x), f(v.y), f(v.z), f(w)]
}

fn ekey(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

struct MeshTri {
    p: [DVec3; 3],
    nf: DVec3,
    nv: [DVec3; 3],
    ne: [DVec3; 3],
}

fn mesh_tris(positions: &[[f32; 3]], triangles: &[[u32; 3]], out: &mut Vec<MeshTri>) {
    let pos: Vec<DVec3> = positions.iter().map(|p| v3(*p)).collect();
    let tris: Vec<[u32; 3]> = triangles.iter().copied().filter(|t| t.iter().all(|&i| (i as usize) < pos.len())).collect();
    let mut face = Vec::with_capacity(tris.len());
    let mut vn = vec![DVec3::ZERO; pos.len()];
    let mut en: HashMap<(u32, u32), DVec3> = HashMap::new();
    for t in &tris {
        let p = [pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]];
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        face.push(n);
        for k in 0..3 {
            let e1 = (p[(k + 1) % 3] - p[k]).normalize_or_zero();
            let e2 = (p[(k + 2) % 3] - p[k]).normalize_or_zero();
            let angle = e1.dot(e2).clamp(-1.0, 1.0).acos();
            vn[t[k] as usize] += n * angle;
            *en.entry(ekey(t[k], t[(k + 1) % 3])).or_insert(DVec3::ZERO) += n;
        }
    }
    for n in vn.iter_mut() {
        *n = n.normalize_or_zero();
    }
    for n in en.values_mut() {
        *n = n.normalize_or_zero();
    }
    for (ti, t) in tris.iter().enumerate() {
        let edge = |k: usize| en.get(&ekey(t[k], t[(k + 1) % 3])).copied().unwrap_or(face[ti]);
        out.push(MeshTri {
            p: [pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]],
            nf: face[ti],
            nv: [vn[t[0] as usize], vn[t[1] as usize], vn[t[2] as usize]],
            ne: [edge(0), edge(1), edge(2)],
        });
    }
}

pub fn pack(colliders: &[Collider]) -> Packed {
    let mut headers: Vec<[[u32; 4]; 3]> = Vec::new();
    let mut tris: Vec<MeshTri> = Vec::new();
    let mut has_mesh = false;
    for c in colliders {
        match c {
            Collider::Plane { point, normal } => headers.push([[0; 4], vec4(v3(*point), 0.0), vec4(v3(*normal).normalize_or(DVec3::Y), 0.0)]),
            Collider::Sphere { center, radius } => headers.push([[1, 0, 0, 0], vec4(v3(*center), *radius as f64), [0; 4]]),
            Collider::Capsule { a, b, radius } => headers.push([[2, 0, 0, 0], vec4(v3(*a), *radius as f64), vec4(v3(*b), 0.0)]),
            Collider::Mesh { positions, triangles } => {
                mesh_tris(positions, triangles, &mut tris);
                has_mesh = true;
            }
        }
    }
    let mesh_slot = headers.len();
    if has_mesh && !tris.is_empty() {
        headers.push([[3, 0, 0, 0], [0; 4], [0; 4]]);
    }
    let mut packed = Packed { count: headers.len() as u32, ..Packed::default() };
    for h in &headers {
        packed.col.extend_from_slice(h);
    }
    if packed.col.is_empty() {
        packed.col.extend_from_slice(&[[0; 4]; 3]);
    }
    if packed.count as usize > mesh_slot {
        let tdata = packed.col.len() as u32;
        for t in &tris {
            for p in &t.p {
                packed.col.push(vec4(*p, 0.0));
            }
            packed.col.push(vec4(t.nf, 0.0));
            for n in &t.nv {
                packed.col.push(vec4(*n, 0.0));
            }
            for n in &t.ne {
                packed.col.push(vec4(*n, 0.0));
            }
        }
        let mut lo = DVec3::splat(f64::MAX);
        let mut hi = DVec3::splat(f64::MIN);
        let mut edge_sum = 0.0;
        for t in &tris {
            for k in 0..3 {
                lo = lo.min(t.p[k]);
                hi = hi.max(t.p[k]);
                edge_sum += (t.p[k] - t.p[(k + 1) % 3]).length();
            }
        }
        let avg_edge = edge_sum / (3.0 * tris.len() as f64);
        let ext = (hi - lo).max(DVec3::splat(1.0e-3));
        let cell = (avg_edge * 2.0).max(ext.max_element() / MAX_GRID as f64).max(1.0e-3);
        let dims = [(ext.x / cell).ceil().max(1.0) as usize, (ext.y / cell).ceil().max(1.0) as usize, (ext.z / cell).ceil().max(1.0) as usize];
        let ncell = dims[0] * dims[1] * dims[2];
        let range = |t: &MeshTri| {
            let tl = t.p[0].min(t.p[1]).min(t.p[2]);
            let th = t.p[0].max(t.p[1]).max(t.p[2]);
            let a = ((tl - lo) / cell).floor();
            let b = ((th - lo) / cell).floor();
            let cl = |v: f64, d: usize| (v.max(0.0) as usize).min(d - 1);
            ([cl(a.x, dims[0]), cl(a.y, dims[1]), cl(a.z, dims[2])], [cl(b.x, dims[0]), cl(b.y, dims[1]), cl(b.z, dims[2])])
        };
        let mut count = vec![0u32; ncell + 1];
        for t in &tris {
            let (a, b) = range(t);
            for k in a[2]..=b[2] {
                for j in a[1]..=b[1] {
                    for i in a[0]..=b[0] {
                        count[(k * dims[1] + j) * dims[0] + i] += 1;
                    }
                }
            }
        }
        let list_base = (ncell + 1) as u32;
        let mut start = vec![0u32; ncell + 1];
        for c in 0..ncell {
            start[c + 1] = start[c] + count[c];
        }
        let mut fill = start.clone();
        let mut lists = vec![0u32; start[ncell] as usize];
        for (ti, t) in tris.iter().enumerate() {
            let (a, b) = range(t);
            for k in a[2]..=b[2] {
                for j in a[1]..=b[1] {
                    for i in a[0]..=b[0] {
                        let c = (k * dims[1] + j) * dims[0] + i;
                        lists[fill[c] as usize] = ti as u32;
                        fill[c] += 1;
                    }
                }
            }
        }
        packed.cidx = start.iter().map(|s| s + list_base).collect();
        packed.cidx.extend_from_slice(&lists);
        let hb = mesh_slot * 3;
        packed.col[hb] = [3, tdata, 0, 0];
        packed.col[hb + 1] = vec4(lo, cell);
        packed.col[hb + 2] = [dims[0] as u32, dims[1] as u32, dims[2] as u32, tris.len() as u32];
    }
    if packed.cidx.is_empty() {
        packed.cidx.push(0);
    }
    packed
}
