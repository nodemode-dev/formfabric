use super::{compact, Raw};

const DATA: u64 = 1 << 63;

fn word(b: &[u8], at: usize) -> Result<u64, String> {
    at.checked_add(8).and_then(|end| b.get(at..end)).map(|s| u64::from_le_bytes(s.try_into().unwrap_or([0; 8]))).ok_or_else(|| "the Alembic file is truncated".to_string())
}

const MAX_DEPTH: u32 = 64;
const MAX_NODES: usize = 1_000_000;

struct Budget {
    nodes: usize,
    bytes: usize,
    vertices: usize,
    out: Vec<Mesh>,
}

impl Budget {
    fn node(&mut self, depth: u32) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err(format!("the Alembic file nests deeper than {MAX_DEPTH} levels"));
        }
        if self.nodes == 0 {
            return Err(format!("the Alembic file holds more than {MAX_NODES} objects and properties"));
        }
        self.nodes -= 1;
        Ok(())
    }

    fn bytes(&mut self, n: usize) -> Result<(), String> {
        if n > self.bytes {
            return Err("the Alembic file expands to more sample data than its size allows".into());
        }
        self.bytes -= n;
        Ok(())
    }
}

fn group(b: &[u8], entry: u64) -> Result<Vec<u64>, String> {
    if entry == 0 {
        return Ok(Vec::new());
    }
    if entry & DATA != 0 {
        return Err("the Alembic file is damaged (data where a group belongs)".into());
    }
    let at = usize::try_from(entry).map_err(|_| "the Alembic file is damaged (offset too large)".to_string())?;
    let n = usize::try_from(word(b, at)?).map_err(|_| "the Alembic file is damaged (group too large)".to_string())?;
    if n == 0 || n > b.len() / 8 {
        return Err("the Alembic file is damaged (bad group size)".into());
    }
    (0..n).map(|i| word(b, at.saturating_add(8).saturating_add(i * 8))).collect()
}

fn data(b: &[u8], entry: u64) -> Result<&[u8], String> {
    if entry & DATA == 0 {
        return Err("the Alembic file is damaged (group where data belongs)".into());
    }
    let at = usize::try_from(entry & !DATA).map_err(|_| "the Alembic file is damaged (offset too large)".to_string())?;
    if at == 0 {
        return Ok(&[]);
    }
    let n = usize::try_from(word(b, at)?).map_err(|_| "the Alembic file is truncated".to_string())?;
    at.checked_add(8).and_then(|s| s.checked_add(n).map(|e| (s, e))).and_then(|(s, e)| b.get(s..e)).ok_or_else(|| "the Alembic file is truncated".to_string())
}

struct Cur<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cur<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).ok_or("the Alembic header table is damaged")?;
        let s = self.b.get(self.at..end).ok_or("the Alembic header table is damaged")?;
        self.at = end;
        Ok(s)
    }

    fn u32(&mut self) -> Result<usize, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap_or([0; 4])) as usize)
    }

    fn var(&mut self, hint: u32) -> Result<usize, String> {
        Ok(match hint {
            0 => self.take(1)?[0] as usize,
            1 => u16::from_le_bytes(self.take(2)?.try_into().unwrap_or([0; 2])) as usize,
            _ => self.u32()?,
        })
    }

    fn text(&mut self, n: usize) -> Result<String, String> {
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }
}

struct Prop {
    name: String,
    compound: bool,
    pod: u32,
    extent: u32,
    n: usize,
    first: usize,
    last: usize,
    constant: bool,
    array: bool,
    entries: Vec<u64>,
    subs: Vec<Prop>,
}

fn meta_of(table: &[String], idx: u8) -> Result<String, String> {
    if idx == 0 {
        return Ok(String::new());
    }
    table.get(idx as usize - 1).cloned().ok_or_else(|| "the Alembic metadata table is damaged".to_string())
}

