use std::collections::HashMap;

use glam::{DVec2, DVec3};

use crate::energy::{HingeElem, LinkElem, TriElem};
use crate::{Cloth, Material};

pub const FOLD_GAIN: f64 = 100.0;

#[derive(Debug, Clone, Copy)]
pub struct Adj {
    pub kind: u8,
    pub slot: u8,
    pub idx: u32,
}

pub const KIND_TRI: u8 = 0;
pub const KIND_HINGE: u8 = 1;
pub const KIND_LINK: u8 = 2;

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub n: usize,
    pub triangles: Vec<[u32; 3]>,
    pub tris: Vec<TriElem>,
    pub tri_pressure: Vec<(u32, f64)>,
    pub hinges: Vec<HingeElem>,
    pub links: Vec<LinkElem>,
    pub mass: Vec<f64>,
    pub friction: Vec<f64>,
    pub thickness: Vec<f64>,
    pub layer: Vec<i32>,
    pub adj_start: Vec<u32>,
    pub adj: Vec<Adj>,
    pub colors: Vec<Vec<u32>>,
    pub neighbors: Vec<Vec<u32>>,
    pub rest_edge: f64,
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

impl Model {
    pub fn build(cloth: &Cloth) -> Model {
        let n = cloth.positions.len();
        let mut model = Model { n, ..Model::default() };
        let mats: Vec<Material> = if cloth.materials.is_empty() { vec![Material::default().effective()] } else { cloth.materials.iter().map(|m| m.effective()).collect() };
        let pos: Vec<DVec3> = cloth.positions.iter().map(|p| DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64)).collect();
        let rest2: Vec<DVec2> = (0..n).map(|i| cloth.rest.get(i).map_or(DVec2::ZERO, |r| DVec2::new(r[0] as f64, r[1] as f64))).collect();
        let mat_of = |t: usize| -> &Material { &mats[(cloth.triangle_material.get(t).copied().unwrap_or(0) as usize).min(mats.len() - 1)] };
        let grain_of = |t: usize| -> f64 { cloth.triangle_grain.get(t).copied().unwrap_or(0.0) as f64 };
        let panel = |v: u32| cloth.vertex_panel.get(v as usize).copied().unwrap_or(0);

