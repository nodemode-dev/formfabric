mod bake;
mod export;
mod game;
mod scene;
mod uv_cmds;

use crate::{Command, Session};

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = export::commands();
    all.extend(uv_cmds::commands());
    all.extend(bake::commands());
    all.extend(game::commands());
    all
}
