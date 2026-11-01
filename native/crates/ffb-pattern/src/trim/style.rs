use serde_json::{json, Value};

use crate::{Style, StyleKind};

pub fn defaults(kind: StyleKind) -> Value {
    match kind {
        StyleKind::Button => json!({"width": 12.0, "thickness": 3.0, "weight": 1.5, "thread": "cross", "collision": true, "stiffness": 1.0}),
        StyleKind::Buttonhole => json!({"length": 14.0, "z_offset": 0.13, "stiffness": 1.0}),
        StyleKind::Zipper => json!({"preset": "nylon", "width": 10.0, "stiffness": 1.0}),
        StyleKind::Topstitch => json!({"count": 1, "shape": "single", "spi": 10.0, "thickness": 0.6, "offset": 0.0, "stiffen": 1.15}),
        StyleKind::Puckering => json!({"density": 50.0, "width": 3.0}),
        StyleKind::Trim => json!({"collision": true}),
    }
}

pub fn param(style: &Style, key: &str, default: f64) -> f64 {
    style.params.get(key).and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(default)
}

pub fn flag_param(style: &Style, key: &str, default: bool) -> bool {
    style.params.get(key).and_then(Value::as_bool).unwrap_or(default)
}

pub fn merge(base: &mut Value, patch: &Value) {
    let Some(patch) = patch.as_object() else { return };
    if !base.is_object() {
        *base = json!({});
    }
    let Some(map) = base.as_object_mut() else { return };
    for (key, value) in patch {
        if value.is_null() {
            map.remove(key);
        } else {
            map.insert(key.clone(), value.clone());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrimGeometry {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

pub const MAX_TRIM_TRIANGLES: usize = 20_000;
pub const MAX_TRIM_VERTICES: usize = 3 * MAX_TRIM_TRIANGLES;
const MAX_TRIM_COORD: f64 = 100_000.0;

fn coordinate(value: Option<&Value>) -> Option<f32> {
    value?.as_f64().filter(|x| x.abs() <= MAX_TRIM_COORD).map(|x| x as f32)
}

fn index(value: Option<&Value>) -> Option<u32> {
    u32::try_from(value?.as_u64()?).ok()
}

pub fn geometry(style: &Style) -> Option<TrimGeometry> {
    let mesh = style.mesh.as_ref().map(|m| m.0.as_ref()).or_else(|| style.params.get("mesh"))?;
    let raw = mesh.get("positions")?.as_array().filter(|l| l.len() <= MAX_TRIM_VERTICES)?;
    let positions: Vec<[f32; 3]> = raw
        .iter()
        .map(|p| {
            let p = p.as_array()?;
            Some([coordinate(p.first())?, coordinate(p.get(1))?, coordinate(p.get(2))?])
        })
        .collect::<Option<_>>()?;
    let faces = mesh.get("triangles")?.as_array().filter(|l| l.len() <= MAX_TRIM_TRIANGLES)?;
    let triangles: Vec<[u32; 3]> = faces
        .iter()
        .map(|t| {
            let t = t.as_array()?;
            Some([index(t.first())?, index(t.get(1))?, index(t.get(2))?])
        })
        .collect::<Option<_>>()?;
    let n = u32::try_from(positions.len()).ok()?;
    (!positions.is_empty() && triangles.iter().all(|t| t.iter().all(|i| *i < n))).then_some(TrimGeometry { positions, triangles })
}
