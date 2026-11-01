use serde_json::{Map, Value};

use crate::units::{self, FIELDS};
use crate::{Fabric, Look, LookKind, Maps, Physical, Surface};

pub const NAME_MAX: usize = 64;
pub const MAP_NAME_MAX: usize = 512;
pub const SCALE_RANGE: [f64; 2] = [0.1, 100_000.0];
pub const OFFSET_RANGE: f64 = 100_000.0;
pub const ROTATION_RANGE: f64 = 360.0;

pub const LOOK_KINDS: [&str; 8] = ["matte", "shiny", "satin", "velvet", "leather", "metal", "plastic", "glass"];
pub const MAP_SLOTS: [&str; 6] = ["base_color", "normal", "roughness", "metalness", "opacity", "displacement"];

fn number(value: &Value, path: &str) -> Result<f64, String> {
    value.as_f64().filter(|x| x.is_finite()).ok_or_else(|| format!("{path} must be a number"))
}

fn within(x: f64, lo: f64, hi: f64, unit: &str, path: &str) -> Result<f64, String> {
    if x < lo || x > hi {
        let unit = if unit.is_empty() || unit == "ratio" { String::new() } else { format!(" {unit}") };
        return Err(format!("{path} must be between {lo} and {hi}{unit}, got {x}"));
    }
    Ok(x)
}

fn object<'a>(value: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    value.as_object().ok_or_else(|| format!("{path} must be an object"))
}

fn only_keys(map: &Map<String, Value>, allowed: &[&str], path: &str) -> Result<(), String> {
    match map.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => Err(format!("{path} has no field {k}")),
        None => Ok(()),
    }
}

pub fn name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > NAME_MAX {
        return Err(format!("name must be 1 to {NAME_MAX} characters"));
    }
    Ok(trimmed.to_string())
}

pub fn physical_patch(p: &mut Physical, patch: &Value) -> Result<(), String> {
    let map = object(patch, "physical")?;
    let keys: Vec<&str> = FIELDS.iter().map(|f| f.key).collect();
    only_keys(map, &keys, "physical")?;
    for f in FIELDS.iter() {
        if let Some(v) = map.get(f.key) {
            let path = format!("physical.{}", f.key);
            let x = within(number(v, &path)?, f.min, f.max, f.unit, &path)?;
            units::set(p, f.key, x);
        }
    }
    Ok(())
}

pub fn check_physical(p: &Physical) -> Result<(), String> {
    for f in FIELDS.iter() {
        let x = units::get(p, f.key).unwrap_or(f64::NAN);
        let path = format!("physical.{}", f.key);
        if !x.is_finite() {
            return Err(format!("{path} must be a number"));
        }
        within(x, f.min, f.max, f.unit, &path)?;
    }
    Ok(())
}

fn unit_interval(v: &Value, path: &str) -> Result<f32, String> {
    Ok(within(number(v, path)?, 0.0, 1.0, "", path)? as f32)
}

fn pair(v: &Value, lo: f64, hi: f64, unit: &str, path: &str) -> Result<[f64; 2], String> {
    let items = v.as_array().filter(|a| a.len() == 2).ok_or_else(|| format!("{path} must be an array of two numbers"))?;
    let mut out = [0.0; 2];
    for (i, item) in items.iter().enumerate() {
        out[i] = within(number(item, path)?, lo, hi, unit, path)?;
    }
    Ok(out)
}

fn map_name(v: &Value, path: &str) -> Result<Option<String>, String> {
    match v {
        Value::Null => Ok(None),
        Value::String(s) if !s.trim().is_empty() && s.chars().count() <= MAP_NAME_MAX && !s.contains('\0') => Ok(Some(s.clone())),
        _ => Err(format!("{path} must be null or a file name of 1 to {MAP_NAME_MAX} characters")),
    }
}

fn maps_patch(m: &mut Maps, patch: &Value, path: &str) -> Result<(), String> {
    let map = object(patch, path)?;
    only_keys(map, &MAP_SLOTS, path)?;
    for slot in MAP_SLOTS {
        if let Some(v) = map.get(slot) {
            let value = map_name(v, &format!("{path}.{slot}"))?;
            match slot {
                "base_color" => m.base_color = value,
                "normal" => m.normal = value,
                "roughness" => m.roughness = value,
                "metalness" => m.metalness = value,
                "opacity" => m.opacity = value,
                _ => m.displacement = value,
            }
        }
    }
    Ok(())
}

