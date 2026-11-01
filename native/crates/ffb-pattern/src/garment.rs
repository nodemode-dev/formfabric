use ffb_core::IdGen;

use crate::{Garment, Panel, Point};

impl Garment {
    pub fn index_of(&self, id: u32) -> Option<usize> {
        self.panels.iter().position(|p| p.id == id)
    }

    pub fn next_id(&self) -> u32 {
        let used = self.panels.iter().flat_map(|p| std::iter::once(p.id).chain(p.inner.iter().map(|i| i.id)));
        IdGen::starting_after(used).next_id()
    }
}

impl Panel {
    pub fn same_mesh_shape(&self, other: &Panel) -> bool {
        self.outline == other.outline && self.inner == other.inner && self.particle_distance == other.particle_distance && self.mirror == other.mirror
    }

    pub fn path(&self, inner: Option<u32>) -> Option<(&Vec<Point>, bool)> {
        match inner {
            None => Some((&self.outline, true)),
            Some(id) => self.inner.iter().find(|i| i.id == id).map(|i| (&i.points, i.closed)),
        }
    }

    pub fn path_mut(&mut self, inner: Option<u32>) -> Option<(&mut Vec<Point>, bool)> {
        match inner {
            None => Some((&mut self.outline, true)),
            Some(id) => self.inner.iter_mut().find(|i| i.id == id).map(|i| (&mut i.points, i.closed)),
        }
    }
}
