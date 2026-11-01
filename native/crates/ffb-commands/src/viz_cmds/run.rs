use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{json, Value};

use super::maps::{self, Component, Kind};
use super::scene::{self, Scene};
use super::store;
use crate::{args, Command, CommandError, FrameSink, Group, Session, READ, SET, WRITE};

const MAX_SNAPSHOTS: usize = 32;
const PERIOD_MS: u64 = 33;

pub(crate) struct Snapshot {
    pub name: String,
    pub panels: Vec<(u32, Vec<[f32; 3]>)>,
}

#[derive(Clone)]
pub(crate) enum Source {
    Live,
    Placed,
    Snap(Arc<Snapshot>),
}

#[derive(Clone, Default)]
pub(crate) enum Job {
    #[default]
    None,
    Map { kind: Kind, component: Component, range: [f32; 2] },
    Compare { a: Source, b: Source, range: [f32; 2] },
}

pub(crate) struct Frames {
    pub frame: u64,
    pub live: Vec<Vec<[f32; 3]>>,
    pub placed: Vec<Vec<[f32; 3]>>,
}

pub(crate) type Fetch = Box<dyn Fn(&Scene) -> Frames + Send + Sync>;

#[derive(Default)]
struct Shared {
    stop: AtomicBool,
    alive: AtomicBool,
    serial: AtomicU64,
    job: Mutex<Job>,
    scene: Mutex<Option<Arc<Scene>>>,
    shown: Mutex<Vec<u32>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub struct VizState {
    shared: Arc<Shared>,
    version: u64,
    snapshots: Vec<Arc<Snapshot>>,
}

impl Default for VizState {
    fn default() -> Self {
        VizState { shared: Arc::new(Shared::default()), version: 0, snapshots: Vec::new() }
    }
}

impl Drop for VizState {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
    }
}

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn fetcher(s: &Session) -> Fetch {
    let core = s.sim.core.clone();
    Box::new(move |scene: &Scene| {
        let c = core.lock().unwrap_or_else(|e| e.into_inner());
        let pull = |all: &[[f32; 3]], panel: &scene::PanelScene| -> Option<Vec<[f32; 3]>> {
            let n = panel.rest.len();
            let slice = c.slices.iter().find(|sl| sl.id == panel.id && sl.count == n)?;
            all.get(slice.start..slice.start + n).map(<[_]>::to_vec)
        };
        let live = scene.panels.iter().map(|p| c.stepper.as_ref().and_then(|st| pull(st.positions(), p)).unwrap_or_else(|| p.positions.clone())).collect();
        let placed = scene.panels.iter().map(|p| pull(&c.base.positions, p).unwrap_or_else(|| p.positions.clone())).collect();
        Frames { frame: c.frame, live, placed }
    })
}

fn positions_of(source: &Source, scene: &Scene, frames: &Frames) -> Vec<Vec<[f32; 3]>> {
    match source {
        Source::Live => frames.live.clone(),
        Source::Placed => frames.placed.clone(),
        Source::Snap(snap) => scene.panels.iter().map(|p| snap.panels.iter().find(|(id, _)| *id == p.id).map(|(_, x)| x.clone()).unwrap_or_default()).collect(),
    }
}

pub(crate) struct Evaluated {
    pub map: u32,
    pub component: u32,
    pub values: Vec<Vec<f32>>,
}

fn evaluate(job: &Job, scene: &Scene, frames: &Frames) -> Option<Evaluated> {
    match job {
        Job::None => None,
        Job::Map { kind, component, .. } => Some(Evaluated { map: kind.id(), component: component.id(), values: maps::compute(scene, &frames.live, *kind, *component) }),
        Job::Compare { a, b, .. } => Some(Evaluated { map: Kind::Compare.id(), component: 0, values: maps::distance(&positions_of(a, scene, frames), &positions_of(b, scene, frames)) }),
    }
}

fn range_of(job: &Job) -> [f32; 2] {
    match job {
        Job::Map { range, .. } | Job::Compare { range, .. } => *range,
        Job::None => [0.0, 1.0],
    }
}

fn emit(sink: &dyn FrameSink, shared: &Shared, scene: &Scene, job: &Job, done: &Evaluated) {
    let range = range_of(job);
    let mut shown = lock(&shared.shown);
    for (panel, values) in scene.panels.iter().zip(&done.values) {
        sink.scalars(panel.id, done.map, done.component, range[0], range[1], values);
        if !shown.contains(&panel.id) {
            shown.push(panel.id);
        }
    }
}

