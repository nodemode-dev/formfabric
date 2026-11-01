use ffb_pattern::geom::{add, dist, lerp, mul, sub, unit};
use ffb_pattern::sew::{end_length, next_seam_id, t_after_length};
use ffb_pattern::{path, polygon, Inner, InnerKind, InnerRole, Point, Seam, SeamEnd, SeamFold};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, SET, WRITE};

fn pleat_schema() -> Value {
    obj(
        json!({
            "panel": panel_id("Panel that receives the pleat fold lines."),
            "inner": int("Existing inner line to take the direction and length of the first pleat line from.", 1.0, 4_294_967_295.0),
            "from": xy("Start of the first pleat line. Give from and to, or inner."),
            "to": xy("End of the first pleat line."),
            "space": space(),
            "kind": {"type": "string", "enum": ["knife", "box"], "default": "knife", "description": "knife: one fold, three lines per pleat, the outer two sewn together. box: two knife folds facing apart, five lines per pleat."},
            "depth": num("Depth of one fold, the distance between neighbouring lines of a pleat.", 1.0, 1000.0, Some("mm")),
            "count": int("Number of pleats.", 1.0, 64.0),
            "interval": num("Distance from the first line of one pleat to the first line of the next. Default is the pleat width plus twice the depth.", 1.0, 100_000.0, Some("mm")),
            "reverse": {"type": "boolean", "default": false, "description": "Fold to the back (360 degrees) instead of the front (0 degrees)."},
            "side": {"type": "string", "enum": ["left", "right"], "default": "left", "description": "Side of the first pleat line, walking from to, where the next lines are added."},
            "strength": num("Fold strength of the crease lines.", 0.0, 20.0, None),
            "sew": {"type": "boolean", "default": true, "description": "Create the seams that close the pleats."}
        }),
        &["panel"],
    )
}

fn crease(points: [[f64; 2]; 2], angle: f64, strength: f64, role: InnerRole) -> Inner {
    Inner {
        id: 0,
        kind: InnerKind::Line,
        points: points.iter().map(|p| Point { p: *p, ..Point::default() }).collect(),
        closed: false,
        fold_angle: angle,
        fold_strength: strength,
        role,
    }
}

fn pleat(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel_id_value = required_id(a, "panel")?;
    let index = s.garment.index_of(panel_id_value).ok_or_else(|| missing(format!("no panel with id {panel_id_value}")))?;
    let place = s.garment.panels[index].placement();
    let world = text(a, "space") != Some("local");
    let (from, to) = match (id(a, "inner"), optional_vec2(a, "from")?, optional_vec2(a, "to")?) {
        (Some(inner), None, None) => {
            let shape = s.garment.panels[index].inner.iter().find(|i| i.id == inner).ok_or_else(|| missing(format!("panel {panel_id_value} has no inner line {inner}")))?;
            let (first, last) = (shape.points.first(), shape.points.last());
            match (first, last) {
                (Some(f), Some(l)) if shape.points.len() >= 2 => (f.p, l.p),
                _ => return Err(bad("the inner line needs two points")),
            }
        }
        (None, Some(f), Some(t)) => {
            if world {
                (place.to_local(f), place.to_local(t))
            } else {
                (f, t)
            }
        }
        _ => return Err(bad("give either inner, or from and to")),
    };
    let dir = unit(sub(to, from)).ok_or_else(|| bad("the pleat line has no length"))?;
    let kind = text(a, "kind").unwrap_or("knife");
    let depth = number_or(a, "depth", 20.0);
    let want = count(a, "count").unwrap_or(1);
    if !(1.0..=1000.0).contains(&depth) || !(1..=64).contains(&want) {
        return Err(bad("depth must be 1 to 1000 mm and count 1 to 64"));
    }
    let lines = if kind == "box" { 5 } else { 3 };
    let width = depth * (lines - 1) as f64;
    let interval = number_or(a, "interval", width + 2.0 * depth);
    if interval < width {
        return Err(bad(format!("interval {interval:.1} is smaller than the pleat width {width:.1}")));
    }
    let strength = number_or(a, "strength", 1.0);
    let crest = if flag(a, "reverse").unwrap_or(false) { 360.0 } else { 0.0 };
    let left = [-dir[1], dir[0]];
    let normal = if text(a, "side") == Some("right") { mul(left, -1.0) } else { left };

    let outline = path::flatten(&s.garment.panels[index].outline, true, 0.1);
    let mut created: Vec<Vec<Inner>> = Vec::new();
    for k in 0..want {
        let mut set = Vec::new();
        for j in 0..lines {
            let shift = mul(normal, k as f64 * interval + j as f64 * depth);
            let (p, q) = (add(from, shift), add(to, shift));
            if !polygon::contains(&outline, lerp(p, q, 0.5)) {
                return Err(geometry(format!("pleat {} line {} falls outside the panel; reduce count, interval or depth", k + 1, j + 1)));
            }
            let creasing = if lines == 3 { j == 1 } else { j == 1 || j == 3 };
            set.push(if creasing { crease([p, q], crest, strength, InnerRole::Fold) } else { crease([p, q], 180.0, 0.0, InnerRole::Fold) });
        }
        created.push(set);
    }

    let before = s.garment.clone();
    let mut ids: Vec<Vec<u32>> = Vec::new();
    for set in created {
        let mut row = Vec::new();
        for mut line in set {
            line.id = s.garment.next_id();
            row.push(line.id);
            s.garment.panels[index].inner.push(line);
        }
        ids.push(row);
    }
    let mut seam_ids = Vec::new();
    if flag(a, "sew").unwrap_or(true) {
        let end = |line: u32| SeamEnd { panel: panel_id_value, inner: Some(line), segment: 0, t0: 0.0, t1: 1.0 };
        for row in &ids {
            let pairs: Vec<(u32, u32)> = if lines == 3 { vec![(row[0], row[2])] } else { vec![(row[0], row[2]), (row[4], row[2])] };
            for (x, y) in pairs {
                let seam = Seam { id: next_seam_id(&s.garment), a: vec![end(x)], b: vec![end(y)], reversed: false, fold: SeamFold::Flat, ..Seam::default() };
                seam_ids.push(seam.id);
                s.garment.seams.push(seam);
            }
        }
    }
    record(s, "Pleats", before);
    Ok(json!({"panel": panel_id_value, "kind": kind, "count": want, "depth": depth, "interval": interval, "width": width, "lines": ids, "seams": seam_ids, "line_length": dist(from, to)}))
}

