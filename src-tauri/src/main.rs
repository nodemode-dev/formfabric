#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::{Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_updater::UpdaterExt;

mod engine_start;
mod packaged;
mod reveal;
mod text_edit;
mod traffic_lights;
#[cfg(windows)]
mod win_job;
mod window_state;

const PROJECT_EXT: &str = "ffbrc";
const FALLBACK_SIZE: (f64, f64) = (1323.0, 860.0);
const MIN_SIZE: (f64, f64) = (980.0, 640.0);
const SCREEN_FRACTION: f64 = 0.9;

struct Backend(Mutex<Option<CommandChild>>);

#[derive(Default)]
struct Opens {
    ready: bool,
    paths: Vec<String>,
}

struct OpenQueue(Mutex<Opens>);
struct ExitGate(AtomicBool);
struct RestartAfterExit(AtomicBool);
struct FoundUpdate(Mutex<Option<tauri_plugin_updater::Update>>);

#[derive(serde::Serialize)]
struct UpdateReport {
    status: &'static str,
    version: Option<String>,
    message: String,
    notes: Option<String>,
}

fn kill_backend(handle: &tauri::AppHandle) {
    let state = handle.state::<Backend>();
    let child = state.0.lock().ok().and_then(|mut slot| slot.take());
    if let Some(child) = child {
        #[cfg(unix)]
        {
            let pid = child.pid().to_string();
            let signal = |sig: &str| {
                std::process::Command::new("kill")
                    .args([sig, pid.as_str()])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success())
            };
            if signal("-TERM") {
                for _ in 0..60 {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    if !signal("-0") {
                        return;
                    }
                }
            }
        }
        let _ = child.kill();
    }
    #[cfg(windows)]
    win_job::end();
}

#[cfg(windows)]
struct Service(Mutex<Option<(u16, String)>>);

#[cfg(windows)]
fn ask_to_close(port: u16, token: &str) -> bool {
    use std::io::{Read, Write};
    let wait = std::time::Duration::from_secs(2);
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&address, wait) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(wait));
    let _ = stream.set_write_timeout(Some(wait));
    let request = format!(
        "POST /api/shutdown HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nx-ffb-window: {token}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut reply = [0u8; 12];
    stream.read_exact(&mut reply).is_ok()
        && reply.starts_with(b"HTTP/1.")
        && reply.ends_with(b" 200")
}

#[cfg(windows)]
fn wait_exit(pid: u32, millis: u32) {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn WaitForSingleObject(handle: *mut std::ffi::c_void, millis: u32) -> u32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    let handle = unsafe { OpenProcess(0x0010_0000, 0, pid) };
    if !handle.is_null() {
        unsafe {
            WaitForSingleObject(handle, millis);
            CloseHandle(handle);
        }
    }
}

fn close_backend(handle: &tauri::AppHandle) {
    open_exit_gate(handle);
    #[cfg(windows)]
    {
        let service = handle
            .state::<Service>()
            .0
            .lock()
            .ok()
            .and_then(|slot| slot.clone());
        let pid = handle
            .state::<Backend>()
            .0
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(|child| child.pid()));
        if let (Some((port, token)), Some(pid)) = (service, pid) {
            if ask_to_close(port, &token) {
                wait_exit(pid, 4000);
            }
        }
    }
    kill_backend(handle);
}

fn renderer_choice(handle: &tauri::AppHandle, choice: window_state::Choice) {
    open_exit_gate(handle);
    match choice {
        window_state::Choice::Restart => {
            kill_backend(handle);
            handle.restart();
        }
        window_state::Choice::Quit => handle.exit(0),
    }
}

fn close_splash(handle: &tauri::AppHandle) {
    if let Some(splash) = handle.get_webview_window("splash") {
        let _ = splash.close();
    }
}

enum Step {
    Event(CommandEvent),
    Ended,
    Late,
}

