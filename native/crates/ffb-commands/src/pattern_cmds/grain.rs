use ffb_pattern::geom::{dist, len, rotate, sub, V2};
use ffb_pattern::grain;
use serde_json::{json, Value};

use super::input::*;
use super::schema as sc;
use super::store::edit_panel;
use crate::{Command, CommandError, Group, Session, SET};

fn grain_schema() -> Value {
    sc::obj(
        json!({
            "panel": sc::panel(),
            "from": sc::xy("Start of the grain line."),
            "to": sc::xy("End of the grain line; the arrow points here."),
            "angle": sc::deg("Direction of the line, counter-clockwise from the pattern x axis; 90 is vertical. The line is centred on the panel unless center is given.", -360.0, 360.0, None),
            "center": sc::xy("Centre of a line made from angle. Defaults to the panel centre."),
            "length": sc::mm("Length of a line made from angle. Defaults to half the panel height, at least 20 mm.", 1.0, sc::SIZE, None),
            "space": sc::space(),
            "clear": sc::flag("Remove the stored line so the default, vertical through the panel centre, applies again.", false),
            "sync_grain": sc::flag("Also set the panel's grain angle to the line direction.", false)
        }),
        &["panel"],
    )
}

fn report(panel: &ffb_pattern::Panel) -> Value {
    let effective = grain::effective(panel);
    let place = panel.placement();
    json!({
        "from": effective.map(|e| e.0[0]),
        "to": effective.map(|e| e.0[1]),
        "computed": effective.is_none_or(|e| e.1),
        "angle": effective.map(|e| grain::angle_of(e.0)),
        "length": effective.map(|e| dist(e.0[0], e.0[1])),
        "world": effective.map(|e| [place.to_world(e.0[0]), place.to_world(e.0[1])])
    })
}

fn set_grain_line(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    check(grain_schema, a)?;
    let (from, to, angle) = (optional_vec2(a, "from")?, optional_vec2(a, "to")?, number(a, "angle"));
    let clear = flag(a, "clear").unwrap_or(false);
    let explicit = from.is_some() || to.is_some();
    if usize::from(explicit) + usize::from(angle.is_some()) + usize::from(clear) != 1 {
        return Err(bad("give from with to, or angle, or clear"));
    }
    if explicit && (from.is_none() || to.is_none()) {
        return Err(bad("from and to go together"));
    }
    let space = space(a);
    edit_panel(s, required_id(a, "panel")?, "Grain Line", |panel| {
        if clear {
            panel.grain_line = None;
        } else if let (Some(f), Some(t)) = (from, to) {
            let line: [V2; 2] = [to_local(panel, space, f), to_local(panel, space, t)];
            if len(sub(line[1], line[0])) < 1.0 {
                return Err(bad("the grain line must be at least 1 mm long"));
            }
            panel.grain_line = Some(line);
        } else if let Some(deg) = angle {
            let (mid, bounds) = grain::anchor(panel).ok_or_else(|| geometry("the panel has no outline"))?;
            let centre = optional_vec2(a, "center")?.map_or(mid, |c| to_local(panel, space, c));
            let direction = vec_to_local(panel, space, rotate([1.0, 0.0], deg));
            let unit = ffb_pattern::geom::unit(direction).ok_or_else(|| bad("the angle gives no direction"))?;
            panel.grain_line = Some(grain::through(centre, unit, number_or(a, "length", grain::default_length(bounds))));
        }
        if flag(a, "sync_grain").unwrap_or(false) {
            if let Some((line, _)) = grain::effective(panel) {
                panel.grain = grain::angle_of(line);
            }
        }
        Ok(report(panel))
    })
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "pattern.grain_line",
        mcp: "pattern_grain_line",
        group: Group::Pattern,
        label: "Grain Line",
        description: "Set the grain line arrow of a panel from two points (from, to) or from an angle (centred on the panel, with an optional length), or clear it. Without a stored line the panel shows a default: vertical through the panel centre, half the panel height long; pattern.get reports it as grain_line_effective with computed true. Does not change the panel grain angle unless sync_grain is set.",
        keys: &[],
        hints: SET,
        schema: grain_schema,
        run: set_grain_line,
    }]
}