fn props(b: &[u8], entry: u64, table: &[String], depth: u32, budget: &mut Budget) -> Result<Vec<Prop>, String> {
    budget.node(depth)?;
    let children = group(b, entry)?;
    let Some((hdr, groups)) = children.split_last() else { return Ok(Vec::new()) };
    let blob = data(b, *hdr)?;
    let mut cur = Cur { b: blob, at: 0 };
    let mut out = Vec::new();
    while cur.at < blob.len() {
        let info = cur.u32()? as u32;
        let hint = (info >> 2) & 3;
        let compound = info & 3 == 0;
        let (mut n, mut first, mut last) = (0usize, 0usize, 0usize);
        let constant = info & 0x800 != 0;
        if !compound {
            n = cur.var(hint)?;
            if info & 0x200 != 0 {
                first = cur.var(hint)?;
                last = cur.var(hint)?.max(first);
            } else if !constant {
                first = 1;
                last = n.saturating_sub(1);
            }
            if info & 0x100 != 0 {
                cur.var(hint)?;
            }
        }
        let l = cur.var(hint)?;
        let name = cur.text(l)?;
        if (info >> 20) & 0xFF == 255 {
            let l = cur.var(hint)?;
            cur.take(l)?;
        } else {
            meta_of(table, ((info >> 20) & 0xFF) as u8)?;
        }
        let g = *groups.get(out.len()).ok_or("the Alembic property list is damaged")?;
        let (subs, entries) = if compound { (props(b, g, table, depth + 1, budget)?, Vec::new()) } else { (Vec::new(), group(b, g)?) };
        budget.bytes(entries.len().saturating_mul(8))?;
        out.push(Prop { name, compound, pod: (info >> 4) & 15, extent: (info >> 12) & 255, n, first, last, constant, array: info & 3 != 1, entries, subs });
    }
    Ok(out)
}

fn payload<'a>(b: &'a [u8], p: &Prop, i: usize) -> Result<&'a [u8], String> {
    if p.compound || p.n == 0 {
        return Err(format!("{} holds no samples", p.name));
    }
    let i = i.min(p.n - 1);
    let k = if p.constant || i < p.first {
        0
    } else if i >= p.last {
        (p.last.saturating_sub(p.first)).saturating_add(1)
    } else {
        (i - p.first).saturating_add(1)
    };
    let at = if p.array { k.saturating_mul(2) } else { k };
    let blob = data(b, *p.entries.get(at).ok_or("the Alembic sample table is damaged")?)?;
    if blob.is_empty() {
        return Ok(blob);
    }
    blob.get(16..).ok_or_else(|| "an Alembic sample is damaged".to_string())
}

fn floats(b: &[u8], p: &Prop, i: usize) -> Result<Vec<f64>, String> {
    let raw = payload(b, p, i)?;
    if p.extent != 3 {
        return Err("P must have three components".into());
    }
    match p.pod {
        10 => Ok(raw.as_chunks::<4>().0.iter().map(|c| f64::from(f32::from_le_bytes(*c))).collect()),
        11 => Ok(raw.as_chunks::<8>().0.iter().map(|c| f64::from_le_bytes(*c)).collect()),
        _ => Err("P is not a float array".into()),
    }
}

fn ints(b: &[u8], p: &Prop, i: usize) -> Result<Vec<i64>, String> {
    let raw = payload(b, p, i)?;
    match p.pod {
        6 => Ok(raw.as_chunks::<4>().0.iter().map(|c| i64::from(i32::from_le_bytes(*c))).collect()),
        5 => Ok(raw.as_chunks::<4>().0.iter().map(|c| i64::from(u32::from_le_bytes(*c))).collect()),
        8 => Ok(raw.as_chunks::<8>().0.iter().map(|c| i64::from_le_bytes(*c)).collect()),
        _ => Err("face data is not an integer array".into()),
    }
}

struct Mesh {
    positions: Vec<[f32; 3]>,
    counts: Vec<i64>,
    indices: Vec<i64>,
}

