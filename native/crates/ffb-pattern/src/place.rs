use crate::geom::*;
use crate::path;
use crate::Panel;

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub position: V2,
    pub rotation: f64,
    pub mirror: bool,
}

impl Placement {
    pub fn vec_to_world(&self, v: V2) -> V2 {
        rotate(if self.mirror { [-v[0], v[1]] } else { v }, self.rotation)
    }

    pub fn to_world(&self, p: V2) -> V2 {
        add(self.position, self.vec_to_world(p))
    }

    pub fn vec_to_local(&self, v: V2) -> V2 {
        let r = rotate(v, -self.rotation);
        if self.mirror {
            [-r[0], r[1]]
        } else {
            r
        }
    }

    pub fn to_local(&self, w: V2) -> V2 {
        self.vec_to_local(sub(w, self.position))
    }
}

impl Panel {
    pub fn placement(&self) -> Placement {
        Placement { position: self.position, rotation: self.rotation, mirror: self.mirror }
    }

    pub fn world_outline(&self, tol: f64) -> Vec<V2> {
        let place = self.placement();
        path::flatten(&self.outline, true, tol).into_iter().map(|p| place.to_world(p)).collect()
    }

    pub fn world_bounds(&self) -> Option<Bounds> {
        bounds_of(self.world_outline(0.1))
    }
}