pub fn surface_patch(s: &mut Surface, patch: &Value, path: &str) -> Result<(), String> {
    let map = object(patch, path)?;
    only_keys(map, &["color", "roughness", "metalness", "sheen", "opacity", "maps", "map_scale", "map_rotation", "map_offset"], path)?;
    if let Some(v) = map.get("color") {
        let p = format!("{path}.color");
        let items = v.as_array().filter(|a| a.len() == 4 || a.len() == 3).ok_or_else(|| format!("{p} must be [r, g, b, a] with values 0 to 1"))?;
        let mut c = s.color;
        for (i, item) in items.iter().enumerate() {
            c[i] = unit_interval(item, &p)?;
        }
        s.color = c;
    }
    for (key, slot) in [("roughness", &mut s.roughness), ("metalness", &mut s.metalness), ("sheen", &mut s.sheen), ("opacity", &mut s.opacity)] {
        if let Some(v) = map.get(key) {
            *slot = unit_interval(v, &format!("{path}.{key}"))?;
        }
    }
    if let Some(v) = map.get("maps") {
        maps_patch(&mut s.maps, v, &format!("{path}.maps"))?;
    }
    if let Some(v) = map.get("map_scale") {
        s.map_scale = pair(v, SCALE_RANGE[0], SCALE_RANGE[1], "mm", &format!("{path}.map_scale"))?;
    }
    if let Some(v) = map.get("map_rotation") {
        let p = format!("{path}.map_rotation");
        s.map_rotation = within(number(v, &p)?, -ROTATION_RANGE, ROTATION_RANGE, "deg", &p)?;
    }
    if let Some(v) = map.get("map_offset") {
        s.map_offset = pair(v, -OFFSET_RANGE, OFFSET_RANGE, "mm", &format!("{path}.map_offset"))?;
    }
    Ok(())
}

fn kind_from(text: &str) -> Option<LookKind> {
    Some(match text {
        "matte" => LookKind::Matte,
        "shiny" => LookKind::Shiny,
        "satin" => LookKind::Satin,
        "velvet" => LookKind::Velvet,
        "leather" => LookKind::Leather,
        "metal" => LookKind::Metal,
        "plastic" => LookKind::Plastic,
        "glass" => LookKind::Glass,
        _ => return None,
    })
}

pub fn look_patch(l: &mut Look, patch: &Value) -> Result<(), String> {
    let map = object(patch, "look")?;
    only_keys(map, &["kind", "front", "back", "side"], "look")?;
    if let Some(v) = map.get("kind") {
        let text = v.as_str().and_then(kind_from);
        l.kind = text.ok_or_else(|| format!("look.kind must be one of {}", LOOK_KINDS.join(", ")))?;
    }
    if let Some(v) = map.get("front") {
        surface_patch(&mut l.front, v, "look.front")?;
    }
    for (key, slot) in [("back", &mut l.back), ("side", &mut l.side)] {
        match map.get(key) {
            None => {}
            Some(Value::Null) => *slot = None,
            Some(v) => {
                let mut surface = slot.clone().unwrap_or_else(|| l.front.clone());
                surface_patch(&mut surface, v, &format!("look.{key}"))?;
                *slot = Some(surface);
            }
        }
    }
    Ok(())
}

fn check_surface(s: &Surface, path: &str) -> Result<(), String> {
    let value = serde_json::to_value(s).map_err(|e| e.to_string())?;
    let mut probe = Surface::default();
    surface_patch(&mut probe, &value, path)
}

pub fn check_look(l: &Look) -> Result<(), String> {
    check_surface(&l.front, "look.front")?;
    if let Some(b) = &l.back {
        check_surface(b, "look.back")?;
    }
    if let Some(s) = &l.side {
        check_surface(s, "look.side")?;
    }
    Ok(())
}

pub fn check_fabric(f: &Fabric) -> Result<(), String> {
    name(&f.name)?;
    check_physical(&f.physical)?;
    check_look(&f.look)
}

pub fn map_refs(l: &Look) -> Vec<String> {
    let mut out = Vec::new();
    for s in [Some(&l.front), l.back.as_ref(), l.side.as_ref()].into_iter().flatten() {
        let m = &s.maps;
        for r in [&m.base_color, &m.normal, &m.roughness, &m.metalness, &m.opacity, &m.displacement].into_iter().flatten() {
            if !out.contains(r) {
                out.push(r.clone());
            }
        }
    }
    out
}

pub fn rewrite_maps(l: &mut Look, f: &mut dyn FnMut(&str) -> Option<String>) {
    let mut surfaces: Vec<&mut Surface> = vec![&mut l.front];
    if let Some(b) = l.back.as_mut() {
        surfaces.push(b);
    }
    if let Some(s) = l.side.as_mut() {
        surfaces.push(s);
    }
    for s in surfaces {
        let m = &mut s.maps;
        for slot in [&mut m.base_color, &mut m.normal, &mut m.roughness, &mut m.metalness, &mut m.opacity, &mut m.displacement] {
            if let Some(old) = slot.as_deref() {
                *slot = f(old);
            }
        }
    }
}
