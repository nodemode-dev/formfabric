mod create;
mod derived;
mod grain;
mod inner;
mod input;
mod lengths;
mod notches;
mod panels;
mod points;
mod query;
mod schema;
mod seam;
mod store;

use crate::Command;

pub(crate) fn commands() -> Vec<Command> {
    let mut all = query::commands();
    all.extend(create::commands());
    all.extend(points::commands());
    all.extend(panels::commands());
    all.extend(seam::commands());
    all.extend(notches::commands());
    all.extend(grain::commands());
    all.extend(inner::commands());
    all.extend(lengths::commands());
    all
}
