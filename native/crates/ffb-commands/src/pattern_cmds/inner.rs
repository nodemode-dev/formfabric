use ffb_pattern::{InnerKind, InnerRole};
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::{edit_panel, min_points};
use crate::{Command, CommandError, Group, Session, SET};

fn inner_schema() -> Value {
    let mut props = json!({
        "panel": sc::panel(),
        "inner": sc::int("Inner shape id inside the panel.", 1.0, 4_294_967_295.0, None),
        "role": sc::text("shape: ordinary inner line. fold: the line folds at fold_angle. dart: a dart. reference: a construction line that does not shape the cloth.", &["shape", "fold", "dart", "reference"]),
        "closed": sc::flag("Whether the shape closes back on its first point.", false)
    });
    sc::fold_fields(&mut props);
    for key in ["fold_angle", "fold_strength", "closed"] {
        if let Some(m) = props[key].as_object_mut() {
            m.remove("default");
        }
    }
    sc::obj(props, &["panel", "inner"])
}

fn set_inner(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(inner_schema, a)?;
    let id = required_id(a, "inner")?;
    let role = match text(a, "role") {
        None => None,
        Some("shape") => Some(InnerRole::Shape),
        Some("fold") => Some(InnerRole::Fold),
        Some("dart") => Some(InnerRole::Dart),
        Some("reference") => Some(InnerRole::Reference),
        Some(other) => return Err(bad(format!("role must be shape, fold, dart or reference, not {other}"))),
    };
    let closed = flag(a, "closed");
    edit_panel(s, required_id(a, "panel")?, "Inner Shape", |panel| {
        let pid = panel.id;
        let shape = panel.inner.iter_mut().find(|n| n.id == id).ok_or_else(|| missing(format!("panel {pid} has no inner shape {id}")))?;
        if let Some(c) = closed {
            if c && shape.points.len() < min_points(true) {
                return Err(bad(format!("a closed shape needs at least {} points", min_points(true))));
            }
            if !c && matches!(shape.kind, InnerKind::Rectangle | InnerKind::Ellipse) {
                return Err(bad("a rectangle or ellipse is always closed"));
            }
            shape.closed = c;
            shape.kind = match (shape.kind, c) {
                (InnerKind::Line | InnerKind::Polyline, true) => InnerKind::Polygon,
                (InnerKind::Polygon, false) => InnerKind::Polyline,
                (kind, _) => kind,
            };
        }
        shape.role = role.unwrap_or(shape.role);
        shape.fold_angle = number_or(a, "fold_angle", shape.fold_angle);
        shape.fold_strength = number_or(a, "fold_strength", shape.fold_strength);
        Ok(json!({"inner": shape.id, "role": shape.role, "closed": shape.closed, "kind": shape.kind, "fold_angle": shape.fold_angle, "fold_strength": shape.fold_strength}))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "pattern.set_inner",
        mcp: "pattern_set_inner",
        group: Group::Pattern,
        label: "Inner Shape Properties",
        description: "Set the role (shape, fold, dart, reference), fold angle (0 to 360, 180 flat, lower folds to the front, higher to the back), fold strength (0 to 20) and closed state of an inner shape. Only the given values change; geometry is untouched.",
        keys: &[],
        hints: SET,
        schema: inner_schema,
        run: set_inner,
    }]
}
