pub mod colliders;
pub mod model;
mod selfcontact;
pub mod solver;
pub mod stepper;

pub use solver::{GpuError, GpuSolver};
pub use stepper::AnyStepper;

pub fn available() -> bool {
    solver::request_adapter().is_some()
}

pub fn adapter_info() -> String {
    match solver::request_adapter() {
        Some((_, adapter)) => {
            let info = adapter.get_info();
            format!("{} ({:?}, {:?}, driver {})", info.name, info.backend, info.device_type, info.driver)
        }
        None => "no adapter".to_string(),
    }
}
