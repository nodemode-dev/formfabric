use std::io::{Cursor, Read, Write};

use serde_json::{json, Value};
use zip::write::SimpleFileOptions;

use crate::validate;
use crate::Fabric;

pub const EXTENSION: &str = "ffab";
pub const FORMAT: &str = "formfabric-fabric";
pub const SCHEMA: u64 = 1;
pub const ENTRY: &str = "fabric.json";
pub const MAP_DIR: &str = "maps/";
pub const MAX_IMAGE: u64 = 64 * 1024 * 1024;
pub const MAX_TOTAL: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct Ffab {
    pub fabric: Fabric,
    pub images: Vec<(String, Vec<u8>)>,
}

pub fn safe_entry(name: &str) -> bool {
    name.starts_with(MAP_DIR)
        && name.len() > MAP_DIR.len()
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.split('/').any(|part| part == ".." || part == ".")
        && !name[MAP_DIR.len()..].contains('/')
}

pub fn file_stem(reference: &str) -> String {
    let base = reference.rsplit(['/', '\\']).next().unwrap_or(reference);
    let cleaned: String = base.chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' }).collect();
    let cleaned = cleaned.trim_start_matches('.').to_string();
    if cleaned.is_empty() {
        "map".into()
    } else {
        cleaned
    }
}

fn options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)
}

pub fn pack(fabric: &Fabric, images: &[(String, Vec<u8>)], app_version: &str) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let body = json!({"format": FORMAT, "schema": SCHEMA, "app_version": app_version, "fabric": fabric});
    let body = serde_json::to_vec_pretty(&body).map_err(|e| e.to_string())?;
    zip.start_file(ENTRY, options()).map_err(|e| e.to_string())?;
    zip.write_all(&body).map_err(|e| e.to_string())?;
    for (name, data) in images {
        if !safe_entry(name) {
            return Err(format!("map entry name {name} is not allowed"));
        }
        zip.start_file(name.as_str(), SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)).map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

pub fn unpack(bytes: Vec<u8>) -> Result<Ffab, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a fabric file (not a zip): {e}"))?;
    let mut body = Vec::new();
    zip.by_name(ENTRY).map_err(|_| format!("the fabric file is missing {ENTRY}"))?.read_to_end(&mut body).map_err(|e| e.to_string())?;
    let body: Value = serde_json::from_slice(&body).map_err(|e| format!("{ENTRY} is not JSON: {e}"))?;
    if body.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err("not a formfabric fabric file".into());
    }
    let schema = body.get("schema").and_then(Value::as_u64).unwrap_or(0);
    if schema == 0 || schema > SCHEMA {
        return Err(format!("fabric file schema {schema} is not supported (this engine reads schema {SCHEMA})"));
    }
    let fabric: Fabric = serde_json::from_value(body.get("fabric").cloned().ok_or("fabric.json has no fabric")?).map_err(|e| format!("fabric.json is not a fabric: {e}"))?;
    validate::check_fabric(&fabric)?;
    let mut images = Vec::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if entry.is_dir() || name == ENTRY {
            continue;
        }
        if !safe_entry(&name) {
            return Err(format!("the fabric file holds an entry that is not allowed: {name}"));
        }
        if entry.size() > MAX_IMAGE {
            return Err(format!("{name} is larger than {} MiB", MAX_IMAGE / 1024 / 1024));
        }
        total += entry.size();
        if total > MAX_TOTAL {
            return Err(format!("the maps are larger than {} MiB together", MAX_TOTAL / 1024 / 1024));
        }
        let mut data = Vec::new();
        entry.take(MAX_IMAGE + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
        if data.len() as u64 > MAX_IMAGE {
            return Err(format!("{name} is larger than {} MiB", MAX_IMAGE / 1024 / 1024));
        }
        images.push((name, data));
    }
    for r in validate::map_refs(&fabric.look) {
        if !images.iter().any(|(n, _)| *n == r) {
            return Err(format!("the fabric names the map {r} but the file does not hold it"));
        }
    }
    Ok(Ffab { fabric, images })
}
