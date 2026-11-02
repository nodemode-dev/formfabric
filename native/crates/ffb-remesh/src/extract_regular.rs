use std::collections::HashSet;

use crate::edges::{twins, NO_TWIN};
use crate::geom::{cross, dot, inside, len, near_on_segment, sub, Bucket};
use crate::regular::{regularize, Regular};
use crate::types::{Report, V2};

pub struct Grid {
    pub sites: Vec<V2>,
    pub quads: Vec<[u32; 4]>,
}

fn root(parent: &mut [u32], mut x: u32) -> u32 {
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
        let (lu, lv) = (len(u), len(v));
        if lu < 0.5 * edge || lv < 0.5 * edge || lu > 1.7 * edge || lv > 1.7 * edge {
            return None;
        }
        let ang = cross(v, u).atan2(dot(u, v)).to_degrees();
        if ang <= 0.0 {
            return None;
        }
        worst = worst.max((ang - 90.0).abs());
    }
    (worst < 38.0).then_some(worst)
}

pub fn collapse(reg: &Regular, edge: f64, loop_pts: &[V2]) -> Grid {
    let n = reg.pos.len();
    let nf = reg.tris.len();
    let mut parent: Vec<u32> = (0..n as u32).collect();
    for f in 0..nf {
        for j in 0..3 {
            if reg.d[3 * f + j] == [0, 0] {
                let (a, b) = (root(&mut parent, reg.tris[f][j]), root(&mut parent, reg.tris[f][(j + 1) % 3]));
                if a != b {
                    parent[b as usize] = a;
                }
            }
        }
    }
    let mut cls = vec![u32::MAX; n];
    let mut anchor: Vec<V2> = Vec::new();
    let mut weight: Vec<f64> = Vec::new();
    let mut ids = vec![0u32; n];
    for (v, id) in ids.iter_mut().enumerate() {
        let r = root(&mut parent, v as u32) as usize;
        if cls[r] == u32::MAX {
            cls[r] = anchor.len() as u32;
            anchor.push([0.0, 0.0]);
            weight.push(0.0);
        }
        let c = cls[r] as usize;
        *id = c as u32;
        anchor[c][0] += reg.pos[v][0];
        anchor[c][1] += reg.pos[v][1];
        weight[c] += 1.0;
    }
    for (a, w) in anchor.iter_mut().zip(&weight) {
        a[0] /= w;
        a[1] /= w;
    }

    let valid: Vec<bool> = (0..nf)
        .map(|f| {
            let t = reg.tris[f];
            let (a, b, c) = (ids[t[0] as usize], ids[t[1] as usize], ids[t[2] as usize]);
            let ok_d = (0..3).all(|j| reg.d[3 * f + j][0].abs() <= 1 && reg.d[3 * f + j][1].abs() <= 1 && reg.d[3 * f + j] != [0, 0]);
            let sum = (0..3).fold([0, 0], |s, j| [s[0] + reg.d[3 * f + j][0], s[1] + reg.d[3 * f + j][1]]);
            a != b && b != c && a != c && ok_d && sum == [0, 0]
        })
        .collect();

    let mut links: Vec<Vec<(u32, V2)>> = vec![Vec::new(); anchor.len()];
    for f in 0..nf {
        if !valid[f] {
            continue;
        }
        let (sn, cs) = reg.phi[f].sin_cos();
        for j in 0..3 {
            let d = reg.d[3 * f + j];
            let off = [edge * (cs * d[0] as f64 - sn * d[1] as f64), edge * (sn * d[0] as f64 + cs * d[1] as f64)];
            let (a, b) = (ids[reg.tris[f][j] as usize], ids[reg.tris[f][(j + 1) % 3] as usize]);
            links[a as usize].push((b, off));
            links[b as usize].push((a, [-off[0], -off[1]]));
        }
    }
    let mut sites = anchor.clone();
    for it in 0..80 {
        let mut moved = 0.0f64;
        for k in 0..sites.len() {
            let c = if it % 2 == 0 { k } else { sites.len() - 1 - k };
            if links[c].is_empty() {
                continue;
            }
            let mut acc = [0.0, 0.0];
            for (o, off) in &links[c] {
                acc[0] += sites[*o as usize][0] - off[0];
                acc[1] += sites[*o as usize][1] - off[1];
            }
            let w = 0.3 * links[c].len() as f64;
            let next = [(acc[0] + w * anchor[c][0]) / (links[c].len() as f64 + w), (acc[1] + w * anchor[c][1]) / (links[c].len() as f64 + w)];
            moved = moved.max(len(sub(next, sites[c])));
            sites[c] = next;
        }
        if moved < 1e-4 * edge {
            break;
        }
    }

    let tw = twins(&reg.tris);
    let mut found: HashSet<[u32; 4]> = HashSet::new();
    let mut cand: Vec<(f64, [u32; 4])> = Vec::new();
    for f in 0..nf {
        if !valid[f] {
            continue;
        }
        for j in 0..3 {
            let h = 3 * f + j;
            let d = reg.d[h];
            if d[0].abs() != 1 || d[1].abs() != 1 || tw[h] == NO_TWIN || (tw[h] as usize) < h {
                continue;
            }
            let g = tw[h] as usize / 3;
            if !valid[g] {
                continue;
            }
            let (a, b, c) = (ids[reg.tris[f][j] as usize], ids[reg.tris[f][(j + 1) % 3] as usize], ids[reg.tris[f][(j + 2) % 3] as usize]);
            let x = ids[reg.tris[g][(tw[h] as usize % 3 + 2) % 3] as usize];
            if x == c || x == a || x == b {
                continue;
            }
            let q = [a, x, b, c];
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
    cand.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));

    let margin = 0.45 * edge;
    let bucket = Bucket::new(loop_pts, edge * 2.0);
    let m = loop_pts.len();
    let near_rim = |q: V2| -> f64 {
        let mut best = f64::MAX;
        for i in bucket.near(q, margin * 1.01 + 2.0 * edge) {
            let i = i as usize;
            best = best.min(near_on_segment(loop_pts[i], loop_pts[(i + 1) % m], q).1);
            best = best.min(near_on_segment(loop_pts[(i + m - 1) % m], loop_pts[i], q).1);
        }
        best
    };
    let ok: Vec<bool> = sites.iter().map(|q| inside(loop_pts, *q) && near_rim(*q) >= margin).collect();
    let mut used: HashSet<(u32, u32)> = HashSet::new();
    let mut quads: Vec<[u32; 4]> = Vec::new();
    for (_, q) in cand {
        if !q.iter().all(|v| ok[*v as usize]) {
            continue;
        }
        let edges = [(q[0], q[1]), (q[1], q[2]), (q[2], q[3]), (q[3], q[0])];
        if edges.iter().any(|e| used.contains(e)) {
            continue;
        }
        used.extend(edges);
        quads.push(q);
    }
    Grid { sites, quads }
}

pub fn regular_grid(pos: &[V2], tris: &[[u32; 3]], theta: &[f64], p: &[V2], edge: f64, loop_pts: &[V2], report: &mut Report) -> Option<Grid> {
    let reg = match regularize(pos, tris, theta, p, edge) {
        Ok(r) => r,
        Err(why) => {
            report.regular_note = why;
            return None;
        }
    };
    let st = reg.stats;
    report.mcf_supply = st.supply as usize;
    report.mcf_flow = st.flow as usize;
    report.mcf_cost = st.cost as usize;
    report.mcf_capacity = st.capacity;
    report.mcf_rounds = st.rounds;
    report.mcf_frozen = st.frozen;
    report.mcf_rim_variables = st.rim_variables;
    report.mcf_splits = st.splits;
    report.singularities_before = st.before;
    report.singularities_after_flow = st.after;
    if st.after > 0 {
        report.regular_note = format!("flow not full: {} triangles keep a nonzero offset sum", st.after);
        return None;
    }
    let grid = collapse(&reg, edge, loop_pts);
    if grid.quads.is_empty() {
        report.regular_note = "the regular extraction produced no quads".into();
        return None;
    }
    report.regular_used = true;
    Some(grid)
}
