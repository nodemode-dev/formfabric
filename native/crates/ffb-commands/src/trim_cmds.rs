mod common;
mod debug;
mod finish;
mod hardware;
mod import;
mod print;
mod quilt;
mod rip;
mod stitch;
mod style;
mod sync;
mod zipper;

#[allow(unused_imports)]
pub use sync::{session_stiffen_bands, session_trim_bindings, stiffen_bands, trim_bindings};

use crate::{Command, Session};

pub(crate) fn after_change(session: &mut Session, command: &str) {
    sync::after_change(session, command);
}

pub(crate) fn follow(session: &mut Session, ops: &[ffb_pattern::sew::Op]) {
    sync::follow(session, ops);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = style::commands();
    all.extend(hardware::commands());
    all.extend(zipper::commands());
    all.extend(stitch::commands());
    all.extend(finish::commands());
    all.extend(print::commands());
    all.extend(rip::commands());
    all.extend(quilt::commands());
    all.extend(import::commands());
    all.extend(debug::commands());
    all
}
