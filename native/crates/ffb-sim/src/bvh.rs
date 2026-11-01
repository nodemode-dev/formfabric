use glam::DVec3;

const LEAF: usize = 4;

#[derive(Debug, Clone, Copy)]
struct Node {
    min: DVec3,
    max: DVec3,
    first: u32,
    count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Feature {
    Face,
    Edge(u8),
    Vertex(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct Closest {
    pub tri: u32,
    pub point: DVec3,
    pub dist2: f64,
    pub feature: Feature,
}

#[derive(Debug, Clone, Default)]
pub struct TriBvh {
    nodes: Vec<Node>,
    order: Vec<u32>,
    pub positions: Vec<DVec3>,
    pub triangles: Vec<[u32; 3]>,
}

fn box_dist2(min: DVec3, max: DVec3, p: DVec3) -> f64 {
    let d = (min - p).max(p - max).max(DVec3::ZERO);
    d.length_squared()
}

pub fn closest_on_triangle(p: DVec3, a: DVec3, b: DVec3, c: DVec3) -> (DVec3, Feature) {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, Feature::Vertex(0));
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, Feature::Vertex(1));
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return (a + ab * v, Feature::Edge(0));
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, Feature::Vertex(2));
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return (a + ac * w, Feature::Edge(2));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (b + (c - b) * w, Feature::Edge(1));
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    (a + ab * v + ac * w, Feature::Face)
}

pub fn barycentric(p: DVec3, a: DVec3, b: DVec3, c: DVec3) -> [f64; 3] {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let den = d00 * d11 - d01 * d01;
    if den.abs() < 1.0e-30 {
        return [1.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / den;
    let w = (d00 * d21 - d01 * d20) / den;
    [1.0 - v - w, v, w]
}

impl TriBvh {
    pub fn build(positions: Vec<DVec3>, triangles: Vec<[u32; 3]>) -> TriBvh {
        let n = triangles.len();
        let mut bvh = TriBvh { nodes: Vec::new(), order: (0..n as u32).collect(), positions, triangles };
        if n == 0 {
            return bvh;
        }
        let boxes: Vec<(DVec3, DVec3, DVec3)> = bvh
            .triangles
            .iter()
            .map(|t| {
                let a = bvh.positions[t[0] as usize];
                let b = bvh.positions[t[1] as usize];
                let c = bvh.positions[t[2] as usize];
                (a.min(b).min(c), a.max(b).max(c), (a + b + c) / 3.0)
            })
            .collect();
        bvh.nodes.push(Node { min: DVec3::ZERO, max: DVec3::ZERO, first: 0, count: n as u32 });
        let mut stack = vec![0usize];
        while let Some(ni) = stack.pop() {
            let (first, count) = (bvh.nodes[ni].first as usize, bvh.nodes[ni].count as usize);
            let mut lo = DVec3::splat(f64::INFINITY);
            let mut hi = DVec3::splat(f64::NEG_INFINITY);
            let mut clo = lo;
            let mut chi = hi;
            for &t in &bvh.order[first..first + count] {
                let b = boxes[t as usize];
                lo = lo.min(b.0);
                hi = hi.max(b.1);
                clo = clo.min(b.2);
                chi = chi.max(b.2);
            }
            bvh.nodes[ni].min = lo;
            bvh.nodes[ni].max = hi;
            if count <= LEAF {
                continue;
            }
            let ext = chi - clo;
            let axis = if ext.x >= ext.y && ext.x >= ext.z {
                0
            } else if ext.y >= ext.z {
                1
            } else {
                2
            };
            let mid = count / 2;
            bvh.order[first..first + count].select_nth_unstable_by(mid, |a, b| boxes[*a as usize].2[axis].total_cmp(&boxes[*b as usize].2[axis]));
            let left = bvh.nodes.len();
            bvh.nodes.push(Node { min: DVec3::ZERO, max: DVec3::ZERO, first: first as u32, count: mid as u32 });
            bvh.nodes.push(Node { min: DVec3::ZERO, max: DVec3::ZERO, first: (first + mid) as u32, count: (count - mid) as u32 });
            bvh.nodes[ni].first = left as u32;
            bvh.nodes[ni].count = 0;
            stack.push(left);
            stack.push(left + 1);
        }
        bvh
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    pub fn closest(&self, p: DVec3, max_dist: f64) -> Option<Closest> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best: Option<Closest> = None;
        let mut best2 = max_dist * max_dist;
        let mut stack: [u32; 64] = [0; 64];
        let mut sp = 1;
        while sp > 0 {
            sp -= 1;
            let node = self.nodes[stack[sp] as usize];
            if box_dist2(node.min, node.max, p) > best2 {
                continue;
            }
            if node.count > 0 {
                for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                    let tr = self.triangles[t as usize];
                    let (cp, feature) = closest_on_triangle(p, self.positions[tr[0] as usize], self.positions[tr[1] as usize], self.positions[tr[2] as usize]);
                    let d2 = (cp - p).length_squared();
                    if d2 <= best2 {
                        best2 = d2;
                        best = Some(Closest { tri: t, point: cp, dist2: d2, feature });
                    }
                }
            } else if sp + 2 <= stack.len() {
                let l = node.first;
                let dl = box_dist2(self.nodes[l as usize].min, self.nodes[l as usize].max, p);
                let dr = box_dist2(self.nodes[l as usize + 1].min, self.nodes[l as usize + 1].max, p);
                if dl < dr {
                    stack[sp] = l + 1;
                    stack[sp + 1] = l;
                } else {
                    stack[sp] = l;
                    stack[sp + 1] = l + 1;
                }
                sp += 2;
            }
        }
        best
    }

    pub fn segment_hit(&self, a: DVec3, b: DVec3) -> Option<(f64, u32)> {
        if self.nodes.is_empty() {
            return None;
        }
        let d = b - a;
        let inv = DVec3::new(1.0 / d.x, 1.0 / d.y, 1.0 / d.z);
        let mut best: Option<(f64, u32)> = None;
        let mut tmax = 1.0;
        let mut stack: [u32; 64] = [0; 64];
        let mut sp = 1;
        while sp > 0 {
            sp -= 1;
            let node = self.nodes[stack[sp] as usize];
            let t0 = (node.min - a) * inv;
            let t1 = (node.max - a) * inv;
            let near = t0.min(t1).max_element().max(0.0);
            let far = t0.max(t1).min_element().min(tmax);
            if near > far {
                continue;
            }
            if node.count > 0 {
                for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                    let tr = self.triangles[t as usize];
                    if let Some(s) = ray_triangle(a, d, self.positions[tr[0] as usize], self.positions[tr[1] as usize], self.positions[tr[2] as usize]) {
                        if s <= tmax {
                            tmax = s;
                            best = Some((s, t));
                        }
                    }
                }
            } else if sp + 2 <= stack.len() {
                stack[sp] = node.first;
                stack[sp + 1] = node.first + 1;
                sp += 2;
            }
        }
        best
    }
}

fn ray_triangle(o: DVec3, d: DVec3, a: DVec3, b: DVec3, c: DVec3) -> Option<f64> {
    let e1 = b - a;
    let e2 = c - a;
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1.0e-14 {
        return None;
    }
    let inv = 1.0 / det;
    let s = o - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (0.0..=1.0).contains(&t).then_some(t)
}
