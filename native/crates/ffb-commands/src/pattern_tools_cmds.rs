use crate::{Command, Session};

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    Vec::new()
}
