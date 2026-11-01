use super::matrix::Pattern;
use super::pcg::{Filter, PINNED, PLANE};

#[derive(Debug, Clone, Default)]
pub struct Coarse {
    pub agg: Vec<u32>,
    pub nc: usize,
    pub bw: usize,
    pub slot_i: Vec<u32>,
    pub slot_j: Vec<u32>,
    pub band: Vec<f64>,
    pub rhs: Vec<f64>,
    pub ok: bool,
}

fn project_rows(b: &mut [f64; 9], n: glam::DVec3) {
    let nv = [n.x, n.y, n.z];
    for c in 0..3 {
        let d = nv[0] * b[c] + nv[1] * b[3 + c] + nv[2] * b[6 + c];
        for r in 0..3 {
            b[3 * r + c] -= d * nv[r];
        }
    }
}

fn project_cols(b: &mut [f64; 9], n: glam::DVec3) {
    let nv = [n.x, n.y, n.z];
    for r in 0..3 {
        let d = b[3 * r] * nv[0] + b[3 * r + 1] * nv[1] + b[3 * r + 2] * nv[2];
        for c in 0..3 {
            b[3 * r + c] -= d * nv[c];
        }
    }
}

fn greedy(adj: &[Vec<u32>]) -> (Vec<u32>, usize) {
    let n = adj.len();
    let mut agg = vec![u32::MAX; n];
    let mut count = 0u32;
    for i in 0..n {
        if agg[i] != u32::MAX || adj[i].iter().any(|&j| agg[j as usize] != u32::MAX) {
            continue;
        }
        agg[i] = count;
        for &j in &adj[i] {
            agg[j as usize] = count;
        }
        count += 1;
    }
    for i in 0..n {
        if agg[i] != u32::MAX {
            continue;
        }
        let mut best = u32::MAX;
        for &j in &adj[i] {
            let a = agg[j as usize];
            if a != u32::MAX {
                best = a;
                break;
            }
        }
        if best == u32::MAX {
            best = count;
            count += 1;
        }
        agg[i] = best;
    }
    (agg, count as usize)
}

fn coarsen(adj: &[Vec<u32>], agg: &[u32], nc: usize) -> Vec<Vec<u32>> {
    let mut out: Vec<Vec<u32>> = vec![Vec::new(); nc];
    for (i, row) in adj.iter().enumerate() {
        let a = agg[i];
        for &j in row {
            let b = agg[j as usize];
            if a != b {
                out[a as usize].push(b);
            }
        }
    }
    for r in &mut out {
        r.sort_unstable();
        r.dedup();
    }
    out
}

fn cuthill_mckee(adj: &[Vec<u32>]) -> Vec<u32> {
    let n = adj.len();
    let mut order: Vec<u32> = Vec::with_capacity(n);
    let mut seen = vec![false; n];
    let mut by_degree: Vec<u32> = (0..n as u32).collect();
    by_degree.sort_by_key(|&i| adj[i as usize].len());
    for &s in &by_degree {
        if seen[s as usize] {
            continue;
        }
        seen[s as usize] = true;
        let mut head = order.len();
        order.push(s);
        while head < order.len() {
            let v = order[head] as usize;
            head += 1;
            let mut nb: Vec<u32> = adj[v].iter().copied().filter(|&j| !seen[j as usize]).collect();
            nb.sort_by_key(|&j| adj[j as usize].len());
            for j in nb {
                seen[j as usize] = true;
                order.push(j);
            }
        }
    }
    order.reverse();
    let mut rank = vec![0u32; n];
    for (k, &v) in order.iter().enumerate() {
        rank[v as usize] = k as u32;
    }
    rank
}

