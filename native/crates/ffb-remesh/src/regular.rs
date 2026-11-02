use std::cmp::Reverse;

use crate::edges::{carry, face_frames, orient_ccw, turns, twins, D2, NO_TWIN};
use crate::fields::site;
use crate::mcf::MinCostFlow;
use crate::types::V2;

const INTERIOR_COST: i32 = 4;
const RIM_COST: i32 = 1;
const MAX_CAPACITY: i32 = 5;

#[derive(Debug, Clone, Copy, Default)]
pub struct FlowStats {
    pub supply: i64,
    pub flow: i64,
    pub cost: i64,
    pub capacity: i32,
    pub rounds: usize,
    pub frozen: usize,
    pub rim_variables: usize,
    pub before: usize,
    pub after: usize,
    pub splits: usize,
}

pub struct Regular {
    pub pos: Vec<V2>,
    pub tris: Vec<[u32; 3]>,
    pub phi: Vec<f64>,
    pub d: Vec<D2>,
    pub stats: FlowStats,
}

struct Scalar {
    var: usize,
    comp: usize,
    row_a: usize,
    coef_a: i32,
    row_b: usize,
    coef_b: i32,
}

struct Var {
    h0: usize,
    h1: u32,
    turn: i32,
    x: D2,
}

fn find(parent: &mut [u32], par: &mut [u8], x: usize) -> (usize, u8) {
    let mut root = x;
    let mut acc = 0u8;
    while parent[root] as usize != root {
        acc ^= par[root];
        root = parent[root] as usize;
    }
    let mut cur = x;
    let mut rem = acc;
    while parent[cur] as usize != cur {
        let next = parent[cur] as usize;
        let old = par[cur];
        parent[cur] = root as u32;
        par[cur] = rem;
        rem ^= old;
        cur = next;
    }
    (root, acc)
}

fn face_sums(dh: &[D2]) -> Vec<D2> {
    (0..dh.len() / 3)
        .map(|f| {
            let (a, b, c) = (dh[3 * f], dh[3 * f + 1], dh[3 * f + 2]);
            [a[0] + b[0] + c[0], a[1] + b[1] + c[1]]
        })
        .collect()
}

fn count_broken(dh: &[D2]) -> usize {
    face_sums(dh).iter().filter(|s| s[0] != 0 || s[1] != 0).count()
}

fn rounded_offsets(tris: &[[u32; 3]], phi: &[f64], tw: &[u32], s: &[V2], edge: f64) -> Vec<D2> {
    let mut dh = vec![[0i32; 2]; tris.len() * 3];
    for (f, t) in tris.iter().enumerate() {
        let (sn, cs) = phi[f].sin_cos();
        for j in 0..3 {
            let h = 3 * f + j;
            if tw[h] != NO_TWIN && (tw[h] as usize) < h {
                continue;
            }
            let g = [s[t[(j + 1) % 3] as usize][0] - s[t[j] as usize][0], s[t[(j + 1) % 3] as usize][1] - s[t[j] as usize][1]];
            let d = [((g[0] * cs + g[1] * sn) / edge).round() as i32, ((-g[0] * sn + g[1] * cs) / edge).round() as i32];
            dh[h] = d;
            if tw[h] != NO_TWIN {
                let t1 = tw[h] as usize;
                dh[t1] = carry(turns(phi[f], phi[t1 / 3]), d);
            }
        }
    }
    dh
}

fn singular_faces(n: usize, tris: &[[u32; 3]], phi: &[f64], tw: &[u32]) -> Vec<bool> {
    let mut hol = vec![0i32; n];
    let mut rim = vec![false; n];
    for (f, t) in tris.iter().enumerate() {
        for j in 0..3 {
            let h = 3 * f + j;
            if tw[h] == NO_TWIN {
                rim[t[j] as usize] = true;
                rim[t[(j + 1) % 3] as usize] = true;
            } else {
                hol[t[j] as usize] += turns(phi[tw[h] as usize / 3], phi[f]);
            }
        }
    }
    let odd: Vec<bool> = (0..n).map(|v| !rim[v] && hol[v].rem_euclid(4) != 0).collect();
    tris.iter().map(|t| t.iter().any(|v| odd[*v as usize])).collect()
}

