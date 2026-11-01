mod library;
mod ops;
mod stream;
mod view;

pub use stream::StreamState;

use crate::{Command, Session};

pub(crate) fn after_change(session: &mut Session, _command: &str) {
    for (changes, forward) in session.history.take_bodies() {
        ffb_body::apply_changes(&mut session.bodies, &changes, forward);
    }
    stream::on_change(session);
}

pub fn publish_all(session: &mut Session) {
    stream::full(session);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = ops::commands();
    all.extend(view::commands());
    all
}
