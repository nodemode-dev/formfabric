mod auto;
mod common;
mod panel;
mod volumes;

use crate::{Command, Session};

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = volumes::commands();
    all.extend(panel::commands());
    all.extend(auto::commands());
    all
}
