use ffb_pattern::Garment;
use serde_json::{json, Value};

use crate::{args, b64, Command, CommandError, Edit, Group, Session, DESTROY, READ, WRITE};

const PREFIX: &str = "snapshot/";
const MAX_SNAPSHOTS: usize = 64;
const MAX_ENTRY: usize = 15 * 1024 * 1024;
const MAX_TOTAL: usize = 256 * 1024 * 1024;
const MAX_PROJECT: usize = 1024 * 1024 * 1024 - 64 * 1024 * 1024;
const FORMAT: &str = "formfabric-snapshot";

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn json_name(id: u32) -> String {
    format!("{PREFIX}{id}.json")
}

fn png_name(id: u32) -> String {
    format!("{PREFIX}{id}.png")
}

fn parse_name(entry: &str) -> Option<(u32, bool)> {
    let rest = entry.strip_prefix(PREFIX)?;
    let (stem, ext) = rest.rsplit_once('.')?;
    let id: u32 = stem.parse().ok().filter(|n| *n >= 1)?;
    match ext {
        "json" => Some((id, true)),
        "png" => Some((id, false)),
        _ => None,
    }
}

fn ids(s: &Session) -> Vec<u32> {
    let mut out: Vec<u32> = s.unknown.iter().filter_map(|(n, _)| parse_name(n)).filter(|(_, j)| *j).map(|(i, _)| i).collect();
    out.sort_unstable();
    out
}

fn entry<'a>(s: &'a Session, name: &str) -> Option<&'a Vec<u8>> {
    s.unknown.iter().find(|(n, _)| n == name).map(|(_, d)| d)
}

fn put(s: &mut Session, name: String, data: Vec<u8>) {
    s.unknown.retain(|(n, _)| *n != name);
    s.unknown.push((name, data));
}

#[cfg(not(target_arch = "wasm32"))]
fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

#[cfg(target_arch = "wasm32")]
fn now() -> u64 {
    0
}

fn snapshot_id(a: &Value) -> Result<u32, CommandError> {
    a.get("id").and_then(Value::as_u64).and_then(|x| u32::try_from(x).ok()).filter(|x| *x >= 1).ok_or_else(|| bad("id must be a snapshot id"))
}

fn save_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 1, "maxLength": 120, "description": "Snapshot name. Defaults to Snapshot N."},
            "thumbnail": {"type": "string", "description": "Optional PNG of the 3D window as base64 (a data URL is accepted), at most 1 MiB decoded."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn save(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let existing = ids(s);
    if existing.len() >= MAX_SNAPSHOTS {
        return Err(CommandError::new("limit", format!("a project keeps at most {MAX_SNAPSHOTS} snapshots; remove one first")));
    }
    let id = existing.last().copied().unwrap_or(0).checked_add(1).filter(|n| *n < u32::MAX / 2).or_else(|| (1..=u32::MAX / 2).find(|n| existing.binary_search(n).is_err())).ok_or_else(|| CommandError::new("limit", "no free snapshot id; remove a snapshot first"))?;
    let name = match a.get("name").and_then(Value::as_str) {
        Some(n) => {
            args::clean_name("name", n)?;
            n.trim().to_string()
        }
        None => format!("Snapshot {id}"),
    };
    if name.is_empty() {
        return Err(bad("name must not be empty"));
    }
    let png = match a.get("thumbnail").and_then(Value::as_str) {
        Some(t) => Some(b64::png_from_base64(t).map_err(bad)?),
        None => None,
    };
    let created = now();
    let garment = serde_json::to_value(&s.garment).map_err(|e| CommandError::new("save_failed", e.to_string()))?;
    let body = serde_json::to_vec(&json!({"format": FORMAT, "schema": 1, "id": id, "name": name, "created": created, "panels": s.garment.panels.len(), "garment": garment}))
        .map_err(|e| CommandError::new("save_failed", e.to_string()))?;
    if body.len() > MAX_ENTRY {
        return Err(CommandError::new("too_large", format!("the garment is {} MiB as JSON; a snapshot holds at most {} MiB", body.len() / 1024 / 1024, MAX_ENTRY / 1024 / 1024)));
    }
    let total: usize = s.unknown.iter().filter(|(n, _)| parse_name(n).is_some()).map(|(_, d)| d.len()).sum();
    if total + body.len() > MAX_TOTAL {
        return Err(CommandError::new("limit", "the snapshots of this project would exceed 256 MiB; remove some first"));
    }
    let held: usize = s.unknown.iter().filter(|(n, _)| *n != json_name(id) && *n != png_name(id)).map(|(_, d)| d.len()).sum();
    if held.saturating_add(body.len()).saturating_add(png.as_ref().map_or(0, Vec::len)) > MAX_PROJECT {
        return Err(CommandError::new("limit", "the project would exceed the 1 GiB project limit and could no longer be opened; remove snapshots or motions first"));
    }
    let bytes = body.len();
    put(s, json_name(id), body);
    if let Some(png) = png {
        put(s, png_name(id), png);
    }
    s.epoch += 1;
    Ok(json!({"id": id, "name": name, "created": created, "panels": s.garment.panels.len(), "bytes": bytes, "thumbnail": entry(s, &png_name(id)).is_some()}))
}

fn list_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "thumbnails": {"type": "boolean", "default": false, "description": "Include each thumbnail as base64 PNG in thumbnail_png. Pair with ids to fetch one at a time."},
            "ids": {"type": "array", "items": {"type": "integer", "minimum": 1}, "maxItems": 64, "description": "Only these snapshots."}
        },
        "required": [],
        "additionalProperties": false
    })
}

