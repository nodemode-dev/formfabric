mod common;
mod fold;
mod region;
mod state;
mod tape;
mod wind;

use crate::{Command, Session};

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = state::commands();
    all.extend(wind::commands());
    all.extend(region::commands());
    all.extend(fold::commands());
    all.extend(tape::commands());
    all
}
