use std::collections::{HashMap, HashSet};

use crate::geom::{add, cross, mul, sub};
use crate::types::V2;

pub struct Poly {
    pub positions: Vec<V2>,
    pub faces: Vec<Vec<u32>>,
    pub rim: Vec<bool>,
}

fn face_ok(positions: &[V2], f: &[u32]) -> bool {
    let n = f.len();
    for k in 0..n {
        let (a, b, c) = (positions[f[(k + n - 1) % n] as usize], positions[f[k] as usize], positions[f[(k + 1) % n] as usize]);
        if cross(sub(b, a), sub(c, b)) <= 1e-12 {
            return false;
        }
    }
    true
}

fn vertex_faces(poly: &Poly) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); poly.positions.len()];
    for (i, f) in poly.faces.iter().enumerate() {
        for v in f {
            out[*v as usize].push(i);
        }
    }
    out
}

fn neighbours(poly: &Poly) -> Vec<Vec<u32>> {
    let mut out: Vec<Vec<u32>> = vec![Vec::new(); poly.positions.len()];
    for f in &poly.faces {
        let n = f.len();
        for k in 0..n {
            out[f[k] as usize].push(f[(k + 1) % n]);
            out[f[(k + 1) % n] as usize].push(f[k]);
        }
    }
    for l in out.iter_mut() {
        l.sort_unstable();
        l.dedup();
    }
    out
}

pub fn smooth(poly: &mut Poly, rounds: usize) {
    let vf = vertex_faces(poly);
    let nbr = neighbours(poly);
    for _ in 0..rounds {
        for v in 0..poly.positions.len() {
            if poly.rim[v] || nbr[v].is_empty() || vf[v].is_empty() {
                continue;
            }
            let mut c = [0.0, 0.0];
            for w in &nbr[v] {
                c = add(c, poly.positions[*w as usize]);
            }
            let target = mul(c, 1.0 / nbr[v].len() as f64);
            let old = poly.positions[v];
            poly.positions[v] = add(old, mul(sub(target, old), 0.5));
            if !vf[v].iter().all(|f| face_ok(&poly.positions, &poly.faces[*f])) {
                poly.positions[v] = old;
            }
        }
    }
}

fn target(rim: bool) -> i64 {
    if rim {
        3
    } else {
        4
    }
}

pub fn singularities(poly: &Poly) -> usize {
    let nbr = neighbours(poly);
    let vf = vertex_faces(poly);
    (0..poly.positions.len())
        .filter(|v| !vf[*v].is_empty() && !poly.rim[*v] && nbr[*v].len() != 4 && vf[*v].iter().all(|f| poly.faces[*f].len() == 4))
        .count()
}

pub fn remove_valence2(poly: &mut Poly) -> usize {
    let mut removed = 0;
    loop {
        let vf = vertex_faces(poly);
        let mut done = false;
        for (v, around) in vf.iter().enumerate() {
            if poly.rim[v] || around.len() != 2 {
                continue;
            }
            let (i, j) = (around[0], around[1]);
            if poly.faces[i].len() != 4 || poly.faces[j].len() != 4 {
                continue;
            }
            let rot = |f: &Vec<u32>| -> Vec<u32> {
                let at = f.iter().position(|x| *x == v as u32).unwrap_or(0);
                (0..4).map(|k| f[(at + k) % 4]).collect()
            };
            let (p, q) = (rot(&poly.faces[i]), rot(&poly.faces[j]));
            if p[1] != q[3] || p[3] != q[1] {
                continue;
            }
            let merged = vec![p[1], p[2], p[3], q[2]];
            if !face_ok(&poly.positions, &merged) {
                continue;
            }
            poly.faces[i] = merged;
            poly.faces.swap_remove(j);
            removed += 1;
            done = true;
            break;
        }
        if !done {
            break;
        }
    }
    removed
}

