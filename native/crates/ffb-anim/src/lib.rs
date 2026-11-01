pub mod bvh;
pub mod cachefmt;
pub mod clip;
#[cfg(not(target_arch = "wasm32"))]
pub mod fbx;
pub mod gltf;
pub mod interp;
pub mod retarget;

pub use ffb_pattern::anim::{Interp, Key};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
