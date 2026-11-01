use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const EMPTY: Aabb = Aabb { min: Vec3::splat(f32::INFINITY), max: Vec3::splat(f32::NEG_INFINITY) };

    pub fn grow(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn merge(&self, other: &Aabb) -> Aabb {
        Aabb { min: self.min.min(other.min), max: self.max.max(other.max) }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extent(&self) -> Vec3 {
        (self.max - self.min).max(Vec3::ZERO)
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x
    }

    pub fn surface_area(&self) -> f32 {
        let e = self.extent();
        2.0 * (e.x * e.y + e.y * e.z + e.z * e.x)
    }

    pub fn ray_entry(&self, origin: Vec3, inv_dir: Vec3, t_max: f32) -> Option<f32> {
        let t0 = (self.min - origin) * inv_dir;
        let t1 = (self.max - origin) * inv_dir;
        let near = t0.min(t1).max_element().max(0.0);
        let far = t0.max(t1).min_element().min(t_max);
        (near <= far).then_some(near)
    }

    pub fn distance_sq_to(&self, p: Vec3) -> f32 {
        let d = (self.min - p).max(p - self.max).max(Vec3::ZERO);
        d.length_squared()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
        let mut mesh = Mesh { positions, indices, ..Mesh::default() };
        mesh.recompute_normals();
        mesh
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn position(&self, i: u32) -> Vec3 {
        Vec3::from(self.positions[i as usize])
    }

    pub fn triangle(&self, t: usize) -> [u32; 3] {
        [self.indices[3 * t], self.indices[3 * t + 1], self.indices[3 * t + 2]]
    }

    pub fn bounds(&self) -> Aabb {
        let mut b = Aabb::EMPTY;
        for p in &self.positions {
            b.grow(Vec3::from(*p));
        }
        b
    }

    pub fn recompute_normals(&mut self) {
        let mut acc = vec![Vec3::ZERO; self.positions.len()];
        for t in 0..self.triangle_count() {
            let [a, b, c] = self.triangle(t);
            let n = (self.position(b) - self.position(a)).cross(self.position(c) - self.position(a));
            acc[a as usize] += n;
            acc[b as usize] += n;
            acc[c as usize] += n;
        }
        self.normals = acc.into_iter().map(|n| n.normalize_or_zero().to_array()).collect();
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.indices.len().is_multiple_of(3) {
            return Err("index count is not a multiple of 3".into());
        }
        let n = self.positions.len() as u32;
        if self.indices.iter().any(|&i| i >= n) {
            return Err("an index points past the last vertex".into());
        }
        if self.positions.iter().flatten().any(|v| !v.is_finite()) {
            return Err("a position is not finite".into());
        }
        Ok(())
    }
}
