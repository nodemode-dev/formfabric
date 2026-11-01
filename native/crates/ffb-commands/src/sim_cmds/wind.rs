use ffb_pattern::Wind;

const AIR_DENSITY: f64 = 1.2;
const DRAG: f64 = 1.2;
const NOISE_SCALE: f64 = 220.0;
const NOISE_RATE: f64 = 0.6;

fn hash(x: i64, y: i64, z: i64, t: i64) -> f64 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ (t as u64).wrapping_mul(0x27D4_EB2F_1656_67C5);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub(crate) fn noise(p: [f64; 3], t: f64, salt: i64) -> f64 {
    let cell = [p[0].floor(), p[1].floor(), p[2].floor(), t.floor()];
    let f = [smooth(p[0] - cell[0]), smooth(p[1] - cell[1]), smooth(p[2] - cell[2]), smooth(t - cell[3])];
    let c = [cell[0] as i64, cell[1] as i64, cell[2] as i64, cell[3] as i64];
    let at = |dx: i64, dy: i64, dz: i64, dt: i64| hash(c[0] + dx, c[1] + dy, c[2] + dz, c[3] + dt + salt * 7919);
    let mut layer = [0.0; 2];
    for (dt, slot) in layer.iter_mut().enumerate() {
        let dt = dt as i64;
        let x00 = lerp(at(0, 0, 0, dt), at(1, 0, 0, dt), f[0]);
        let x10 = lerp(at(0, 1, 0, dt), at(1, 1, 0, dt), f[0]);
        let x01 = lerp(at(0, 0, 1, dt), at(1, 0, 1, dt), f[0]);
        let x11 = lerp(at(0, 1, 1, dt), at(1, 1, 1, dt), f[0]);
        *slot = lerp(lerp(x00, x10, f[1]), lerp(x01, x11, f[1]), f[2]);
    }
    lerp(layer[0], layer[1], f[3])
}

fn unit(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    (len > 1e-9).then(|| [v[0] / len, v[1] / len, v[2] / len])
}

fn velocity(wind: &Wind, at: [f64; 3], time: f64) -> [f64; 3] {
    let radial = wind.kind.eq_ignore_ascii_case("radial");
    let rel = [at[0] - wind.origin[0], at[1] - wind.origin[1], at[2] - wind.origin[2]];
    let (dir, run) = if radial {
        let len = (rel[0] * rel[0] + rel[1] * rel[1] + rel[2] * rel[2]).sqrt();
        (unit(rel).unwrap_or([0.0, 0.0, 1.0]), len)
    } else {
        let d = unit(wind.direction).unwrap_or([0.0, 0.0, 1.0]);
        (d, (rel[0] * d[0] + rel[1] * d[1] + rel[2] * d[2]).max(0.0))
    };
    let decay = (-wind.decay.max(0.0) * run / 100_000.0).exp();
    let mut speed = wind.strength * decay;
    let mut out = dir;
    let turb = (wind.turbulence / 100.0).clamp(0.0, 1.0);
    if turb > 0.0 {
        let q = [at[0] / NOISE_SCALE, at[1] / NOISE_SCALE, at[2] / NOISE_SCALE];
        let t = time * NOISE_RATE;
        speed *= (1.0 + turb * noise(q, t, 0)).max(0.0);
        for (k, o) in out.iter_mut().enumerate() {
            *o += turb * 0.6 * noise(q, t, k as i64 + 1);
        }
        out = unit(out).unwrap_or(dir);
    }
    [out[0] * speed, out[1] * speed, out[2] * speed]
}

pub(crate) fn forces(wind: &Wind, positions: &[[f32; 3]], triangles: &[[u32; 3]], time: f64, out: &mut Vec<[f32; 3]>) {
    out.clear();
    out.resize(positions.len(), [0.0; 3]);
    if !wind.enabled || wind.strength.abs() < 1e-9 {
        return;
    }
    let n = positions.len() as u32;
    for t in triangles {
        if t.iter().any(|&i| i >= n) {
            continue;
        }
        let p = t.map(|i| positions[i as usize].map(f64::from));
        let u = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let v = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let c = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        let area2 = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        if area2 < 1e-9 {
            continue;
        }
        let nrm = [c[0] / area2, c[1] / area2, c[2] / area2];
        let centre = [(p[0][0] + p[1][0] + p[2][0]) / 3.0, (p[0][1] + p[1][1] + p[2][1]) / 3.0, (p[0][2] + p[1][2] + p[2][2]) / 3.0];
        let w = velocity(wind, centre, time);
        let along = w[0] * nrm[0] + w[1] * nrm[1] + w[2] * nrm[2];
        let push = 0.5 * AIR_DENSITY * DRAG * along * along.abs() * 0.5 * area2 / 3.0;
        for &i in t {
            let f = &mut out[i as usize];
            f[0] += (nrm[0] * push) as f32;
            f[1] += (nrm[1] * push) as f32;
            f[2] += (nrm[2] * push) as f32;
        }
    }
}
