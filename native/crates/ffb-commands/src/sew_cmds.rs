mod change;
mod common;
mod create;
mod pleat;
mod query;
mod sync;
mod tack;

#[allow(unused_imports)]
pub use sync::{seam_bindings, session_bindings};

use crate::{Command, Session};

pub(crate) fn after_change(session: &mut Session, command: &str) {
    sync::after_change(session, command);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = create::commands();
    all.extend(change::commands());
    all.extend(query::commands());
    all.extend(tack::commands());
    all.extend(pleat::commands());
    all
}
