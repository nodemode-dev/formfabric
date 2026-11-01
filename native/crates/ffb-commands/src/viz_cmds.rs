mod capture;
mod draw;
mod maps;
mod measure;
mod run;
mod scene;
mod schematic;
mod stage;
mod store;

use crate::{Command, Session};

pub use run::VizState;

pub(crate) fn after_change(session: &mut Session, command: &str) {
    run::after_change(session, command);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = run::commands();
    all.extend(measure::commands());
    all.extend(stage::commands());
    all.extend(schematic::commands());
    all.extend(capture::commands());
    all
}