pub fn collapse_diagonals(poly: &mut Poly) -> usize {
    let mut applied = 0;
    for _ in 0..4000 {
        let nbr = neighbours(poly);
        let vf = vertex_faces(poly);
        let val = |v: u32| -> i64 { nbr[v as usize].len() as i64 };
        let mut best: Option<(i64, usize, u32, u32)> = None;
        for (fi, f) in poly.faces.iter().enumerate() {
            if f.len() != 4 {
                continue;
            }
            for k in 0..2 {
                let (a, b, c, d) = (f[k], f[k + 1], f[k + 2], f[(k + 3) % 4]);
                if poly.rim[a as usize] || poly.rim[c as usize] {
                    continue;
                }
                if vf[a as usize].iter().chain(vf[c as usize].iter()).any(|x| poly.faces[*x].len() != 4) {
                    continue;
                }
                let common: HashSet<u32> = nbr[a as usize].iter().filter(|x| nbr[c as usize].contains(x)).copied().collect();
                if common.len() != 2 || !common.contains(&b) || !common.contains(&d) {
                    continue;
                }
                let before = (val(a) - 4).abs() + (val(c) - 4).abs() + (val(b) - target(poly.rim[b as usize])).abs() + (val(d) - target(poly.rim[d as usize])).abs();
                let merged = val(a) + val(c) - 4;
                let after = (merged - 4).abs() + (val(b) - 1 - target(poly.rim[b as usize])).abs() + (val(d) - 1 - target(poly.rim[d as usize])).abs();
                if val(b) <= 2 || val(d) <= 2 {
                    continue;
                }
                let gain = before - after;
                if gain >= 2 && best.is_none_or(|x| gain > x.0) {
                    best = Some((gain, fi, a, c));
                }
            }
        }
        let Some((_, fi, a, c)) = best else { break };
        let mut trial = poly.faces.clone();
        let mid = mul(add(poly.positions[a as usize], poly.positions[c as usize]), 0.5);
        trial.swap_remove(fi);
        for f in trial.iter_mut() {
            for x in f.iter_mut() {
                if *x == c {
                    *x = a;
                }
            }
        }
        let old = poly.positions[a as usize];
        poly.positions[a as usize] = mid;
        let sound = trial.iter().filter(|f| f.contains(&a)).all(|f| {
            let mut s: Vec<u32> = f.to_vec();
            s.sort_unstable();
            s.windows(2).all(|w| w[0] != w[1]) && face_ok(&poly.positions, f)
        });
        if sound {
            poly.faces = trial;
            applied += 1;
        } else {
            poly.positions[a as usize] = old;
            break;
        }
    }
    applied
}

pub fn pure_quads(poly: &Poly) -> (Poly, HashMap<(u32, u32), u32>) {
    let mut positions = poly.positions.clone();
    let mut rim = poly.rim.clone();
    let mut mid: HashMap<(u32, u32), u32> = HashMap::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for f in &poly.faces {
        let n = f.len();
        let mut m: Vec<u32> = Vec::with_capacity(n);
        for k in 0..n {
            let (a, b) = (f[k], f[(k + 1) % n]);
            let key = (a.min(b), a.max(b));
            let id = *mid.entry(key).or_insert_with(|| {
                let p = mul(add(poly.positions[a as usize], poly.positions[b as usize]), 0.5);
                positions.push(p);
                rim.push(false);
                positions.len() as u32 - 1
            });
            m.push(id);
        }
        let mut c = [0.0, 0.0];
        for v in f {
            c = add(c, poly.positions[*v as usize]);
        }
        positions.push(mul(c, 1.0 / n as f64));
        rim.push(false);
        let centre = positions.len() as u32 - 1;
        for k in 0..n {
            faces.push(vec![f[k], m[k], centre, m[(k + n - 1) % n]]);
        }
    }
    let mut on_rim: HashMap<(u32, u32), u32> = HashMap::new();
    for f in &poly.faces {
        let n = f.len();
        for k in 0..n {
            let (a, b) = (f[k], f[(k + 1) % n]);
            *on_rim.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    for (key, m) in &mid {
        if on_rim.get(key) == Some(&1) {
            rim[*m as usize] = true;
        }
    }
    (Poly { positions, faces, rim }, mid)
}
