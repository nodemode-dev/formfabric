use crate::geom::Bucket;
use crate::types::V2;

pub fn lift(source: &[V2], triangles: &[[u32; 3]], values: &[[f64; 3]], points: &[V2]) -> Vec<[f64; 3]> {
    if triangles.is_empty() || source.is_empty() {
        return vec![[0.0; 3]; points.len()];
    }
    let centres: Vec<V2> = triangles
        .iter()
        .map(|t| {
            let (a, b, c) = (source[t[0] as usize], source[t[1] as usize], source[t[2] as usize]);
            [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0]
        })
        .collect();
    let mut reach = 0.0f64;
    for (t, c) in triangles.iter().zip(&centres) {
        for v in t {
            let p = source[*v as usize];
            reach = reach.max((p[0] - c[0]).hypot(p[1] - c[1]));
        }
    }
    let bucket = Bucket::new(&centres, (reach * 2.0).max(1e-6));
    points
        .iter()
        .map(|q| {
            let mut best: Option<(f64, usize, [f64; 3])> = None;
            let mut radius = reach * 1.05;
            for _ in 0..4 {
                for ti in bucket.near(*q, radius) {
                    let t = triangles[ti as usize];
                    let (a, b, c) = (source[t[0] as usize], source[t[1] as usize], source[t[2] as usize]);
                    let det = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
                    if det.abs() < 1e-18 {
                        continue;
                    }
                    let l0 = ((b[1] - c[1]) * (q[0] - c[0]) + (c[0] - b[0]) * (q[1] - c[1])) / det;
                    let l1 = ((c[1] - a[1]) * (q[0] - c[0]) + (a[0] - c[0]) * (q[1] - c[1])) / det;
                    let l2 = 1.0 - l0 - l1;
                    let miss = (-l0).max(-l1).max(-l2).max(0.0);
                    if best.as_ref().is_none_or(|x| miss < x.0) {
                        best = Some((miss, ti as usize, [l0, l1, l2]));
                    }
                }
                if best.as_ref().is_some_and(|b| b.0 <= 1e-9) {
                    break;
                }
                radius *= 2.0;
            }
            match best {
                Some((_, ti, l)) => {
                    let t = triangles[ti];
                    let w = [l[0].max(0.0), l[1].max(0.0), l[2].max(0.0)];
                    let s = (w[0] + w[1] + w[2]).max(1e-12);
                    let mut out = [0.0; 3];
                    for k in 0..3 {
                        let v = values[t[k] as usize];
                        for d in 0..3 {
                            out[d] += v[d] * w[k] / s;
                        }
                    }
                    out
                }
                None => [0.0; 3],
            }
        })
        .collect()
}