        let mut mass = vec![0.0f64; n];
        let mut edge_sum = 0.0;
        let mut edge_cnt = 0.0;
        let mut tri_of_tri: Vec<Option<u32>> = vec![None; cloth.triangles.len()];
        for (ti, t) in cloth.triangles.iter().enumerate() {
            if t.iter().any(|&i| i as usize >= n) {
                continue;
            }
            model.triangles.push(*t);
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
            let elem = TriElem {
                v: *t,
                alpha: [-(ia + ic), ia, ic],
                beta: [-(ib + id), ib, id],
                area: 0.5 * det.abs(),
                ku: m.stretch_weft as f64,
                kv: m.stretch_warp as f64,
                ks: m.shear as f64,
                kd: m.kd(),
            };
            tri_of_tri[ti] = Some(model.tris.len() as u32);
            model.tris.push(elem);
            if m.pressure != 0.0 {
                model.tri_pressure.push((model.triangles.len() as u32 - 1, m.pressure as f64));
            }
        }
        model.rest_edge = if edge_cnt > 0.0 { edge_sum / edge_cnt } else { 10.0 };
        let floor_mass = model.rest_edge * model.rest_edge * 0.5 * crate::material::DEFAULT_DENSITY as f64 * 0.01;
        for m in mass.iter_mut() {
            if *m < floor_mass {
                *m = floor_mass;
            }
        }
        model.mass = mass;
        model.friction = vec![0.0; n];
        model.thickness = vec![0.0; n];
        for (ti, t) in cloth.triangles.iter().enumerate() {
            if t.iter().all(|&i| (i as usize) < n) {
                let f = mat_of(ti).friction as f64;
                let th = mat_of(ti).thickness as f64;
                for &i in t {
                    model.friction[i as usize] = f;
                    model.thickness[i as usize] = model.thickness[i as usize].max(th);
                }
            }
        }
        model.layer = (0..n).map(|i| cloth.vertex_layer.get(i).copied().unwrap_or(0)).collect();

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
            hinge_of.insert(key, model.hinges.len());
            model.hinges.push(hinge);
        }
        for fold in &cloth.folds {
            let key = ekey(fold.edge[0], fold.edge[1]);
            let boost = 1.0 + FOLD_GAIN * fold.strength.clamp(0.0, 1.0) as f64;
            if let Some(&hi) = hinge_of.get(&key) {
                let h = &mut model.hinges[hi];
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
            let tri = cloth.triangles.iter().position(|t| t.contains(&v[0]) && t.contains(&v[1])).unwrap_or(0);
            let mut h = make_hinge(&pos, &rest2, &mats, cloth, v, tri, None, fold.angle as f64, &panel);
            h.coef *= boost;
            model.hinges.push(h);
        }

        let default_link_k = mats.iter().map(|m| m.stretch_weft.max(m.stretch_warp) as f64).fold(0.0, f64::max).max(1.0);
        for s in &cloth.seams {
            if s.a as usize >= n || s.b as usize >= n || s.a == s.b {
                continue;
            }
            let k = if s.stiffness > 0.0 { s.stiffness as f64 } else { default_link_k };
            model.links.push(LinkElem { v: [s.a, s.b, 0], c: [1.0, -1.0, 0.0], n: 2, k, rest: s.rest_length.max(0.0) as f64 });
        }
        for b in &cloth.binds {
            if b.a as usize >= n || b.b.iter().any(|&i| i as usize >= n) {
                continue;
            }
            let k = if b.stiffness > 0.0 { b.stiffness as f64 } else { default_link_k };
            let w = (b.w as f64).clamp(0.0, 1.0);
            model.links.push(LinkElem { v: [b.a, b.b[0], b.b[1]], c: [1.0, -(1.0 - w), -w], n: 3, k, rest: b.rest_length.max(0.0) as f64 });
        }

        model.build_adjacency();
        model
    }

    fn build_adjacency(&mut self) {
        let n = self.n;
        let mut count = vec![0u32; n + 1];
        for t in &self.tris {
            for &v in &t.v {
                count[v as usize] += 1;
            }
        }
        for h in &self.hinges {
            for &v in &h.v {
                count[v as usize] += 1;
            }
        }
        for l in &self.links {
            for &v in &l.v[..l.n] {
                count[v as usize] += 1;
            }
        }
        let mut start = vec![0u32; n + 1];
        for i in 0..n {
            start[i + 1] = start[i] + count[i];
        }
        let mut fill = start.clone();
        let mut adj = vec![Adj { kind: 0, slot: 0, idx: 0 }; start[n] as usize];
        let mut put = |v: u32, a: Adj| {
            adj[fill[v as usize] as usize] = a;
            fill[v as usize] += 1;
        };
        for (i, t) in self.tris.iter().enumerate() {
            for (s, &v) in t.v.iter().enumerate() {
                put(v, Adj { kind: KIND_TRI, slot: s as u8, idx: i as u32 });
            }
        }
        for (i, h) in self.hinges.iter().enumerate() {
            for (s, &v) in h.v.iter().enumerate() {
                put(v, Adj { kind: KIND_HINGE, slot: s as u8, idx: i as u32 });
            }
        }
        for (i, l) in self.links.iter().enumerate() {
            for (s, &v) in l.v[..l.n].iter().enumerate() {
                put(v, Adj { kind: KIND_LINK, slot: s as u8, idx: i as u32 });
            }
        }
        self.adj_start = start;
        self.adj = adj;

        let mut neigh: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut link = |vs: &[u32]| {
            for &a in vs {
                for &b in vs {
                    if a != b {
                        neigh[a as usize].push(b);
                    }
                }
            }
        };
        for t in &self.tris {
            link(&t.v);
        }
        for h in &self.hinges {
            link(&h.v);
        }
        for l in &self.links {
            link(&l.v[..l.n]);
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
        let mut colors = vec![Vec::new(); ncol];
        for (i, &c) in color.iter().enumerate() {
            colors[c as usize].push(i as u32);
        }
        self.colors = colors;
        self.neighbors = neigh;
    }
}

#[allow(clippy::too_many_arguments)]
fn make_hinge(pos: &[DVec3], rest2: &[DVec2], mats: &[Material], cloth: &Cloth, v: [u32; 4], t0: usize, t1: Option<usize>, theta0: f64, panel: &dyn Fn(u32) -> u32) -> HingeElem {
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
    let ratio = 0.5 * (m0.buckling_ratio + m1.buckling_ratio) as f64;
    let floor = 0.5 * (m0.buckling_stiffness + m1.buckling_stiffness) as f64;
    HingeElem {
        v,
        coef: b * 3.0 * le * le / area,
        theta0,
        h_rest: [2.0 * a1 / le, 2.0 * a2 / le],
        buckle_ratio: ratio,
        buckle_floor: floor.clamp(0.0, 1.0),
        kd: 0.5 * (m0.kd() + m1.kd()),
    }
}
