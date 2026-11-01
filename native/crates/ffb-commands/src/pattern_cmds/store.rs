use ffb_pattern::{Panel, Point};
use serde_json::{json, Value};

use super::input::{bad, missing};
use crate::{CommandError, Edit, Session};

pub fn index_of(s: &Session, id: u32) -> Result<usize, CommandError> {
    s.garment.index_of(id).ok_or_else(|| missing(format!("no panel with id {id}")))
}

pub fn replace(index: usize, label: &str, before: Panel, after: Panel) -> Edit {
    Edit::Panel { label: label.into(), index, before: Some(Box::new(before)), after: Some(Box::new(after)) }
}

pub fn commit(s: &mut Session, label: &str, mut edits: Vec<Edit>) {
    match edits.len() {
        0 => {}
        1 => s.record(edits.remove(0)),
        _ => s.record(Edit::Many { label: label.into(), edits }),
    }
}

pub fn edit_panel<F>(s: &mut Session, id: u32, label: &str, f: F) -> Result<Value, CommandError>
where
    F: FnOnce(&mut Panel) -> Result<Value, CommandError>,
{
    let index = index_of(s, id)?;
    let before = s.garment.panels[index].clone();
    let mut after = before.clone();
    let mut out = f(&mut after)?;
    let changed = after != before;
    if changed {
        s.garment.panels[index] = after.clone();
        s.record(replace(index, label, before, after));
    }
    out["changed"] = json!(changed);
    Ok(out)
}

pub fn path_of(panel: &mut Panel, inner: Option<u32>) -> Result<(&mut Vec<Point>, bool), CommandError> {
    let id = panel.id;
    panel.path_mut(inner).ok_or_else(|| missing(format!("panel {id} has no inner shape {}", inner.unwrap_or(0))))
}

pub fn check_index(len: usize, index: usize, what: &str) -> Result<(), CommandError> {
    if index < len {
        Ok(())
    } else {
        Err(bad(format!("{what} {index} is out of range, there are {len}")))
    }
}

pub fn min_points(closed: bool) -> usize {
    if closed {
        3
    } else {
        2
    }
}
