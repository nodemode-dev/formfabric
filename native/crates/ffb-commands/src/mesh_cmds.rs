use crate::{Command, Session};

#[derive(Debug, Default)]
pub struct MeshState {}

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    Vec::new()
}
