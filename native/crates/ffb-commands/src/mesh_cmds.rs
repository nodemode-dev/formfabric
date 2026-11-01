mod arrange;
mod ops;
mod publish;
mod state;

pub use state::MeshState;

use crate::{Command, Session};

pub(crate) fn after_change(session: &mut Session, _command: &str) {
    publish::on_change(session);
}

pub fn publish_all(session: &mut Session) {
    publish::full(session);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = ops::commands();
    all.extend(arrange::commands());
    all
}
