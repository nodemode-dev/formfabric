pub type V3 = [f32; 3];
pub type Q = [f32; 4];

pub const IDENTITY: Q = [0.0, 0.0, 0.0, 1.0];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn length(a: V3) -> f32 {
    dot(a, a).sqrt()
}

pub fn normalize(a: V3) -> V3 {
    let l = length(a);
    if l > 1e-12 {
        scale(a, 1.0 / l)
    } else {
        [0.0, 1.0, 0.0]
    }
}

pub fn lerp(a: V3, b: V3, t: f32) -> V3 {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn qmul(a: Q, b: Q) -> Q {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

pub fn qconj(q: Q) -> Q {
    [-q[0], -q[1], -q[2], q[3]]
}

pub fn qnorm(q: Q) -> Q {
    let l = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if l < 1e-9 || !l.is_finite() {
        IDENTITY
    } else {
        [q[0] / l, q[1] / l, q[2] / l, q[3] / l]
    }
}

pub fn qrot(q: Q, v: V3) -> V3 {
    let u = [q[0], q[1], q[2]];
    let t = scale(cross(u, v), 2.0);
    add(add(v, scale(t, q[3])), cross(u, t))
}

pub fn qaxis(axis: V3, angle: f32) -> Q {
    let a = normalize(axis);
    let h = angle * 0.5;
    let s = h.sin();
    [a[0] * s, a[1] * s, a[2] * s, h.cos()]
}

pub fn qarc(from: V3, to: V3) -> Q {
    let a = normalize(from);
    let b = normalize(to);
    let d = dot(a, b);
    if d > 0.999_999 {
        return IDENTITY;
    }
    if d < -0.999_999 {
        let mut axis = cross(a, [1.0, 0.0, 0.0]);
        if length(axis) < 1e-3 {
            axis = cross(a, [0.0, 0.0, 1.0]);
        }
        return qaxis(axis, std::f32::consts::PI);
    }
    let c = cross(a, b);
    qnorm([c[0], c[1], c[2], 1.0 + d])
}

pub fn qeuler(degrees: V3) -> Q {
    let r = scale(degrees, std::f32::consts::PI / 180.0);
    let qx = qaxis([1.0, 0.0, 0.0], r[0]);
    let qy = qaxis([0.0, 1.0, 0.0], r[1]);
    let qz = qaxis([0.0, 0.0, 1.0], r[2]);
    qmul(qz, qmul(qy, qx))
}

pub fn qfrom_columns(c0: V3, c1: V3, c2: V3) -> Q {
    let (m00, m10, m20) = (c0[0], c0[1], c0[2]);
    let (m01, m11, m21) = (c1[0], c1[1], c1[2]);
    let (m02, m12, m22) = (c2[0], c2[1], c2[2]);
    let tr = m00 + m11 + m22;
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s]
    };
    qnorm(q)
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn hash_f32s(h: &mut u64, values: &[f32]) {
    for v in values {
        *h ^= u64::from(v.to_bits());
        *h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

pub const FNV_START: u64 = 0xcbf2_9ce4_8422_2325;
