mod create;
mod input;
mod panels;
mod points;
mod query;
mod schema;
mod store;

use crate::Command;

pub(crate) fn commands() -> Vec<Command> {
    let mut all = query::commands();
    all.extend(create::commands());
    all.extend(points::commands());
    all.extend(panels::commands());
    all
}
