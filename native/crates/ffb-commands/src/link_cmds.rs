use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use ffb_io::export::{self, Format};
use serde_json::{json, Value};

use crate::output_cmds::{link_scene, LinkScene};
use crate::{args, Command, CommandError, Group, Session, EXTERNAL, READ, SET};

#[cfg(not(target_arch = "wasm32"))]
const PERIOD_MS: u64 = 33;
#[cfg(not(target_arch = "wasm32"))]
const SETTLE_MS: u128 = 250;
const MAX_EVERY: u64 = 100_000;

#[derive(Default)]
struct Status {
    writes: u64,
    last_frame: u64,
    last_bytes: usize,
    last_error: Option<String>,
    stopped: Option<String>,
}

struct Config {
    folder: PathBuf,
    file: String,
    every: u64,
    thickness: bool,
    z_up: bool,
}

impl Config {
    fn path(&self) -> PathBuf {
        self.folder.join(format!("{}.usda", self.file))
    }
}

#[derive(Default)]
struct Shared {
    generation: AtomicU64,
    running: AtomicBool,
    version: AtomicU64,
    scene: Mutex<Option<Arc<LinkScene>>>,
    config: Mutex<Option<Arc<Config>>>,
    status: Mutex<Status>,
}

#[derive(Default)]
pub struct LinkState {
    shared: Arc<Shared>,
}

