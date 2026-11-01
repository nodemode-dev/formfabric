use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::Interp;

pub fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut b = b;
    let mut d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    if d < 0.0 {
        d = -d;
        b = [-b[0], -b[1], -b[2], -b[3]];
    }
    let (wa, wb) = if d > 0.9995 {
        (1.0 - t, t)
    } else {
        let theta = d.clamp(-1.0, 1.0).acos();
        let s = theta.sin();
        (((1.0 - t) * theta).sin() / s, (t * theta).sin() / s)
    };
    let q = [a[0] * wa + b[0] * wb, a[1] * wa + b[1] * wb, a[2] * wa + b[2] * wb, a[3] * wa + b[3] * wb];
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n > 1e-12 {
        [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

pub fn hermite(p0: f64, m0: f64, p1: f64, m1: f64, t: f64, dt: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0 + (t3 - 2.0 * t2 + t) * dt * m0 + (-2.0 * t3 + 3.0 * t2) * p1 + (t3 - t2) * dt * m1
}

fn flatten(v: &Value) -> Option<Vec<f64>> {
    match v {
        Value::Number(n) => n.as_f64().map(|x| vec![x]),
        Value::Array(a) => a.iter().map(Value::as_f64).collect(),
        _ => None,
    }
}

fn rebuild(like: &Value, flat: Vec<f64>) -> Value {
    match like {
        Value::Array(_) => json!(flat),
        _ => json!(flat.first().copied().unwrap_or(0.0)),
    }
}

fn is_quat_map(v: &Value) -> bool {
    v.as_object().is_some_and(|m| !m.is_empty() && m.values().all(|q| q.as_array().is_some_and(|a| a.len() == 4 && a.iter().all(Value::is_number))))
}

fn quat_of(v: Option<&Value>) -> [f32; 4] {
    let Some(a) = v.and_then(Value::as_array) else { return [0.0, 0.0, 0.0, 1.0] };
    let g = |i: usize| a.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    [g(0), g(1), g(2), if a.len() > 3 { g(3) } else { 1.0 }]
}

fn blend_quats(a: &Value, b: &Value, t: f64) -> Value {
    let mut out = Map::new();
    let empty = Map::new();
    let (ma, mb) = (a.as_object().unwrap_or(&empty), b.as_object().unwrap_or(&empty));
    for k in ma.keys().chain(mb.keys()) {
        if out.contains_key(k) {
            continue;
        }
        let q = slerp(quat_of(ma.get(k)), quat_of(mb.get(k)), t as f32);
        out.insert(k.clone(), json!([q[0], q[1], q[2], q[3]]));
    }
    Value::Object(out)
}

fn tangent(keys: &[(f64, Vec<f64>)], i: usize, c: usize) -> f64 {
    let n = keys.len();
    let (a, b) = (i.saturating_sub(1), (i + 1).min(n - 1));
    let dt = keys[b].0 - keys[a].0;
    if dt.abs() < 1e-12 {
        0.0
    } else {
        (keys[b].1[c] - keys[a].1[c]) / dt
    }
}

pub fn evaluate(keys: &[crate::Key], frame: f64) -> Option<Value> {
    if keys.is_empty() {
        return None;
    }
    let mut order: Vec<&crate::Key> = keys.iter().collect();
    order.sort_by_key(|k| k.frame);
    let first = order[0];
    let last = order[order.len() - 1];
    if frame <= f64::from(first.frame) {
        return Some(first.value.clone());
    }
    if frame >= f64::from(last.frame) {
        return Some(last.value.clone());
    }
    let hi = order.iter().position(|k| f64::from(k.frame) > frame).unwrap_or(order.len() - 1);
    let (a, b) = (order[hi - 1], order[hi]);
    let (fa, fb) = (f64::from(a.frame), f64::from(b.frame));
    let t = (frame - fa) / (fb - fa).max(1e-9);
    if a.interp == Interp::Step {
        return Some(a.value.clone());
    }
    if is_quat_map(&a.value) || is_quat_map(&b.value) {
        let t = if a.interp == Interp::Smooth { t * t * (3.0 - 2.0 * t) } else { t };
        return Some(blend_quats(&a.value, &b.value, t));
    }
    let (Some(va), Some(vb)) = (flatten(&a.value), flatten(&b.value)) else { return Some(a.value.clone()) };
    if va.len() != vb.len() {
        return Some(a.value.clone());
    }
    let mut out = Vec::with_capacity(va.len());
    if a.interp == Interp::Smooth {
        let series: Vec<(f64, Vec<f64>)> = order.iter().filter_map(|k| flatten(&k.value).filter(|v| v.len() == va.len()).map(|v| (f64::from(k.frame), v))).collect();
        let (ia, ib) = (hi - 1, hi);
        if series.len() == order.len() {
            for c in 0..va.len() {
                let m0 = tangent(&series, ia, c);
                let m1 = tangent(&series, ib, c);
                out.push(hermite(va[c], m0, vb[c], m1, t, fb - fa));
            }
            return Some(rebuild(&a.value, out));
        }
    }
    for c in 0..va.len() {
        out.push(va[c] + (vb[c] - va[c]) * t);
    }
    Some(rebuild(&a.value, out))
}

pub fn evaluate_number(keys: &[crate::Key], frame: f64) -> Option<f64> {
    evaluate(keys, frame).and_then(|v| v.as_f64())
}

pub fn quat_pose(v: &Value) -> BTreeMap<String, [f32; 4]> {
    let mut out = BTreeMap::new();
    if let Some(m) = v.as_object() {
        for (k, q) in m {
            out.insert(k.clone(), quat_of(Some(q)));
        }
    }
    out
}
