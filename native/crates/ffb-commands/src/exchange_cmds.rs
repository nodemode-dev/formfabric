mod export;
mod import;

use crate::Command;

pub(crate) fn commands() -> Vec<Command> {
    vec![import::command(), export::command()]
}
