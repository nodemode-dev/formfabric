mod links;
mod support;
mod tools;

use crate::{Command, Session};

pub(crate) fn after_change(session: &mut Session, command: &str) {
    links::after_change(session, command);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = tools::commands();
    all.extend(links::commands());
    all
}