impl Coarse {
    pub fn build(pat: &Pattern, levels: usize) -> Coarse {
        let n = pat.n;
        let mut adj: Vec<Vec<u32>> = (0..n)
            .map(|i| pat.cols[pat.start[i] as usize..pat.start[i + 1] as usize].iter().copied().filter(|&j| j as usize != i).collect())
            .collect();
        let mut agg: Vec<u32> = (0..n as u32).collect();
        let mut nc = n;
        for _ in 0..levels {
            let (a, c) = greedy(&adj);
            if c == nc || c < 2 {
                break;
            }
            adj = coarsen(&adj, &a, c);
            for v in &mut agg {
                *v = a[*v as usize];
            }
            nc = c;
        }
        let rank = cuthill_mckee(&adj);
        for v in &mut agg {
            *v = rank[*v as usize];
        }
        let mut span = 0usize;
        for (i, row) in adj.iter().enumerate() {
            for &j in row {
                span = span.max((rank[i] as i64 - rank[j as usize] as i64).unsigned_abs() as usize);
            }
        }
        let bw = 3 * span + 2;
        let mut slot_i = vec![0u32; pat.nnz()];
        let mut slot_j = vec![0u32; pat.nnz()];
        for i in 0..n {
            for s in pat.start[i] as usize..pat.start[i + 1] as usize {
                slot_i[s] = agg[i];
                slot_j[s] = agg[pat.cols[s] as usize];
            }
        }
        Coarse { agg, nc, bw, slot_i, slot_j, band: vec![0.0; 3 * nc * (bw + 1)], rhs: vec![0.0; 3 * nc], ok: false }
    }

    pub fn factor(&mut self, pat: &Pattern, a: &[f64], f: &Filter) {
        let w = self.bw + 1;
        self.band.fill(0.0);
        for i in 0..pat.n {
            let ki = f.kind[i];
            if ki == PINNED {
                continue;
            }
            for s in pat.start[i] as usize..pat.start[i + 1] as usize {
                let j = pat.cols[s] as usize;
                let kj = f.kind[j];
                if kj == PINNED {
                    continue;
                }
                let (ci, cj) = (3 * self.slot_i[s] as usize, 3 * self.slot_j[s] as usize);
                if cj > ci + 2 {
                    continue;
                }
                let mut blk = [0.0f64; 9];
                blk.copy_from_slice(&a[9 * s..9 * s + 9]);
                if ki == PLANE {
                    project_rows(&mut blk, f.nrm[i]);
                }
                if kj == PLANE {
                    project_cols(&mut blk, f.nrm[j]);
                }
                for r in 0..3 {
                    for c in 0..3 {
                        let (rr, cc) = (ci + r, cj + c);
                        if rr >= cc {
                            self.band[rr * w + self.bw - (rr - cc)] += blk[3 * r + c];
                        }
                    }
                }
            }
        }
        let n = 3 * self.nc;
        let bw = self.bw;
        let mut maxd = 0.0f64;
        for i in 0..n {
            maxd = maxd.max(self.band[i * w + bw]);
        }
        let ridge = 1.0e-12 * maxd;
        self.ok = true;
        for i in 0..n {
            let j0 = i.saturating_sub(bw);
            for j in j0..=i {
                let k0 = j0.max(j.saturating_sub(bw));
                let mut sum = self.band[i * w + bw - (i - j)];
                let (ri, rj) = (i * w + bw, j * w + bw);
                for k in k0..j {
                    sum -= self.band[ri - (i - k)] * self.band[rj - (j - k)];
                }
                if i == j {
                    if !sum.is_finite() {
                        self.ok = false;
                        return;
                    }
                    let d = if sum <= ridge { maxd } else { sum };
                    self.band[i * w + bw] = d.sqrt();
                } else {
                    self.band[i * w + bw - (i - j)] = sum / self.band[j * w + bw];
                }
            }
        }
    }

    pub fn apply(&mut self, r: &[f64], z: &mut [f64]) {
        let w = self.bw + 1;
        let bw = self.bw;
        let n = 3 * self.nc;
        self.rhs.fill(0.0);
        for (i, &g) in self.agg.iter().enumerate() {
            let o = 3 * g as usize;
            self.rhs[o] += r[3 * i];
            self.rhs[o + 1] += r[3 * i + 1];
            self.rhs[o + 2] += r[3 * i + 2];
        }
        for i in 0..n {
            let j0 = i.saturating_sub(bw);
            let mut s = self.rhs[i];
            let base = i * w + bw;
            for j in j0..i {
                s -= self.band[base - (i - j)] * self.rhs[j];
            }
            self.rhs[i] = s / self.band[base];
        }
        for i in (0..n).rev() {
            let x = self.rhs[i] / self.band[i * w + bw];
            self.rhs[i] = x;
            let j0 = i.saturating_sub(bw);
            for j in j0..i {
                self.rhs[j] -= self.band[i * w + bw - (i - j)] * x;
            }
        }
        for (i, &g) in self.agg.iter().enumerate() {
            let o = 3 * g as usize;
            z[3 * i] += self.rhs[o];
            z[3 * i + 1] += self.rhs[o + 1];
            z[3 * i + 2] += self.rhs[o + 2];
        }
    }
}
