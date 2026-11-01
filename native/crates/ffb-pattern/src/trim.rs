mod bands;
mod bindings;
mod finish;
mod ids;
mod locate;
mod remap;
mod style;
mod zipper;

pub use bands::stiffen_bands;
pub use bindings::{trim_bindings, Fastening, HardwareBinding, TrimBindings};
pub use finish::{build_strip, crease_points, span_path, Strip, StripKind};
pub use ids::{next_finish_id, next_hardware_id, next_print_id, next_stitch_id, next_style_id, next_zipper_id};
pub use locate::{locate, Barycentric, TrimMesh};
pub use remap::{follow, prune};
pub use style::{defaults, flag_param, geometry, merge, param, TrimGeometry};
pub use zipper::{closed_ends, zipper_bindings};
