mod bind;
mod lengths;
mod pending;
mod remap;
mod side;

pub use side::resolve_side;
pub use bind::{seam_bindings, Binding, MeshView, VertexRef};
pub use lengths::{end_length, end_path, end_valid, is_full, side_length, span_curve, t_after_length, SeamLengths};
pub use pending::{note_reproject, note_reverse, note_split, take_pending, Op};
pub use remap::{apply_pending, prune, split_end};

use crate::{Garment, Seam};

pub fn next_seam_id(garment: &Garment) -> u32 {
    garment.seams.iter().map(|s| s.id).max().unwrap_or(0) + 1
}

pub fn next_tack_id(garment: &Garment) -> u32 {
    garment.tacks.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

pub fn lengths_of(garment: &Garment, seam: &Seam) -> Option<SeamLengths> {
    let a = side_length(garment, &seam.a)?;
    let b = side_length(garment, &seam.b)?;
    Some(SeamLengths { a, b, mismatch: b - seam.ratio * a })
}
