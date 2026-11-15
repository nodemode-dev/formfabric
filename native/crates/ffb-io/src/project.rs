use std::io::{Cursor, Read, Write};

use ffb_pattern::Garment;
use serde_json::{json, Value};
use zip::write::SimpleFileOptions;

pub const EXTENSION: &str = "ffbrc";
pub const MIME: &str = "application/x-formfabric";
const MANIFEST: &str = "manifest.json";
const GARMENT: &str = "garment.json";
const SCHEMA: u64 = 1;

type Parts = (Option<Vec<u8>>, Option<Vec<u8>>);

#[derive(Debug, Clone, Default)]
pub struct Project {
    pub garment: Garment,
    pub unknown: Vec<(String, Vec<u8>)>,
    pub bodies: Vec<ffb_body::Body>,
}

fn options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)
}

pub fn save(garment: &Garment, unknown: &[(String, Vec<u8>)], app_version: &str) -> Result<Vec<u8>, String> {
    save_with_bodies(garment, &[], unknown, app_version)
}

pub fn save_with_bodies(garment: &Garment, bodies: &[ffb_body::Body], unknown: &[(String, Vec<u8>)], app_version: &str) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let manifest = json!({"format": "formfabric", "schema": SCHEMA, "app_version": app_version});
    let manifest = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    let body = serde_json::to_vec(garment).map_err(|e| e.to_string())?;
    serde_json::from_slice::<Garment>(&body).map_err(|e| format!("the garment was not saved because it would not open again; it most likely holds a number that is not finite ({e}); undo the last change and save again"))?;
    let mut put =|name: &str, data: &[u8]| -> Result<(), String> {
        zip.start_file(name, options()).map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())
    };
    put(MANIFEST, &manifest)?;
    put(GARMENT, &body)?;
    for body in bodies {
        let (json, bin) = crate::body::encode(body)?;
        crate::body::decode(&json, &bin).map_err(|e| format!("body {} was not saved because it would not open again: {e}", body.id))?;
        put(&crate::body::json_name(body.id), &json)?;
        put(&crate::body::bin_name(body.id), &bin)?;
    }
    for (name, data) in unknown.iter().filter(|(n, _)| n != MANIFEST && n != GARMENT && crate::body::parse_id(n).is_none()) {
        put(name, data)?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

pub const MAX_ENTRIES: usize = 4096;
pub const MAX_JSON: u64 = 16 * 1024 * 1024;
pub const MAX_BLOB: u64 = 512 * 1024 * 1024;
pub const MAX_TOTAL: u64 = 1024 * 1024 * 1024;

fn read_bounded(entry: &mut impl Read, name: &str, declared: u64, spent: &mut u64) -> Result<Vec<u8>, String> {
    let limit = if name.ends_with(".json") { MAX_JSON } else { MAX_BLOB };
    if declared > limit {
        return Err(format!("{name} is larger than {} MiB uncompressed", limit / 1024 / 1024));
    }
    let mut data = Vec::new();
    entry.take(limit + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() as u64 > limit {
        return Err(format!("{name} is larger than {} MiB uncompressed", limit / 1024 / 1024));
    }
    *spent += data.len() as u64;
    if *spent > MAX_TOTAL {
        return Err(format!("the project is larger than {} MiB uncompressed", MAX_TOTAL / 1024 / 1024));
    }
    Ok(data)
}

pub fn open(bytes: Vec<u8>) -> Result<Project, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a formfabric project (not a zip): {e}"))?;
    if zip.len() > MAX_ENTRIES {
        return Err(format!("the project holds {} entries; the limit is {MAX_ENTRIES}", zip.len()));
    }
    let mut spent = 0u64;
    let manifest = {
        let mut entry = zip.by_name(MANIFEST).map_err(|_| format!("the project is missing {MANIFEST}"))?;
        let declared = entry.size();
        read_bounded(&mut entry, MANIFEST, declared, &mut spent)?
    };
    let manifest: Value = serde_json::from_slice(&manifest).map_err(|e| format!("{MANIFEST} is not JSON: {e}"))?;
    if manifest.get("format").and_then(Value::as_str) != Some("formfabric") {
        return Err("not a formfabric project".into());
    }
    let schema = manifest.get("schema").and_then(Value::as_u64).unwrap_or(0);
    if schema == 0 || schema > SCHEMA {
        return Err(format!("project schema {schema} is not supported (this engine reads schema {SCHEMA})"));
    }
    let body = {
        let mut entry = zip.by_name(GARMENT).map_err(|_| format!("the project is missing {GARMENT}"))?;
        let declared = entry.size();
        read_bounded(&mut entry, GARMENT, declared, &mut spent)?
    };
    let garment: Garment = serde_json::from_slice(&body).map_err(|e| format!("{GARMENT} is not a garment: {e}"))?;
    let mut unknown = Vec::new();
    let mut parts: std::collections::BTreeMap<u32, Parts> = std::collections::BTreeMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if entry.is_dir() || name == MANIFEST || name == GARMENT {
            continue;
        }
        let declared = entry.size();
        let data = read_bounded(&mut entry, &name, declared, &mut spent)?;
        match crate::body::parse_id(&name) {
            Some((id, true)) => parts.entry(id).or_default().0 = Some(data),
            Some((id, false)) => parts.entry(id).or_default().1 = Some(data),
            None => unknown.push((name, data)),
        }
    }
    let mut bodies = Vec::new();
    for (id, (json, bin)) in parts {
        let (Some(json), Some(bin)) = (json, bin) else { return Err(format!("body {id} is missing its json or bin entry")) };
        let mut body = crate::body::decode(&json, &bin)?;
        body.id = id;
        bodies.push(body);
    }
    Ok(Project { garment, unknown, bodies })
}
