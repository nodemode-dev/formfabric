use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{args, CommandError};

pub(crate) const MAX_ITEM: u64 = 256 * 1024 * 1024;
const MAX_INDEX: u64 = 16 * 1024 * 1024;
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

const DENY_TREES: [&str; 16] = ["bin", "boot", "dev", "etc", "lib", "lib32", "lib64", "libx32", "proc", "sbin", "sys", "usr", "system", "library", "run", "snap"];
const DENY_TOP: [&str; 12] = ["home", "users", "root", "var", "opt", "mnt", "media", "volumes", "applications", "private", "tmp", "srv"];
const DENY_WINDOWS: [&str; 6] = ["windows", "program files", "program files (x86)", "programdata", "system volume information", "$recycle.bin"];

pub(crate) fn refuse_folder(p: &Path) -> Result<(), CommandError> {
    let no = |why: &str| Err(CommandError::new("bad_args", format!("{} {why}", p.display())));
    let mut names: Vec<String> = Vec::new();
    let mut drive = false;
    for c in p.components() {
        match c {
            std::path::Component::Prefix(_) => drive = true,
            std::path::Component::Normal(n) => names.push(n.to_string_lossy().to_lowercase()),
            std::path::Component::RootDir => {}
            _ => return no("is not a plain absolute folder"),
        }
    }
    let Some(first) = names.first() else { return no("is the filesystem root and cannot be a library folder") };
    if drive {
        if DENY_WINDOWS.contains(&first.as_str()) || (names.len() == 1 && first == "users") {
            return no("is a system folder and cannot be a library folder");
        }
    } else if DENY_TREES.contains(&first.as_str()) || (names.len() == 1 && DENY_TOP.contains(&first.as_str())) || (names.len() == 2 && first == "private" && matches!(names[1].as_str(), "etc" | "var" | "tmp")) {
        return no("is a system folder and cannot be a library folder");
    }
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(PathBuf::from).and_then(|h| h.canonicalize().ok());
    if home.as_deref() == Some(p) {
        return no("is the home folder; choose a folder inside it");
    }
    let data = data_dir();
    let data = data.canonicalize().unwrap_or(data);
    if data.starts_with(p) {
        return no("contains the application data folder and cannot be a library folder");
    }
    Ok(())
}

pub(crate) fn future_path(path: &Path) -> Result<PathBuf, CommandError> {
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::CurDir)) {
        return Err(CommandError::new("bad_args", format!("{} must not contain . or .. parts", path.display())));
    }
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let mut at = path.to_path_buf();
    while !at.exists() {
        let name = at.file_name().ok_or_else(|| CommandError::new("bad_args", format!("{} has no existing parent folder", path.display())))?.to_os_string();
        tail.push(name);
        if !at.pop() {
            return Err(CommandError::new("bad_args", format!("{} has no existing parent folder", path.display())));
        }
    }
    let mut base = at.canonicalize().map_err(|e| io_error(&at, e))?;
    for name in tail.into_iter().rev() {
        base.push(name);
    }
    Ok(base)
}

