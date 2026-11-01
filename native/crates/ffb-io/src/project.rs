use std::io::{Cursor, Read, Write};

use ffb_pattern::Garment;
use serde_json::{json, Value};
use zip::write::SimpleFileOptions;

pub const EXTENSION: &str = "ffbrc";
pub const MIME: &str = "application/x-formfabric";
const MANIFEST: &str = "manifest.json";
const GARMENT: &str = "garment.json";
const SCHEMA: u64 = 1;

#[derive(Debug, Clone, Default)]
pub struct Project {
    pub garment: Garment,
    pub unknown: Vec<(String, Vec<u8>)>,
}

fn options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)
}

pub fn save(garment: &Garment, unknown: &[(String, Vec<u8>)], app_version: &str) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let manifest = json!({"format": "formfabric", "schema": SCHEMA, "app_version": app_version});
    let manifest = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    let body = serde_json::to_vec(garment).map_err(|e| e.to_string())?;
    let mut put = |name: &str, data: &[u8]| -> Result<(), String> {
        zip.start_file(name, options()).map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())
    };
    put(MANIFEST, &manifest)?;
    put(GARMENT, &body)?;
    for (name, data) in unknown.iter().filter(|(n, _)| n != MANIFEST && n != GARMENT) {
        put(name, data)?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

pub fn open(bytes: Vec<u8>) -> Result<Project, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a formfabric project (not a zip): {e}"))?;
    let mut manifest = Vec::new();
    zip.by_name(MANIFEST).map_err(|_| format!("the project is missing {MANIFEST}"))?.read_to_end(&mut manifest).map_err(|e| e.to_string())?;
    let manifest: Value = serde_json::from_slice(&manifest).map_err(|e| format!("{MANIFEST} is not JSON: {e}"))?;
    if manifest.get("format").and_then(Value::as_str) != Some("formfabric") {
        return Err("not a formfabric project".into());
    }
    let schema = manifest.get("schema").and_then(Value::as_u64).unwrap_or(0);
    if schema == 0 || schema > SCHEMA {
        return Err(format!("project schema {schema} is not supported (this engine reads schema {SCHEMA})"));
    }
    let mut body = Vec::new();
    zip.by_name(GARMENT).map_err(|_| format!("the project is missing {GARMENT}"))?.read_to_end(&mut body).map_err(|e| e.to_string())?;
    let garment: Garment = serde_json::from_slice(&body).map_err(|e| format!("{GARMENT} is not a garment: {e}"))?;
    let mut unknown = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if entry.is_dir() || name == MANIFEST || name == GARMENT {
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
        unknown.push((name, data));
    }
    Ok(Project { garment, unknown })
}
