use std::collections::HashMap;

use ffb_sim::material::DEFAULT_DENSITY;
use ffb_sim::{Cloth, Material};
use glam::{DVec2, DVec3};

pub const FOLD_GAIN: f64 = 100.0;

#[derive(Debug, Clone, Default)]
pub struct GpuModel {
    pub n: usize,
    pub mass: Vec<f32>,
    pub friction: Vec<f32>,
    pub elem: Vec<[u32; 4]>,
    pub tri_off: u32,
    pub hinge_off: u32,
    pub link_off: u32,
    pub idx: Vec<u32>,
    pub order_off: u32,
    pub hdr_off: u32,
    pub adj_off: u32,
    pub colors: Vec<(u32, u32)>,
    pub tri_count: usize,
    pub hinge_count: usize,
    pub link_count: usize,
}

struct Tri {
    v: [u32; 3],
    alpha: [f64; 3],
    beta: [f64; 3],
    area: f64,
    ku: f64,
    kv: f64,
    ks: f64,
    kd: f64,
    pressure: f64,
}

struct Hinge {
    v: [u32; 4],
    coef: f64,
    theta0: f64,
    h_rest: [f64; 2],
    ratio: f64,
    floor: f64,
    kd: f64,
}

struct Link {
    v: [u32; 3],
    c: [f64; 3],
    n: u32,
    k: f64,
    rest: f64,
}

type EdgeFace = (u32, u32, u32);

fn grain_axes(angle: f64) -> (DVec2, DVec2) {
    let (s, c) = angle.sin_cos();
    (DVec2::new(c, s), DVec2::new(-s, c))
}

fn grain_rest(rest: DVec2, angle: f64, mat: &Material) -> DVec2 {
    let (w, p) = grain_axes(angle);
    DVec2::new(rest.dot(w) * mat.shrink_weft as f64, rest.dot(p) * mat.shrink_warp as f64)
}

fn bend_dir(mat: [&Material; 2], phi: f64) -> f64 {
    let c2 = phi.cos().powi(2);
    let s2 = 1.0 - c2;
    let wb = 4.0 * c2 * s2;
    let wu = (1.0 - wb) * c2;
    let wv = (1.0 - wb) * s2;
    let b = |m: &Material| wu * m.bending_weft as f64 + wv * m.bending_warp as f64 + wb * m.bending_bias as f64;
    0.5 * (b(mat[0]) + b(mat[1]))
}

