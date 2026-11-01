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

pub fn geometry(style: &Style) -> Option<TrimGeometry> {
    let mesh = style.params.get("mesh")?;
    let positions: Vec<[f32; 3]> = mesh
        .get("positions")?
        .as_array()?
        .iter()
        .map(|p| {
            let p = p.as_array()?;
            Some([p.first()?.as_f64()? as f32, p.get(1)?.as_f64()? as f32, p.get(2)?.as_f64()? as f32])
        })
        .collect::<Option<_>>()?;
    let triangles: Vec<[u32; 3]> = mesh
        .get("triangles")?
        .as_array()?
        .iter()
        .map(|t| {
            let t = t.as_array()?;
            Some([t.first()?.as_u64()? as u32, t.get(1)?.as_u64()? as u32, t.get(2)?.as_u64()? as u32])
        })
        .collect::<Option<_>>()?;
    let n = positions.len() as u32;
    (!positions.is_empty() && triangles.iter().all(|t| t.iter().all(|i| *i < n))).then_some(TrimGeometry { positions, triangles })
}
