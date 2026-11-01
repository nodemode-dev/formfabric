use ffb_sim::{Cloth, Collider, Method, Params, PinPoint, Solver, StepStats};

use crate::GpuSolver;

pub enum AnyStepper {
    Cpu(Box<Solver>),
    Gpu(Box<GpuSolver>),
}

macro_rules! both {
    ($self:expr, $s:ident => $body:expr) => {
        match $self {
            AnyStepper::Cpu($s) => $body,
            AnyStepper::Gpu($s) => $body,
        }
    };
}

impl AnyStepper {
    pub fn cpu(cloth: Cloth, params: Params) -> AnyStepper {
        AnyStepper::Cpu(Box::new(Solver::new(cloth, params)))
    }

    pub fn new(cloth: Cloth, params: Params, want_gpu: bool) -> (AnyStepper, String) {
        if !want_gpu {
            return (AnyStepper::cpu(cloth, params), "cpu".to_string());
        }
        if params.method != Method::Vbd {
            return (AnyStepper::cpu(cloth, params), "cpu (gpu path is vbd only)".to_string());
        }
        if !crate::available() {
            return (AnyStepper::cpu(cloth, params), "cpu (no gpu adapter)".to_string());
        }
        match GpuSolver::new(cloth.clone(), params) {
            Ok(g) => {
                let name = format!("gpu: {}", g.adapter_name());
                (AnyStepper::Gpu(Box::new(g)), name)
            }
            Err(e) => (AnyStepper::cpu(cloth, params), format!("cpu (gpu failed: {e})")),
        }
    }

    pub fn is_gpu(&self) -> bool {
        matches!(self, AnyStepper::Gpu(_))
    }

    pub fn accepts(&self, params: &Params) -> bool {
        !self.is_gpu() || params.method == Method::Vbd
    }

    pub fn set_colliders(&mut self, colliders: Vec<Collider>) {
        both!(self, s => s.set_colliders(colliders))
    }

    pub fn set_pins(&mut self, pins: Vec<PinPoint>) {
        both!(self, s => s.set_pins(pins))
    }

    pub fn set_params(&mut self, params: Params) {
        both!(self, s => s.set_params(params))
    }

    pub fn params(&self) -> Params {
        both!(self, s => s.params())
    }

    pub fn positions(&self) -> &[[f32; 3]] {
        both!(self, s => s.positions())
    }

    pub fn vertex_count(&self) -> usize {
        both!(self, s => s.vertex_count())
    }

    pub fn set_positions(&mut self, p: &[[f32; 3]]) {
        both!(self, s => s.set_positions(p))
    }

    pub fn push_out(&mut self, margin: f32) -> usize {
        both!(self, s => s.push_out(margin))
    }

    pub fn scale_links(&mut self, factor: f64) {
        both!(self, s => s.scale_links(factor))
    }

    pub fn link_count(&self) -> usize {
        both!(self, s => s.link_count())
    }

    pub fn max_link_gap(&self) -> f32 {
        both!(self, s => s.max_link_gap())
    }

    pub fn damp_velocities(&mut self, keep: f64) {
        both!(self, s => s.damp_velocities(keep))
    }

    pub fn grab(&mut self, vertex: u32, target: Option<[f32; 3]>) {
        both!(self, s => s.grab(vertex, target))
    }

    pub fn kinetic_energy(&self) -> f32 {
        both!(self, s => s.kinetic_energy())
    }

    pub fn max_penetration(&self) -> f32 {
        both!(self, s => s.max_penetration())
    }

    pub fn max_stretch(&self) -> f32 {
        both!(self, s => s.max_stretch())
    }

    pub fn max_compression(&self) -> f32 {
        both!(self, s => s.max_compression())
    }

    pub fn set_external_forces(&mut self, forces: &[[f32; 3]]) {
        both!(self, s => s.set_external_forces(forces))
    }

    pub fn step(&mut self) -> StepStats {
        both!(self, s => s.step())
    }

    pub fn self_intersections(&self) -> Option<Vec<(u32, u32)>> {
        match self {
            AnyStepper::Cpu(s) => Some(s.self_intersections()),
            AnyStepper::Gpu(_) => None,
        }
    }

    pub fn untangle(&mut self, steps: u32) -> Option<usize> {
        match self {
            AnyStepper::Cpu(s) => Some(s.untangle(steps)),
            AnyStepper::Gpu(_) => None,
        }
    }

    pub fn try_clone(&self) -> Option<AnyStepper> {
        match self {
            AnyStepper::Cpu(s) => Some(AnyStepper::Cpu(s.clone())),
            AnyStepper::Gpu(_) => None,
        }
    }
}
