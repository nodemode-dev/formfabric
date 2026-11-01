use std::collections::HashMap;

use glam::DVec3;

use crate::bvh::{Feature, TriBvh};
use crate::Collider;

#[derive(Debug, Clone)]
pub struct MeshCollider {
    pub bvh: TriBvh,
    face_normal: Vec<DVec3>,
    vertex_normal: Vec<DVec3>,
    edge_normal: HashMap<(u32, u32), DVec3>,
}

#[derive(Debug, Clone)]
pub enum Prepared {
    Plane { point: DVec3, normal: DVec3 },
    Sphere { center: DVec3, radius: f64 },
    Capsule { a: DVec3, b: DVec3, radius: f64 },
    Mesh(Box<MeshCollider>),
}

#[derive(Debug, Clone, Copy)]
pub struct Contact {
    pub normal: DVec3,
    pub depth: f64,
}

fn v3(p: [f32; 3]) -> DVec3 {
    DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64)
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

impl MeshCollider {
    fn new(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> MeshCollider {
        let pos: Vec<DVec3> = positions.iter().map(|p| v3(*p)).collect();
        let tris: Vec<[u32; 3]> = triangles.iter().copied().filter(|t| t.iter().all(|&i| (i as usize) < pos.len())).collect();
        let mut face_normal = Vec::with_capacity(tris.len());
        let mut vertex_normal = vec![DVec3::ZERO; pos.len()];
        let mut edge_normal: HashMap<(u32, u32), DVec3> = HashMap::new();
        for t in &tris {
            let p = [pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]];
            let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
            face_normal.push(n);
            for k in 0..3 {
                let e1 = (p[(k + 1) % 3] - p[k]).normalize_or_zero();
                let e2 = (p[(k + 2) % 3] - p[k]).normalize_or_zero();
                let angle = e1.dot(e2).clamp(-1.0, 1.0).acos();
                vertex_normal[t[k] as usize] += n * angle;
                *edge_normal.entry(edge_key(t[k], t[(k + 1) % 3])).or_insert(DVec3::ZERO) += n;
            }
        }
        for n in vertex_normal.iter_mut() {
            *n = n.normalize_or_zero();
        }
        for n in edge_normal.values_mut() {
            *n = n.normalize_or_zero();
        }
        MeshCollider { bvh: TriBvh::build(pos, tris), face_normal, vertex_normal, edge_normal }
    }

    fn pseudo_normal(&self, tri: u32, feature: Feature) -> DVec3 {
        let t = self.bvh.triangles[tri as usize];
        match feature {
            Feature::Face => self.face_normal[tri as usize],
            Feature::Vertex(k) => self.vertex_normal[t[k as usize] as usize],
            Feature::Edge(k) => {
                let (a, b) = (t[k as usize], t[(k as usize + 1) % 3]);
                self.edge_normal.get(&edge_key(a, b)).copied().unwrap_or(self.face_normal[tri as usize])
            }
        }
    }

    fn signed(&self, x: DVec3, range: f64) -> Option<(f64, DVec3)> {
        let c = self.bvh.closest(x, range)?;
        let n = self.pseudo_normal(c.tri, c.feature);
        let d = c.dist2.sqrt();
        let dir = x - c.point;
        let side = if d < 1.0e-9 { 1.0 } else { dir.dot(n).signum() };
        let normal = if d < 1.0e-6 { n } else { (dir / d) * side };
        let normal = if normal.length_squared() < 0.5 { n } else { normal };
        Some((d * side, if side < 0.0 { n } else { normal }))
    }
}

impl Prepared {
    pub fn from_collider(c: &Collider) -> Prepared {
        match c {
            Collider::Plane { point, normal } => Prepared::Plane { point: v3(*point), normal: v3(*normal).normalize_or(DVec3::Y) },
            Collider::Sphere { center, radius } => Prepared::Sphere { center: v3(*center), radius: *radius as f64 },
            Collider::Capsule { a, b, radius } => Prepared::Capsule { a: v3(*a), b: v3(*b), radius: *radius as f64 },
            Collider::Mesh { positions, triangles } => Prepared::Mesh(Box::new(MeshCollider::new(positions, triangles))),
        }
    }

