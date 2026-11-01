use ffb_pattern::{grain, notch, offset, Garment};
use serde_json::{json, Value};

pub fn annotate(out: &mut Value, garment: &Garment) {
    let Some(list) = out.get_mut("panels").and_then(Value::as_array_mut) else { return };
    for entry in list {
        let Some(panel) = entry.get("id").and_then(Value::as_u64).and_then(|id| garment.panels.iter().find(|p| u64::from(p.id) == id)) else { continue };
        let place = panel.placement();
        entry["grain_line_effective"] = match grain::effective(panel) {
            Some((line, computed)) => json!({"from": line[0], "to": line[1], "computed": computed, "angle": grain::angle_of(line)}),
            None => Value::Null,
        };
        entry["notch_points"] = json!(panel
            .notches
            .iter()
            .filter_map(|n| notch::locate(panel, n).map(|(p, normal)| json!({"id": n.id, "point": p, "normal": normal, "world": place.to_world(p)})))
            .collect::<Vec<_>>());
        entry["seam_outline"] = match offset::seam_outline(panel) {
            Some(Ok(points)) => serde_json::to_value(points).unwrap_or(Value::Null),
            _ => Value::Null,
        };
    }
}
