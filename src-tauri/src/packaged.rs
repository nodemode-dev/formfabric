use std::ffi::OsString;
use std::process::Command;

pub fn launcher() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

const ENGINE: &str = if cfg!(windows) {
    "formfabric-engine.exe"
} else {
    "formfabric-engine"
};

pub fn engine_folder() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    [dir.join("../Resources/engine"), dir.join("engine")]
        .into_iter()
        .map(|folder| folder.join(ENGINE))
        .find(|program| program.is_file())
}

pub fn command_line() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    let first = args.next()?;
    match first.to_str()? {
        "mcp" | "agents" => Some(hand_over(first, args.collect())),
        _ => None,
    }
}

fn hand_over(mode: OsString, rest: Vec<OsString>) -> i32 {
    let Ok(exe) = std::env::current_exe() else {
        eprintln!("formfabric: cannot tell where the program is.");
        return 1;
    };
    let mut command = Command::new(engine_folder().unwrap_or_else(|| exe.with_file_name(ENGINE)));
    command.arg(mode).args(rest);
    if let Some(path) = launcher() {
        command.env("FORMFABRIC_LAUNCHER", path);
    }
    run(command)
}

#[cfg(unix)]
fn run(mut command: Command) -> i32 {
    use std::os::unix::process::CommandExt;
    let err = command.exec();
    eprintln!("formfabric: the engine could not start: {err}");
    1
}

#[cfg(not(unix))]
fn run(mut command: Command) -> i32 {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    match command.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            eprintln!("formfabric: the engine could not start: {err}");
            1
        }
    }
}
