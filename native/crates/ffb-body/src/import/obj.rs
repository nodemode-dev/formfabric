use super::{compact, Raw};

fn index(token: &str, count: usize) -> Result<u32, String> {
    let head = token.split('/').next().unwrap_or("");
    let n: i64 = head.parse().map_err(|_| format!("bad face index {token}"))?;
    let i = if n < 0 { count as i64 + n } else { n - 1 };
    if i < 0 || i >= count as i64 {
        return Err(format!("face index {token} is out of range"));
    }
    Ok(i as u32)
}

pub fn read(bytes: &[u8]) -> Result<Raw, String> {
    let text = String::from_utf8_lossy(bytes);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(tag) = parts.next() else { continue };
        match tag {
            "v" => {
                let v: Vec<f32> = parts.take(3).map(|t| t.parse::<f32>().map_err(|_| format!("line {}: bad vertex", line_no + 1))).collect::<Result<_, _>>()?;
                if v.len() < 3 || v.iter().any(|x| !x.is_finite()) {
                    return Err(format!("line {}: a vertex needs three finite numbers", line_no + 1));
                }
                positions.push([v[0], v[1], v[2]]);
            }
            "f" => {
                let ids: Vec<u32> = parts.map(|t| index(t, positions.len())).collect::<Result<_, _>>().map_err(|e| format!("line {}: {e}", line_no + 1))?;
                for k in 1..ids.len().saturating_sub(1) {
                    triangles.push([ids[0], ids[k], ids[k + 1]]);
                }
            }
            _ => {}
        }
    }
    if positions.is_empty() || triangles.is_empty() {
        return Err("the OBJ file has no vertices or faces".into());
    }
    let mut weights = Vec::new();
    compact(&mut positions, &mut triangles, &mut weights);
    Ok(Raw { positions, triangles, ..Raw::default() })
}
