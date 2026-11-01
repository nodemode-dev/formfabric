use std::collections::{HashMap, HashSet};

use crate::fields::site;
use crate::geom::{cross, dist, dot, len, near_on_segment, sub, Bucket};
use crate::types::V2;

pub struct Lattice {
    pub sites: Vec<V2>,
    pub quads: Vec<[u32; 4]>,
}

fn find(parent: &mut [u32], mut x: u32) -> u32 {
    while parent[x as usize] != x {
        parent[x as usize] = parent[parent[x as usize] as usize];
        x = parent[x as usize];
    }
    x
}

fn quad_score(p: [V2; 4], edge: f64) -> Option<f64> {
    let mut worst = 0.0f64;
    for k in 0..4 {
        let (a, b, c) = (p[(k + 3) % 4], p[k], p[(k + 1) % 4]);
        let u = sub(a, b);
        let v = sub(c, b);
        let lu = len(u);
        let lv = len(v);
        if lu < 0.55 * edge || lv < 0.55 * edge || lu > 1.6 * edge || lv > 1.6 * edge {
            return None;
        }
        let ang = cross(v, u).atan2(dot(u, v)).to_degrees();
        if ang <= 0.0 {
            return None;
        }
        worst = worst.max((ang - 90.0).abs());
    }
    (worst < 32.0).then_some(worst)
}

pub fn extract(pos: &[V2], nbr: &[Vec<u32>], theta: &[f64], p: &[V2], edge: f64, loop_pts: &[V2]) -> Lattice {
    let n = pos.len();
    let s: Vec<V2> = (0..n).map(|v| site(p[v], theta[v], edge, pos[v])).collect();
    let mut parent: Vec<u32> = (0..n as u32).collect();
    for v in 0..n {
        for &w in &nbr[v] {
            if (w as usize) > v && dist(s[v], s[w as usize]) < 0.5 * edge {
                let (a, b) = (find(&mut parent, v as u32), find(&mut parent, w));
                if a != b {
                    parent[b as usize] = a;
                }
            }
        }
    }
    let mut id_of: HashMap<u32, u32> = HashMap::new();
    let mut sites: Vec<V2> = Vec::new();
    let mut weight: Vec<f64> = Vec::new();
    let mut cluster = vec![0u32; n];
    for v in 0..n {
        let root = find(&mut parent, v as u32);
        let id = *id_of.entry(root).or_insert_with(|| {
            sites.push([0.0, 0.0]);
            weight.push(0.0);
            sites.len() as u32 - 1
        });
        cluster[v] = id;
        sites[id as usize][0] += s[v][0];
        sites[id as usize][1] += s[v][1];
        weight[id as usize] += 1.0;
    }
    for (c, w) in sites.iter_mut().zip(&weight) {
        c[0] /= w;
        c[1] /= w;
    }

    let mut links: Vec<Vec<u32>> = vec![Vec::new(); sites.len()];
    for v in 0..n {
        let (sn, cs) = (theta[v].sin(), theta[v].cos());
        for &w in &nbr[v] {
            let (a, b) = (cluster[v], cluster[w as usize]);
            if a == b {
                continue;
            }
            let d = sub(sites[b as usize], sites[a as usize]);
            let l = len(d);
            if l < 0.6 * edge || l > 1.5 * edge {
                continue;
            }
            let u = (d[0] * cs + d[1] * sn).abs() / l;
            let t = (-d[0] * sn + d[1] * cs).abs() / l;
            if u.max(t) < 0.9 {
                continue;
            }
            links[a as usize].push(b);
            links[b as usize].push(a);
        }
    }
    for l in links.iter_mut() {
        l.sort_unstable();
        l.dedup();
    }

    let mut found: HashSet<[u32; 4]> = HashSet::new();
    let mut cand: Vec<(f64, [u32; 4])> = Vec::new();
    for a in 0..sites.len() {
        for &b in &links[a] {
            for &d in &links[a] {
                if b == d {
                    continue;
                }
                let (u, v) = (sub(sites[b as usize], sites[a]), sub(sites[d as usize], sites[a]));
                let ang = cross(u, v).atan2(dot(u, v)).to_degrees();
                if !(58.0..=122.0).contains(&ang) {
                    continue;
                }
                let want = [sites[b as usize][0] + sites[d as usize][0] - sites[a][0], sites[b as usize][1] + sites[d as usize][1] - sites[a][1]];
                let mut best: Option<(f64, u32)> = None;
                for &c in &links[b as usize] {
                    if c as usize == a || !links[d as usize].contains(&c) {
                        continue;
                    }
                    let e = dist(sites[c as usize], want);
                    if e < 0.4 * edge && best.is_none_or(|x| e < x.0) {
                        best = Some((e, c));
                    }
                }
                let Some((_, c)) = best else { continue };
                let q = [a as u32, b, c, d];
                let first = (0..4).min_by_key(|k| q[*k]).unwrap_or(0);
                let key = [q[first], q[(first + 1) % 4], q[(first + 2) % 4], q[(first + 3) % 4]];
                if !found.insert(key) {
                    continue;
                }
                let pts = [sites[key[0] as usize], sites[key[1] as usize], sites[key[2] as usize], sites[key[3] as usize]];
                if let Some(score) = quad_score(pts, edge) {
                    cand.push((score, key));
                }
            }
        }
    }
    cand.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));

    let margin = 0.45 * edge;
    let bucket = Bucket::new(loop_pts, edge * 2.0);
    let near_rim = |q: V2| -> f64 {
        let reach = margin * 1.01;
        let mut best = f64::MAX;
        for i in bucket.near(q, reach + 2.0 * edge) {
            let i = i as usize;
            let (a, b) = (loop_pts[i], loop_pts[(i + 1) % loop_pts.len()]);
            best = best.min(near_on_segment(a, b, q).1);
            let (a, b) = (loop_pts[(i + loop_pts.len() - 1) % loop_pts.len()], loop_pts[i]);
            best = best.min(near_on_segment(a, b, q).1);
        }
        best
    };
    let mut ok_site: HashMap<u32, bool> = HashMap::new();
    let mut check = |id: u32| -> bool {
        *ok_site.entry(id).or_insert_with(|| {
            let q = sites[id as usize];
            crate::geom::inside(loop_pts, q) && near_rim(q) >= margin
        })
    };

    let mut used: HashSet<(u32, u32)> = HashSet::new();
    let mut quads: Vec<[u32; 4]> = Vec::new();
    for (_, q) in cand {
        if !q.iter().all(|v| check(*v)) {
            continue;
        }
        let edges = [(q[0], q[1]), (q[1], q[2]), (q[2], q[3]), (q[3], q[0])];
        if edges.iter().any(|e| used.contains(e)) {
            continue;
        }
        used.extend(edges);
        quads.push(q);
    }
    Lattice { sites, quads }
}