fn start_limit(late: &tauri::async_runtime::Sender<Step>, limit: std::time::Duration) {
    let late = late.clone();
    std::thread::spawn(move || {
        std::thread::sleep(limit);
        let _ = late.blocking_send(Step::Late);
    });
}

fn engine_failed(handle: &tauri::AppHandle, failure: engine_start::Failure, tail: &engine_start::Tail) {
    let text = engine_start::message(&failure, &tail.lines(), engine_start::engine_log().as_deref());
    eprintln!("formfabric: {text}");
    let _ = handle
        .dialog()
        .message(text)
        .title(engine_start::APP)
        .kind(MessageDialogKind::Error)
        .blocking_show();
    close_splash(handle);
    kill_backend(handle);
    handle.exit(1);
}

fn engine_stopped(handle: &tauri::AppHandle) {
    if handle.state::<ExitGate>().0.load(Ordering::SeqCst) {
        return;
    }
    let ours = handle
        .state::<Backend>()
        .0
        .lock()
        .map(|mut slot| slot.take().is_none())
        .unwrap_or(true);
    if ours {
        return;
    }
    open_exit_gate(handle);
    let restart = handle
        .dialog()
        .message("The formfabric engine has stopped, so this window can no longer work. After a restart formfabric offers to recover the last autosave and the changes made after it.")
        .title("FormFabric")
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Restart".into(),
            "Quit".into(),
        ))
        .blocking_show();
    if restart {
        handle.restart();
    }
    handle.exit(1);
}

fn updater_configured(handle: &tauri::AppHandle) -> bool {
    handle
        .config()
        .plugins
        .0
        .get("updater")
        .and_then(|section| section.get("pubkey"))
        .and_then(|key| key.as_str())
        .is_some_and(|key| !key.is_empty())
}

fn open_exit_gate(handle: &tauri::AppHandle) {
    handle.state::<ExitGate>().0.store(true, Ordering::SeqCst);
}

fn ask_page_to_quit(handle: &tauri::AppHandle) -> bool {
    if handle.state::<ExitGate>().0.load(Ordering::SeqCst) || window_state::loop_page_shown() {
        return false;
    }
    let ready = handle
        .state::<OpenQueue>()
        .0
        .lock()
        .map(|opens| opens.ready)
        .unwrap_or(false);
    if !ready || handle.get_webview_window("main").is_none() {
        return false;
    }
    handle.emit("formfabric://exit-requested", ()).is_ok()
}

fn show_main_window(handle: &tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn open_paths(handle: &tauri::AppHandle, paths: Vec<String>) {
    if paths.is_empty() {
        return;
    }
    let send = match handle.state::<OpenQueue>().0.lock() {
        Ok(mut opens) => {
            if opens.ready {
                Some(paths)
            } else {
                opens.paths.extend(paths);
                None
            }
        }
        Err(_) => None,
    };
    if let Some(paths) = send {
        let _ = handle.emit("formfabric://open-paths", paths);
        show_main_window(handle);
    }
}

fn is_project(path: &std::path::Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(PROJECT_EXT))
}

fn paths_from_args(args: &[String], cwd: Option<&str>) -> Vec<String> {
    args.iter()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .filter_map(|arg| {
            let path = std::path::PathBuf::from(arg);
            let full = match cwd {
                Some(dir) if path.is_relative() => std::path::Path::new(dir).join(path),
                _ => path,
            };
            is_project(&full).then(|| full.to_string_lossy().into_owned())
        })
        .collect()
}