fn list(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let with = a.get("thumbnails").and_then(Value::as_bool).unwrap_or(false);
    let only: Option<Vec<u64>> = a.get("ids").and_then(Value::as_array).map(|l| l.iter().filter_map(Value::as_u64).collect());
    let mut items = Vec::new();
    for id in ids(s) {
        if only.as_ref().is_some_and(|o| !o.contains(&u64::from(id))) {
            continue;
        }
        let Some(data) = entry(s, &json_name(id)) else { continue };
        let Ok(v) = serde_json::from_slice::<Value>(data) else { continue };
        let png = entry(s, &png_name(id));
        let mut item = json!({
            "id": id,
            "name": v.get("name").cloned().unwrap_or(json!(format!("Snapshot {id}"))),
            "created": v.get("created").cloned().unwrap_or(json!(0)),
            "panels": v.get("panels").cloned().unwrap_or(json!(0)),
            "bytes": data.len(),
            "thumbnail": png.is_some()
        });
        if with {
            if let Some(png) = png {
                item["thumbnail_png"] = json!(b64::encode(png));
            }
        }
        items.push(item);
    }
    Ok(json!({"snapshots": items}))
}

fn id_schema() -> Value {
    json!({"type": "object", "properties": {"id": {"type": "integer", "minimum": 1, "description": "Snapshot id from snapshot.list."}}, "required": ["id"], "additionalProperties": false})
}

fn restore(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = snapshot_id(a)?;
    let data = entry(s, &json_name(id)).ok_or_else(|| CommandError::new("not_found", format!("no snapshot with id {id}")))?;
    let v: Value = serde_json::from_slice(data).map_err(|e| CommandError::new("open_failed", format!("snapshot {id} is not JSON: {e}")))?;
    let name = v.get("name").and_then(Value::as_str).unwrap_or("Snapshot").to_string();
    let garment: Garment = serde_json::from_value(v.get("garment").cloned().ok_or_else(|| CommandError::new("open_failed", format!("snapshot {id} holds no garment")))?)
        .map_err(|e| CommandError::new("open_failed", format!("snapshot {id} is not a garment: {e}")))?;
    let before = s.garment.clone();
    let changed = before != garment;
    if changed {
        s.garment = garment.clone();
        s.record(Edit::Garment { label: format!("Restore {name}"), before: Box::new(before), after: Box::new(garment) });
    }
    Ok(json!({"id": id, "name": name, "panels": s.garment.panels.len(), "changed": changed}))
}

fn remove(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let id = snapshot_id(a)?;
    if entry(s, &json_name(id)).is_none() {
        return Err(CommandError::new("not_found", format!("no snapshot with id {id}")));
    }
    let (j, p) = (json_name(id), png_name(id));
    s.unknown.retain(|(n, _)| *n != j && *n != p);
    s.epoch += 1;
    Ok(json!({"removed": id}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![
        Command {
            id: "snapshot.save",
            mcp: "snapshot_save",
            group: Group::File,
            label: "Save Snapshot",
            description: "Store the whole garment state as a snapshot inside the project (entry snapshot/ID.json, plus snapshot/ID.png when a thumbnail is given as base64 PNG, at most 1 MiB). Does not change the garment, so it is not an undo step. At most 64 snapshots per project.",
            keys: &[],
            hints: WRITE,
            schema: save_schema,
            run: save,
        },
        Command {
            id: "snapshot.list",
            mcp: "snapshot_list",
            group: Group::File,
            label: "Snapshots",
            description: "List the snapshots stored in the project: id, name, created (unix seconds), panels, bytes and whether a thumbnail exists. With thumbnails true each item carries thumbnail_png (base64).",
            keys: &[],
            hints: READ,
            schema: list_schema,
            run: list,
        },
        Command {
            id: "snapshot.restore",
            mcp: "snapshot_restore",
            group: Group::File,
            label: "Restore Snapshot",
            description: "Replace the garment with a snapshot's state. One undo step; bodies, the history and the other snapshots are kept.",
            keys: &[],
            hints: WRITE,
            schema: id_schema,
            run: restore,
        },
        Command {
            id: "snapshot.remove",
            mcp: "snapshot_remove",
            group: Group::File,
            label: "Remove Snapshot",
            description: "Delete a snapshot and its thumbnail from the project. Not an undo step.",
            keys: &[],
            hints: DESTROY,
            schema: id_schema,
            run: remove,
        },
    ]
}