fn ekey(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn bits(v: f64) -> u32 {
    (v as f32).to_bits()
}

#[allow(clippy::too_many_arguments)]
fn make_hinge(pos: &[DVec3], rest2: &[DVec2], mats: &[Material], cloth: &Cloth, v: [u32; 4], t0: usize, t1: Option<usize>, theta0: f64, panel: &dyn Fn(u32) -> u32) -> Hinge {
    let mat_idx = |t: usize| (cloth.triangle_material.get(t).copied().unwrap_or(0) as usize).min(mats.len() - 1);
    let m0 = &mats[mat_idx(t0)];
    let m1 = &mats[mat_idx(t1.unwrap_or(t0))];
    let g = cloth.triangle_grain.get(t0).copied().unwrap_or(0.0) as f64;
    let same = panel(v[0]) == panel(v[1]) && panel(v[0]) == panel(v[2]) && panel(v[0]) == panel(v[3]);
    let (le, a1, a2, phi) = if same {
        let r: Vec<DVec2> = v.iter().map(|&i| grain_rest(rest2[i as usize], g, m0)).collect();
        let e = r[1] - r[0];
        let le = e.length().max(1.0e-9);
        let a1 = 0.5 * e.perp_dot(r[2] - r[0]).abs();
        let a2 = 0.5 * e.perp_dot(r[3] - r[0]).abs();
        let perp = DVec2::new(-e.y, e.x) / le;
        (le, a1, a2, perp.y.atan2(perp.x))
    } else {
        let p: Vec<DVec3> = v.iter().map(|&i| pos[i as usize]).collect();
        let e = p[1] - p[0];
        let le = e.length().max(1.0e-9);
        (le, 0.5 * e.cross(p[2] - p[0]).length(), 0.5 * e.cross(p[3] - p[0]).length(), std::f64::consts::FRAC_PI_4)
    };
    let b = bend_dir([m0, m1], phi);
    let area = (a1 + a2).max(1.0e-9);
    Hinge {
        v,
        coef: b * 3.0 * le * le / area,
        theta0,
        h_rest: [2.0 * a1 / le, 2.0 * a2 / le],
        ratio: 0.5 * (m0.buckling_ratio + m1.buckling_ratio) as f64,
        floor: (0.5 * (m0.buckling_stiffness + m1.buckling_stiffness) as f64).clamp(0.0, 1.0),
        kd: 0.5 * (m0.kd() + m1.kd()),
    }
}

impl GpuModel {
    pub fn build(cloth: &Cloth) -> GpuModel {
        let n = cloth.positions.len();
        let mats: Vec<Material> = if cloth.materials.is_empty() { vec![Material::default().effective()] } else { cloth.materials.iter().map(|m| m.effective()).collect() };
        let pos: Vec<DVec3> = cloth.positions.iter().map(|p| DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64)).collect();
        let rest2: Vec<DVec2> = (0..n).map(|i| cloth.rest.get(i).map_or(DVec2::ZERO, |r| DVec2::new(r[0] as f64, r[1] as f64))).collect();
        let mat_of = |t: usize| -> &Material { &mats[(cloth.triangle_material.get(t).copied().unwrap_or(0) as usize).min(mats.len() - 1)] };
        let grain_of = |t: usize| -> f64 { cloth.triangle_grain.get(t).copied().unwrap_or(0.0) as f64 };
        let panel = |v: u32| cloth.vertex_panel.get(v as usize).copied().unwrap_or(0);

        let mut mass = vec![0.0f64; n];
        let mut edge_sum = 0.0;
        let mut edge_cnt = 0.0;
        let mut tris: Vec<Tri> = Vec::new();
        for (ti, t) in cloth.triangles.iter().enumerate() {
            if t.iter().any(|&i| i as usize >= n) {
                continue;
            }
            let m = mat_of(ti);
            let g = grain_of(ti);
            let r0 = rest2[t[0] as usize];
            let r1 = rest2[t[1] as usize];
            let r2 = rest2[t[2] as usize];
            let raw_area = 0.5 * (r1 - r0).perp_dot(r2 - r0).abs();
            for &i in t {
                mass[i as usize] += raw_area / 3.0 * m.density as f64;
            }
            for k in 0..3 {
                edge_sum += (pos[t[k] as usize] - pos[t[(k + 1) % 3] as usize]).length();
                edge_cnt += 1.0;
            }
            let a = grain_rest(r0, g, m);
            let b = grain_rest(r1, g, m);
            let c = grain_rest(r2, g, m);
            let (p, q, r, s) = (b.x - a.x, c.x - a.x, b.y - a.y, c.y - a.y);
            let det = p * s - q * r;
            if det.abs() < 1.0e-9 {
                continue;
            }
            let (ia, ib, ic, id) = (s / det, -q / det, -r / det, p / det);
            tris.push(Tri {
                v: *t,
                alpha: [-(ia + ic), ia, ic],
                beta: [-(ib + id), ib, id],
                area: 0.5 * det.abs(),
                ku: m.stretch_weft as f64,
                kv: m.stretch_warp as f64,
                ks: m.shear as f64,
                kd: m.kd(),
                pressure: m.pressure as f64,
            });
        }
        let rest_edge = if edge_cnt > 0.0 { edge_sum / edge_cnt } else { 10.0 };
        let floor_mass = rest_edge * rest_edge * 0.5 * DEFAULT_DENSITY as f64 * 0.01;
        for m in mass.iter_mut() {
            if *m < floor_mass {
                *m = floor_mass;
            }
        }
        let mut friction = vec![0.0f64; n];
        for (ti, t) in cloth.triangles.iter().enumerate() {
            if t.iter().all(|&i| (i as usize) < n) {
                let f = mat_of(ti).friction as f64;
                for &i in t {
                    friction[i as usize] = f;
                }
            }
        }

        let mut edges: HashMap<(u32, u32), Vec<EdgeFace>> = HashMap::new();
        for (ti, t) in cloth.triangles.iter().enumerate() {
            if t.iter().any(|&i| i as usize >= n) {
                continue;
            }
            for k in 0..3 {
                let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
                edges.entry(ekey(a, b)).or_default().push((ti as u32, a, c));
            }
        }
        let mut keys: Vec<(u32, u32)> = edges.keys().copied().collect();
        keys.sort_unstable();
        let mut hinge_of: HashMap<(u32, u32), usize> = HashMap::new();
        let mut hinges: Vec<Hinge> = Vec::new();
        for key in keys {
            let faces = &edges[&key];
            if faces.len() != 2 {
                continue;
            }
            let (f0, f1) = (faces[0], faces[1]);
            if f0.1 == f1.1 {
                continue;
            }
            let (i, j, k, l) = (f0.1, if f0.1 == key.0 { key.1 } else { key.0 }, f0.2, f1.2);
            let hinge = make_hinge(&pos, &rest2, &mats, cloth, [i, j, k, l], f0.0 as usize, Some(f1.0 as usize), 0.0, &panel);
            hinge_of.insert(key, hinges.len());
            hinges.push(hinge);
        }
        for fold in &cloth.folds {
            let key = ekey(fold.edge[0], fold.edge[1]);
            let boost = 1.0 + FOLD_GAIN * fold.strength.clamp(0.0, 1.0) as f64;
            if let Some(&hi) = hinge_of.get(&key) {
                let h = &mut hinges[hi];
                h.theta0 = fold.angle as f64;
                h.coef *= boost;
                continue;
            }
            let ids = [fold.edge[0], fold.edge[1], fold.opposite[0], fold.opposite[1]];
            if ids.iter().any(|&i| i as usize >= n) {
                continue;
            }
            let mut v = ids;
            if panel(v[0]) == panel(v[2]) && panel(v[0]) == panel(v[3]) {
                let e = rest2[v[1] as usize] - rest2[v[0] as usize];
                let s2 = e.perp_dot(rest2[v[2] as usize] - rest2[v[0] as usize]);
                let s3 = e.perp_dot(rest2[v[3] as usize] - rest2[v[0] as usize]);
                if s2 * s3 >= 0.0 {
                    continue;
                }
                if s2 < 0.0 {
                    v.swap(2, 3);
                }
            }
            let Some(tri) = cloth.triangles.iter().position(|t| t.contains(&v[0]) && t.contains(&v[1])) else { continue };
            let mut h = make_hinge(&pos, &rest2, &mats, cloth, v, tri, None, fold.angle as f64, &panel);
            h.coef *= boost;
            hinges.push(h);
        }

        let default_link_k = mats.iter().map(|m| m.stretch_weft.max(m.stretch_warp) as f64).fold(0.0, f64::max).max(1.0);
        let mut links: Vec<Link> = Vec::new();
        for s in &cloth.seams {
            if s.a as usize >= n || s.b as usize >= n || s.a == s.b {
                continue;
            }
            let k = if s.stiffness > 0.0 { s.stiffness as f64 } else { default_link_k };
            links.push(Link { v: [s.a, s.b, 0], c: [1.0, -1.0, 0.0], n: 2, k, rest: s.rest_length.max(0.0) as f64 });
        }
        for b in &cloth.binds {
            if b.a as usize >= n || b.b.iter().any(|&i| i as usize >= n) {
                continue;
            }
            let k = if b.stiffness > 0.0 { b.stiffness as f64 } else { default_link_k };
            let w = (b.w as f64).clamp(0.0, 1.0);
            links.push(Link { v: [b.a, b.b[0], b.b[1]], c: [1.0, -(1.0 - w), -w], n: 3, k, rest: b.rest_length.max(0.0) as f64 });
        }

        let mut model = GpuModel { n, tri_count: tris.len(), hinge_count: hinges.len(), link_count: links.len(), ..GpuModel::default() };
        model.mass = mass.iter().map(|&m| m as f32).collect();
        model.friction = friction.iter().map(|&m| m as f32).collect();
        for t in &tris {
            model.elem.push([t.v[0], t.v[1], t.v[2], 0]);
            model.elem.push([bits(t.alpha[0]), bits(t.alpha[1]), bits(t.alpha[2]), bits(t.area)]);
            model.elem.push([bits(t.beta[0]), bits(t.beta[1]), bits(t.beta[2]), bits(t.kd)]);
            model.elem.push([bits(t.ku), bits(t.kv), bits(t.ks), bits(t.pressure)]);
        }
        model.hinge_off = model.elem.len() as u32;
        for h in &hinges {
            model.elem.push([h.v[0], h.v[1], h.v[2], h.v[3]]);
            model.elem.push([bits(h.coef), bits(h.theta0), bits(h.h_rest[0]), bits(h.h_rest[1])]);
            model.elem.push([bits(h.ratio), bits(h.floor), bits(h.kd), 0]);
        }
        model.link_off = model.elem.len() as u32;
        for l in &links {
            model.elem.push([l.v[0], l.v[1], l.v[2], l.n]);
            model.elem.push([bits(l.c[0]), bits(l.c[1]), bits(l.c[2]), bits(l.k)]);
            model.elem.push([bits(l.rest), 0, 0, 0]);
        }
        if model.elem.is_empty() {
            model.elem.push([0; 4]);
        }

        let mut adj_tri: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut adj_hinge: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut adj_link: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut neigh: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut join = |vs: &[u32]| {
            for &a in vs {
                for &b in vs {
                    if a != b {
                        neigh[a as usize].push(b);
                    }
                }
            }
        };
        for (i, t) in tris.iter().enumerate() {
            for (s, &v) in t.v.iter().enumerate() {
                adj_tri[v as usize].push(i as u32 * 4 + s as u32);
            }
            join(&t.v);
        }
        for (i, h) in hinges.iter().enumerate() {
            for (s, &v) in h.v.iter().enumerate() {
                adj_hinge[v as usize].push(i as u32 * 4 + s as u32);
            }
            join(&h.v);
        }
        for (i, l) in links.iter().enumerate() {
            for (s, &v) in l.v[..l.n as usize].iter().enumerate() {
                adj_link[v as usize].push(i as u32 * 4 + s as u32);
            }
            join(&l.v[..l.n as usize]);
        }
        for v in neigh.iter_mut() {
            v.sort_unstable();
            v.dedup();
        }
        let mut color = vec![u32::MAX; n];
        let mut ncol = 0usize;
        let mut used: Vec<bool> = Vec::new();
        for i in 0..n {
            used.clear();
            used.resize(ncol + 1, false);
            for &j in &neigh[i] {
                let c = color[j as usize];
                if c != u32::MAX {
                    used[c as usize] = true;
                }
            }
            let c = used.iter().position(|u| !u).unwrap_or(ncol);
            color[i] = c as u32;
            ncol = ncol.max(c + 1);
        }
        let mut groups: Vec<Vec<u32>> = vec![Vec::new(); ncol];
        for (i, &c) in color.iter().enumerate() {
            groups[c as usize].push(i as u32);
        }
        model.order_off = 0;
        for g in &groups {
            let start = model.idx.len() as u32;
            model.idx.extend_from_slice(g);
            model.colors.push((start, g.len() as u32));
        }
        model.hdr_off = model.idx.len() as u32;
        let mut adj: Vec<u32> = Vec::new();
        let mut hdr: Vec<u32> = Vec::with_capacity(4 * n);
        for i in 0..n {
            hdr.extend_from_slice(&[adj.len() as u32, adj_tri[i].len() as u32, adj_hinge[i].len() as u32, adj_link[i].len() as u32]);
            adj.extend_from_slice(&adj_tri[i]);
            adj.extend_from_slice(&adj_hinge[i]);
            adj.extend_from_slice(&adj_link[i]);
        }
        model.idx.extend_from_slice(&hdr);
        model.adj_off = model.idx.len() as u32;
        model.idx.extend_from_slice(&adj);
        if model.idx.is_empty() {
            model.idx.push(0);
        }
        model
    }
}