#[tauri::command]
fn take_pending_opens(queue: tauri::State<OpenQueue>) -> Vec<String> {
    match queue.0.lock() {
        Ok(mut opens) => {
            opens.ready = true;
            std::mem::take(&mut opens.paths)
        }
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
fn confirm_exit(app: tauri::AppHandle, cancel: Option<bool>) {
    let restart = app
        .state::<RestartAfterExit>()
        .0
        .swap(false, Ordering::SeqCst);
    if cancel == Some(true) {
        return;
    }
    open_exit_gate(&app);
    if restart {
        close_backend(&app);
        app.restart();
    }
    app.exit(0);
}

fn update_report(status: &'static str, version: Option<String>, message: String) -> UpdateReport {
    UpdateReport {
        status,
        version,
        message,
        notes: None,
    }
}

#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> UpdateReport {
    if !updater_configured(&app) {
        return update_report(
            "unconfigured",
            None,
            "This build has no update key configured, so it cannot check for updates.".into(),
        );
    }
    let updater = match app.updater() {
        Ok(updater) => updater,
        Err(err) => {
            return update_report(
                "failed",
                None,
                format!("Could not start the update check: {err}"),
            )
        }
    };
    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone().filter(|text| !text.trim().is_empty());
            if let Ok(mut slot) = app.state::<FoundUpdate>().0.lock() {
                *slot = Some(update);
            }
            let mut report = update_report(
                "available",
                Some(version.clone()),
                format!("Version {version} is available."),
            );
            report.notes = notes;
            report
        }
        Ok(None) => update_report("current", None, "formfabric is up to date.".into()),
        Err(err) => update_report(
            "failed",
            None,
            format!("Could not check for updates: {err}"),
        ),
    }
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let update = app
        .state::<FoundUpdate>()
        .0
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
        .ok_or_else(|| "No update is waiting; check for updates again.".to_string())?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|err| format!("Could not install the update: {err}"))?;
    open_exit_gate(&app);
    close_backend(&app);
    app.restart()
}

fn user_data_dir_arg(args: &[String]) -> Option<String> {
    args.iter()
        .enumerate()
        .find_map(|(i, a)| {
            a.strip_prefix("--user-data-dir=")
                .map(str::to_string)
                .or_else(|| {
                    (a == "--user-data-dir")
                        .then(|| args.get(i + 1).cloned())
                        .flatten()
                })
        })
        .filter(|dir| !dir.is_empty())
}

#[cfg(target_os = "linux")]
fn data_folder() -> std::path::PathBuf {
    use std::path::PathBuf;
    let args: Vec<String> = std::env::args().collect();
    let var = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
    let home = || var("HOME").map(PathBuf::from).unwrap_or_default();
    let folder = match user_data_dir_arg(&args).or_else(|| var("FORMFABRIC_DATA_DIR")) {
        Some(dir) => match dir.trim().strip_prefix("~/") {
            Some(rest) => home().join(rest),
            None => PathBuf::from(dir.trim()),
        },
        None => home().join("Formfabric"),
    };
    let absolute = if folder.is_absolute() {
        folder
    } else {
        std::env::current_dir().unwrap_or_default().join(folder)
    };
    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

fn single_instance() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let plugin =
        tauri_plugin_single_instance::Builder::<tauri::Wry>::new().callback(|app, args, cwd| {
            open_paths(app, paths_from_args(&args, Some(&cwd)));
            show_main_window(app);
        });
    #[cfg(target_os = "linux")]
    let plugin = {
        let hash = data_folder()
            .to_string_lossy()
            .bytes()
            .fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
            });
        plugin.dbus_id(format!("ai.nodemode.formfabric.d{hash:016x}"))
    };
    plugin.build()
}

fn window_script(url: &str, key: Option<&str>) -> String {
    let url = serde_json::to_string(url).unwrap_or_default();
    let key = match key {
        Some(value) => format!("window.__FFB_WINDOW__={};", serde_json::to_string(value).unwrap_or_default()),
        None => String::new(),
    };
    format!("(function(){{if(location.origin!=={url})return;window.__FFB_ENGINE__={url};{key}}})();")
}

