use ffb_pattern::Garment;
use serde_json::{json, Map, Value};

const KEY: &str = "edit3d2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    None,
    Stiffen,
    Fold,
}

impl Mode {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Mode::None => "none",
            Mode::Stiffen => "stiffen",
            Mode::Fold => "fold",
        }
    }

    pub(crate) fn parse(s: &str) -> Option<Mode> {
        match s {
            "none" => Some(Mode::None),
            "stiffen" => Some(Mode::Stiffen),
            "fold" => Some(Mode::Fold),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LineFx {
    pub mode: Mode,
    pub stiffen: f64,
    pub width: f64,
    pub fold_angle: f64,
    pub fold_strength: f64,
}

impl Default for LineFx {
    fn default() -> Self {
        LineFx { mode: Mode::None, stiffen: 8.0, width: 12.0, fold_angle: 150.0, fold_strength: 0.5 }
    }
}

impl LineFx {
    pub(crate) fn to_json(self) -> Value {
        json!({"mode": self.mode.name(), "stiffen": self.stiffen, "width": self.width, "fold_angle": self.fold_angle, "fold_strength": self.fold_strength})
    }

    pub(crate) fn from_json(v: &Value) -> LineFx {
        let d = LineFx::default();
        let num = |k: &str, fallback: f64| v.get(k).and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(fallback);
        LineFx {
            mode: v.get("mode").and_then(Value::as_str).and_then(Mode::parse).unwrap_or(d.mode),
            stiffen: num("stiffen", d.stiffen),
            width: num("width", d.width),
            fold_angle: num("fold_angle", d.fold_angle),
            fold_strength: num("fold_strength", d.fold_strength),
        }
    }
}

fn get<'a>(g: &'a Garment, section: &str, key: &str) -> Option<&'a Value> {
    g.selection.get(KEY)?.get(section)?.get(key)
}

pub(crate) fn line_fx(g: &Garment, id: u32) -> LineFx {
    get(g, "lines", &id.to_string()).map(LineFx::from_json).unwrap_or_default()
}

pub(crate) fn shell_of(g: &Garment, body: u32) -> Option<(f64, bool)> {
    let v = get(g, "fit_shell", &body.to_string())?;
    Some((v.get("offset").and_then(Value::as_f64)?, v.get("enabled").and_then(Value::as_bool).unwrap_or(false)))
}

pub(crate) fn put(g: &mut Garment, section: &str, key: &str, value: Option<Value>) {
    let mut root = match g.selection.take() {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    let ext = root.entry(KEY).or_insert_with(|| json!({}));
    if !ext.is_object() {
        *ext = json!({});
    }
    if let Some(ext) = ext.as_object_mut() {
        let sec = ext.entry(section).or_insert_with(|| json!({}));
        if !sec.is_object() {
            *sec = json!({});
        }
        if let Some(sec) = sec.as_object_mut() {
            match value {
                Some(v) => {
                    sec.insert(key.to_string(), v);
                }
                None => {
                    sec.remove(key);
                }
            }
        }
    }
    g.selection = Value::Object(root);
}

pub(crate) fn signature(g: &Garment) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    g.selection.get(KEY).map(Value::to_string).unwrap_or_default().hash(&mut h);
    for l in &g.style_lines {
        l.id.hash(&mut h);
        l.panel.hash(&mut h);
        l.closed.hash(&mut h);
        for p in &l.points {
            p[0].to_bits().hash(&mut h);
            p[1].to_bits().hash(&mut h);
        }
    }
    h.finish()
}
