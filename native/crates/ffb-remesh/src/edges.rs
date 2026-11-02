use std::collections::HashMap;
use std::f64::consts::FRAC_PI_2;

pub const NO_TWIN: u32 = u32::MAX;

pub type D2 = [i32; 2];

pub fn twins(tris: &[[u32; 3]]) -> Vec<u32> {
    let mut at: HashMap<(u32, u32), u32> = HashMap::with_capacity(tris.len() * 3);
    for (f, t) in tris.iter().enumerate() {
        for j in 0..3 {
            at.entry((t[j], t[(j + 1) % 3])).or_insert((f * 3 + j) as u32);
        }
    }
    let mut out = vec![NO_TWIN; tris.len() * 3];
    for (f, t) in tris.iter().enumerate() {
        for j in 0..3 {
            if let Some(h) = at.get(&(t[(j + 1) % 3], t[j])) {
                out[f * 3 + j] = *h;
            }
        }
    }
    out
}

pub fn face_frames(theta: &[f64], tris: &[[u32; 3]]) -> Vec<f64> {
    tris.iter()
        .map(|t| {
            let mut z = [0.0f64, 0.0];
            for v in t {
                let a = theta[*v as usize] * 4.0;
                z[0] += a.cos();
                z[1] += a.sin();
            }
            z[1].atan2(z[0]) / 4.0
        })
        .collect()
}

pub fn turns(from: f64, to: f64) -> i32 {
    (((to - from) / FRAC_PI_2).round() as i32).rem_euclid(4)
}

pub fn rot(quarter: i32, d: D2) -> D2 {
    match quarter.rem_euclid(4) {
        0 => d,
        1 => [-d[1], d[0]],
        2 => [-d[0], -d[1]],
        _ => [d[1], -d[0]],
    }
}

pub fn carry(turns: i32, d: D2) -> D2 {
    rot(4 - turns.rem_euclid(4), [-d[0], -d[1]])
}

pub fn orient_ccw(pos: &[[f64; 2]], tris: &mut [[u32; 3]]) {
    for t in tris.iter_mut() {
        let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area < 0.0 {
            t.swap(1, 2);
        }
    }
}
