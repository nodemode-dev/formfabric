#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use std::sync::atomic::AtomicBool;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use ffb_anim::cachefmt::{Header, PanelSpec, Writer};
use ffb_anim::cachefmt::{BodyFrame, BodySpec, FrameStat};
use ffb_anim::interp;
use ffb_body::math::IDENTITY;
use ffb_body::{skin, Body};
use ffb_pattern::anim::Anim;
use ffb_pattern::Wind;
use serde_json::{json, Value};

#[cfg(not(target_arch = "wasm32"))]
use super::rig::resized;
use super::rig::{blend_positions, build_rigs, posed, BodyRig};
#[cfg(not(target_arch = "wasm32"))]
use super::{Finished, OBJECT_BASE};
use super::{lock_job, Job};
use crate::{CommandError, FrameSink, Session};

pub(crate) const MAX_RECORD_FRAMES: u32 = 20_000;

pub(crate) struct CoreOps {
    pub begin: Box<dyn FnOnce() -> Result<Vec<ffb_sim::Collider>, String> + Send>,
    pub snapshot: Box<dyn Fn() -> Vec<(u32, Vec<f32>)> + Send>,
    pub step: Box<dyn Fn(Vec<ffb_sim::Collider>, Wind) + Send>,
    pub measure: Box<dyn Fn(bool) -> (f32, f32, u32) + Send>,
    pub end: Box<dyn Fn() + Send>,
    pub rekey: Option<Rekey>,
}

pub(crate) type Rekey = Box<dyn FnMut(&ffb_pattern::Garment, &[Body]) -> Result<(), String> + Send>;

pub(crate) fn has_cloth_keys(anim: &Anim) -> bool {
    anim.tracks.iter().any(|t| (t.target.starts_with("panel.") || t.target.starts_with("fabric.")) && !t.keys.is_empty())
}

fn truth(v: &Value) -> Option<bool> {
    v.as_bool().or_else(|| v.as_f64().map(|x| x >= 0.5))
}

