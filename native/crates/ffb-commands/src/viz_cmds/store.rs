use ffb_pattern::Garment;
use serde_json::{json, Map, Value};

use crate::{CommandError, Edit, Session};

pub(crate) fn edit(s: &mut Session, label: &str, change: impl FnOnce(&mut Garment) -> Result<Value, CommandError>) -> Result<Value, CommandError> {
    let before = s.garment.clone();
    match change(&mut s.garment) {
        Ok(out) => {
            let after = s.garment.clone();
            s.record(Edit::Garment { label: label.into(), before: Box::new(before), after: Box::new(after) });
            Ok(out)
        }
        Err(e) => {
            s.garment = before;
            Err(e)
        }
    }
}

pub(crate) fn compare_obj(g: &mut Garment) -> &mut Map<String, Value> {
    if !g.viz.compare.is_object() {
        g.viz.compare = json!({});
    }
    g.viz.compare.as_object_mut().expect("object just set")
}

pub(crate) fn set_compare(g: &mut Garment, pair: Option<(&str, &str)>) {
    let obj = compare_obj(g);
    match pair {
        Some((a, b)) => {
            obj.insert("a".into(), json!(a));
            obj.insert("b".into(), json!(b));
        }
        None => {
            obj.remove("a");
            obj.remove("b");
        }
    }
}
