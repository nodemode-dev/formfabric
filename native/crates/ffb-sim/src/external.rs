use glam::DVec3;

use crate::solver::Solver;

impl Solver {
    pub fn set_external_forces(&mut self, forces: &[[f32; 3]]) {
        self.outside.clear();
        self.outside.extend(forces.iter().map(|f| DVec3::new(f64::from(f[0]), f64::from(f[1]), f64::from(f[2]))));
    }
}
