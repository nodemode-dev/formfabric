use crate::model::Model;

#[derive(Debug, Clone, Default)]
pub struct Pattern {
    pub n: usize,
    pub start: Vec<u32>,
    pub cols: Vec<u32>,
    pub diag: Vec<u32>,
    pub tri: Vec<[u32; 9]>,
    pub hinge: Vec<[u32; 16]>,
    pub link: Vec<[u32; 9]>,
}

impl Pattern {
    pub fn build(m: &Model) -> Pattern {
        let n = m.n;
        let mut start = vec![0u32; n + 1];
        let mut cols: Vec<u32> = Vec::new();
        let mut diag = vec![0u32; n];
        for i in 0..n {
            let mut row: Vec<u32> = m.neighbors[i].clone();
            row.push(i as u32);
            row.sort_unstable();
            row.dedup();
            diag[i] = (cols.len() + row.binary_search(&(i as u32)).unwrap_or(0)) as u32;
            cols.extend_from_slice(&row);
            start[i + 1] = cols.len() as u32;
        }
        let mut p = Pattern { n, start, cols, diag, tri: Vec::new(), hinge: Vec::new(), link: Vec::new() };
        p.tri = m
            .tris
            .iter()
            .map(|t| {
                let mut s = [0u32; 9];
                for a in 0..3 {
                    for b in 0..3 {
                        s[a * 3 + b] = p.slot(t.v[a], t.v[b]);
                    }
                }
                s
            })
            .collect();
        p.hinge = m
            .hinges
            .iter()
            .map(|h| {
                let mut s = [0u32; 16];
                for a in 0..4 {
                    for b in 0..4 {
                        s[a * 4 + b] = p.slot(h.v[a], h.v[b]);
                    }
                }
                s
            })
            .collect();
        p.link = m
            .links
            .iter()
            .map(|l| {
                let mut s = [0u32; 9];
                for a in 0..l.n {
                    for b in 0..l.n {
                        s[a * 3 + b] = p.slot(l.v[a], l.v[b]);
                    }
                }
                s
            })
            .collect();
        p
    }

    pub fn slot(&self, i: u32, j: u32) -> u32 {
        let (s, e) = (self.start[i as usize] as usize, self.start[i as usize + 1] as usize);
        match self.cols[s..e].binary_search(&j) {
            Ok(k) => (s + k) as u32,
            Err(_) => self.diag[i as usize],
        }
    }

    pub fn nnz(&self) -> usize {
        self.cols.len()
    }
}

pub fn matvec(p: &Pattern, a: &[f64], x: &[f64], y: &mut [f64]) {
    for i in 0..p.n {
        let (s, e) = (p.start[i] as usize, p.start[i + 1] as usize);
        let (mut a0, mut a1, mut a2) = (0.0, 0.0, 0.0);
        for (b, &c) in a[9 * s..9 * e].chunks_exact(9).zip(&p.cols[s..e]) {
            let c = 3 * c as usize;
            let (x0, x1, x2) = (x[c], x[c + 1], x[c + 2]);
            a0 += b[0] * x0 + b[1] * x1 + b[2] * x2;
            a1 += b[3] * x0 + b[4] * x1 + b[5] * x2;
            a2 += b[6] * x0 + b[7] * x1 + b[8] * x2;
        }
        y[3 * i] = a0;
        y[3 * i + 1] = a1;
        y[3 * i + 2] = a2;
    }
}

pub fn row_apply(p: &Pattern, a: &[f64], x: &[f64], i: usize) -> [f64; 3] {
    let (s, e) = (p.start[i] as usize, p.start[i + 1] as usize);
    let (mut a0, mut a1, mut a2) = (0.0, 0.0, 0.0);
    for (b, &c) in a[9 * s..9 * e].chunks_exact(9).zip(&p.cols[s..e]) {
        let c = 3 * c as usize;
        let (x0, x1, x2) = (x[c], x[c + 1], x[c + 2]);
        a0 += b[0] * x0 + b[1] * x1 + b[2] * x2;
        a1 += b[3] * x0 + b[4] * x1 + b[5] * x2;
        a2 += b[6] * x0 + b[7] * x1 + b[8] * x2;
    }
    [a0, a1, a2]
}

pub fn inverse3(m: &[f64]) -> Option<[f64; 9]> {
    let c0 = m[4] * m[8] - m[5] * m[7];
    let c1 = m[5] * m[6] - m[3] * m[8];
    let c2 = m[3] * m[7] - m[4] * m[6];
    let det = m[0] * c0 + m[1] * c1 + m[2] * c2;
    if det.abs() < 1.0e-300 || !det.is_finite() {
        return None;
    }
    let id = 1.0 / det;
    Some([
        c0 * id,
        (m[2] * m[7] - m[1] * m[8]) * id,
        (m[1] * m[5] - m[2] * m[4]) * id,
        c1 * id,
        (m[0] * m[8] - m[2] * m[6]) * id,
        (m[2] * m[3] - m[0] * m[5]) * id,
        c2 * id,
        (m[1] * m[6] - m[0] * m[7]) * id,
        (m[0] * m[4] - m[1] * m[3]) * id,
    ])
}
