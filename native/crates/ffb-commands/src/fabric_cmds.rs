mod colorway;
mod common;
mod library;
mod lab_cmd;
mod package;
mod schema;

use ffb_pattern::Garment;

use crate::Command;

pub(crate) fn new_garment() -> Garment {
    let mut garment = Garment::default();
    common::ensure_default(&mut garment);
    garment
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = library::commands();
    all.extend(lab_cmd::commands());
    all.extend(package::commands());
    all.extend(colorway::commands());
    all
}