fn collect(b: &[u8], entry: u64, meta: &str, table: &[String], sample: usize, depth: u32, budget: &mut Budget) -> Result<(), String> {
    budget.node(depth)?;
    let children = group(b, entry)?;
    let Some((hdr, rest)) = children.split_last() else { return Ok(()) };
    let blob = data(b, *hdr)?;
    if blob.len() < 32 {
        return Err("an Alembic object header is damaged".into());
    }
    if meta.contains("schema=AbcGeom_PolyMesh_v1") || meta.contains("schemaObjTitle=AbcGeom_PolyMesh_v1") {
        let top = match rest.first() {
            Some(p) => props(b, *p, table, depth + 1, budget)?,
            None => Vec::new(),
        };
        if let Some(geom) = top.iter().find(|p| p.name == ".geom") {
            let find = |name: &str| geom.subs.iter().find(|p| p.name == name);
            if let (Some(p), Some(fi), Some(fc)) = (find("P"), find(".faceIndices"), find(".faceCounts")) {
                if sample >= p.n.max(1) {
                    return Err(format!("sample {sample} is past the last sample ({}) of the mesh", p.n.saturating_sub(1)));
                }
                let raw_p = payload(b, p, sample)?.len();
                let points = raw_p / (p.extent.max(1) as usize * if p.pod == 11 { 8 } else { 4 });
                budget.vertices = budget.vertices.checked_sub(points).ok_or_else(|| format!("the Alembic mesh has more than {} vertices", super::MAX_VERTICES))?;
                budget.bytes(raw_p.saturating_mul(3))?;
                budget.bytes(payload(b, fc, sample)?.len().saturating_mul(3))?;
                budget.bytes(payload(b, fi, sample)?.len().saturating_mul(3))?;
                let v = floats(b, p, sample)?;
                let positions = v.as_chunks::<3>().0.iter().map(|c| [c[0] as f32, c[1] as f32, c[2] as f32]).collect();
                budget.out.push(Mesh { positions, counts: ints(b, fc, sample)?, indices: ints(b, fi, sample)? });
            }
        }
    }
    let table_bytes = &blob[..blob.len() - 32];
    let mut cur = Cur { b: table_bytes, at: 0 };
    let mut k = 0;
    while cur.at < table_bytes.len() {
        let l = cur.u32()?;
        cur.text(l)?;
        let mi = cur.take(1)?[0];
        let cmeta = if mi == 255 {
            let l = cur.u32()?;
            cur.text(l)?
        } else {
            meta_of(table, mi)?
        };
        let g = *rest.get(1 + k).ok_or("an Alembic child list is damaged")?;
        collect(b, g, &cmeta, table, sample, depth + 1, budget)?;
        k += 1;
    }
    Ok(())
}

pub fn read(b: &[u8], sample: Option<usize>) -> Result<Raw, String> {
    if b.len() < 16 || &b[0..5] != b"Ogawa" {
        return Err("this is not an Ogawa Alembic file (HDF5 Alembic files are not supported)".into());
    }
    if b[5] != 0xFF {
        return Err("the Alembic file was not closed cleanly".into());
    }
    let root = group(b, word(b, 8)?)?;
    if root.len() < 6 {
        return Err("the Alembic archive is damaged".into());
    }
    let version = |e: u64| -> Result<i32, String> {
        let d = data(b, e)?;
        Ok(i32::from_le_bytes(d.try_into().map_err(|_| "the Alembic version fields are damaged".to_string())?))
    };
    if version(root[0])? != 0 || version(root[1])? < 9999 {
        return Err("this Alembic file version is not supported".into());
    }
    let raw = data(b, root[5])?;
    let mut table = Vec::new();
    let mut cur = Cur { b: raw, at: 0 };
    while cur.at < raw.len() {
        let l = cur.take(1)?[0] as usize;
        table.push(cur.text(l)?);
    }
    let mut budget = Budget { nodes: MAX_NODES, bytes: b.len().saturating_mul(4).max(1 << 20), vertices: super::MAX_VERTICES, out: Vec::new() };
    collect(b, root[2], "", &table, sample.unwrap_or(0), 0, &mut budget)?;
    let meshes = std::mem::take(&mut budget.out);
    if meshes.is_empty() {
        return Err("the Alembic file holds no polygon mesh".into());
    }
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    for m in &meshes {
        let base = positions.len();
        if base + m.positions.len() > super::MAX_VERTICES {
            return Err(format!("the Alembic mesh has more than {} vertices", super::MAX_VERTICES));
        }
        let n = m.positions.len() as i64;
        let mut at = 0usize;
        for c in &m.counts {
            let c = *c;
            let Some(end) = usize::try_from(c).ok().filter(|c| *c > 0).and_then(|c| at.checked_add(c)).filter(|e| *e <= m.indices.len()) else { break };
            let corners = &m.indices[at..end];
            at = end;
            if corners.iter().any(|i| *i < 0 || *i >= n) {
                continue;
            }
            let face: Vec<u32> = corners.iter().rev().map(|i| base as u32 + *i as u32).collect();
            for k in 1..face.len().saturating_sub(1) {
                triangles.push([face[0], face[k], face[k + 1]]);
            }
            if triangles.len() > super::MAX_TRIANGLES {
                return Err(format!("the Alembic mesh has more than {} triangles", super::MAX_TRIANGLES));
            }
        }
        positions.extend_from_slice(&m.positions);
    }
    if positions.iter().any(|p| p.iter().any(|x| !x.is_finite())) {
        return Err("the Alembic mesh has non-finite points".into());
    }
    let mut weights = Vec::new();
    compact(&mut positions, &mut triangles, &mut weights);
    Ok(Raw { positions, triangles, ..Raw::default() })
}
