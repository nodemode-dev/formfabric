use std::collections::HashMap;

pub(super) type V2 = [f64; 2];

pub(super) fn seg_closest(p: V2, a: V2, b: V2) -> (f64, V2, f64) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 < 1e-12 { 0.0 } else { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) };
    let q = [a[0] + dx * t, a[1] + dy * t];
    (((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt(), q, t)
}

pub(super) fn segments(points: &[V2], closed: bool) -> Vec<(V2, V2)> {
    let mut out: Vec<(V2, V2)> = points.windows(2).map(|w| (w[0], w[1])).collect();
    if closed && points.len() > 2 {
        out.push((points[points.len() - 1], points[0]));
    }
    out
}

pub(super) fn dist_to(segs: &[(V2, V2)], p: V2) -> f64 {
    segs.iter().map(|s| seg_closest(p, s.0, s.1).0).fold(f64::INFINITY, f64::min)
}

pub(super) struct Locator<'a> {
    pos: &'a [V2],
    tris: &'a [[u32; 3]],
    origin: V2,
    cell: f64,
    cols: usize,
    rows: usize,
    grid: HashMap<(usize, usize), Vec<u32>>,
}

pub(super) struct Hit {
    pub tri: usize,
    pub bary: [f64; 3],
}

fn bary(p: V2, a: V2, b: V2, c: V2) -> Option<[f64; 3]> {
    let d = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    if d.abs() < 1e-14 {
        return None;
    }
    let u = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / d;
    let v = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / d;
    Some([u, v, 1.0 - u - v])
}

fn outside(w: &[f64; 3]) -> f64 {
    (-w[0]).max(-w[1]).max(-w[2]).max(0.0)
}

impl<'a> Locator<'a> {
    pub(super) fn new(pos: &'a [V2], tris: &'a [[u32; 3]]) -> Locator<'a> {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in pos {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        if pos.is_empty() {
            lo = [0.0; 2];
            hi = [1.0; 2];
        }
        let extent = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-6);
        let target = (tris.len().max(1) as f64).sqrt().clamp(1.0, 256.0);
        let cell = extent / target;
        let cols = ((hi[0] - lo[0]) / cell).floor() as usize + 1;
        let rows = ((hi[1] - lo[1]) / cell).floor() as usize + 1;
        let mut grid: HashMap<(usize, usize), Vec<u32>> = HashMap::new();
        for (k, t) in tris.iter().enumerate() {
            let c = [pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]];
            let (mut tl, mut th) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
            for p in &c {
                for a in 0..2 {
                    tl[a] = tl[a].min(p[a]);
                    th[a] = th[a].max(p[a]);
                }
            }
            let (x0, x1) = (((tl[0] - lo[0]) / cell).floor() as usize, (((th[0] - lo[0]) / cell).floor() as usize).min(cols - 1));
            let (y0, y1) = (((tl[1] - lo[1]) / cell).floor() as usize, (((th[1] - lo[1]) / cell).floor() as usize).min(rows - 1));
            for x in x0..=x1 {
                for y in y0..=y1 {
                    grid.entry((x, y)).or_default().push(k as u32);
                }
            }
        }
        Locator { pos, tris, origin: lo, cell, cols, rows, grid }
    }

    fn eval(&self, k: usize, p: V2) -> Option<(f64, [f64; 3])> {
        let t = self.tris[k];
        let w = bary(p, self.pos[t[0] as usize], self.pos[t[1] as usize], self.pos[t[2] as usize])?;
        Some((outside(&w), w))
    }

    pub(super) fn locate(&self, p: V2) -> Option<Hit> {
        let cx = (((p[0] - self.origin[0]) / self.cell).floor().max(0.0) as usize).min(self.cols - 1);
        let cy = (((p[1] - self.origin[1]) / self.cell).floor().max(0.0) as usize).min(self.rows - 1);
        let mut best: Option<(f64, usize, [f64; 3])> = None;
        if let Some(list) = self.grid.get(&(cx, cy)) {
            for &k in list {
                if let Some((o, w)) = self.eval(k as usize, p) {
                    if best.as_ref().is_none_or(|b| o < b.0) {
                        best = Some((o, k as usize, w));
                    }
                    if o <= 1e-9 {
                        break;
                    }
                }
            }
        }
        if best.as_ref().is_none_or(|b| b.0 > 1e-9) {
            for k in 0..self.tris.len() {
                if let Some((o, w)) = self.eval(k, p) {
                    if best.as_ref().is_none_or(|b| o < b.0) {
                        best = Some((o, k, w));
                    }
                }
            }
        }
        let (_, tri, w) = best?;
        let mut c = [w[0].max(0.0), w[1].max(0.0), w[2].max(0.0)];
        let sum = c[0] + c[1] + c[2];
        if sum < 1e-12 {
            return None;
        }
        for x in &mut c {
            *x /= sum;
        }
        Some(Hit { tri, bary: c })
    }
}