fn clear(sink: &dyn FrameSink, shared: &Shared) {
    for id in std::mem::take(&mut *lock(&shared.shown)) {
        sink.scalars(id, 0, 0, 0.0, 0.0, &[]);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn follow(shared: Arc<Shared>, sink: Arc<dyn FrameSink>, fetch: Fetch) {
    let mut last: Option<(u64, u64, u64)> = None;
    loop {
        if shared.stop.load(Ordering::Acquire) {
            break;
        }
        let job = lock(&shared.job).clone();
        if matches!(job, Job::None) {
            shared.alive.store(false, Ordering::Release);
            let still_idle = matches!(*lock(&shared.job), Job::None);
            if still_idle || shared.alive.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
                break;
            }
            continue;
        }
        let scene = lock(&shared.scene).clone();
        if let Some(scene) = scene {
            let frames = fetch(&scene);
            let stamp = (frames.frame, scene.version, shared.serial.load(Ordering::Acquire));
            if last != Some(stamp) {
                last = Some(stamp);
                if let Some(done) = evaluate(&job, &scene, &frames) {
                    emit(sink.as_ref(), &shared, &scene, &job, &done);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(PERIOD_MS));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_thread(s: &Session) {
    let Some(sink) = s.sink.clone() else { return };
    let shared = s.viz.shared.clone();
    if shared.alive.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
        return;
    }
    let fetch = fetcher(s);
    let again = shared.clone();
    if std::thread::Builder::new().name("ffb-viz-map".into()).spawn(move || follow(shared, sink, fetch)).is_err() {
        again.alive.store(false, Ordering::Release);
    }
}

#[cfg(target_arch = "wasm32")]
fn ensure_thread(_s: &Session) {}

fn next_scene(s: &mut Session) -> Result<Arc<Scene>, CommandError> {
    s.viz.version += 1;
    let scene = Arc::new(scene::build(s, s.viz.version)?);
    *lock(&s.viz.shared.scene) = Some(scene.clone());
    Ok(scene)
}

fn activate(s: &mut Session, scene: &Scene, job: Job) -> Option<Evaluated> {
    let fetch = fetcher(s);
    let frames = fetch(scene);
    let done = evaluate(&job, scene, &frames);
    let shared = s.viz.shared.clone();
    *lock(&shared.job) = job.clone();
    shared.serial.fetch_add(1, Ordering::AcqRel);
    if let (Some(sink), Some(done)) = (s.sink.as_ref(), done.as_ref()) {
        emit(sink.as_ref(), &shared, scene, &job, done);
    }
    ensure_thread(s);
    done
}

fn deactivate(s: &mut Session) {
    let shared = s.viz.shared.clone();
    *lock(&shared.job) = Job::None;
    shared.serial.fetch_add(1, Ordering::AcqRel);
    if let Some(sink) = s.sink.as_ref() {
        clear(sink.as_ref(), &shared);
    }
}

const QUIET: [&str; 7] = ["light.", "stage.", "view.", "fur.", "measure3d.", "compare.", "map."];

pub(crate) fn after_change(s: &mut Session, command: &str) {
    if QUIET.iter().any(|p| command.starts_with(p)) || matches!(*lock(&s.viz.shared.job), Job::None) {
        return;
    }
    if next_scene(s).is_err() {
        deactivate(s);
    }
}

fn kind_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": ["none", "strain", "stress", "pressure", "fit", "contact", "thickness"], "description": "Map to show. none clears the overlay."},
            "component": {"type": "string", "enum": ["max", "min", "weft", "warp", "shear"], "default": "max", "description": "Strain and stress component: max or min principal, along the weft, along the warp, or shear."},
            "range": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2, "description": "Value range quantised into the frames, [low, high]. Default: taken from the data when the map is shown and then held."}
        },
        "required": ["kind"],
        "additionalProperties": false
    })
}

fn stats_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": ["strain", "stress", "pressure", "fit", "contact", "thickness"]},
            "component": {"type": "string", "enum": ["max", "min", "weft", "warp", "shear"], "default": "max"}
        },
        "required": ["kind"],
        "additionalProperties": false
    })
}

