use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

type Q = [f64; 10];

#[derive(Debug, Clone)]
pub struct DecimateOptions {
    pub target_triangles: usize,
    pub lock: Vec<bool>,
    pub boundary_weight: f64,
    pub max_error: f64,
}

impl DecimateOptions {
    pub fn new(target_triangles: usize) -> DecimateOptions {
        DecimateOptions { target_triangles, lock: Vec::new(), boundary_weight: 20.0, max_error: f64::INFINITY }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Decimated {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub origin: Vec<u32>,
}

struct Item {
    cost: f64,
    a: u32,
    b: u32,
    va: u32,
    vb: u32,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.total_cmp(&self.cost).then(other.a.cmp(&self.a)).then(other.b.cmp(&self.b))
    }
}

fn plane_quadric(n: [f64; 3], d: f64, w: f64) -> Q {
    [
        w * n[0] * n[0],
        w * n[0] * n[1],
        w * n[0] * n[2],
        w * n[0] * d,
        w * n[1] * n[1],
        w * n[1] * n[2],
        w * n[1] * d,
        w * n[2] * n[2],
        w * n[2] * d,
        w * d * d,
    ]
}

fn add_q(a: &Q, b: &Q) -> Q {
    let mut o = [0.0; 10];
    for i in 0..10 {
        o[i] = a[i] + b[i];
    }
    o
}

fn error(q: &Q, p: [f64; 3]) -> f64 {
    let (x, y, z) = (p[0], p[1], p[2]);
    q[0] * x * x + 2.0 * q[1] * x * y + 2.0 * q[2] * x * z + 2.0 * q[3] * x + q[4] * y * y + 2.0 * q[5] * y * z + 2.0 * q[6] * y + q[7] * z * z + 2.0 * q[8] * z + q[9]
}

fn solve(q: &Q) -> Option<[f64; 3]> {
    let m = [[q[0], q[1], q[2]], [q[1], q[4], q[5]], [q[2], q[5], q[7]]];
    let r = [-q[3], -q[6], -q[8]];
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let scale = (q[0].abs() + q[4].abs() + q[7].abs()).max(1e-30);
    if det.abs() < 1e-9 * scale * scale * scale {
        return None;
    }
    let dx = r[0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (r[1] * m[2][2] - m[1][2] * r[2]) + m[0][2] * (r[1] * m[2][1] - m[1][1] * r[2]);
    let dy = m[0][0] * (r[1] * m[2][2] - m[1][2] * r[2]) - r[0] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * r[2] - r[1] * m[2][0]);
    let dz = m[0][0] * (m[1][1] * r[2] - r[1] * m[2][1]) - m[0][1] * (m[1][0] * r[2] - r[1] * m[2][0]) + r[0] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    Some([dx / det, dy / det, dz / det])
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

struct State {
    pos: Vec<[f64; 3]>,
    q: Vec<Q>,
    faces: Vec<[u32; 3]>,
    alive_face: Vec<bool>,
    vf: Vec<Vec<u32>>,
    version: Vec<u32>,
    lock: Vec<bool>,
}

impl State {
    fn neighbours(&self, v: u32) -> HashSet<u32> {
        let mut out = HashSet::new();
        for f in self.vf[v as usize].iter().filter(|f| self.alive_face[**f as usize]) {
            for x in self.faces[*f as usize] {
                if x != v {
                    out.insert(x);
                }
            }
        }
        out
    }

    fn choose(&self, a: u32, b: u32) -> Option<([f64; 3], f64)> {
        let (la, lb) = (self.lock[a as usize], self.lock[b as usize]);
        if la && lb {
            return None;
        }
        let q = add_q(&self.q[a as usize], &self.q[b as usize]);
        let (pa, pb) = (self.pos[a as usize], self.pos[b as usize]);
        let mut options: Vec<[f64; 3]> = Vec::new();
        if la {
            options.push(pa);
        } else if lb {
            options.push(pb);
        } else {
            if let Some(p) = solve(&q) {
                options.push(p);
            }
            options.push(pa);
            options.push(pb);
            options.push([(pa[0] + pb[0]) * 0.5, (pa[1] + pb[1]) * 0.5, (pa[2] + pb[2]) * 0.5]);
        }
        options.into_iter().map(|p| (p, error(&q, p).max(0.0))).min_by(|x, y| x.1.total_cmp(&y.1))
    }

    fn legal(&self, a: u32, b: u32, target: [f64; 3]) -> bool {
        let na = self.neighbours(a);
        let nb = self.neighbours(b);
        let shared_faces = self.vf[a as usize].iter().filter(|f| self.alive_face[**f as usize] && self.faces[**f as usize].contains(&b)).count();
        let common = na.intersection(&nb).count();
        if common > shared_faces {
            return false;
        }
        for (this, other) in [(a, b), (b, a)] {
            for f in self.vf[this as usize].iter().filter(|f| self.alive_face[**f as usize]) {
                let t = self.faces[*f as usize];
                if t.contains(&other) {
                    continue;
                }
                let p: Vec<[f64; 3]> = t.iter().map(|x| self.pos[*x as usize]).collect();
                let before = cross3(sub3(p[1], p[0]), sub3(p[2], p[0]));
                let moved: Vec<[f64; 3]> = t.iter().map(|x| if *x == this { target } else { self.pos[*x as usize] }).collect();
                let after = cross3(sub3(moved[1], moved[0]), sub3(moved[2], moved[0]));
                let (lb, la) = (dot3(before, before).sqrt(), dot3(after, after).sqrt());
                if la < 1e-12 || dot3(before, after) < 0.2 * lb * la {
                    return false;
                }
            }
        }
        true
    }
}

pub fn decimate(positions: &[[f32; 3]], triangles: &[[u32; 3]], options: &DecimateOptions) -> Decimated {
    let n = positions.len();
    let mut s = State {
        pos: positions.iter().map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]).collect(),
        q: vec![[0.0; 10]; n],
        faces: triangles.iter().copied().filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2] && t.iter().all(|x| (*x as usize) < n)).collect(),
        alive_face: Vec::new(),
        vf: vec![Vec::new(); n],
        version: vec![0; n],
        lock: (0..n).map(|i| options.lock.get(i).copied().unwrap_or(false)).collect(),
    };
    s.alive_face = vec![true; s.faces.len()];
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (fi, t) in s.faces.iter().enumerate() {
        let p: Vec<[f64; 3]> = t.iter().map(|x| s.pos[*x as usize]).collect();
        let c = cross3(sub3(p[1], p[0]), sub3(p[2], p[0]));
        let area2 = dot3(c, c).sqrt();
        if area2 > 1e-18 {
            let nrm = [c[0] / area2, c[1] / area2, c[2] / area2];
            let d = -dot3(nrm, p[0]);
            let qf = plane_quadric(nrm, d, area2 * 0.5);
            for x in t {
                s.q[*x as usize] = add_q(&s.q[*x as usize], &qf);
            }
        }
        for k in 0..3 {
            s.vf[t[k] as usize].push(fi as u32);
            let (a, b) = (t[k], t[(k + 1) % 3]);
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(fi);
        }
    }
    for ((a, b), fs) in &edge_faces {
        if fs.len() != 1 {
            continue;
        }
        let t = s.faces[fs[0]];
        let p: Vec<[f64; 3]> = t.iter().map(|x| s.pos[*x as usize]).collect();
        let nrm_raw = cross3(sub3(p[1], p[0]), sub3(p[2], p[0]));
        let ln = dot3(nrm_raw, nrm_raw).sqrt();
        if ln < 1e-18 {
            continue;
        }
        let nrm = [nrm_raw[0] / ln, nrm_raw[1] / ln, nrm_raw[2] / ln];
        let e = sub3(s.pos[*b as usize], s.pos[*a as usize]);
        let side = cross3(e, nrm);
        let ls = dot3(side, side).sqrt();
        if ls < 1e-18 {
            continue;
        }
        let m = [side[0] / ls, side[1] / ls, side[2] / ls];
        let d = -dot3(m, s.pos[*a as usize]);
        let qe = plane_quadric(m, d, options.boundary_weight * ls * ls);
        for v in [*a, *b] {
            s.q[v as usize] = add_q(&s.q[v as usize], &qe);
        }
    }

    let mut heap: BinaryHeap<Item> = BinaryHeap::new();
    for (a, b) in edge_faces.keys() {
        if let Some((_, cost)) = s.choose(*a, *b) {
            heap.push(Item { cost, a: *a, b: *b, va: 0, vb: 0 });
        }
    }
    let mut alive = s.faces.len();
    let mut removed = vec![false; n];
    while alive > options.target_triangles {
        let Some(item) = heap.pop() else { break };
        let (a, b) = (item.a, item.b);
        if removed[a as usize] || removed[b as usize] || s.version[a as usize] != item.va || s.version[b as usize] != item.vb {
            continue;
        }
        if item.cost > options.max_error {
            break;
        }
        let Some((target, _)) = s.choose(a, b) else { continue };
        if !s.legal(a, b, target) {
            continue;
        }
        let keep_b = s.lock[b as usize];
        let (keep, gone) = if keep_b { (b, a) } else { (a, b) };
        s.q[keep as usize] = add_q(&s.q[keep as usize], &s.q[gone as usize]);
        s.pos[keep as usize] = target;
        let moved: Vec<u32> = s.vf[gone as usize].clone();
        for f in moved {
            if !s.alive_face[f as usize] {
                continue;
            }
            let t = &mut s.faces[f as usize];
            for x in t.iter_mut() {
                if *x == gone {
                    *x = keep;
                }
            }
            let t = s.faces[f as usize];
            if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                s.alive_face[f as usize] = false;
                alive -= 1;
            } else {
                s.vf[keep as usize].push(f);
            }
        }
        let live: Vec<u32> = s.vf[keep as usize].iter().copied().filter(|f| s.alive_face[*f as usize]).collect();
        let mut seen: HashSet<u32> = HashSet::new();
        s.vf[keep as usize] = live.into_iter().filter(|f| seen.insert(*f)).collect();
        s.vf[gone as usize].clear();
        removed[gone as usize] = true;
        s.version[keep as usize] += 1;
        for w in s.neighbours(keep) {
            if let Some((_, cost)) = s.choose(keep, w) {
                let (x, y) = (keep.min(w), keep.max(w));
                heap.push(Item { cost, a: x, b: y, va: s.version[x as usize], vb: s.version[y as usize] });
            }
        }
    }

    let mut map = vec![u32::MAX; n];
    let mut out = Decimated::default();
    for (fi, t) in s.faces.iter().enumerate() {
        if !s.alive_face[fi] {
            continue;
        }
        let mut tri = [0u32; 3];
        for k in 0..3 {
            let v = t[k] as usize;
            if map[v] == u32::MAX {
                map[v] = out.positions.len() as u32;
                out.positions.push([s.pos[v][0] as f32, s.pos[v][1] as f32, s.pos[v][2] as f32]);
                out.origin.push(v as u32);
            }
            tri[k] = map[v];
        }
        out.triangles.push(tri);
    }
    out
}
