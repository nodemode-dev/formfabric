use std::path::{Path, PathBuf};

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub license: Option<String>,
}

fn roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(dir) = std::env::var("FF_BODIES_DIR") {
        out.push(PathBuf::from(dir));
    }
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            starts.push(parent.to_path_buf());
        }
    }
    for start in starts {
        for dir in start.ancestors().take(7) {
            out.push(dir.join("assets").join("bodies"));
        }
    }
    out
}

fn title(stem: &str) -> String {
    stem.split(['-', '_']).filter(|w| !w.is_empty()).map(|w| if w.chars().any(|c| c.is_ascii_digit()) { w.to_uppercase() } else { w.chars().take(1).flat_map(char::to_uppercase).chain(w.chars().skip(1)).collect() }).collect::<Vec<_>>().join(" ")
}

fn license_of(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("LICENSE.txt")).ok()?;
    let head = text.to_lowercase();
    if head.contains("cc0") || head.contains("creative commons zero") || head.contains("cc0 1.0") {
        Some("CC0-1.0".into())
    } else if head.contains("attribution 4.0") {
        Some("CC-BY-4.0".into())
    } else {
        None
    }
}

pub fn entries() -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    for root in roots() {
        let Ok(read) = std::fs::read_dir(&root) else { continue };
        let mut dirs: Vec<PathBuf> = read.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect();
        dirs.sort();
        for dir in dirs {
            let Some(stem) = dir.file_name().and_then(|s| s.to_str()) else { continue };
            for file in ["body.glb", "body.gltf", "body.fbx", "body.obj"] {
                let path = dir.join(file);
                if path.is_file() && !out.iter().any(|e| e.name == title(stem)) {
                    out.push(Entry { name: title(stem), path, license: license_of(&dir) });
                    break;
                }
            }
        }
    }
    out
}

pub fn find(name: &str) -> Option<Entry> {
    let key = name.trim().to_lowercase();
    entries().into_iter().find(|e| e.name.to_lowercase() == key).or_else(|| entries().into_iter().find(|e| e.name.to_lowercase().contains(&key)))
}
