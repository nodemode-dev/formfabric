use std::collections::BTreeMap;

use ffb_pattern::anim::{Interp, Key};
use serde_json::Value;

use crate::{Edit, Session};

const SKIP: [&str; 9] = ["history.", "file.", "key.", "anim.", "cache.", "job.", "motion.", "morph.", "sample."];

pub(crate) fn pin_target(target: &str) -> Option<u32> {
    target.strip_prefix("pin:").and_then(|n| n.parse().ok())
}

pub(crate) fn current_value(s: &Session, target: &str) -> Option<Value> {
    if let Some(id) = pin_target(target) {
        let pin = s.garment.pins.iter().find(|p| p.id == id)?;
        return pin.target.map(|t| serde_json::json!([t[0], t[1], t[2]]));
    }
    let parts: Vec<&str> = target.split('.').collect();
    match parts.as_slice() {
        ["wind", field] => serde_json::to_value(&s.garment.wind).ok()?.get(*field).cloned(),
        ["panel", id, "state", field] => {
            let panel = s.garment.panels.iter().find(|p| p.id == id.parse::<u32>().ok().unwrap_or(0))?;
            serde_json::to_value(&panel.state).ok()?.get(*field).cloned()
        }
        ["fabric", id, field] => {
            let fabric = s.garment.fabrics.iter().find(|f| f.id == id.parse::<u32>().ok().unwrap_or(0))?;
            serde_json::to_value(&fabric.physical).ok()?.get(*field).cloned()
        }
        _ => None,
    }
}

fn snapshot(s: &Session) -> BTreeMap<String, Value> {
    s.garment.anim.tracks.iter().filter_map(|t| current_value(s, &t.target).map(|v| (t.target.clone(), v))).collect()
}

pub(crate) fn run(s: &mut Session, command: &str) {
    let skipped = SKIP.iter().any(|p| command.starts_with(p));
    let now = snapshot(s);
    if skipped || !s.garment.anim.auto_key {
        s.anim.auto_prev = now;
        return;
    }
    let frame = s.garment.anim.current;
    let changed: Vec<(String, Value)> = now.iter().filter(|(k, v)| s.anim.auto_prev.get(*k).is_some_and(|old| old != *v)).map(|(k, v)| (k.clone(), v.clone())).collect();
    s.anim.auto_prev = now;
    if changed.is_empty() {
        return;
    }
    let before = s.garment.clone();
    for (target, value) in changed {
        let Some(track) = s.garment.anim.tracks.iter_mut().find(|t| t.target == target) else { continue };
        let interp = track.keys.iter().find(|k| k.frame == frame).map_or(Interp::Linear, |k| k.interp);
        track.keys.retain(|k| k.frame != frame);
        track.keys.push(Key { frame, value, interp });
        track.keys.sort_by_key(|k| k.frame);
    }
    if s.garment == before {
        return;
    }
    let after = s.garment.clone();
    s.history.amend_last(Edit::Garment { label: "Auto key".into(), before: Box::new(before), after: Box::new(after) });
}