pub fn regularize(pos: &[V2], tris_in: &[[u32; 3]], theta: &[f64], p: &[V2], edge: f64) -> Result<Regular, String> {
    let n = pos.len();
    let mut tris = tris_in.to_vec();
    orient_ccw(pos, &mut tris);
    let nf = tris.len();
    if nf == 0 {
        return Err("the working mesh has no triangles".into());
    }
    let s: Vec<V2> = (0..n).map(|v| site(p[v], theta[v], edge, pos[v])).collect();
    let phi = face_frames(theta, &tris);
    let tw = twins(&tris);
    let mut dh = rounded_offsets(&tris, &phi, &tw, &s, edge);
    let mut stats = FlowStats { before: count_broken(&dh), ..FlowStats::default() };

    let mut vars: Vec<Var> = Vec::new();
    for h in 0..3 * nf {
        if tw[h] == NO_TWIN || h < tw[h] as usize {
            let turn = if tw[h] == NO_TWIN { 0 } else { turns(phi[h / 3], phi[tw[h] as usize / 3]) };
            vars.push(Var { h0: h, h1: tw[h], turn, x: dh[h] });
        }
    }
    let mut scalars: Vec<Scalar> = Vec::with_capacity(vars.len() * 2);
    for (i, v) in vars.iter().enumerate() {
        for c in 0..2 {
            let (row_b, coef_b) = if v.h1 == NO_TWIN {
                (usize::MAX, 0)
            } else {
                let mut unit = [0i32; 2];
                unit[c] = 1;
                let w = carry(v.turn, unit);
                let cb = if w[0] != 0 { 0 } else { 1 };
                (2 * (v.h1 as usize / 3) + cb, w[cb])
            };
            scalars.push(Scalar { var: i, comp: c, row_a: 2 * (v.h0 / 3) + c, coef_a: 1, row_b, coef_b });
        }
    }

    let odd = singular_faces(n, &tris, &phi, &tw);
    let mut order: Vec<usize> = (0..scalars.len()).filter(|i| scalars[*i].row_b != usize::MAX).collect();
    let weight = |i: usize| -> usize {
        let sc = &scalars[i];
        usize::from(odd[sc.row_a / 2]) + usize::from(odd[sc.row_b / 2])
    };
    order.sort_by_key(|i| (weight(*i), *i));
    let rows = 2 * nf;
    let mut parent: Vec<u32> = (0..rows as u32).collect();
    let mut par = vec![0u8; rows];
    let mut balanced = vec![false; scalars.len()];
    for i in order {
        let sc = &scalars[i];
        let want = u8::from(sc.coef_a * sc.coef_b != -1);
        let (ra, pa) = find(&mut parent, &mut par, sc.row_a);
        let (rb, pb) = find(&mut parent, &mut par, sc.row_b);
        if ra == rb {
            balanced[i] = pa ^ pb == want;
        } else {
            parent[rb] = ra as u32;
            par[rb] = pa ^ pb ^ want;
            balanced[i] = true;
        }
    }
    let mut eps = vec![1i32; rows];
    for (r, e) in eps.iter_mut().enumerate() {
        let (_, parity) = find(&mut parent, &mut par, r);
        if parity == 1 {
            *e = -1;
        }
    }
    stats.frozen = scalars.iter().enumerate().filter(|(i, sc)| sc.row_b != usize::MAX && !balanced[*i]).count();
    stats.rim_variables = scalars.iter().filter(|sc| sc.row_b == usize::MAX).count();

    let residual = |vars: &[Var]| -> Vec<i32> {
        let mut raw = vec![0i32; rows];
        for sc in &scalars {
            let x = vars[sc.var].x[sc.comp];
            raw[sc.row_a] += sc.coef_a * x;
            if sc.row_b != usize::MAX {
                raw[sc.row_b] += sc.coef_b * x;
            }
        }
        raw.iter().zip(&eps).map(|(r, e)| r * e).collect()
    };

    let mut capacity = 2;
    while capacity <= MAX_CAPACITY {
        let res = residual(&vars);
        if res.iter().all(|r| *r == 0) {
            break;
        }
        stats.rounds += 1;
        stats.capacity = capacity;
        let (out, src, snk) = (rows, rows + 1, rows + 2);
        let mut net = MinCostFlow::new(rows + 3);
        let mut arcs: Vec<(usize, usize, usize)> = vec![(usize::MAX, usize::MAX, 0); scalars.len()];
        for (i, sc) in scalars.iter().enumerate() {
            let alpha = sc.coef_a * eps[sc.row_a];
            let x = vars[sc.var].x[sc.comp];
            let (fwd, bwd) = if alpha == 1 { (x + capacity, capacity - x) } else { (capacity - x, x + capacity) };
            if sc.row_b == usize::MAX {
                let a = net.add_arc(sc.row_a, out, fwd, RIM_COST);
                let b = net.add_arc(out, sc.row_a, bwd, RIM_COST);
                arcs[i] = (a, b, 1);
            } else if balanced[i] {
                let a = net.add_arc(sc.row_a, sc.row_b, fwd, INTERIOR_COST);
                let b = net.add_arc(sc.row_b, sc.row_a, bwd, INTERIOR_COST);
                arcs[i] = (a, b, 2);
            }
        }
        let mut supply = 0i64;
        let total: i64 = res.iter().map(|r| *r as i64).sum();
        for (r, v) in res.iter().enumerate() {
            if *v > 0 {
                net.add_arc(src, r, *v, 0);
                supply += *v as i64;
            } else if *v < 0 {
                net.add_arc(r, snk, -*v, 0);
            }
        }
        if total < 0 {
            net.add_arc(src, out, (-total) as i32, 0);
            supply += -total;
        } else if total > 0 {
            net.add_arc(out, snk, total as i32, 0);
        }
        let solved = net.solve(src, snk, supply);
        if stats.rounds == 1 {
            stats.supply = supply;
        }
        stats.flow += solved.flow;
        stats.cost += solved.cost;
        for (i, sc) in scalars.iter().enumerate() {
            let (a, b, kind) = arcs[i];
            if kind == 0 {
                continue;
            }
            let moved = net.flow_on(a) - net.flow_on(b);
            let alpha = sc.coef_a * eps[sc.row_a];
            vars[sc.var].x[sc.comp] -= alpha * moved;
        }
        capacity += 1;
    }

    for v in &vars {
        dh[v.h0] = v.x;
        if v.h1 != NO_TWIN {
            dh[v.h1 as usize] = carry(v.turn, v.x);
        }
    }
    stats.after = count_broken(&dh);
    let mut reg = Regular { pos: pos.to_vec(), tris, phi, d: dh, stats };
    reg.stats.splits = split_long(&mut reg)?;
    Ok(reg)
}