impl Drop for LinkState {
    fn drop(&mut self) {
        self.shared.running.store(false, Ordering::Release);
        self.shared.generation.fetch_add(1, Ordering::AcqRel);
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

type Fetch = Box<dyn Fn(&LinkScene) -> Vec<[f32; 3]> + Send + Sync>;
type FrameNow = Box<dyn Fn() -> u64 + Send + Sync>;

fn fetcher(s: &Session) -> (Fetch, FrameNow) {
    let core = s.sim.core.clone();
    let again = s.sim.core.clone();
    let fetch: Fetch = Box::new(move |scene: &LinkScene| {
        let c = core.lock().unwrap_or_else(|e| e.into_inner());
        let Some(stepper) = c.stepper.as_ref() else { return scene.raw.clone() };
        let all = stepper.positions();
        let mut out = Vec::with_capacity(scene.raw.len());
        let mut at = 0;
        for (id, n) in scene.ids.iter().zip(&scene.counts) {
            let live = c.slices.iter().find(|sl| sl.id == *id && sl.count == *n).and_then(|sl| all.get(sl.start..sl.start + *n));
            match live {
                Some(part) => out.extend_from_slice(part),
                None => out.extend_from_slice(&scene.raw[at..at + *n]),
            }
            at += *n;
        }
        out
    });
    let frame_now: FrameNow = Box::new(move || again.lock().unwrap_or_else(|e| e.into_inner()).frame);
    (fetch, frame_now)
}

fn write_layer(scene: &LinkScene, config: &Config, raw: &[[f32; 3]]) -> Result<usize, String> {
    let positions = scene.built.frame(raw)?;
    let mut layer = scene.built.scene.clone();
    let mut at = 0;
    for object in layer.objects.iter_mut() {
        let n = object.positions.len();
        let part = positions.get(at..at + n).ok_or("the frame does not match the scene")?;
        object.positions.copy_from_slice(part);
        object.normals = export::vertex_normals(&object.positions, &object.indices);
        at += n;
    }
    let files = export::write(Format::Usd, &layer, None, &scene.options)?;
    let file = files.into_iter().find(|f| f.name.ends_with(".usda")).ok_or("the exporter wrote no usda file")?;
    let target = config.path();
    let text = target.to_str().ok_or("the folder path is not valid text")?;
    args::write_atomic(text, &file.bytes).map_err(|e| e.message)?;
    Ok(file.bytes.len())
}

#[cfg(not(target_arch = "wasm32"))]
fn record(shared: &Shared, frame: u64, result: Result<usize, String>) {
    let mut st = lock(&shared.status);
    match result {
        Ok(bytes) => {
            st.writes += 1;
            st.last_frame = frame;
            st.last_bytes = bytes;
            st.last_error = None;
        }
        Err(e) => st.last_error = Some(e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn follow(shared: Arc<Shared>, mine: u64, fetch: Fetch, frame_now: FrameNow) {
    let started = std::time::Instant::now();
    let mut last_version = shared.version.load(Ordering::Acquire);
    let mut written = frame_now();
    let mut seen = written;
    let mut moved_at = started;
    let mut dirty = false;
    loop {
        if shared.generation.load(Ordering::Acquire) != mine {
            break;
        }
        let version = shared.version.load(Ordering::Acquire);
        let frame = frame_now();
        let (scene, config) = (lock(&shared.scene).clone(), lock(&shared.config).clone());
        if let (Some(scene), Some(config)) = (scene, config) {
            if frame != seen {
                seen = frame;
                moved_at = std::time::Instant::now();
                dirty = frame != written;
            }
            let due = version != last_version || frame < written || frame >= written + config.every || (dirty && moved_at.elapsed().as_millis() >= SETTLE_MS);
            if due {
                let raw = fetch(&scene);
                if shared.generation.load(Ordering::Acquire) != mine {
                    break;
                }
                record(&shared, frame, write_layer(&scene, &config, &raw));
                last_version = version;
                written = frame;
                dirty = false;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(PERIOD_MS));
    }
}

fn halt(shared: &Shared, reason: Option<&str>) {
    shared.generation.fetch_add(1, Ordering::AcqRel);
    if shared.running.swap(false, Ordering::AcqRel) {
        lock(&shared.status).stopped = reason.map(str::to_string);
    }
}

fn report(shared: &Shared) -> Value {
    let config = lock(&shared.config).clone();
    let st = lock(&shared.status);
    json!({
        "running": shared.running.load(Ordering::Acquire),
        "format": "usd",
        "folder": config.as_ref().map(|c| c.folder.to_string_lossy().into_owned()),
        "path": config.as_ref().map(|c| c.path().to_string_lossy().into_owned()),
        "every_frames": config.as_ref().map(|c| c.every),
        "writes": st.writes,
        "last_frame": st.last_frame,
        "last_bytes": st.last_bytes,
        "last_error": st.last_error,
        "stopped": st.stopped
    })
}

pub(crate) fn after_change(s: &mut Session, command: &str) {
    let shared = s.link.shared.clone();
    if !shared.running.load(Ordering::Acquire) {
        return;
    }
    if command.starts_with("file.") {
        halt(&shared, Some("a file command ran"));
        return;
    }
    let Some(config) = lock(&shared.config).clone() else { return };
    match link_scene(s, &config.file, config.thickness, config.z_up) {
        Ok(scene) => {
            *lock(&shared.scene) = Some(Arc::new(scene));
            shared.version.fetch_add(1, Ordering::AcqRel);
        }
        Err(e) => halt(&shared, Some(&format!("the garment can no longer be exported: {}", e.message))),
    }
}

fn start_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "folder": {"type": "string", "description": "Absolute folder the layer is written into; created when missing. Point Unreal's USD Stage at the file inside it."},
            "every_frames": {"type": "integer", "minimum": 1, "maximum": MAX_EVERY, "default": 1, "description": "Write a new layer every this many simulation frames. Edits to the garment write at once, and a stopped simulation writes its last frame after a quarter second."},
            "format": {"type": "string", "enum": ["usd"], "default": "usd", "description": "Layer format. Only usd (usda text) is written."},
            "file": {"type": "string", "pattern": "^[A-Za-z0-9_-]{1,64}$", "default": "garment", "description": "File name without extension; the layer is [folder]/[file].usda."},
            "thickness": {"type": "boolean", "default": false, "description": "Write the back face and side strips set by thickness.set."},
            "axis": {"type": "string", "enum": ["y_up", "z_up"], "default": "y_up", "description": "Up axis stored in the layer. Unreal converts from the stage's upAxis."}
        },
        "required": ["folder"],
        "additionalProperties": false
    })
}

fn start(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    if cfg!(target_arch = "wasm32") {
        return Err(CommandError::new("unsupported", "Live Link writes files and needs the native engine"));
    }
    let folder = args::file_path(a, "folder", None)?;
    let every = match a.get("every_frames") {
        None | Some(Value::Null) => 1,
        Some(v) => v.as_u64().filter(|n| (1..=MAX_EVERY).contains(n)).ok_or_else(|| bad(format!("every_frames must be a whole number from 1 to {MAX_EVERY}")))?,
    };
    if let Some(f) = a.get("format").and_then(Value::as_str) {
        if f != "usd" {
            return Err(bad(format!("format must be usd, got {f}")));
        }
    }
    let file = a.get("file").and_then(Value::as_str).unwrap_or("garment").to_string();
    if file.is_empty() || file.len() > 64 || !file.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(bad("file must be 1 to 64 letters, digits, hyphens or underscores"));
    }
    let thickness = a.get("thickness").and_then(Value::as_bool).unwrap_or(false);
    let z_up = match a.get("axis").and_then(Value::as_str).unwrap_or("y_up") {
        "y_up" => false,
        "z_up" => true,
        other => return Err(bad(format!("axis must be y_up or z_up, got {other}"))),
    };
    std::fs::create_dir_all(&folder).map_err(|e| CommandError::new("io", format!("could not create {folder}: {e}")))?;
    let shared = s.link.shared.clone();
    halt(&shared, None);
    let scene = Arc::new(link_scene(s, &file, thickness, z_up)?);
    let config = Arc::new(Config { folder: PathBuf::from(&folder), file, every, thickness, z_up });
    let (fetch, frame_now) = fetcher(s);
    let frame = frame_now();
    let bytes = write_layer(&scene, &config, &fetch(&scene)).map_err(|e| CommandError::new("export_failed", e))?;
    *lock(&shared.status) = Status { writes: 1, last_frame: frame, last_bytes: bytes, last_error: None, stopped: None };
    *lock(&shared.scene) = Some(scene);
    *lock(&shared.config) = Some(config);
    shared.version.fetch_add(1, Ordering::AcqRel);
    shared.running.store(true, Ordering::Release);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mine = shared.generation.load(Ordering::Acquire);
        let thread_shared = shared.clone();
        if let Err(e) = std::thread::Builder::new().name("ffb-link".into()).spawn(move || follow(thread_shared, mine, fetch, frame_now)) {
            shared.running.store(false, Ordering::Release);
            return Err(CommandError::new("internal", format!("could not start the link thread: {e}")));
        }
    }
    Ok(report(&shared))
}

