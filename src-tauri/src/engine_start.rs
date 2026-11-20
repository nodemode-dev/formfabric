use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const APP: &str = "FormFabric";
pub const SHELL_ENV: &str = "FORMFABRIC_SHELL_PID";
pub const ADDRESS_WAIT: Duration = Duration::from_secs(90);
pub const EXIT_WAIT: Duration = Duration::from_secs(1);
pub const BUSY_CODE: i32 = 2;
const DATA_ENV: &str = "FORMFABRIC_DATA_DIR";
const DATA_ARG: &str = "--user-data-dir";
const HOME_NAME: &str = "Formfabric";
const LOG_NAME: &str = "formfabric-engine.log";
const TAIL_LINES: usize = 12;
const LINE_CHARS: usize = 300;

#[derive(Debug, PartialEq)]
pub enum Failure {
    Missing,
    NotStarted(String),
    Stopped(Option<i32>),
    TimedOut,
    Window,
}

#[derive(Default)]
pub struct Tail(VecDeque<String>);

impl Tail {
    pub fn push(&mut self, line: &str) {
        let clean: String = line.chars().filter(|c| !c.is_control()).take(LINE_CHARS).collect();
        if clean.trim().is_empty() {
            return;
        }
        if self.0.len() == TAIL_LINES {
            self.0.pop_front();
        }
        self.0.push_back(clean);
    }

    pub fn lines(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}

pub fn sentence(failure: &Failure) -> String {
    match failure {
        Failure::Missing => format!("{APP} could not start: its engine program is missing from the install. Installing {APP} again puts it back."),
        Failure::NotStarted(reason) => format!("{APP} could not start its engine: {reason}"),
        Failure::Stopped(Some(BUSY_CODE)) => format!("{APP} could not start: another copy of its engine holds its data folder."),
        Failure::Stopped(Some(code)) => format!("{APP} could not start: its engine stopped with code {code} before the window could open."),
        Failure::Stopped(None) => format!("{APP} could not start: its engine stopped before the window could open."),
        Failure::TimedOut => format!("{APP} could not start: it waited {} seconds for its engine to say which address it serves on and that did not happen.", ADDRESS_WAIT.as_secs()),
        Failure::Window => format!("{APP} could not open its window."),
    }
}

pub fn message(failure: &Failure, tail: &[String], log: Option<&Path>) -> String {
    let mut text = sentence(failure);
    if !tail.is_empty() {
        text.push_str("\n\nThe engine's last lines:\n");
        text.push_str(&tail.join("\n"));
    } else if matches!(failure, Failure::Stopped(_) | Failure::TimedOut) {
        text.push_str("\n\nThe engine wrote nothing before that.");
    }
    if let Some(path) = log {
        text.push_str("\n\nThe engine's log: ");
        text.push_str(&path.to_string_lossy());
    }
    text
}

pub fn named_folder(args: &[String]) -> Option<String> {
    args.iter()
        .enumerate()
        .find_map(|(at, arg)| match arg.strip_prefix(DATA_ARG) {
            Some("") => args.get(at + 1).cloned(),
            Some(rest) => rest.strip_prefix('=').map(str::to_string),
            None => None,
        })
        .filter(|value| !value.trim().is_empty())
}

pub fn log_path(named: Option<String>, var: &dyn Fn(&str) -> Option<String>, home: Option<&Path>) -> Option<PathBuf> {
    let raw = named.or_else(|| var(DATA_ENV)).map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let folder = match raw {
        Some(raw) => match (raw.strip_prefix('~'), home) {
            (Some(rest), Some(home)) if rest.is_empty() || rest.starts_with('/') || rest.starts_with('\\') => {
                home.join(rest.trim_start_matches(['/', '\\']))
            }
            _ => PathBuf::from(raw),
        },
        None => home?.join(HOME_NAME),
    };
    Some(folder.join("Logs").join(LOG_NAME))
}

pub fn engine_log() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    log_path(named_folder(&args), &|name: &str| std::env::var(name).ok(), std::env::home_dir().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_keeps_the_last_lines() {
        let mut tail = Tail::default();
        for n in 0..15 {
            tail.push(&format!("line {n}"));
        }
        tail.push("   ");
        tail.push(&"x".repeat(1000));
        let lines = tail.lines();
        assert_eq!(lines.len(), TAIL_LINES);
        assert_eq!(lines[0], "line 4");
        assert_eq!(lines[TAIL_LINES - 1].len(), LINE_CHARS);
    }

    #[test]
    fn every_failure_has_its_sentence_the_last_lines_and_the_log() {
        let log = Path::new("/data/Logs/formfabric-engine.log");
        let said = vec!["formfabric is already running on /data (engine process 7).".to_string()];
        let busy = message(&Failure::Stopped(Some(BUSY_CODE)), &said, Some(log));
        assert!(busy.starts_with("FormFabric could not start: another copy of its engine holds its data folder."));
        assert!(busy.contains("The engine's last lines:\nformfabric is already running on /data (engine process 7)."));
        assert!(busy.ends_with("The engine's log: /data/Logs/formfabric-engine.log"));
        assert!(message(&Failure::Stopped(Some(3)), &[], Some(log)).contains("stopped with code 3"));
        assert!(message(&Failure::Stopped(None), &[], None).ends_with("The engine wrote nothing before that."));
        assert!(message(&Failure::Missing, &[], None).ends_with("puts it back."));
        assert!(sentence(&Failure::NotStarted("Access is denied.".to_string())).ends_with("its engine: Access is denied."));
        assert!(sentence(&Failure::TimedOut).contains("waited 90 seconds for its engine to say which address it serves on"));
        assert_eq!(sentence(&Failure::Window), "FormFabric could not open its window.");
    }

    #[test]
    fn the_log_is_where_the_engine_writes_it() {
        let home = Path::new("/home/u");
        let none = |_: &str| None;
        let set = |name: &str| (name == DATA_ENV).then(|| " ~/x ".to_string());
        let tail = Path::new("Logs").join(LOG_NAME);
        assert_eq!(log_path(None, &none, Some(home)), Some(home.join("Formfabric").join(&tail)));
        assert_eq!(log_path(None, &set, Some(home)), Some(home.join("x").join(&tail)));
        assert_eq!(log_path(Some("/d/y".to_string()), &set, Some(home)), Some(Path::new("/d/y").join(&tail)));
        assert_eq!(log_path(None, &none, None), None);
    }

    #[test]
    fn a_data_folder_named_on_the_command_line_is_read_both_ways() {
        let args = |list: &[&str]| list.iter().map(|item| item.to_string()).collect::<Vec<String>>();
        assert_eq!(named_folder(&args(&["formfabric", "--user-data-dir=/d/x"])), Some("/d/x".to_string()));
        assert_eq!(named_folder(&args(&["formfabric", "--user-data-dir", "/d/y"])), Some("/d/y".to_string()));
        assert_eq!(named_folder(&args(&["formfabric", "--portable"])), None);
    }
}