pub(crate) fn roots() -> Vec<PathBuf> {
    let list = configured().unwrap_or_else(|| vec![default_root()]);
    let mut out: Vec<PathBuf> = Vec::new();
    for (i, p) in list.iter().enumerate() {
        if i == 0 && future_path(p).is_ok_and(|f| refuse_folder(&f).is_ok()) {
            let _ = std::fs::create_dir_all(p);
        }
        if let Ok(c) = p.canonicalize() {
            if c.is_dir() && !out.contains(&c) && refuse_folder(&c).is_ok() {
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
        refuse_folder(&future_path(&path)?)?;
        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(|e| io_error(&path, e))?;
        }
        let c = path.canonicalize().map_err(|e| io_error(&path, e))?;
        refuse_folder(&c)?;
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

fn tied(mut i: Item) -> Option<Item> {
    let kind = Kind::parse(&i.kind)?;
    let (folder, base) = i.file.split_once('/')?;
    let stem = base.strip_suffix(&format!(".{}", kind.extension()))?;
    if folder != kind.name() || stem.is_empty() || base.contains('/') || !plain(&i.file) || i.id != format!("{}/{stem}", kind.name()) {
        return None;
    }
    if i.thumbnail.as_deref().is_some_and(|t| !plain(t) || !t.starts_with("thumbs/") || t.matches('/').count() != 1 || !t.ends_with(".png")) {
        i.thumbnail = None;
    }
    i.name = i.name.chars().take(120).collect();
    i.tags.truncate(MAX_TAGS);
    Some(i)
}

fn read_index(root: &Path) -> Result<Vec<Item>, String> {
    let file = root.join(INDEX);
    let meta = match std::fs::symlink_metadata(&file) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    if !meta.is_file() {
        return Err("index.json is not a plain file".into());
    }
    if meta.len() > MAX_INDEX {
        return Err(format!("index.json is larger than {} MiB", MAX_INDEX / 1024 / 1024));
    }
    let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let items: Vec<Item> = serde_json::from_value(v.get("items").cloned().ok_or("index.json has no items list")?).map_err(|e| e.to_string())?;
    Ok(items.into_iter().filter_map(tied).collect())
}

fn quarantine(root: &Path) -> Result<(), CommandError> {
    let file = root.join(INDEX);
    if std::fs::symlink_metadata(&file).is_err() {
        return Ok(());
    }
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    for n in 0..1000 {
        let to = root.join(format!("index.corrupt-{secs}-{n}.json"));
        if std::fs::symlink_metadata(&to).is_err() {
            return std::fs::rename(&file, &to).map_err(|e| io_error(&file, e));
        }
    }
    Err(CommandError::new("io", format!("{} could not be backed up", file.display())))
}

fn index_for_write(root: &Path) -> Result<Vec<Item>, CommandError> {
    match read_index(root) {
        Ok(items) => Ok(items),
        Err(_) => {
            quarantine(root)?;
            let mut items = Vec::new();
            scan(root, &mut items);
            Ok(items)
        }
    }
}

fn write_index(root: &Path, items: &[Item]) -> Result<(), CommandError> {
    let body = serde_json::to_vec(&json!({"format": "formfabric-library", "schema": 1, "items": items})).map_err(|e| CommandError::new("io", e.to_string()))?;
    if body.len() as u64 > MAX_INDEX {
        return Err(CommandError::new("limit", "the library index would be too large; remove items first"));
    }
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

pub(crate) fn would_be_inside_roots(path: &Path) -> bool {
    future_path(path).is_ok_and(|f| roots().iter().any(|r| f.starts_with(r)))
}

fn scan(root: &Path, items: &mut Vec<Item>) {
    let mut known: std::collections::HashSet<String> = items.iter().map(|i| i.file.clone()).collect();
    for kind in Kind::ALL {
        let Ok(dir) = std::fs::read_dir(root.join(kind.name())) else { continue };
        for entry in dir.flatten().take(MAX_ITEMS) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(&format!(".{}", kind.extension())) else { continue };
            let file = format!("{}/{name}", kind.name());
            if !entry.file_type().is_ok_and(|t| t.is_file()) || !plain(&file) || !known.insert(file.clone()) {
                continue;
            }
            let bytes = entry.metadata().map_or(0, |m| m.len());
            let thumb = format!("thumbs/{}-{stem}.png", kind.name());
            let thumbnail = std::fs::symlink_metadata(root.join(&thumb)).is_ok_and(|m| m.is_file()).then_some(thumb);
            items.push(Item { id: format!("{}/{stem}", kind.name()), kind: kind.name().into(), name: stem.into(), file, bytes, thumbnail, ..Item::default() });
        }
    }
}

pub(crate) fn all_items() -> Vec<(PathBuf, Item)> {
    let mut out: Vec<(PathBuf, Item)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for root in roots() {
        let mut items = read_index(&root).unwrap_or_default();
        scan(&root, &mut items);
        for item in items {
            if !seen.contains(&item.id) && resolve(&root, &item.file).is_ok_and(|p| p.is_file()) {
                seen.insert(item.id.clone());
                out.push((root.clone(), item));
            }
        }
    }
    out
}

fn thumb_path(root: &Path, rel: &str) -> Option<String> {
    use std::io::Read;
    let canon = resolve(root, rel).ok()?;
    if !canon.is_file() || !canon.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("png")) {
        return None;
    }
    let mut head = [0u8; 8];
    std::fs::File::open(&canon).and_then(|mut f| f.read_exact(&mut head)).ok()?;
    (head == [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).then(|| canon.to_string_lossy().into_owned())
}

pub(crate) fn item_json(root: &Path, item: &Item) -> Value {
    json!({
        "id": item.id,
        "kind": item.kind,
        "name": item.name,
        "tags": item.tags,
        "path": root.join(&item.file).to_string_lossy(),
        "thumbnail": item.thumbnail.as_deref().and_then(|t| thumb_path(root, t)),
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
    if !meta.is_file() {
        return Err(CommandError::new("bad_args", format!("{} is not a file", path.display())));
    }
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

fn inner_dir(root: &Path, name: &str) -> Result<PathBuf, CommandError> {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).map_err(|e| io_error(&dir, e))?;
    let canon = dir.canonicalize().map_err(|e| io_error(&dir, e))?;
    if !canon.starts_with(root) || !canon.is_dir() {
        return Err(CommandError::new("outside_library", format!("{} leads outside the library folder", dir.display())));
    }
    Ok(canon)
}

pub(crate) fn reserve(kind: Kind, name: &str) -> Result<Reserved, CommandError> {
    let root = roots().into_iter().next().ok_or_else(|| CommandError::new("io", "the library folder could not be created"))?;
    if read_index(&root).map_or(0, |v| v.len()) >= MAX_ITEMS {
        return Err(CommandError::new("limit", format!("a library folder holds at most {MAX_ITEMS} items")));
    }
    let dir = inner_dir(&root, kind.name())?;
    let base = slug(name, kind);
    let mut n = 1;
    loop {
        let stem = if n == 1 { base.clone() } else { format!("{base}-{n}") };
        let file = format!("{}/{stem}.{}", kind.name(), kind.extension());
        let abs = dir.join(format!("{stem}.{}", kind.extension()));
        if std::fs::symlink_metadata(&abs).is_err() {
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
        let dir = inner_dir(&r.root, "thumbs")?;
        args::write_atomic(&dir.join(format!("{}-{}.png", r.kind.name(), r.slug)).to_string_lossy(), &png)?;
        thumbnail = Some(rel);
    }
    let created = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let item = Item { id: format!("{}/{}", r.kind.name(), r.slug), kind: r.kind.name().into(), name: name.into(), tags, file: r.file.clone(), thumbnail, bytes, created };
    let mut items = index_for_write(&r.root)?;
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
    let mut items = index_for_write(&root)?;
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
