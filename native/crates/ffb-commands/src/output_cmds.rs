mod bake;
mod export;
mod game;
mod link_scene;
mod scene;
mod uv_cmds;
mod weld;

pub(crate) use link_scene::{link_scene, LinkScene};

use crate::{Command, Session};

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = export::commands();
    all.extend(uv_cmds::commands());
    all.extend(bake::commands());
    all.extend(game::commands());
    all
}
