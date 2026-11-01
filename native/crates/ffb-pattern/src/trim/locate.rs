use crate::geom::{closest_on_segment, sub, V2};
use crate::sew::MeshView;

#[derive(Debug, Clone, Copy)]
pub struct TrimMesh<'a> {
    pub positions: &'a [[f64; 2]],
    pub triangles: &'a [[u32; 3]],
    pub boundary: &'a [u32],
    pub boundary_param: &'a [(usize, f64)],
    pub inner_edges: &'a [(u32, Vec<u32>)],
}

impl<'a> TrimMesh<'a> {
    pub fn view(&self) -> MeshView<'a> {
        MeshView { positions: self.positions, boundary: self.boundary, boundary_param: self.boundary_param, inner_edges: self.inner_edges }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Barycentric {
    pub panel: u32,
    pub vertices: [u32; 3],
    pub weights: [f64; 3],
}

fn weights_of(a: V2, b: V2, c: V2, p: V2) -> Option<[f64; 3]> {
    let (v0, v1, v2) = (sub(b, a), sub(c, a), sub(p, a));
    let d = v0[0] * v1[1] - v0[1] * v1[0];
    if d.abs() < 1e-12 {
        return None;
    }
    let w1 = (v2[0] * v1[1] - v2[1] * v1[0]) / d;
    let w2 = (v0[0] * v2[1] - v0[1] * v2[0]) / d;
    Some([1.0 - w1 - w2, w1, w2])
}

pub fn locate(panel: u32, mesh: &TrimMesh, p: V2) -> Option<(Barycentric, f64)> {
    let at = |i: u32| mesh.positions.get(i as usize).copied();
    let mut best: Option<(f64, Barycentric)> = None;
    for tri in mesh.triangles {
        let (Some(a), Some(b), Some(c)) = (at(tri[0]), at(tri[1]), at(tri[2])) else { continue };
        let Some(w) = weights_of(a, b, c, p) else { continue };
        if w.iter().all(|x| *x >= -1e-9) {
            let weights = [w[0].max(0.0), w[1].max(0.0), w[2].max(0.0)];
            let sum: f64 = weights.iter().sum();
            return Some((Barycentric { panel, vertices: *tri, weights: weights.map(|x| x / sum) }, 0.0));
        }
        let corners = [a, b, c];
        for k in 0..3 {
            let (j, l) = (k, (k + 1) % 3);
            let (t, q) = closest_on_segment(p, corners[j], corners[l]);
            let d = crate::geom::dist(p, q);
            if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                let mut weights = [0.0; 3];
                weights[j] = 1.0 - t;
                weights[l] = t;
                best = Some((d, Barycentric { panel, vertices: *tri, weights }));
            }
        }
    }
    best.map(|(d, b)| (b, d))
}
