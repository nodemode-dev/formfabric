use ffb_body::{skin, Body};
use serde_json::{json, Value};

pub const FORMAT: &str = "formfabric-body";
pub const SCHEMA: u64 = 1;

pub fn json_name(id: u32) -> String {
    format!("bodies/{id}.json")
}

pub fn bin_name(id: u32) -> String {
    format!("bodies/{id}.bin")
}

pub fn parse_id(name: &str) -> Option<(u32, bool)> {
    let rest = name.strip_prefix("bodies/")?;
    let (stem, ext) = rest.rsplit_once('.')?;
    let id: u32 = stem.parse().ok()?;
    match ext {
        "json" => Some((id, true)),
        "bin" => Some((id, false)),
        _ => None,
    }
}

pub fn encode(body: &Body) -> Result<(Vec<u8>, Vec<u8>), String> {
    let n = body.positions.len();
    let has_weights = body.weights.len() == n && !body.joints.is_empty();
    let light = Body { positions: Vec::new(), normals: Vec::new(), triangles: Vec::new(), weights: Vec::new(), ..body.clone() };
    let mut meta = serde_json::to_value(&light).map_err(|e| e.to_string())?;
    let Some(map) = meta.as_object_mut() else { return Err("a body did not serialise to an object".into()) };
    map.remove("positions");
    map.remove("normals");
    map.remove("triangles");
    map.remove("weights");
    map.insert("format".into(), json!(FORMAT));
    map.insert("schema".into(), json!(SCHEMA));
    map.insert("vertex_count".into(), json!(n));
    map.insert("triangle_count".into(), json!(body.triangles.len()));
    map.insert("has_weights".into(), json!(has_weights));
    let json = serde_json::to_vec_pretty(&meta).map_err(|e| e.to_string())?;
    let mut bin = Vec::with_capacity(n * 12 + body.triangles.len() * 12 + if has_weights { n * 24 } else { 0 });
    for p in &body.positions {
        for c in p {
            bin.extend_from_slice(&c.to_le_bytes());
        }
    }
    for t in &body.triangles {
        for i in t {
            bin.extend_from_slice(&i.to_le_bytes());
        }
    }
    if has_weights {
        for w in &body.weights {
            for (j, _) in w {
                bin.extend_from_slice(&j.to_le_bytes());
            }
            for (_, x) in w {
                bin.extend_from_slice(&x.to_le_bytes());
            }
        }
    }
    Ok((json, bin))
}

struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

impl Cursor<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let end = self.at + N;
        let slice = self.data.get(self.at..end).ok_or("the body data file is shorter than its description")?;
        self.at = end;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take::<4>()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take::<4>()?))
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take::<2>()?))
    }
}

pub fn decode(json: &[u8], bin: &[u8]) -> Result<Body, String> {
    let meta: Value = serde_json::from_slice(json).map_err(|e| format!("a body description is not JSON: {e}"))?;
    if meta.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err("a body description has the wrong format".into());
    }
    if meta.get("schema").and_then(Value::as_u64).unwrap_or(0) > SCHEMA {
        return Err("a body was written by a newer version".into());
    }
    let count = |k: &str| meta.get(k).and_then(Value::as_u64).map(|v| v as usize).ok_or_else(|| format!("a body description lacks {k}"));
    let (n, t) = (count("vertex_count")?, count("triangle_count")?);
    let has_weights = meta.get("has_weights").and_then(Value::as_bool).unwrap_or(false);
    let mut body: Body = serde_json::from_value(meta).map_err(|e| format!("a body description is not a body: {e}"))?;
    let expected = n.checked_mul(if has_weights { 36 } else { 12 }).zip(t.checked_mul(12)).and_then(|(a, b)| a.checked_add(b));
    if expected != Some(bin.len()) {
        return Err(format!("body {} data holds {} bytes, which does not match {n} vertices and {t} triangles", body.id, bin.len()));
    }
    let mut c = Cursor { data: bin, at: 0 };
    body.positions = Vec::with_capacity(n);
    for _ in 0..n {
        body.positions.push([c.f32()?, c.f32()?, c.f32()?]);
    }
    let limit = crate::dxf::MAX_COORD_MM as f32;
    if body.positions.iter().flatten().any(|v| !v.is_finite() || v.abs() > limit) {
        return Err(format!("body {} has a coordinate outside plus or minus {limit} mm or not a number", body.id));
    }
    body.triangles = Vec::with_capacity(t);
    for _ in 0..t {
        let tri = [c.u32()?, c.u32()?, c.u32()?];
        if tri.iter().any(|i| *i as usize >= n) {
            return Err(format!("body {} has a triangle index outside its vertices", body.id));
        }
        body.triangles.push(tri);
    }
    body.weights = Vec::new();
    if has_weights {
        for _ in 0..n {
            let mut j = [0u16; 4];
            for slot in j.iter_mut() {
                *slot = c.u16()?;
            }
            let mut w = [(0u16, 0.0f32); 4];
            for (k, slot) in w.iter_mut().enumerate() {
                *slot = (j[k], c.f32()?);
            }
            body.weights.push(w);
        }
    }
    body.normals = skin::compute_normals(&body.positions, &body.triangles);
    Ok(body)
}
