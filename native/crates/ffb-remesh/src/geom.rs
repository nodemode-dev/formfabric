use crate::types::V2;

pub fn sub(a: V2, b: V2) -> V2 {
    [a[0] - b[0], a[1] - b[1]]
}

pub fn add(a: V2, b: V2) -> V2 {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn mul(a: V2, k: f64) -> V2 {
    [a[0] * k, a[1] * k]
}

pub fn dot(a: V2, b: V2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

pub fn cross(a: V2, b: V2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

pub fn len(a: V2) -> f64 {
    a[0].hypot(a[1])
}

pub fn dist(a: V2, b: V2) -> f64 {
    len(sub(a, b))
}

pub fn unit(a: V2) -> V2 {
    let n = len(a);
    if n < 1e-15 {
        [1.0, 0.0]
    } else {
        [a[0] / n, a[1] / n]
    }
}

pub fn lerp(a: V2, b: V2, t: f64) -> V2 {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

pub fn signed_area(poly: &[V2]) -> f64 {
    let n = poly.len();
    0.5 * (0..n).map(|i| cross(poly[i], poly[(i + 1) % n])).sum::<f64>()
}

pub fn inside(poly: &[V2], q: V2) -> bool {
    let n = poly.len();
    let mut odd = false;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a[1] > q[1]) != (b[1] > q[1]) && q[0] < a[0] + (q[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            odd = !odd;
        }
    }
    odd
}

pub fn near_on_segment(a: V2, b: V2, q: V2) -> (V2, f64) {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 < 1e-18 { 0.0 } else { (dot(sub(q, a), ab) / l2).clamp(0.0, 1.0) };
    let foot = add(a, mul(ab, t));
    (foot, dist(foot, q))
}

pub fn corner_angle(prev: V2, here: V2, next: V2) -> f64 {
    let u = sub(prev, here);
    let v = sub(next, here);
    cross(u, v).atan2(dot(u, v)).abs().to_degrees()
}

pub struct Bucket {
    origin: V2,
    cell: f64,
    nx: usize,
    ny: usize,
    cells: Vec<Vec<u32>>,
}

impl Bucket {
    pub fn new(points: &[V2], cell: f64) -> Bucket {
        let mut lo = [f64::MAX; 2];
        let mut hi = [f64::MIN; 2];
        for p in points {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        if points.is_empty() {
            lo = [0.0; 2];
            hi = [0.0; 2];
        }
        let cell = cell.max(1e-6);
        let nx = (((hi[0] - lo[0]) / cell).floor() as usize + 1).min(2048);
        let ny = (((hi[1] - lo[1]) / cell).floor() as usize + 1).min(2048);
        let mut b = Bucket { origin: lo, cell, nx, ny, cells: vec![Vec::new(); nx * ny] };
        for (i, p) in points.iter().enumerate() {
            let (cx, cy) = b.at(*p);
            b.cells[cy * nx + cx].push(i as u32);
        }
        b
    }

    fn at(&self, p: V2) -> (usize, usize) {
        let cx = (((p[0] - self.origin[0]) / self.cell).floor().max(0.0) as usize).min(self.nx - 1);
        let cy = (((p[1] - self.origin[1]) / self.cell).floor().max(0.0) as usize).min(self.ny - 1);
        (cx, cy)
    }

    pub fn near(&self, p: V2, reach: f64) -> Vec<u32> {
        let (x0, y0) = self.at([p[0] - reach, p[1] - reach]);
        let (x1, y1) = self.at([p[0] + reach, p[1] + reach]);
        let mut out = Vec::new();
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                out.extend_from_slice(&self.cells[cy * self.nx + cx]);
            }
        }
        out
    }
}
