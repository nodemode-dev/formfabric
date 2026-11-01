use glam::DVec3;

use crate::bvh::{barycentric, closest_on_triangle};

#[derive(Debug, Clone, Default)]
pub struct SpatialHash {
    cell: f64,
    entries: Vec<(u64, u32)>,
}

fn cell_key(ix: i64, iy: i64, iz: i64) -> u64 {
    (ix.wrapping_mul(73_856_093) ^ iy.wrapping_mul(19_349_663) ^ iz.wrapping_mul(83_492_791)) as u64
}

impl SpatialHash {
    pub fn build(x: &[DVec3], tris: &[[u32; 3]], cell: f64, margin: f64) -> SpatialHash {
        let cell = cell.max(1.0e-3);
        let mut entries = Vec::with_capacity(tris.len() * 4);
        for (ti, t) in tris.iter().enumerate() {
            let a = x[t[0] as usize];
            let b = x[t[1] as usize];
            let c = x[t[2] as usize];
            let lo = (a.min(b).min(c) - DVec3::splat(margin)) / cell;
            let hi = (a.max(b).max(c) + DVec3::splat(margin)) / cell;
            let (lo, hi) = ((lo.x.floor() as i64, lo.y.floor() as i64, lo.z.floor() as i64), (hi.x.floor() as i64, hi.y.floor() as i64, hi.z.floor() as i64));
            if (hi.0 - lo.0 + 1) * (hi.1 - lo.1 + 1) * (hi.2 - lo.2 + 1) > 64 {
                continue;
            }
            for ix in lo.0..=hi.0 {
                for iy in lo.1..=hi.1 {
                    for iz in lo.2..=hi.2 {
                        entries.push((cell_key(ix, iy, iz), ti as u32));
                    }
                }
            }
        }
        entries.sort_unstable();
        SpatialHash { cell, entries }
    }

    pub fn query(&self, p: DVec3, out: &mut Vec<u32>) {
        out.clear();
        let key = cell_key((p.x / self.cell).floor() as i64, (p.y / self.cell).floor() as i64, (p.z / self.cell).floor() as i64);
        let start = self.entries.partition_point(|e| e.0 < key);
        for e in &self.entries[start..] {
            if e.0 != key {
                break;
            }
            out.push(e.1);
        }
    }
}

pub struct SelfInput<'a> {
    pub tris: &'a [[u32; 3]],
    pub neighbors: &'a [Vec<u32>],
    pub inv_mass: &'a [f64],
    pub thickness: f64,
}

pub fn resolve_pass(x: &mut [DVec3], hash: &SpatialHash, input: &SelfInput) -> u32 {
    let mut cand = Vec::new();
    let mut count = 0;
    for v in 0..x.len() {
        let wv = input.inv_mass[v];
        hash.query(x[v], &mut cand);
        for &ti in &cand {
            let t = input.tris[ti as usize];
            if t.contains(&(v as u32)) {
                continue;
            }
            let nb = &input.neighbors[v];
            if t.iter().any(|i| nb.binary_search(i).is_ok()) {
                continue;
            }
            let (a, b, c) = (x[t[0] as usize], x[t[1] as usize], x[t[2] as usize]);
            let (cp, _) = closest_on_triangle(x[v], a, b, c);
            let d = x[v] - cp;
            let dist = d.length();
            if dist >= input.thickness {
                continue;
            }
            let tn = (b - a).cross(c - a).normalize_or_zero();
            let n = if dist > 1.0e-9 { d / dist } else { tn };
            let bary = barycentric(cp, a, b, c);
            let wt = [input.inv_mass[t[0] as usize], input.inv_mass[t[1] as usize], input.inv_mass[t[2] as usize]];
            let wsum = wv + bary[0] * bary[0] * wt[0] + bary[1] * bary[1] * wt[1] + bary[2] * bary[2] * wt[2];
            if wsum < 1.0e-18 {
                continue;
            }
            let lambda = (input.thickness - dist) / wsum;
            x[v] += n * (lambda * wv);
            for k in 0..3 {
                x[t[k] as usize] -= n * (lambda * wt[k] * bary[k]);
            }
            count += 1;
        }
    }
    count
}
