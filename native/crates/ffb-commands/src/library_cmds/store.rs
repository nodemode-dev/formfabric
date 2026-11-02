use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{args, CommandError};

pub(crate) const MAX_ITEM: u64 = 256 * 1024 * 1024;
const MAX_INDEX: u64 = 4 * 1024 * 1024;
const MAX_ITEMS: usize = 5000;
const MAX_ROOTS: usize = 8;
const MAX_TAGS: usize = 16;
const INDEX: &str = "index.json";
const CONFIG: &str = "library_roots.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Fabric,
    Trim,
    Style,
    Component,
    Garment,
    Pose,
    Motion,
}

impl Kind {
    pub(crate) const ALL: [Kind; 7] = [Kind::Fabric, Kind::Trim, Kind::Style, Kind::Component, Kind::Garment, Kind::Pose, Kind::Motion];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Kind::Fabric => "fabric",
            Kind::Trim => "trim",
            Kind::Style => "style",
            Kind::Component => "component",
            Kind::Garment => "garment",
            Kind::Pose => "pose",
            Kind::Motion => "motion",
        }
    }

    pub(crate) fn extension(self) -> &'static str {
        match self {
            Kind::Fabric => "ffab",
            Kind::Trim | Kind::Style => "ffstyle",
            Kind::Component => "ffcomp",
            Kind::Garment => "ffbrc",
            Kind::Pose => "ffpose",
            Kind::Motion => "ffmotion",
        }
    }

    pub(crate) fn parse(text: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct Item {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub tags: Vec<String>,
    pub file: String,
    pub thumbnail: Option<String>,
    pub bytes: u64,
    pub created: u64,
}

fn io_error(path: &Path, e: std::io::Error) -> CommandError {
    CommandError::new("io", format!("{}: {e}", path.display()))
}

pub(crate) fn data_dir() -> PathBuf {
    match std::env::var("FORMFABRIC_DATA_DIR") {
        Ok(raw) if !raw.trim().is_empty() => PathBuf::from(raw.trim()),
        _ => PathBuf::from(std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".into())).join("Formfabric"),
    }
}

pub(crate) fn default_root() -> PathBuf {
    data_dir().join("library")
}

fn configured() -> Option<Vec<PathBuf>> {
    let text = std::fs::read_to_string(data_dir().join(CONFIG)).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let list: Vec<PathBuf> = v.get("roots")?.as_array()?.iter().filter_map(Value::as_str).map(PathBuf::from).filter(|p| p.is_absolute()).collect();
    (!list.is_empty()).then_some(list)
}

pub(crate) fn roots() -> Vec<PathBuf> {
    let list = configured().unwrap_or_else(|| vec![default_root()]);
    let mut out: Vec<PathBuf> = Vec::new();
    for (i, p) in list.iter().enumerate() {
        if i == 0 {
            let _ = std::fs::create_dir_all(p);
        }
        if let Ok(c) = p.canonicalize() {
            if c.is_dir() && !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

pub(crate) fn set_roots(paths: &[String]) -> Result<Vec<PathBuf>, CommandError> {
    if paths.len() > MAX_ROOTS {
        return Err(CommandError::new("bad_args", format!("at most {MAX_ROOTS} library folders")));
    }
    let mut list = Vec::new();
    for p in paths {
        let path = args::file_path(&json!({"p": p}), "p", None)?;
        let path = PathBuf::from(path);
        std::fs::create_dir_all(&path).map_err(|e| io_error(&path, e))?;
        let c = path.canonicalize().map_err(|e| io_error(&path, e))?;
        if !c.is_dir() {
            return Err(CommandError::new("bad_args", format!("{} is not a folder", c.display())));
        }
        if !list.contains(&c) {
            list.push(c);
        }
    }
    let file = data_dir().join(CONFIG);
    if list.is_empty() {
        let _ = std::fs::remove_file(&file);
        return Ok(roots());
    }
    let body = serde_json::to_vec_pretty(&json!({"roots": list.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()})).map_err(|e| CommandError::new("io", e.to_string()))?;
    std::fs::create_dir_all(data_dir()).map_err(|e| io_error(&data_dir(), e))?;
    args::write_atomic(&file.to_string_lossy(), &body)?;
    Ok(roots())
}

fn plain(rel: &str) -> bool {
    !rel.is_empty() && !rel.contains('\0') && !rel.starts_with('/') && !rel.contains('\\') && rel.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn read_index(root: &Path) -> Vec<Item> {
    let file = root.join(INDEX);
    let Ok(meta) = std::fs::metadata(&file) else { return Vec::new() };
    if meta.len() > MAX_INDEX {
        return Vec::new();
    }
    let Ok(text) = std::fs::read_to_string(&file) else { return Vec::new() };
    let Ok(v) = serde_json::from_str::<Value>(&text) else { return Vec::new() };
    let items: Vec<Item> = v.get("items").and_then(|i| serde_json::from_value(i.clone()).ok()).unwrap_or_default();
    items.into_iter().filter(|i| plain(&i.file) && Kind::parse(&i.kind).is_some() && i.thumbnail.as_deref().is_none_or(plain)).take(MAX_ITEMS).collect()
}

fn write_index(root: &Path, items: &[Item]) -> Result<(), CommandError> {
    let body = serde_json::to_vec_pretty(&json!({"format": "formfabric-library", "schema": 1, "items": items})).map_err(|e| CommandError::new("io", e.to_string()))?;
    args::write_atomic(&root.join(INDEX).to_string_lossy(), &body)
}

pub(crate) fn resolve(root: &Path, rel: &str) -> Result<PathBuf, CommandError> {
    if !plain(rel) {
        return Err(CommandError::new("bad_args", format!("{rel} is not a library file name")));
    }
    let full = root.join(rel);
    let canon = full.canonicalize().map_err(|e| CommandError::new(if e.kind() == std::io::ErrorKind::NotFound { "not_found" } else { "io" }, format!("{}: {e}", full.display())))?;
    if !canon.starts_with(root) {
        return Err(CommandError::new("outside_library", format!("{} is outside the library folders", canon.display())));
    }
    Ok(canon)
}

pub(crate) fn inside_roots(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let canon = path.canonicalize().ok()?;
    roots().into_iter().find(|r| canon.starts_with(r)).map(|r| (r, canon))
}

fn scan(root: &Path, items: &mut Vec<Item>) {
    for kind in Kind::ALL {
        let Ok(dir) = std::fs::read_dir(root.join(kind.name())) else { continue };
        for entry in dir.flatten().take(MAX_ITEMS) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(&format!(".{}", kind.extension())) else { continue };
            let file = format!("{}/{name}", kind.name());
            if items.iter().any(|i| i.file == file) || !entry.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }
            let bytes = entry.metadata().map_or(0, |m| m.len());
            items.push(Item { id: format!("{}/{stem}", kind.name()), kind: kind.name().into(), name: stem.into(), file, bytes, ..Item::default() });
        }
    }
}

pub(crate) fn all_items() -> Vec<(PathBuf, Item)> {
    let mut out: Vec<(PathBuf, Item)> = Vec::new();
    for root in roots() {
        let mut items = read_index(&root);
        scan(&root, &mut items);
        for item in items {
            if root.join(&item.file).is_file() && !out.iter().any(|(_, o)| o.id == item.id) {
                out.push((root.clone(), item));
            }
        }
    }
    out
}

pub(crate) fn item_json(root: &Path, item: &Item) -> Value {
    json!({
        "id": item.id,
        "kind": item.kind,
        "name": item.name,
        "tags": item.tags,
        "path": root.join(&item.file).to_string_lossy(),
        "thumbnail": item.thumbnail.as_ref().map(|t| root.join(t).to_string_lossy().into_owned()).filter(|t| Path::new(t).is_file()),
        "bytes": item.bytes,
        "created": item.created,
        "root": root.to_string_lossy()
    })
}

pub(crate) fn find(id: &str) -> Result<(PathBuf, Item), CommandError> {
    all_items().into_iter().find(|(_, i)| i.id == id).ok_or_else(|| CommandError::new("not_found", format!("no library item {id}")))
}

pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>, CommandError> {
    let meta = std::fs::metadata(path).map_err(|e| io_error(path, e))?;
    if meta.len() > MAX_ITEM {
        return Err(CommandError::new("too_large", format!("{} is larger than {} MiB", path.display(), MAX_ITEM / 1024 / 1024)));
    }
    std::fs::read(path).map_err(|e| io_error(path, e))
}

pub(crate) fn locate(reference: &str, kinds: &[Kind]) -> Result<(PathBuf, Option<Item>), CommandError> {
    if reference.starts_with('/') || reference.chars().nth(1) == Some(':') {
        let path = PathBuf::from(args::file_path(&json!({"p": reference}), "p", None)?);
        let (_, canon) = inside_roots(&path).ok_or_else(|| CommandError::new("outside_library", format!("{reference} is not inside a library folder")))?;
        let ext = canon.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !kinds.iter().any(|k| k.extension().eq_ignore_ascii_case(ext)) {
            return Err(CommandError::new("bad_args", format!("{reference} is not a {} file", kinds.iter().map(|k| k.name()).collect::<Vec<_>>().join(" or "))));
        }
        return Ok((canon, None));
    }
    let (root, item) = find(reference)?;
    let kind = Kind::parse(&item.kind).unwrap_or(Kind::Garment);
    if !kinds.contains(&kind) {
        return Err(CommandError::new("bad_args", format!("{reference} is a {} item, not a {}", item.kind, kinds.iter().map(|k| k.name()).collect::<Vec<_>>().join(" or "))));
    }
    let path = resolve(&root, &item.file)?;
    Ok((path, Some(item)))
}

pub(crate) struct Reserved {
    pub root: PathBuf,
    pub kind: Kind,
    pub slug: String,
    pub file: String,
    pub abs: PathBuf,
}

fn slug(name: &str, kind: Kind) -> String {
    let mut s: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c.to_ascii_lowercase() } else { '-' }).collect();
    s = s.trim_matches('-').to_string();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s.truncate(60);
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        kind.name().to_string()
    } else {
        s
    }
}

pub(crate) fn reserve(kind: Kind, name: &str) -> Result<Reserved, CommandError> {
    let root = roots().into_iter().next().ok_or_else(|| CommandError::new("io", "the library folder could not be created"))?;
    if read_index(&root).len() >= MAX_ITEMS {
        return Err(CommandError::new("limit", format!("a library folder holds at most {MAX_ITEMS} items")));
    }
    let dir = root.join(kind.name());
    std::fs::create_dir_all(&dir).map_err(|e| io_error(&dir, e))?;
    let base = slug(name, kind);
    let mut n = 1;
    loop {
        let stem = if n == 1 { base.clone() } else { format!("{base}-{n}") };
        let file = format!("{}/{stem}.{}", kind.name(), kind.extension());
        if !root.join(&file).exists() {
            let abs = root.join(&file);
            return Ok(Reserved { root, kind, slug: stem, file, abs });
        }
        n += 1;
        if n > 9999 {
            return Err(CommandError::new("limit", "too many items with this name"));
        }
    }
}

pub(crate) fn register(r: &Reserved, name: &str, tags: Vec<String>, thumb: Option<Vec<u8>>) -> Result<Value, CommandError> {
    let bytes = std::fs::metadata(&r.abs).map_err(|e| io_error(&r.abs, e))?.len();
    let mut thumbnail = None;
    if let Some(png) = thumb {
        let rel = format!("thumbs/{}-{}.png", r.kind.name(), r.slug);
        let dir = r.root.join("thumbs");
        std::fs::create_dir_all(&dir).map_err(|e| io_error(&dir, e))?;
        args::write_atomic(&r.root.join(&rel).to_string_lossy(), &png)?;
        thumbnail = Some(rel);
    }
    let created = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let item = Item { id: format!("{}/{}", r.kind.name(), r.slug), kind: r.kind.name().into(), name: name.into(), tags, file: r.file.clone(), thumbnail, bytes, created };
    let mut items = read_index(&r.root);
    items.retain(|i| i.file != item.file);
    items.push(item.clone());
    write_index(&r.root, &items)?;
    Ok(item_json(&r.root, &item))
}

pub(crate) fn add_bytes(kind: Kind, name: &str, tags: Vec<String>, bytes: &[u8], thumb: Option<Vec<u8>>) -> Result<Value, CommandError> {
    if bytes.len() as u64 > MAX_ITEM {
        return Err(CommandError::new("too_large", format!("the item is larger than {} MiB", MAX_ITEM / 1024 / 1024)));
    }
    let r = reserve(kind, name)?;
    args::write_atomic(&r.abs.to_string_lossy(), bytes)?;
    register(&r, name, tags, thumb)
}

pub(crate) fn remove(id: &str) -> Result<Value, CommandError> {
    let (root, item) = find(id)?;
    let file = resolve(&root, &item.file)?;
    std::fs::remove_file(&file).map_err(|e| io_error(&file, e))?;
    if let Some(t) = &item.thumbnail {
        if let Ok(p) = resolve(&root, t) {
            let _ = std::fs::remove_file(p);
        }
    }
    let mut items = read_index(&root);
    items.retain(|i| i.file != item.file);
    write_index(&root, &items)?;
    Ok(json!({"removed": id}))
}

pub(crate) fn clean_tags(a: &Value) -> Result<Vec<String>, CommandError> {
    let Some(list) = a.get("tags") else { return Ok(Vec::new()) };
    let list = list.as_array().ok_or_else(|| CommandError::new("bad_args", "tags must be an array of strings"))?;
    if list.len() > MAX_TAGS {
        return Err(CommandError::new("bad_args", format!("at most {MAX_TAGS} tags")));
    }
    let mut out = Vec::new();
    for t in list {
        let t = t.as_str().ok_or_else(|| CommandError::new("bad_args", "tags must be strings"))?.trim();
        if t.is_empty() || t.chars().count() > 40 || args::has_control(t) {
            return Err(CommandError::new("bad_args", "a tag is 1 to 40 characters without control characters"));
        }
        if !out.iter().any(|x: &String| x == t) {
            out.push(t.to_string());
        }
    }
    Ok(out)
}