fn main() {
    reveal::mark_launch();
    if let Some(code) = packaged::command_line() {
        std::process::exit(code);
    }
    let builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let builder = builder.on_web_content_process_terminate(window_state::on_renderer_stopped);
    let app = builder
        .plugin(single_instance())
        .manage(OpenQueue(Mutex::new(Opens::default())))
        .manage(ExitGate(AtomicBool::new(false)))
        .manage(RestartAfterExit(AtomicBool::new(false)))
        .manage(FoundUpdate(Mutex::new(None)))
        .manage(window_state::Memory::default())
        .on_window_event(|window, event| {
            window_state::track(window, event);
            if window.label() == "main"
                && matches!(
                    event,
                    WindowEvent::Resized(_)
                        | WindowEvent::Focused(_)
                        | WindowEvent::ThemeChanged(_)
                        | WindowEvent::ScaleFactorChanged { .. }
                )
            {
                traffic_lights::place(window);
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" && ask_page_to_quit(window.app_handle()) {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            take_pending_opens,
            confirm_exit,
            check_for_update,
            install_update,
            window_state::ui_zoom,
            traffic_lights::set_window_title,
            reveal::app_ready,
            text_edit::text_edit
        ])
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Backend(Mutex::new(None)))
        .setup(|app| {
            let handle = app.handle().clone();
            reveal::round_splash(&handle);
            #[cfg(windows)]
            app.manage(Service(Mutex::new(None)));
            let launch_args: Vec<String> = std::env::args().collect();
            open_paths(&handle, paths_from_args(&launch_args, None));
            let engine = match packaged::engine_folder() {
                Some(program) => Ok(app.shell().command(program)),
                None => app
                    .shell()
                    .sidecar("formfabric-engine")
                    .map_err(|_| engine_start::Failure::Missing),
            };
            let started = engine.and_then(|engine| {
                let mut sidecar = engine
                    .env("FFB_PORT", "0")
                    .env(engine_start::KEY_STDOUT_ENV, "1")
                    .env(engine_start::SHELL_ENV, std::process::id().to_string());
                if let Some(path) = packaged::launcher() {
                    sidecar = sidecar.env("FORMFABRIC_LAUNCHER", path);
                }
                if let Some(dir) = user_data_dir_arg(&launch_args) {
                    sidecar = sidecar.env("FORMFABRIC_DATA_DIR", dir);
                }
                sidecar
                    .spawn()
                    .map_err(|err| engine_start::Failure::NotStarted(err.to_string()))
            });
            let (mut rx, child) = match started {
                Ok(pair) => pair,
                Err(failure) => {
                    std::thread::spawn(move || {
                        engine_failed(&handle, failure, &engine_start::Tail::default())
                    });
                    return Ok(());
                }
            };
            #[cfg(windows)]
            win_job::hold_pid(child.pid());
            *app.state::<Backend>().0.lock().unwrap() = Some(child);

            tauri::async_runtime::spawn(async move {
                let pattern = regex::Regex::new(r"http://127\.0\.0\.1:(\d+)").unwrap();
                let key_pattern = regex::Regex::new(r"formfabric-window-key: ([A-Za-z0-9_-]{32,})").unwrap();
                let mut token: Option<String> = None;
                let (tx, mut events) = tauri::async_runtime::channel::<Step>(64);
                let late = tx.clone();
                tauri::async_runtime::spawn(async move {
                    while let Some(event) = rx.recv().await {
                        if tx.send(Step::Event(event)).await.is_err() {
                            break;
                        }
                    }
                    let _ = tx.send(Step::Ended).await;
                });
                start_limit(&late, engine_start::ADDRESS_WAIT);
                let mut port: Option<u16> = None;
                let mut stopped = false;
                let mut tail = engine_start::Tail::default();
                let mut failure = engine_start::Failure::Stopped(None);
                while let Some(step) = events.recv().await {
                    let text = match step {
                        Step::Event(CommandEvent::Stdout(bytes) | CommandEvent::Stderr(bytes)) => {
                            String::from_utf8_lossy(&bytes).into_owned()
                        }
                        Step::Event(CommandEvent::Terminated(ended)) => {
                            failure = engine_start::Failure::Stopped(ended.code);
                            stopped = true;
                            start_limit(&late, engine_start::EXIT_WAIT);
                            continue;
                        }
                        Step::Event(_) => continue,
                        Step::Ended => break,
                        Step::Late if stopped => break,
                        Step::Late => {
                            failure = engine_start::Failure::TimedOut;
                            break;
                        }
                    };
                    for line in text.lines() {
                        tail.push(line);
                    }
                    if token.is_none() {
                        token = key_pattern.captures(&text).map(|found| found[1].to_string());
                    }
                    port = pattern
                        .captures(&text)
                        .and_then(|found| found[1].parse::<u16>().ok());
                    if port.is_some() && !stopped {
                        break;
                    }
                    port = None;
                }
                drop(late);
                match port {
                    Some(port) => {
                        #[cfg(windows)]
                        {
                            if let Ok(mut slot) = handle.state::<Service>().0.lock() {
                                *slot = token.clone().map(|key| (port, key));
                            }
                        }
                        let url = format!("http://127.0.0.1:{port}");
                        let place = window_state::load(&handle);
                        let (width, height) = if place.width > 0.0 {
                            (place.width, place.height)
                        } else {
                            handle
                                .primary_monitor()
                                .ok()
                                .flatten()
                                .map(|monitor| {
                                    let logical =
                                        monitor.size().to_logical::<f64>(monitor.scale_factor());
                                    (
                                        logical.width * SCREEN_FRACTION,
                                        logical.height * SCREEN_FRACTION,
                                    )
                                })
                                .unwrap_or(FALLBACK_SIZE)
                        };
                        let builder = WebviewWindowBuilder::new(
                            &handle,
                            "main",
                            WebviewUrl::External(url.parse().unwrap()),
                        )
                        .title("FormFabric")
                        .visible(reveal::BUILD_VISIBLE)
                        .background_color(reveal::BACKGROUND)
                        .inner_size(width, height)
                        .min_inner_size(MIN_SIZE.0, MIN_SIZE.1)
                        .disable_drag_drop_handler()
                        .initialization_script(window_script(&url, token.as_deref()))
                        .on_navigation({
                            let handle = handle.clone();
                            move |url| match window_state::shell_choice(url) {
                                Some(choice) => {
                                    renderer_choice(&handle, choice);
                                    false
                                }
                                None => true,
                            }
                        });
                        let builder = if place.width > 0.0 {
                            builder.position(place.x, place.y)
                        } else {
                            builder.center()
                        };
                        #[cfg(target_os = "macos")]
                        let builder = builder
                            .title_bar_style(tauri::TitleBarStyle::Overlay)
                            .hidden_title(true);
                        let builder = builder.on_page_load(|window, payload| {
                            if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished) {
                                traffic_lights::place(&window.as_ref().window());
                            }
                        });
                        if let Ok(window) = builder.build() {
                            reveal::conceal(&window);
                            traffic_lights::place(&window.as_ref().window());
                            window_state::restore(&window, &place);
                            window_state::watch_renderer(&window);
                        } else {
                            return engine_failed(
                                &handle,
                                engine_start::Failure::Window,
                                &engine_start::Tail::default(),
                            );
                        }
                    }
                    None => return engine_failed(&handle, failure, &tail),
                }
                while let Some(step) = events.recv().await {
                    if matches!(step, Step::Event(CommandEvent::Terminated(_)) | Step::Ended) {
                        break;
                    }
                }
                engine_stopped(&handle);
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building the formfabric app");

    app.run(|handle, event| match event {
        RunEvent::Exit => {
            window_state::flush(handle);
            if window_state::loop_page_shown() {
                kill_backend(handle);
            } else {
                close_backend(handle);
            }
        }
        RunEvent::ExitRequested { api, .. } => {
            if ask_page_to_quit(handle) {
                api.prevent_exit();
            }
        }
        #[cfg(target_os = "macos")]
        RunEvent::Opened { urls } => {
            let paths = urls
                .iter()
                .filter_map(|url| url.to_file_path().ok())
                .filter(|path| is_project(path))
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
            open_paths(handle, paths);
            show_main_window(handle);
        }
        _ => {}
    });
}