    pub fn signed_distance(&self, x: DVec3, range: f64) -> Option<(f64, DVec3)> {
        match self {
            Prepared::Plane { point, normal } => Some(((x - *point).dot(*normal), *normal)),
            Prepared::Sphere { center, radius } => {
                let d = x - *center;
                let l = d.length();
                let n = if l > 1.0e-9 { d / l } else { DVec3::Y };
                Some((l - radius, n))
            }
            Prepared::Capsule { a, b, radius } => {
                let ab = *b - *a;
                let len2 = ab.length_squared();
                let t = if len2 > 1.0e-12 { ((x - *a).dot(ab) / len2).clamp(0.0, 1.0) } else { 0.0 };
                let c = *a + ab * t;
                let d = x - c;
                let l = d.length();
                let n = if l > 1.0e-9 { d / l } else { DVec3::Y };
                Some((l - radius, n))
            }
            Prepared::Mesh(m) => m.signed(x, range),
        }
    }

    pub fn contact(&self, x: DVec3, offset: f64) -> Option<Contact> {
        let (sd, n) = self.signed_distance(x, offset.max(1.0e-6))?;
        (sd < offset).then_some(Contact { normal: n, depth: offset - sd })
    }

    pub fn sweep(&self, from: DVec3, to: DVec3, offset: f64) -> Option<Contact> {
        let Prepared::Mesh(m) = self else {
            return None;
        };
        let (t, tri) = m.bvh.segment_hit(from, to)?;
        let mut n = m.face_normal[tri as usize];
        let d = to - from;
        if d.dot(n) > 0.0 {
            n = -n;
        }
        let hit = from + d * t;
        let along = (to - hit).dot(n);
        Some(Contact { normal: n, depth: offset - along })
    }
}

pub const CONTACT_STIFFNESS: f64 = 1000.0;
pub const FRICTION_EPS: f64 = 0.01;

pub fn deepest(colliders: &[Prepared], p: DVec3, offset: f64) -> Option<Contact> {
    let mut best: Option<Contact> = None;
    for c in colliders {
        if let Some(ct) = c.contact(p, offset) {
            if ct.depth > 0.0 && best.is_none_or(|b| b.depth < ct.depth) {
                best = Some(ct);
            }
        }
    }
    best
}

pub struct ContactForce {
    pub grad: DVec3,
    pub hess: glam::DMat3,
}

pub fn contact_force(ct: Contact, m_over_h2: f64, mu: f64, displacement: DVec3) -> ContactForce {
    let n = ct.normal;
    let kc = CONTACT_STIFFNESS * m_over_h2;
    let nn = glam::DMat3::from_cols(n * n.x, n * n.y, n * n.z);
    let mut grad = -n * (kc * ct.depth);
    let mut hess = nn * kc;
    if mu > 0.0 {
        let lam = kc * ct.depth;
        let ut = displacement - n * displacement.dot(n);
        let s = (ut.length_squared() + FRICTION_EPS * FRICTION_EPS).sqrt();
        let raw = mu * lam / s;
        let w = raw.min(kc);
        grad += ut * w;
        hess += (glam::DMat3::IDENTITY - nn) * w;
    }
    ContactForce { grad, hess }
}

pub fn pair_friction(global: f64, material: f64) -> f64 {
    if material > 0.0 {
        (global * material).sqrt()
    } else {
        global
    }
}

pub fn resolve(colliders: &[Prepared], x: DVec3, offset: f64) -> (DVec3, Option<Contact>) {
    let mut pos = x;
    let mut out: Option<Contact> = None;
    for _ in 0..2 {
        let mut moved = false;
        for c in colliders {
            if let Some(ct) = c.contact(pos, offset) {
                if ct.depth > 1.0e-9 {
                    pos += ct.normal * ct.depth;
                    moved = true;
                    out = Some(match out {
                        Some(o) if o.depth >= ct.depth => o,
                        _ => ct,
                    });
                }
            }
        }
        if !moved {
            break;
        }
    }
    (pos, out)
}

pub fn max_penetration(colliders: &[Prepared], x: &[DVec3]) -> f64 {
    let mut worst: f64 = 0.0;
    for &p in x {
        for c in colliders {
            if let Some((sd, _)) = c.signed_distance(p, 1.0e9) {
                worst = worst.max(-sd);
            }
        }
    }
    worst
}