fn parse_kind(a: &Value) -> Result<(Kind, Component), CommandError> {
    let name = args::string(a, "kind")?;
    let kind = Kind::parse(name).ok_or_else(|| bad(format!("unknown map {name}")))?;
    let component = match a.get("component").and_then(Value::as_str) {
        Some(c) => Component::parse(c).ok_or_else(|| bad(format!("unknown component {c}")))?,
        None => Component::Max,
    };
    Ok((kind, component))
}

fn parse_range(a: &Value) -> Result<Option<[f32; 2]>, CommandError> {
    let Some(list) = a.get("range").and_then(Value::as_array) else { return Ok(None) };
    let v: Vec<f64> = list.iter().filter_map(Value::as_f64).collect();
    if v.len() != 2 || !v.iter().all(|x| x.is_finite()) || v[0] >= v[1] {
        return Err(bad("range must be [low, high] with low below high"));
    }
    Ok(Some([v[0] as f32, v[1] as f32]))
}

fn set_map_name(s: &mut Session, name: Option<&str>) {
    s.garment.viz.map = name.map(str::to_string);
}

fn show(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    if args::string(a, "kind")? == "none" {
        deactivate(s);
        set_map_name(s, None);
        return Ok(json!({"kind": "none"}));
    }
    let (kind, component) = parse_kind(a)?;
    let given = parse_range(a)?;
    let scene = next_scene(s)?;
    let frames = fetcher(s)(&scene);
    let values = maps::compute(&scene, &frames.live, kind, component);
    let range = given.or(kind.fixed_range()).unwrap_or_else(|| maps::auto_range(&values, kind));
    let streaming = s.sink.is_some();
    activate(s, &scene, Job::Map { kind, component, range });
    set_map_name(s, Some(kind.name()));
    Ok(json!({
        "kind": kind.name(),
        "map_id": kind.id(),
        "component": component.name(),
        "unit": kind.unit(),
        "range": range,
        "panels": scene.panels.len(),
        "vertices": scene.panels.iter().map(|p| p.rest.len()).sum::<usize>(),
        "streaming": streaming,
        "stats": maps::stats(&scene, &values)
    }))
}

fn stats(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (kind, component) = parse_kind(a)?;
    let scene = next_scene(s)?;
    let frames = fetcher(s)(&scene);
    let values = maps::compute(&scene, &frames.live, kind, component);
    let mut out = maps::stats(&scene, &values);
    out["kind"] = json!(kind.name());
    out["component"] = json!(component.name());
    out["unit"] = json!(kind.unit());
    out["frame"] = json!(frames.frame);
    Ok(out)
}

fn name_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"name": {"type": "string", "minLength": 1, "maxLength": 80, "description": "Snapshot name; saving over an existing name replaces it."}},
        "required": ["name"],
        "additionalProperties": false
    })
}

fn snapshot(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let name = args::string(a, "name")?.trim().to_string();
    args::clean_name("name", &name)?;
    if name.is_empty() || name == "live" || name == "placed" {
        return Err(bad("name must be non-empty and not live or placed"));
    }
    let scene = next_scene(s)?;
    let frames = fetcher(s)(&scene);
    let panels: Vec<(u32, Vec<[f32; 3]>)> = scene.panels.iter().zip(frames.live).map(|(p, x)| (p.id, x)).collect();
    let vertices = panels.iter().map(|(_, x)| x.len()).sum::<usize>();
    s.viz.snapshots.retain(|x| x.name != name);
    if s.viz.snapshots.len() >= MAX_SNAPSHOTS {
        return Err(bad(format!("at most {MAX_SNAPSHOTS} snapshots are kept; save over an existing name or restart")));
    }
    s.viz.snapshots.push(Arc::new(Snapshot { name: name.clone(), panels }));
    Ok(json!({"name": name, "panels": scene.panels.len(), "vertices": vertices, "snapshots": s.viz.snapshots.iter().map(|x| x.name.clone()).collect::<Vec<_>>()}))
}

fn set_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "a": {"type": "string", "description": "First state: live (current positions), placed (the start of the simulation), or a snapshot name."},
            "b": {"type": "string", "description": "Second state, same choices."},
            "range": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}
        },
        "required": ["a", "b"],
        "additionalProperties": false
    })
}

