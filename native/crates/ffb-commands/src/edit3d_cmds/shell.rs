use std::collections::HashMap;

pub(crate) struct Shell {
    pub positions: Vec<[f32; 3]>,
    pub distances: Vec<f32>,
    pub clamped: usize,
}

type V3 = [f32; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: V3) -> V3 {
    let l = dot(a, a).sqrt();
    if l < 1e-20 {
        [0.0; 3]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

pub(crate) fn offset_surface(positions: &[V3], triangles: &[[u32; 3]], offset: f32) -> Shell {
    let n = positions.len();
    let mut ids: HashMap<[i64; 3], u32> = HashMap::new();
    let mut weld = vec![0u32; n];
    let mut rep: Vec<usize> = Vec::new();
    for (i, p) in positions.iter().enumerate() {
        let key = [(p[0] * 100.0).round() as i64, (p[1] * 100.0).round() as i64, (p[2] * 100.0).round() as i64];
        let next = rep.len() as u32;
        let id = *ids.entry(key).or_insert(next);
        if id == next {
            rep.push(i);
        }
        weld[i] = id;
    }
    let w = rep.len();
    let at = |k: usize| positions[rep[k]];
    let mut normal = vec![[0.0f32; 3]; w];
    let mut near: Vec<Vec<u32>> = vec![Vec::new(); w];
    for t in triangles {
        let c = [weld[t[0] as usize], weld[t[1] as usize], weld[t[2] as usize]];
        if c[0] == c[1] || c[1] == c[2] || c[0] == c[2] {
            continue;
        }
        let face = cross(sub(at(c[1] as usize), at(c[0] as usize)), sub(at(c[2] as usize), at(c[0] as usize)));
        for k in 0..3 {
            let v = c[k] as usize;
            for a in 0..3 {
                normal[v][a] += face[a];
            }
            for other in [c[(k + 1) % 3], c[(k + 2) % 3]] {
                if !near[v].contains(&other) {
                    near[v].push(other);
                }
            }
        }
    }
    let volume: f64 = triangles.iter().map(|t| f64::from(dot(positions[t[0] as usize], cross(positions[t[1] as usize], positions[t[2] as usize])))).sum();
    let sign = if volume < 0.0 { -1.0 } else { 1.0 };
    for v in &mut normal {
        *v = unit([v[0] * sign, v[1] * sign, v[2] * sign]);
    }
    for _ in 0..3 {
        let prev = normal.clone();
        for i in 0..w {
            if near[i].is_empty() {
                continue;
            }
            let mut acc = prev[i];
            for &j in &near[i] {
                for a in 0..3 {
                    acc[a] += prev[j as usize][a] / near[i].len() as f32;
                }
            }
            normal[i] = unit(acc);
        }
    }
    let mut d = vec![offset; w];
    for i in 0..w {
        let mut limit = f32::INFINITY;
        for &j in &near[i] {
            let e = sub(at(j as usize), at(i));
            let l2 = dot(e, e);
            if l2 < 1e-10 {
                continue;
            }
            let k = dot(sub(normal[j as usize], normal[i]), e) / l2;
            if k < -1e-6 {
                limit = limit.min(0.7 / -k);
            }
        }
        d[i] = d[i].min(limit);
    }
    for _ in 0..4 {
        for i in 0..w {
            for &j in &near[i] {
                let e = dot(sub(at(j as usize), at(i)), sub(at(j as usize), at(i))).sqrt();
                let cap = d[j as usize] + 0.5 * e;
                if d[i] > cap {
                    d[i] = cap;
                }
            }
        }
    }
    let clamped = d.iter().filter(|x| **x < offset - 0.01).count();
    let distances: Vec<f32> = (0..n).map(|i| d[weld[i] as usize]).collect();
    let out = (0..n)
        .map(|i| {
            let (nv, dv) = (normal[weld[i] as usize], distances[i]);
            [positions[i][0] + nv[0] * dv, positions[i][1] + nv[1] * dv, positions[i][2] + nv[2] * dv]
        })
        .collect();
    Shell { positions: out, distances, clamped }
}