fn gather_schema() -> Value {
    obj(
        json!({
            "seam": panel_id("Seam id as listed by sew.list."),
            "ratio": num("Arc length of side b over side a, 1 or more. Side a is shortened from its end until it fits; side b keeps its spans.", 1.0, 20.0, None)
        }),
        &["seam", "ratio"],
    )
}

fn gather(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let index = seam_index(s, required_id(a, "seam")?)?;
    let ratio = number_or(a, "ratio", 1.0);
    if !(1.0..=20.0).contains(&ratio) {
        return Err(bad("ratio must be between 1 and 20"));
    }
    let before = s.garment.clone();
    let mut seam = s.garment.seams[index].clone();
    let total_b: f64 = seam.b.iter().filter_map(|e| end_length(&s.garment, e)).sum();
    let target = total_b / ratio;
    let last = seam.a.len() - 1;
    let others: f64 = seam.a[..last].iter().filter_map(|e| end_length(&s.garment, e)).sum();
    let wanted = target - others;
    let end = seam.a[last].clone();
    if end.t0 > end.t1 {
        return Err(bad("the last span of side a runs against its line; flip the span before gathering"));
    }
    if wanted < 0.5 {
        return Err(bad(format!("a ratio of {ratio} would leave side a with {wanted:.2} mm; use a smaller ratio")));
    }
    let t1 = t_after_length(&s.garment, &end, end.t0, wanted).ok_or_else(|| bad("side a refers to a segment that does not exist"))?;
    let reach = end_length(&s.garment, &SeamEnd { t1: 1.0, ..end.clone() }).unwrap_or(0.0);
    if wanted > reach + 1e-6 {
        return Err(bad(format!("side a cannot reach {wanted:.2} mm on its segment, the most is {reach:.2} mm; use a larger ratio")));
    }
    seam.a[last].t1 = t1.clamp(end.t0, 1.0);
    seam.ratio = ratio;
    s.garment.seams[index] = seam.clone();
    record(s, "Gather", before);
    Ok(json!({"changed": true, "seam": describe(&s.garment, &seam)}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "sew.pleat",
            mcp: "sew_pleat",
            group: Group::Sew,
            label: "Pleats",
            description: "Create knife or box pleats on a panel: parallel fold lines from a start line (from and to, or an existing inner line) added to one side, with fold angles 0 (front) or 360 (back, reverse) on the creases, and seams that close each pleat. A knife pleat has three lines with the middle one creased and the outer two sewn together; a box pleat has five lines, creases at the second and fourth, both outer lines sewn to the middle one. Returns the new inner line ids per pleat and the seam ids.",
            keys: &[],
            hints: WRITE,
            schema: pleat_schema,
            run: pleat,
        },
        Command {
            id: "sew.gather",
            mcp: "sew_gather",
            group: Group::Sew,
            label: "Gather",
            description: "Gather side b onto side a at a ratio of 1 or more: side a is shortened from its end so that side b is the given multiple of it, and the seam ratio is stored. The cloth of side b bunches up to fit when simulated.",
            keys: &[],
            hints: SET,
            schema: gather_schema,
            run: gather,
        },
    ]
}