fn source(s: &Session, name: &str) -> Result<Source, CommandError> {
    match name {
        "live" => Ok(Source::Live),
        "placed" => Ok(Source::Placed),
        other => s
            .viz
            .snapshots
            .iter()
            .find(|x| x.name == other)
            .map(|x| Source::Snap(x.clone()))
            .ok_or_else(|| CommandError::new("not_found", format!("no snapshot named {other}; save one with compare.snapshot"))),
    }
}

fn compare_set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (an, bn) = (args::string(a, "a")?.to_string(), args::string(a, "b")?.to_string());
    let (sa, sb) = (source(s, &an)?, source(s, &bn)?);
    let given = parse_range(a)?;
    let scene = next_scene(s)?;
    let frames = fetcher(s)(&scene);
    let values = maps::distance(&positions_of(&sa, &scene, &frames), &positions_of(&sb, &scene, &frames));
    let range = given.unwrap_or_else(|| maps::auto_range(&values, Kind::Compare));
    activate(s, &scene, Job::Compare { a: sa, b: sb, range });
    store::set_compare(&mut s.garment, Some((&an, &bn)));
    set_map_name(s, Some("compare"));
    Ok(json!({"a": an, "b": bn, "unit": "mm", "map_id": Kind::Compare.id(), "range": range, "stats": maps::stats(&scene, &values)}))
}

fn compare_clear(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    deactivate(s);
    store::set_compare(&mut s.garment, None);
    set_map_name(s, None);
    Ok(json!({"cleared": true}))
}

fn compare_get(s: &mut Session, _: &Value) -> Result<Value, CommandError> {
    Ok(json!({
        "a": s.garment.viz.compare.get("a").cloned().unwrap_or(Value::Null),
        "b": s.garment.viz.compare.get("b").cloned().unwrap_or(Value::Null),
        "snapshots": s.viz.snapshots.iter().map(|x| x.name.clone()).collect::<Vec<_>>(),
        "active": !matches!(*lock(&s.viz.shared.job), Job::None)
    }))
}

fn no_args() -> Value {
    crate::no_args()
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "map.show",
            mcp: "map_show",
            group: Group::View,
            label: "Show Fit Map",
            description: "Show a per-vertex analysis map on the draped garment: strain (principal stretch in percent against the rest shape), stress (kPa from the fabric model), pressure (kPa the garment exerts on the body, Laplace estimate from tension and curvature), fit (signed distance to the body in mm), contact (1 in contact) or thickness (mm). The values stream as frame kind 3 per panel (object = panel id), every frame while the simulation runs and once when it is static. none clears the map. Not an undo step.",
            keys: &[],
            hints: SET,
            schema: kind_schema,
            run: show,
        },
        Command {
            id: "map.stats",
            mcp: "map_stats",
            group: Group::View,
            label: "Map Statistics",
            description: "Compute a map for the current positions and return min, max, mean, percentiles and the same per panel. Does not stream or change the display.",
            keys: &[],
            hints: READ,
            schema: stats_schema,
            run: stats,
        },
        Command {
            id: "compare.snapshot",
            mcp: "compare_snapshot",
            group: Group::View,
            label: "Save Pose Snapshot",
            description: "Keep the current draped positions in session memory under a name for compare.set. Snapshots are not saved in the document.",
            keys: &[],
            hints: WRITE,
            schema: name_schema,
            run: snapshot,
        },
        Command {
            id: "compare.set",
            mcp: "compare_set",
            group: Group::View,
            label: "Compare States",
            description: "Stream the per-vertex distance in mm between two states (live, placed or snapshot names) as a map (id 7). Panels whose vertex counts differ between the states show no data. The names are kept in Garment.viz.compare.",
            keys: &[],
            hints: SET,
            schema: set_schema,
            run: compare_set,
        },
        Command {
            id: "compare.clear",
            mcp: "compare_clear",
            group: Group::View,
            label: "Clear Compare",
            description: "Stop the comparison map and forget the compared state names. Snapshots stay.",
            keys: &[],
            hints: SET,
            schema: no_args,
            run: compare_clear,
        },
        Command {
            id: "compare.get",
            mcp: "compare_get",
            group: Group::View,
            label: "Compare State",
            description: "Read the compared state names, the snapshot names and whether a map is streaming.",
            keys: &[],
            hints: READ,
            schema: no_args,
            run: compare_get,
        },
    ]
}