fn stop(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    let shared = s.link.shared.clone();
    let was = shared.running.load(Ordering::Acquire);
    halt(&shared, Some("link.stop"));
    let mut out = report(&shared);
    out["was_running"] = json!(was);
    Ok(out)
}

fn state(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    Ok(report(&s.link.shared))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "link.start",
            mcp: "link_start",
            group: Group::Export,
            label: "Start Live Link",
            description: "Start the Live Link for Unreal through USD: write the current draped garment as a usda layer into a folder at once, then replace it atomically every N simulation frames while the simulation runs (and after an edit, and when the simulation settles). Unreal's USD Stage reloads the layer when the file changes. The link follows the live positions without touching the document or undo; any file command (new, open) stops it. Starting again replaces the running link.",
            keys: &[],
            hints: EXTERNAL,
            schema: start_schema,
            run: start,
        },
        Command {
            id: "link.stop",
            mcp: "link_stop",
            group: Group::Export,
            label: "Stop Live Link",
            description: "Stop the Live Link. The last layer stays in the folder.",
            keys: &[],
            hints: SET,
            schema: crate::no_args,
            run: stop,
        },
        Command {
            id: "link.state",
            mcp: "link_state",
            group: Group::Export,
            label: "Live Link State",
            description: "Read whether the Live Link is running, the folder and layer path, the write interval, how many layers were written, the frame of the last one, its size, the last error and why it stopped.",
            keys: &[],
            hints: READ,
            schema: crate::no_args,
            run: state,
        },
    ]
}
