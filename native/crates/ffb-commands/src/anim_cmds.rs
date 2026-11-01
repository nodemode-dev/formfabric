mod ops;
mod play;
mod record;
mod rig;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use ffb_anim::cachefmt::FrameStat;

use crate::{Command, CommandError, Edit, Session};

pub(crate) const OBJECT_BASE: u32 = 1_000_000;
const MAX_TRASH: usize = 6;

pub(crate) struct Finished {
    pub bytes: Vec<u8>,
    pub name: String,
    pub start: u32,
    pub frames: u32,
}

pub(crate) struct Job {
    pub phase: &'static str,
    pub frame: u32,
    pub total: u32,
    pub start: u32,
    pub end: u32,
    pub name: String,
    pub message: String,
    pub ms: f32,
    pub stats: Vec<FrameStat>,
    pub warnings: Vec<String>,
    pub finished: Option<Finished>,
}

impl Default for Job {
    fn default() -> Self {
        Job { phase: "idle", frame: 0, total: 0, start: 0, end: 0, name: String::new(), message: String::new(), ms: 0.0, stats: Vec::new(), warnings: Vec::new(), finished: None }
    }
}

pub(crate) fn lock_job(job: &Arc<Mutex<Job>>) -> MutexGuard<'_, Job> {
    job.lock().unwrap_or_else(|e| e.into_inner())
}

pub struct AnimState {
    pub(crate) job: Arc<Mutex<Job>>,
    pub(crate) cancel: Arc<AtomicBool>,
    pub(crate) playing: Arc<AtomicBool>,
    pub(crate) play_generation: Arc<AtomicU64>,
    pub(crate) play_frame: Arc<AtomicU32>,
    pub(crate) trash: Vec<(String, Vec<u8>)>,
}

impl Default for AnimState {
    fn default() -> Self {
        AnimState {
            job: Arc::new(Mutex::new(Job::default())),
            cancel: Arc::new(AtomicBool::new(false)),
            playing: Arc::new(AtomicBool::new(false)),
            play_generation: Arc::new(AtomicU64::new(0)),
            play_frame: Arc::new(AtomicU32::new(0)),
            trash: Vec::new(),
        }
    }
}

impl Drop for AnimState {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.playing.store(false, Ordering::Release);
        self.play_generation.fetch_add(1, Ordering::AcqRel);
    }
}

pub(crate) fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub(crate) fn entry<'a>(s: &'a Session, name: &str) -> Option<&'a [u8]> {
    s.unknown.iter().find(|(n, _)| n == name).map(|(_, d)| d.as_slice())
}

pub(crate) fn put_entry(s: &mut Session, name: String, data: Vec<u8>) {
    s.unknown.retain(|(n, _)| *n != name);
    s.unknown.push((name, data));
}

pub(crate) fn edit_garment(s: &mut Session, label: &str, change: impl FnOnce(&mut ffb_pattern::Garment) -> Result<(), CommandError>) -> Result<bool, CommandError> {
    let before = s.garment.clone();
    let result = change(&mut s.garment);
    if let Err(e) = result {
        s.garment = before;
        return Err(e);
    }
    if s.garment == before {
        return Ok(false);
    }
    let after = s.garment.clone();
    s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
    Ok(true)
}

fn referenced(s: &Session) -> Vec<String> {
    let mut names: Vec<String> = s.garment.anim.caches.iter().map(|c| c.entry.clone()).collect();
    names.extend(s.garment.anim.motions.iter().map(|m| rig::motion_entry(m.id)));
    names
}

fn sweep(s: &mut Session) {
    let names = referenced(s);
    let mut moved = Vec::new();
    s.unknown.retain(|(n, d)| {
        let ours = n.starts_with("cache/") || n.starts_with("motion/");
        if ours && !names.contains(n) {
            moved.push((n.clone(), d.clone()));
            return false;
        }
        true
    });
    for item in moved {
        s.anim.trash.retain(|(n, _)| *n != item.0);
        s.anim.trash.push(item);
    }
    for name in names {
        if s.unknown.iter().any(|(n, _)| *n == name) {
            continue;
        }
        if let Some(at) = s.anim.trash.iter().position(|(n, _)| *n == name) {
            let item = s.anim.trash.remove(at);
            s.unknown.push(item);
        }
    }
    while s.anim.trash.len() > MAX_TRASH {
        s.anim.trash.remove(0);
    }
}

pub(crate) fn after_change(session: &mut Session, command: &str) {
    if command.starts_with("file.") {
        session.anim.cancel.store(true, Ordering::Release);
        session.anim.playing.store(false, Ordering::Release);
        session.anim.play_generation.fetch_add(1, Ordering::AcqRel);
        session.anim.trash.clear();
        *lock_job(&session.anim.job) = Job::default();
    }
    sweep(session);
}

pub(crate) fn commands() -> Vec<Command> {
    let mut all = ops::commands();
    all.extend(play::commands());
    all
}
