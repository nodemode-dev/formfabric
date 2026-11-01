#[allow(dead_code)]
mod bands;
mod common;
mod fitshell;
mod fold;
#[allow(dead_code)]
mod fx;
#[allow(dead_code)]
mod geo;
mod region;
mod shell;
mod state;
mod styleline;
mod tape;
mod wind;

#[allow(unused_imports)]
pub(crate) use bands::{styleline_folds, styleline_stiffen};
#[allow(unused_imports)]
pub(crate) use fitshell::{fit_shell_active, fit_shell_collider};
#[allow(unused_imports)]
pub(crate) use fx::signature as fit_and_style_signature;

use crate::{Command, Session};

pub(crate) fn after_change(_session: &mut Session, _command: &str) {}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = state::commands();
    all.extend(wind::commands());
    all.extend(region::commands());
    all.extend(fold::commands());
    all.extend(tape::commands());
    all.extend(styleline::commands());
    all.extend(fitshell::commands());
    all
}