fn apply_cloth_keys(g: &mut ffb_pattern::Garment, anim: &Anim, frame: f64) -> bool {
    let mut changed = false;
    for t in anim.tracks.iter() {
        let parts: Vec<&str> = t.target.split('.').collect();
        let Some(v) = interp::evaluate(&t.keys, frame) else { continue };
        match parts.as_slice() {
            ["panel", id, "state", field] => {
                let Some(k) = id.parse::<u32>().ok().and_then(|i| g.index_of(i)) else { continue };
                let st = &mut g.panels[k].state;
                let num = v.as_f64().filter(|x| x.is_finite());
                match *field {
                    "frozen" => {
                        if let Some(b) = truth(&v).filter(|b| *b != st.frozen) {
                            st.frozen = b;
                            changed = true;
                        }
                    }
                    "hold_shape" => {
                        if let Some(b) = truth(&v).filter(|b| *b != st.hold_shape) {
                            st.hold_shape = b;
                            changed = true;
                        }
                    }
                    "stiffen" => {
                        if let Some(x) = num.filter(|x| *x != st.stiffen) {
                            st.stiffen = x;
                            changed = true;
                        }
                    }
                    "shrink_weft" => {
                        if let Some(x) = num.filter(|x| *x != st.shrink_weft) {
                            st.shrink_weft = x;
                            changed = true;
                        }
                    }
                    "shrink_warp" => {
                        if let Some(x) = num.filter(|x| *x != st.shrink_warp) {
                            st.shrink_warp = x;
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
            ["fabric", id, field] => {
                let Some(f) = id.parse::<u32>().ok().and_then(|i| g.fabrics.iter_mut().find(|f| f.id == i)) else { continue };
                let Some(x) = v.as_f64().filter(|x| x.is_finite()) else { continue };
                let Ok(mut json) = serde_json::to_value(&f.physical) else { continue };
                if json.get(*field).and_then(Value::as_f64) == Some(x) {
                    continue;
                }
                json[*field] = json!(x);
                if let Ok(next) = serde_json::from_value(json) {
                    f.physical = next;
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub(crate) struct Plan {
    pub ops: CoreOps,
    pub shared: Arc<Mutex<ffb_sim::Params>>,
    pub job: Arc<Mutex<Job>>,
    pub cancel: Arc<AtomicBool>,
    pub anim: Anim,
    pub bodies: Vec<Body>,
    pub rigs: Vec<BodyRig>,
    pub mesh_bodies: Vec<usize>,
    pub original_mesh_count: usize,
    pub start: u32,
    pub end: u32,
    pub steps: u32,
    pub validate: bool,
    pub quantized: bool,
    pub name: String,
    pub wind: Wind,
    pub sink: Option<Arc<dyn FrameSink>>,
    pub garment: Option<ffb_pattern::Garment>,
}

fn wind_at(base: &Wind, anim: &Anim, frame: f64) -> Wind {
    let mut w = base.clone();
    for t in anim.tracks.iter().filter(|t| t.target.starts_with("wind.")) {
        let Some(v) = interp::evaluate(&t.keys, frame) else { continue };
        let num = |v: &Value| v.as_f64().or_else(|| v.as_bool().map(|b| if b { 1.0 } else { 0.0 }));
        let vec3 = |v: &Value| -> Option<[f64; 3]> {
            let a = v.as_array()?;
            (a.len() == 3).then(|| [a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0), a[2].as_f64().unwrap_or(0.0)])
        };
        match &t.target["wind.".len()..] {
            "enabled" => {
                if let Some(x) = num(&v) {
                    w.enabled = x >= 0.5;
                }
            }
            "strength" => w.strength = num(&v).unwrap_or(w.strength),
            "decay" => w.decay = num(&v).unwrap_or(w.decay),
            "turbulence" => w.turbulence = num(&v).unwrap_or(w.turbulence),
            "direction" => w.direction = vec3(&v).unwrap_or(w.direction),
            "origin" => w.origin = vec3(&v).unwrap_or(w.origin),
            "kind" => {
                if let Some(k) = v.as_str() {
                    w.kind = k.to_string();
                }
            }
            _ => {}
        }
    }
    w
}

pub(crate) fn mesh_bodies(bodies: &[Body]) -> Vec<usize> {
    (0..bodies.len()).filter(|i| bodies[*i].visible && !bodies[*i].triangles.is_empty()).collect()
}

pub(crate) fn body_frame(body: &Body, e: &super::rig::Evaluated, spec: &BodySpec) -> BodyFrame {
    BodyFrame { id: body.id, offset: e.offset, morph: e.morph, quats: spec.joints.iter().map(|n| e.pose.get(n).copied().unwrap_or(IDENTITY)).collect() }
}

pub(crate) fn rig_positions(rig: &BodyRig, working: &mut Body, bodies: &[Body], ev: &super::rig::Evaluated) -> Vec<[f32; 3]> {
    let mut pos = posed(working, ev);
    if let Some(m) = rig.morph.as_ref() {
        let to = skin::posed_positions(&bodies[m.to]);
        if to.len() == pos.len() {
            pos = blend_positions(&pos, &to, ev.morph);
        }
    }
    pos
}

pub(crate) fn flat(points: &[[f32; 3]]) -> Vec<f32> {
    points.iter().flat_map(|p| p.iter().copied()).collect()
}

fn fail(job: &Arc<Mutex<Job>>, message: String) {
    let mut j = lock_job(job);
    j.phase = "failed";
    j.message = message;
}

pub(crate) fn plan_for(s: &mut Session, start: u32, end: u32, name: String, steps: u32, validate: bool, quantized: bool) -> Result<Plan, CommandError> {
    let bodies: Vec<Body> = s.bodies.clone();
    let mut warnings = Vec::new();
    let rigs = build_rigs(&s.garment.anim, &bodies, &s.unknown, &mut warnings);
    let hidden_targets: Vec<usize> = rigs.iter().filter_map(|r| r.morph.as_ref().map(|m| m.to)).collect();
    let original_mesh_count = mesh_bodies(&bodies).len();
    let mesh_list: Vec<usize> = mesh_bodies(&bodies).into_iter().filter(|i| !hidden_targets.contains(i)).collect();
    {
        let mut j = lock_job(&s.anim.job);
        j.warnings = warnings;
    }
    let (a, b, c, d, e) = (s.sim.core.clone(), s.sim.core.clone(), s.sim.core.clone(), s.sim.core.clone(), s.sim.core.clone());
    let keyed = has_cloth_keys(&s.garment.anim);
    let ops = CoreOps {
        begin: Box::new(move || {
            let mut core = a.lock().unwrap_or_else(|x| x.into_inner());
            if core.stepper.is_none() {
                return Err("nothing is built; run sim.reset first".to_string());
            }
            core.closing_over = true;
            Ok(core.colliders.clone())
        }),
        snapshot: Box::new(move || b.lock().unwrap_or_else(|x| x.into_inner()).snapshot()),
        step: Box::new(move |colliders, wind| {
            let mut core = c.lock().unwrap_or_else(|x| x.into_inner());
            core.wind = wind;
            core.colliders.clone_from(&colliders);
            if let Some(st) = core.stepper.as_mut() {
                st.set_colliders(colliders);
            }
            core.step_frames(1);
        }),
        measure: Box::new(move |validate| {
            let core = d.lock().unwrap_or_else(|x| x.into_inner());
            match core.stepper.as_ref() {
                Some(st) if validate => (st.max_link_gap(), st.max_penetration().max(0.0), crate::sim_cmds::self_hits(st)),
                Some(st) => (st.max_link_gap(), 0.0, 0),
                None => (0.0, 0.0, 0),
            }
        }),
        end: Box::new(move || e.lock().unwrap_or_else(|x| x.into_inner()).stale = true),
        rekey: if keyed { Some(Box::new(crate::sim_cmds::keyed_rebuilder(s.sim.core.clone(), s.mesh.forced.clone()))) } else { None },
    };
    Ok(Plan {
        ops,
        shared: s.sim.params.clone(),
        job: s.anim.job.clone(),
        cancel: s.anim.cancel.clone(),
        anim: s.garment.anim.clone(),
        bodies,
        rigs,
        mesh_bodies: mesh_list,
        original_mesh_count,
        start,
        end,
        steps,
        validate,
        quantized,
        name,
        wind: s.garment.wind.clone(),
        sink: s.sink.clone(),
        garment: if keyed { Some(s.garment.clone()) } else { None },
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn spawn(plan: Plan) -> bool {
    let body = move || {
        let job = plan.job.clone();
        let shared = plan.shared.clone();
        let original = *shared.lock().unwrap_or_else(|e| e.into_inner());
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(plan)));
        *shared.lock().unwrap_or_else(|e| e.into_inner()) = original;
        if outcome.is_err() {
            fail(&job, "the recording thread stopped unexpectedly".into());
        }
    };
    std::thread::Builder::new().name("ffb-anim".into()).spawn(body).is_ok()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn spawn(_plan: Plan) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn run(mut p: Plan) {
    use std::time::Instant;
    let fps = p.anim.fps.max(1.0);
    let k = p.steps.max(1);
    {
        let mut sh = p.shared.lock().unwrap_or_else(|e| e.into_inner());
        sh.time_step = 1.0 / (fps * k as f32);
    }
    let mut rekey = p.ops.rekey.take();
    let begin = std::mem::replace(&mut p.ops.begin, Box::new(|| Err(String::new())));
    let existing = match begin() {
        Ok(c) => c,
        Err(e) => {
            fail(&p.job, e);
            return;
        }
    };
    if existing.len() < p.original_mesh_count {
        fail(&p.job, "the colliders changed; run sim.reset and record again".into());
        return;
    }
    let tail: Vec<ffb_sim::Collider> = existing[p.original_mesh_count..].to_vec();
    let mut statics: Vec<Option<Vec<[f32; 3]>>> = vec![None; p.bodies.len()];
    for (i, slot) in statics.iter_mut().enumerate() {
        if !p.rigs.iter().any(|r| r.index == i) {
            *slot = Some(skin::posed_positions(&p.bodies[i]));
        }
    }
    let mut working: Vec<Body> = p.bodies.clone();
    let specs: Vec<BodySpec> = p
        .mesh_bodies
        .iter()
        .map(|i| BodySpec { id: p.bodies[*i].id, joints: p.bodies[*i].joints.iter().map(|j| j.name.clone()).collect(), morph_to: p.rigs.iter().find(|r| r.index == *i).and_then(|r| r.morph.as_ref()).map(|m| p.bodies[m.to].id) })
        .collect();
    let first = (p.ops.snapshot)();
    let header = Header {
        name: p.name.clone(),
        fps,
        start: p.start,
        frames: 0,
        quantized: p.quantized,
        panels: first.iter().map(|(id, v)| PanelSpec { id: *id, vertices: (v.len() / 3) as u32 }).collect(),
        bodies: specs.clone(),
        stats: Vec::new(),
    };
    let mut writer = Writer::new(header);
    let mut previous: Vec<Vec<f32>> = first.iter().map(|(_, v)| v.clone()).collect();
    let total = p.end - p.start + 1;
    {
        let mut j = lock_job(&p.job);
        j.total = total;
        j.frame = 0;
        j.phase = "running";
    }
    let mut stopped_at = None;
    for f in p.start..=p.end {
        if p.cancel.load(Ordering::Acquire) {
            stopped_at = Some(f);
            break;
        }
        let began = Instant::now();
        if let (Some(g), Some(rekey)) = (p.garment.as_mut(), rekey.as_mut()) {
            if apply_cloth_keys(g, &p.anim, f64::from(f)) {
                if let Err(e) = rekey(g, &[]) {
                    fail(&p.job, format!("the cloth could not be rebuilt for the panel or fabric keys at frame {f}: {e}"));
                    return;
                }
            }
        }
        for r in p.rigs.iter_mut() {
            if r.measure_keys.is_empty() {
                continue;
            }
            let targets = r.measurement_targets(f64::from(f));
            let changed = targets.len() != r.measured.len() || targets.iter().zip(&r.measured).any(|(a, b)| a.0 != b.0 || (a.1 - b.1).abs() > 0.05);
            if changed {
                if let Some(b) = resized(&r.pristine, &targets) {
                    working[r.index] = b;
                }
                r.measured = targets;
            }
        }
        for sub in 1..=k {
            let ft = f64::from(f) - 1.0 + f64::from(sub) / f64::from(k);
            let ft = ft.max(f64::from(p.start));
            let mut colliders: Vec<ffb_sim::Collider> = Vec::with_capacity(p.mesh_bodies.len() + tail.len());
            for i in &p.mesh_bodies {
                let positions = match p.rigs.iter().position(|r| r.index == *i) {
                    Some(ri) => {
                        let ev = p.rigs[ri].evaluate(&p.anim, ft);
                        rig_positions(&p.rigs[ri], &mut working[*i], &p.bodies, &ev)
                    }
                    None => statics[*i].clone().unwrap_or_default(),
                };
                colliders.push(ffb_sim::Collider::Mesh { positions, triangles: working[*i].triangles.clone() });
            }
            colliders.extend(tail.iter().cloned());
            (p.ops.step)(colliders, wind_at(&p.wind, &p.anim, ft));
        }
        let snap = (p.ops.snapshot)();
        let (gap, penetration, crossings) = (p.ops.measure)(p.validate);
        let mut stat = FrameStat { frame: f, seam_gap_mm: gap, penetration_mm: penetration, self_intersections: crossings, ..FrameStat::default() };
        if snap.iter().any(|(_, v)| v.iter().any(|x| !x.is_finite())) {
            fail(&p.job, format!("the cloth went non-finite at frame {f}; the cache stops at frame {}", f.saturating_sub(1)));
            break;
        }
        let mut worst = 0.0f32;
        for ((_, now), before) in snap.iter().zip(&previous) {
            for (a, b) in now.chunks_exact(3).zip(before.chunks_exact(3)) {
                let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                worst = worst.max(d);
            }
        }
        stat.max_displacement_mm = worst;
        stat.max_velocity_mm_s = worst * fps;
        stat.ms = began.elapsed().as_secs_f32() * 1000.0;
        previous = snap.iter().map(|(_, v)| v.clone()).collect();
        let panels: Vec<Vec<f32>> = snap.iter().map(|(_, v)| v.clone()).collect();
        let mut frames_bodies = Vec::with_capacity(specs.len());
        let mut streamed: Vec<(u32, Vec<[f32; 3]>)> = Vec::new();
        for (slot, i) in p.mesh_bodies.iter().enumerate() {
            match p.rigs.iter().position(|r| r.index == *i) {
                Some(ri) => {
                    let ev = p.rigs[ri].evaluate(&p.anim, f64::from(f));
                    frames_bodies.push(body_frame(&p.bodies[*i], &ev, &specs[slot]));
                    if p.sink.is_some() {
                        streamed.push((p.bodies[*i].id, rig_positions(&p.rigs[ri], &mut working[*i], &p.bodies, &ev)));
                    }
                }
                None => {
                    let ev = super::rig::Evaluated { pose: p.bodies[*i].pose.clone(), offset: [0.0; 3], morph: 0.0 };
                    frames_bodies.push(body_frame(&p.bodies[*i], &ev, &specs[slot]));
                }
            }
        }
        if let Err(e) = writer.push(&panels, &frames_bodies, stat.clone()) {
            fail(&p.job, e);
            return;
        }
        if let Some(sink) = &p.sink {
            for (id, v) in &snap {
                sink.positions(*id, v);
            }
            for (id, pos) in &streamed {
                sink.positions(OBJECT_BASE + id, &flat(pos));
            }
        }
        let mut j = lock_job(&p.job);
        j.frame += 1;
        j.stats.push(stat);
        j.ms += began.elapsed().as_secs_f32() * 1000.0;
    }
    (p.ops.end)();
    let header_frames = writer.header().frames;
    if lock_job(&p.job).phase == "failed" && header_frames == 0 {
        return;
    }
    if header_frames == 0 {
        fail(&p.job, "no frame was recorded".into());
        return;
    }
    match writer.finish() {
        Ok(bytes) => {
            let mut j = lock_job(&p.job);
            let was_failed = j.phase == "failed";
            j.finished = Some(Finished { bytes, name: p.name.clone(), start: p.start, frames: header_frames });
            if !was_failed {
                j.phase = "done";
            }
            if let Some(f) = stopped_at {
                j.message = format!("stopped before frame {f}; {header_frames} frames kept");
            }
        }
        Err(e) => fail(&p.job, e),
    }
}

pub(crate) fn summary(job: &Job) -> Value {
    let max = |f: fn(&FrameStat) -> f32| job.stats.iter().map(f).fold(0.0f32, f32::max);
    json!({
        "frames": job.stats.len(),
        "max_penetration_mm": max(|s| s.penetration_mm),
        "max_self_intersections": job.stats.iter().map(|s| s.self_intersections).max().unwrap_or(0),
        "max_displacement_mm": max(|s| s.max_displacement_mm),
        "max_seam_gap_mm": max(|s| s.seam_gap_mm),
        "max_velocity_mm_s": max(|s| s.max_velocity_mm_s),
    })
}
