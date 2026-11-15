use ffb_fabric::ffab;
use ffb_fabric::validate;
use serde_json::{json, Value};

use super::common::*;
use super::schema as sc;
use crate::{args, Command, CommandError, Group, Session, EXTERNAL};

const MAP_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "webp", "tif", "tiff", "bmp", "gif"];

fn export_schema() -> Value {
    sc::obj(json!({"fabric": sc::fabric_id("Fabric to export."), "path": sc::path("File path of the .ffab file to write.")}), &["fabric", "path"])
}

fn read_map(s: &Session, reference: &str) -> Result<Vec<u8>, CommandError> {
    if let Some((_, data)) = s.unknown.iter().find(|(n, _)| n == reference) {
        return Ok(data.clone());
    }
    let image = std::path::Path::new(reference).extension().and_then(|e| e.to_str()).is_some_and(|e| MAP_EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)));
    if !image || !std::fs::metadata(reference).is_ok_and(|m| m.is_file() && m.len() <= ffab::MAX_IMAGE) {
        return Err(bad(format!("the map {reference} is not an image file ({}) of at most {} MiB", MAP_EXTENSIONS.join(", "), ffab::MAX_IMAGE / 1024 / 1024)));
    }
    std::fs::read(reference).map_err(|e| CommandError::new(if e.kind() == std::io::ErrorKind::NotFound { "not_found" } else { "io" }, format!("could not read the map {reference}: {e}")))
}

fn export_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = id_arg(a, "fabric")?;
    let path = args::file_path(a, "path", Some(ffab::EXTENSION))?;
    let mut fabric = fabric_of(&s.garment, id)?.clone();
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();
    let mut failure: Option<CommandError> = None;
    validate::rewrite_maps(&mut fabric.look, &mut |reference| {
        match read_map(s, reference) {
            Ok(data) => {
                let stem = ffab::file_stem(reference);
                let mut name = format!("{}{stem}", ffab::MAP_DIR);
                let mut n = 1;
                while images.iter().any(|(existing, d)| *existing == name && *d != data) {
                    n += 1;
                    name = format!("{}{n}_{stem}", ffab::MAP_DIR);
                }
                if !images.iter().any(|(existing, _)| *existing == name) {
                    images.push((name.clone(), data));
                }
                Some(name)
            }
            Err(e) => {
                failure.get_or_insert(e);
                None
            }
        }
    });
    if let Some(e) = failure {
        return Err(e);
    }
    fabric.id = 1;
    let bytes = ffab::pack(&fabric, &images, ffb_core::ENGINE_VERSION).map_err(|e| CommandError::new("save_failed", e))?;
    args::write_atomic(&path, &bytes)?;
    Ok(json!({"path": path, "bytes": bytes.len(), "maps": images.len(), "fabric": id}))
}

fn import_schema() -> Value {
    sc::obj(json!({"path": sc::path("File path of the .ffab file to read."), "name": sc::name("Name for the imported fabric. Defaults to the name in the file.")}), &["path"])
}

fn import_run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::file_path(a, "path", None)?;
    let name = text(a, "name")?;
    let bytes = std::fs::read(&path).map_err(|e| CommandError::new(if e.kind() == std::io::ErrorKind::NotFound { "not_found" } else { "io" }, format!("could not read {path}: {e}")))?;
    let parsed = ffab::unpack(bytes).map_err(|e| CommandError::new("open_failed", e))?;
    let id = next_fabric_id(&s.garment);
    let mut fabric = parsed.fabric;
    fabric.id = id;
    if let Some(n) = name {
        fabric.name = n;
    }
    let mut stored: Vec<(String, Vec<u8>)> = Vec::new();
    let rename = |old: &str| format!("{}f{id}_{}", ffab::MAP_DIR, &old[ffab::MAP_DIR.len().min(old.len())..]);
    validate::rewrite_maps(&mut fabric.look, &mut |reference| {
        let target = rename(reference);
        if let Some((_, data)) = parsed.images.iter().find(|(n, _)| n == reference) {
            if !stored.iter().any(|(n, _)| *n == target) {
                stored.push((target.clone(), data.clone()));
            }
        }
        Some(target)
    });
    let maps = stored.len();
    let out = mutate(s, "Import Fabric", |g| {
        g.fabrics.push(fabric.clone());
        ensure_default(g);
        Ok(json!({"fabric": fabric, "maps": maps}))
    })?;
    for (name, data) in stored {
        s.unknown.retain(|(n, _)| *n != name);
        s.unknown.push((name, data));
    }
    Ok(out)
}

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "fabric.export",
            mcp: "fabric_export",
            group: Group::Fabric,
            label: "Export Fabric",
            description: "Write a fabric to a .ffab file: a zip with fabric.json and the map images it uses under maps/.",
            keys: &[],
            hints: EXTERNAL,
            schema: export_schema,
            run: export_run,
        },
        Command {
            id: "fabric.import",
            mcp: "fabric_import",
            group: Group::Fabric,
            label: "Import Fabric",
            description: "Add the fabric in a .ffab file to the garment as a new fabric. Its map images are stored inside the project and saved with it.",
            keys: &[],
            hints: EXTERNAL,
            schema: import_schema,
            run: import_run,
        },
    ]
}
