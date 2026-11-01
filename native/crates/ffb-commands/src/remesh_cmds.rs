mod ops;
mod out;
mod state;

pub use out::{quad_meshes, QuadPanel};
pub use state::RemeshState;

use crate::{Command, Session};

pub(crate) fn after_change(session: &mut Session, _command: &str) {
    let Session { garment, remesh, .. } = session;
    remesh.meshes.retain(|id, _| garment.panels.iter().any(|p| p.id == *id));
}

pub(crate) fn commands() -> Vec<Command> {
    ops::commands()
}