fn split_long(reg: &mut Regular) -> Result<usize, String> {
    let mut total = 0;
    for _ in 0..12 {
        let tw = twins(&reg.tris);
        let nf = reg.tris.len();
        let mut cand: Vec<(i32, usize)> = Vec::new();
        for (h, twin) in tw.iter().enumerate() {
            if *twin == NO_TWIN || h < *twin as usize {
                let m = reg.d[h][0].abs().max(reg.d[h][1].abs());
                if m > 1 {
                    cand.push((m, h));
                }
            }
        }
        if cand.is_empty() {
            return Ok(total);
        }
        cand.sort_by_key(|(m, h)| (Reverse(*m), *h));
        let mut used = vec![false; nf];
        let mut chosen: Vec<usize> = Vec::new();
        for (_, h) in cand {
            let f = h / 3;
            let other = (tw[h] != NO_TWIN).then(|| tw[h] as usize / 3);
            if used[f] || other.is_some_and(|o| used[o]) {
                continue;
            }
            used[f] = true;
            if let Some(o) = other {
                used[o] = true;
            }
            chosen.push(h);
        }
        for h in chosen {
            let (f, j) = (h / 3, h % 3);
            let (a, b, c) = (reg.tris[f][j], reg.tris[f][(j + 1) % 3], reg.tris[f][(j + 2) % 3]);
            let x = reg.d[h];
            let d1 = reg.d[3 * f + (j + 1) % 3];
            let d2 = reg.d[3 * f + (j + 2) % 3];
            let lo = [x[0] / 2, x[1] / 2];
            let hi = [x[0] - lo[0], x[1] - lo[1]];
            let m = reg.pos.len() as u32;
            let (pa, pb) = (reg.pos[a as usize], reg.pos[b as usize]);
            reg.pos.push([(pa[0] + pb[0]) * 0.5, (pa[1] + pb[1]) * 0.5]);
            let dmc = [-lo[0] - d2[0], -lo[1] - d2[1]];
            let phi_f = reg.phi[f];
            reg.tris[f] = [a, m, c];
            reg.d[3 * f] = lo;
            reg.d[3 * f + 1] = dmc;
            reg.d[3 * f + 2] = d2;
            reg.tris.push([m, b, c]);
            reg.phi.push(phi_f);
            reg.d.extend([hi, d1, [-dmc[0], -dmc[1]]]);
            if tw[h] != NO_TWIN {
                let (f1, j1) = (tw[h] as usize / 3, tw[h] as usize % 3);
                let e = reg.tris[f1][(j1 + 2) % 3];
                let t1 = reg.d[3 * f1 + (j1 + 1) % 3];
                let t2 = reg.d[3 * f1 + (j1 + 2) % 3];
                let r = turns(phi_f, reg.phi[f1]);
                let chi = carry(r, hi);
                let clo = carry(r, lo);
                let dme = [-chi[0] - t2[0], -chi[1] - t2[1]];
                let phi_g = reg.phi[f1];
                reg.tris[f1] = [b, m, e];
                reg.d[3 * f1] = chi;
                reg.d[3 * f1 + 1] = dme;
                reg.d[3 * f1 + 2] = t2;
                reg.tris.push([m, a, e]);
                reg.phi.push(phi_g);
                reg.d.extend([clo, t1, [-dme[0], -dme[1]]]);
            }
            total += 1;
        }
    }
    if reg.d.iter().any(|d| d[0].abs() > 1 || d[1].abs() > 1) {
        return Err("edges with offsets above one remain after splitting".into());
    }
    Ok(total)
}
