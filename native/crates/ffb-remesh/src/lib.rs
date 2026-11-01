pub mod boundary;
pub mod decimate;
pub mod lift;
pub mod metrics;
pub mod pipeline;
pub mod types;

mod band;
mod cleanup;
mod extract;
mod fields;
mod geom;

pub use decimate::{decimate, DecimateOptions, Decimated};
pub use lift::lift;
pub use metrics::{measure, Metrics};
pub use pipeline::remesh_panel;
pub use types::{Params, QuadMesh, RemeshError, Report, Rim};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
