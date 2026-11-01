use std::collections::VecDeque;

use crate::geom::{add, corner_angle, dot, len, lerp, mul, near_on_segment, sub, unit};
use crate::types::V2;

pub struct Pin {
    pub point: V2,
    pub dir: V2,
}

pub struct FieldInput<'a> {
    pub pos: &'a [V2],
    pub nbr: &'a [Vec<u32>],
    pub rim: &'a [u32],
    pub guides: &'a [Vec<V2>],
    pub grain: f64,
    pub grain_weight: f64,
    pub h: f64,
}

pub struct Fields {
    pub theta: Vec<f64>,
    pub pins: Vec<Option<Pin>>,
    pub field_iterations: usize,
    pub position_iterations: usize,
}

fn quad_rep(dir: V2) -> V2 {
    let a = dir[1].atan2(dir[0]) * 4.0;
    [a.cos(), a.sin()]
}

pub fn adjacency(n: usize, tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    let mut nbr: Vec<Vec<u32>> = vec![Vec::new(); n];
    for t in tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            nbr[a as usize].push(b);
            nbr[b as usize].push(a);
        }
    }
    for l in nbr.iter_mut() {
        l.sort_unstable();
        l.dedup();
    }
    nbr
}

pub fn orientation(input: &FieldInput) -> Fields {
    let n = input.pos.len();
    let mut fixed: Vec<Option<V2>> = vec![None; n];
    let mut pins: Vec<Option<Pin>> = (0..n).map(|_| None).collect();
    let r = input.rim.len();
    for i in 0..r {
        let (prev, here, next) = (input.rim[(i + r - 1) % r], input.rim[i], input.rim[(i + 1) % r]);
        let (pp, ph, pn) = (input.pos[prev as usize], input.pos[here as usize], input.pos[next as usize]);
        let tangent = unit(sub(pn, pp));
        if corner_angle(pp, ph, pn) < 155.0 {
            let a = quad_rep(unit(sub(ph, pp)));
            let b = quad_rep(unit(sub(pn, ph)));
            let m = add(a, b);
            if len(m) > 0.5 {
                fixed[here as usize] = Some(unit(m));
            }
        } else {
            fixed[here as usize] = Some(quad_rep(tangent));
            pins[here as usize] = Some(Pin { point: ph, dir: tangent });
        }
    }
    for g in input.guides {
        if g.len() < 2 {
            continue;
        }
        for v in 0..n {
            let q = input.pos[v];
            let mut best: Option<(f64, V2, V2)> = None;
            for w in g.windows(2) {
                if len(sub(w[1], w[0])) < 1e-9 {
                    continue;
                }
                let (foot, d) = near_on_segment(w[0], w[1], q);
                if best.as_ref().is_none_or(|b| d < b.0) {
                    best = Some((d, foot, unit(sub(w[1], w[0]))));
                }
            }
            if let Some((d, foot, dir)) = best {
                if d <= input.h * 1.2 {
                    fixed[v] = Some(quad_rep(dir));
                    pins[v] = Some(Pin { point: foot, dir });
                }
            }
        }
    }

    let grain_z = quad_rep([input.grain.cos(), input.grain.sin()]);
    let mut z: Vec<V2> = vec![grain_z; n];
    let mut seen = vec![false; n];
    let mut queue: VecDeque<u32> = VecDeque::new();
    for v in 0..n {
        if let Some(f) = fixed[v] {
            z[v] = f;
            seen[v] = true;
            queue.push_back(v as u32);
        }
    }
    while let Some(v) = queue.pop_front() {
        for &w in &input.nbr[v as usize] {
            if !seen[w as usize] {
                seen[w as usize] = true;
                z[w as usize] = z[v as usize];
                queue.push_back(w);
            }
        }
    }

    let free: Vec<usize> = (0..n).filter(|v| fixed[*v].is_none()).collect();
    let max_iter = ((2.0e7 / (n.max(1) as f64 * 6.0)) as usize).clamp(60, 4000);
    let omega = 1.6;
    let mut iterations = 0;
    for it in 0..max_iter {
        let mut delta = 0.0f64;
        let backward = it % 2 == 1;
        for k in 0..free.len() {
            let v = if backward { free[free.len() - 1 - k] } else { free[k] };
            let mut acc = [0.0, 0.0];
            for &w in &input.nbr[v] {
                acc = add(acc, z[w as usize]);
            }
            acc = add(acc, mul(grain_z, input.grain_weight * input.nbr[v].len() as f64));
            if len(acc) < 1e-12 {
                continue;
            }
            let target = unit(acc);
            let blended = add(z[v], mul(sub(target, z[v]), omega));
            let next = if len(blended) < 1e-12 { target } else { unit(blended) };
            delta = delta.max(len(sub(next, z[v])));
            z[v] = next;
        }
        iterations = it + 1;
        if delta < 1e-6 {
            break;
        }
    }
    let theta: Vec<f64> = z.iter().map(|c| c[1].atan2(c[0]) / 4.0).collect();
    Fields { theta, pins, field_iterations: iterations, position_iterations: 0 }
}

pub fn site(origin: V2, theta: f64, edge: f64, q: V2) -> V2 {
    site_cs(origin, theta.sin_cos(), edge, q)
}

fn site_cs(origin: V2, sc: (f64, f64), edge: f64, q: V2) -> V2 {
    let (s, c) = sc;
    let d = sub(q, origin);
    let u = ((d[0] * c + d[1] * s) / edge).round();
    let v = ((-d[0] * s + d[1] * c) / edge).round();
    [origin[0] + edge * (u * c - v * s), origin[1] + edge * (u * s + v * c)]
}

pub fn positions(pos: &[V2], nbr: &[Vec<u32>], fields: &mut Fields, edge: f64, seed: usize) -> Vec<V2> {
    let n = pos.len();
    let mut p: Vec<V2> = pos.to_vec();
    let mut seen = vec![false; n];
    let mut queue: VecDeque<u32> = VecDeque::new();
    if n == 0 {
        return p;
    }
    let root = seed.min(n - 1);
    seen[root] = true;
    queue.push_back(root as u32);
    while let Some(v) = queue.pop_front() {
        for &w in &nbr[v as usize] {
            if !seen[w as usize] {
                seen[w as usize] = true;
                p[w as usize] = site(p[v as usize], fields.theta[v as usize], edge, pos[w as usize]);
                queue.push_back(w);
            }
        }
    }
    let max_iter = ((1.5e7 / (n as f64 * 6.0)) as usize).clamp(40, 2500);
    let sc: Vec<(f64, f64)> = fields.theta.iter().map(|t| t.sin_cos()).collect();
    let mut iterations = 0;
    for it in 0..max_iter {
        let mut delta = 0.0f64;
        for k in 0..n {
            let v = if it % 2 == 1 { n - 1 - k } else { k };
            if nbr[v].is_empty() {
                continue;
            }
            let mut acc = [0.0, 0.0];
            for &w in &nbr[v] {
                acc = add(acc, site_cs(p[w as usize], sc[w as usize], edge, p[v]));
            }
            let mut q = mul(acc, 1.0 / nbr[v].len() as f64);
            if let Some(pin) = &fields.pins[v] {
                let along = dot(sub(q, pin.point), pin.dir);
                let on_line = add(pin.point, mul(pin.dir, along));
                q = lerp(q, on_line, 0.35);
            }
            delta = delta.max(len(sub(q, p[v])));
            p[v] = q;
        }
        iterations = it + 1;
        if delta < 5e-4 * edge {
            break;
        }
    }
    fields.position_iterations = iterations;
    p
}
