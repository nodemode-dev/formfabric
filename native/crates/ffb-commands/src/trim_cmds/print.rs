use ffb_pattern::trim::next_print_id;
use ffb_pattern::{Print, SeamSide};
use serde_json::{json, Value};

use super::common::*;
use crate::{Command, CommandError, Group, Session, DESTROY, SET, WRITE};

const EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "svg"];

fn image_of(a: &Value) -> Result<Option<String>, CommandError> {
    if a.get("image").is_none() {
        return Ok(None);
    }
    let path = crate::args::file_path(a, "image", None)?;
    let ext = std::path::Path::new(&path).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !EXTENSIONS.contains(&ext.as_str()) {
        return Err(bad(format!("image must be a {} file", EXTENSIONS.join(", "))));
    }
    if !std::path::Path::new(&path).is_file() {
        return Err(CommandError::new("io", format!("no image file at {path}")));
    }
    Ok(Some(path))
}

fn side_of(a: &Value) -> Result<Option<SeamSide>, CommandError> {
    Ok(match text(a, "side") {
        None => None,
        Some("auto") => Some(SeamSide::Auto),
        Some("front") => Some(SeamSide::Front),
        Some("back") => Some(SeamSide::Back),
        Some(other) => return Err(bad(format!("side must be auto, front or back, not {other}"))),
    })
}

fn size_of(a: &Value) -> Result<Option<[f64; 2]>, CommandError> {
    let size = optional_vec2(a, "size")?;
    if size.is_some_and(|v| v[0] <= 0.0 || v[1] <= 0.0 || v[0] > COORD || v[1] > COORD) {
        return Err(bad("size must be positive"));
    }
    Ok(size)
}

fn props() -> Value {
    json!({
        "image": {"type": "string", "description": "Absolute path of a png, jpg, jpeg, webp or svg image."},
        "name": {"type": "string", "description": "Print name."},
        "position": xy("Centre of the print on the panel."),
        "space": space(),
        "rotation": num("Rotation of the print.", -360.0, 360.0, Some("deg")),
        "size": xy("Width and height of the print on the panel."),
        "side": {"type": "string", "enum": ["auto", "front", "back"], "description": "Which face of the fabric carries the print."},
        "emboss": num("Raised height of the print for rendering.", -5.0, 5.0, Some("mm"))
    })
}

fn add_schema() -> Value {
    let mut p = props();
    p["panel"] = ident("Panel the print is placed on.");
    obj(p, &["panel", "image", "size"])
}

fn describe(panel: u32, p: &Print) -> Value {
    let mut out = serde_json::to_value(p).unwrap_or(Value::Null);
    out["panel"] = json!(panel);
    out
}

fn add(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let panel = panel_of(&s.garment, required_id(a, "panel")?)?;
    let image = image_of(a)?.ok_or_else(|| bad("image is required"))?;
    let size = size_of(a)?.ok_or_else(|| bad("size is required"))?;
    let position = local_point(panel, optional_vec2(a, "position")?.unwrap_or([0.0, 0.0]), text(a, "space") != Some("local") && a.get("position").is_some());
    let panel_id = panel.id;
    let id = next_print_id(&s.garment);
    let name = clean_text(a, "name")?.unwrap_or_else(|| format!("Print {id}"));
    let print = Print { id, name, image, position, rotation: number_or(a, "rotation", 0.0), size, side: side_of(a)?.unwrap_or_default(), emboss: number_or(a, "emboss", 0.0) };
    let before = s.garment.clone();
    let out = describe(panel_id, &print);
    if let Some(p) = s.garment.panels.iter_mut().find(|p| p.id == panel_id) {
        p.prints.push(print);
    }
    record(s, "Add Print", before);
    Ok(out)
}

fn set_schema() -> Value {
    let mut p = props();
    p["print"] = ident("Print id.");
    obj(p, &["print"])
}

fn find(s: &Session, id: u32) -> Result<(usize, usize), CommandError> {
    s.garment
        .panels
        .iter()
        .enumerate()
        .find_map(|(i, p)| p.prints.iter().position(|x| x.id == id).map(|j| (i, j)))
        .ok_or_else(|| missing(format!("no print with id {id}")))
}

fn set(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let (pi, qi) = find(s, required_id(a, "print")?)?;
    let image = image_of(a)?;
    let size = size_of(a)?;
    let side = side_of(a)?;
    let name = clean_text(a, "name")?;
    let position = optional_vec2(a, "position")?.map(|v| local_point(&s.garment.panels[pi], v, text(a, "space") != Some("local")));
    let before = s.garment.clone();
    let panel = s.garment.panels[pi].id;
    let p = &mut s.garment.panels[pi].prints[qi];
    p.image = image.unwrap_or_else(|| p.image.clone());
    p.name = name.unwrap_or_else(|| p.name.clone());
    p.size = size.unwrap_or(p.size);
    p.side = side.unwrap_or(p.side);
    p.position = position.unwrap_or(p.position);
    p.rotation = number_or(a, "rotation", p.rotation);
    p.emboss = number_or(a, "emboss", p.emboss);
    let out = describe(panel, p);
    if s.garment == before {
        return Ok(json!({"changed": false, "print": out}));
    }
    record(s, "Set Print", before);
    Ok(json!({"changed": true, "print": out}))
}

fn delete_schema() -> Value {
    obj(json!({"print": ident("Print id.")}), &["print"])
}

fn delete(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = required_id(a, "print")?;
    let (pi, qi) = find(s, id)?;
    let before = s.garment.clone();
    s.garment.panels[pi].prints.remove(qi);
    record(s, "Delete Print", before);
    Ok(json!({"changed": true, "removed": id}))
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "print.add",
            mcp: "print_add",
            group: Group::Pattern,
            label: "Add Print",
            description: "Place an image on a panel at a position (centre), rotation and size in mm, on the front, back or either face, with an optional emboss height. Render data. The image path must exist now; it is stored as given.",
            keys: &["Shift+I"],
            hints: WRITE,
            schema: add_schema,
            run: add,
        },
        Command {
            id: "print.set",
            mcp: "print_set",
            group: Group::Pattern,
            label: "Set Print",
            description: "Change the image, position, rotation, size, side or emboss of a print. Only the given values change.",
            keys: &[],
            hints: SET,
            schema: set_schema,
            run: set,
        },
        Command {
            id: "print.delete",
            mcp: "print_delete",
            group: Group::Pattern,
            label: "Delete Print",
            description: "Remove a print from its panel.",
            keys: &[],
            hints: DESTROY,
            schema: delete_schema,
            run: delete,
        },
    ]
}
