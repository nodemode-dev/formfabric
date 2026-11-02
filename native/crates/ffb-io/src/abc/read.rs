use super::ogawa::{DATA_BIT, HEADER_LEN};

#[derive(Debug, Clone, PartialEq)]
pub struct Sampling {
    pub max: u32,
    pub per: f64,
    pub times: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropType {
    Compound,
    Scalar,
    Array,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub meta: String,
    pub meta_index: u8,
    pub info: u32,
    pub hint: u8,
    pub kind: PropType,
    pub scalar_like: bool,
    pub pod: u8,
    pub extent: u8,
    pub n: u32,
    pub first: u32,
    pub last: u32,
    pub ts: u32,
    pub homogenous: bool,
    pub constant: bool,
    pub explicit_range: bool,
    pub subs: Vec<Property>,
    pub entries: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub meta: String,
    pub meta_index: u8,
    pub props: Vec<Property>,
    pub children: Vec<Object>,
    pub hash: Vec<u8>,
    pub group_len: usize,
}

pub struct Sample<'a> {
    pub key: Option<[u8; 16]>,
    pub payload: &'a [u8],
    pub dims_entry: u64,
}

pub struct Archive<'a> {
    pub bytes: &'a [u8],
    pub frozen: u8,
    pub version: u16,
    pub root_offset: u64,
    pub root_children: usize,
    pub file_version: i32,
    pub library_version: i32,
    pub app_meta: String,
    pub samplings: Vec<Sampling>,
    pub indexed: Vec<String>,
    pub indexed_raw_len: usize,
    pub top: Object,
}

fn rd_u64(b: &[u8], at: usize) -> Result<u64, String> {
    at.checked_add(8).and_then(|end| b.get(at..end)).map(|s| u64::from_le_bytes(s.try_into().unwrap_or([0; 8]))).ok_or_else(|| format!("read past the end at {at}"))
}

pub const MAX_DEPTH: u32 = 64;
pub const MAX_NODES: usize = 1_000_000;

fn spend(budget: &mut usize, depth: u32) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(format!("the archive nests deeper than {MAX_DEPTH} levels"));
    }
    if *budget == 0 {
        return Err(format!("the archive holds more than {MAX_NODES} objects and properties"));
    }
    *budget -= 1;
    Ok(())
}

fn group(b: &[u8], entry: u64) -> Result<Vec<u64>, String> {
    if entry == 0 {
        return Ok(Vec::new());
    }
    if entry & DATA_BIT != 0 {
        return Err("expected a group but found data".into());
    }
    let at = usize::try_from(entry).map_err(|_| "a group offset is too large".to_string())?;
    let n = usize::try_from(rd_u64(b, at)?).map_err(|_| "a group is too large".to_string())?;
    if n == 0 || n > b.len() / 8 {
        return Err(format!("group at {at} has a bad child count {n}"));
    }
    (0..n).map(|i| rd_u64(b, at.saturating_add(8).saturating_add(i * 8))).collect()
}

fn data(b: &[u8], entry: u64) -> Result<&[u8], String> {
    if entry & DATA_BIT == 0 {
        return Err("expected data but found a group".into());
    }
    let at = usize::try_from(entry & !DATA_BIT).map_err(|_| "a data offset is too large".to_string())?;
    if at == 0 {
        return Ok(&[]);
    }
    let n = usize::try_from(rd_u64(b, at)?).map_err(|_| "a data blob is too large".to_string())?;
    at.checked_add(8).and_then(|s| s.checked_add(n).map(|e| (s, e))).and_then(|(s, e)| b.get(s..e)).ok_or_else(|| format!("data at {at} runs past the end"))
}

struct Cur<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cur<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).ok_or("header table ends early")?;
        let s = self.b.get(self.at..end).ok_or("header table ends early")?;
        self.at = end;
        Ok(s)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap_or([0; 4])))
    }

    fn var(&mut self, hint: u8) -> Result<usize, String> {
        Ok(match hint {
            0 => self.take(1)?[0] as usize,
            1 => u16::from_le_bytes(self.take(2)?.try_into().unwrap_or([0; 2])) as usize,
            _ => self.u32()? as usize,
        })
    }

    fn text(&mut self, n: usize) -> Result<String, String> {
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "a name is not UTF-8".to_string())
    }
}

fn resolve(indexed: &[String], idx: u8) -> Result<String, String> {
    if idx == 0 {
        return Ok(String::new());
    }
    indexed.get(idx as usize - 1).cloned().ok_or_else(|| format!("metadata index {idx} is not in the table"))
}

fn compound(b: &[u8], entry: u64, indexed: &[String], depth: u32, budget: &mut usize) -> Result<Vec<Property>, String> {
    spend(budget, depth)?;
    let children = group(b, entry)?;
    let Some((last, groups)) = children.split_last() else { return Ok(Vec::new()) };
    let blob = data(b, *last)?;
    let mut cur = Cur { b: blob, at: 0 };
    let mut out = Vec::new();
    while cur.at < blob.len() {
        let info = cur.u32()?;
        let hint = ((info >> 2) & 3) as u8;
        let ptype = info & 3;
        let kind = match ptype {
            0 => PropType::Compound,
            1 => PropType::Scalar,
            _ => PropType::Array,
        };
        let mut n = 0u32;
        let (mut first, mut last_changed) = (0u32, 0u32);
        let mut ts = 0u32;
        let constant = info & 0x800 != 0;
        let explicit_range = info & 0x200 != 0;
        if kind != PropType::Compound {
            n = cur.var(hint)? as u32;
            if explicit_range {
                first = cur.var(hint)? as u32;
                last_changed = (cur.var(hint)? as u32).max(first);
            } else if !constant {
                first = 1;
                last_changed = n.saturating_sub(1);
            }
            if info & 0x100 != 0 {
                ts = cur.var(hint)? as u32;
            }
        }
        let name_len = cur.var(hint)?;
        if name_len == 0 {
            return Err("a property has an empty name".into());
        }
        let name = cur.text(name_len)?;
        let meta_index = ((info >> 20) & 0xFF) as u8;
        let meta = if meta_index == 255 {
            let l = cur.var(hint)?;
            cur.text(l)?
        } else {
            resolve(indexed, meta_index)?
        };
        let at = out.len();
        let entry = *groups.get(at).ok_or_else(|| format!("property {name} has no group"))?;
        let (subs, entries) = if kind == PropType::Compound { (compound(b, entry, indexed, depth + 1, budget)?, Vec::new()) } else { (Vec::new(), group(b, entry)?) };
        out.push(Property {
            name,
            meta,
            meta_index,
            info,
            hint,
            kind,
            scalar_like: ptype == 3,
            pod: ((info >> 4) & 0xF) as u8,
            extent: ((info >> 12) & 0xFF) as u8,
            n,
            first,
            last: last_changed,
            ts,
            homogenous: info & 0x400 != 0,
            constant,
            explicit_range,
            subs,
            entries,
        });
    }
    if out.len() != groups.len() {
        return Err(format!("{} property headers for {} property groups", out.len(), groups.len()));
    }
    Ok(out)
}

fn object(b: &[u8], entry: u64, head: (String, String, u8), indexed: &[String], depth: u32, budget: &mut usize) -> Result<Object, String> {
    let (name, meta, meta_index) = head;
    spend(budget, depth)?;
    let children = group(b, entry)?;
    let Some((last, rest)) = children.split_last() else { return Err(format!("object {name} has no header blob")) };
    let blob = data(b, *last)?;
    if blob.len() < 32 {
        return Err(format!("object {name} header blob is {} bytes", blob.len()));
    }
    let (table, hash) = blob.split_at(blob.len() - 32);
    let props = match rest.first() {
        Some(p) => compound(b, *p, indexed, depth + 1, budget)?,
        None => Vec::new(),
    };
    let mut cur = Cur { b: table, at: 0 };
    let mut kids = Vec::new();
    while cur.at < table.len() {
        let l = cur.u32()? as usize;
        let cname = cur.text(l)?;
        let mi = cur.take(1)?[0];
        let cmeta = if mi == 255 {
            let l = cur.u32()? as usize;
            cur.text(l)?
        } else {
            resolve(indexed, mi)?
        };
        let g = *rest.get(1 + kids.len()).ok_or_else(|| format!("child {cname} has no group"))?;
        kids.push(object(b, g, (cname, cmeta, mi), indexed, depth + 1, budget)?);
    }
    if kids.len() + 1 != rest.len().max(1) {
        return Err(format!("object {name}: {} child headers for {} child groups", kids.len(), rest.len().saturating_sub(1)));
    }
    Ok(Object { name, meta, meta_index, props, children: kids, hash: hash.to_vec(), group_len: children.len() })
}

pub fn open(bytes: &[u8]) -> Result<Archive<'_>, String> {
    if bytes.len() < HEADER_LEN || &bytes[0..5] != b"Ogawa" {
        return Err("not an Ogawa file".into());
    }
    let frozen = bytes[5];
    if frozen != 0xFF {
        return Err("the file was not cleanly closed".into());
    }
    let version = (u16::from(bytes[6]) << 8) | u16::from(bytes[7]);
    if version != 1 {
        return Err(format!("Ogawa version {version} is not supported"));
    }
    let root_offset = rd_u64(bytes, 8)?;
    let root = group(bytes, root_offset)?;
    if root.len() < 6 {
        return Err(format!("the archive root has {} children, needs 6", root.len()));
    }
    let i32_of = |e: u64| -> Result<i32, String> {
        let d = data(bytes, e)?;
        Ok(i32::from_le_bytes(d.try_into().map_err(|_| "a version field is not 4 bytes".to_string())?))
    };
    let file_version = i32_of(root[0])?;
    let library_version = i32_of(root[1])?;
    if !(0..=0).contains(&file_version) || library_version < 9999 {
        return Err(format!("unsupported versions {file_version} and {library_version}"));
    }
    let app_meta = String::from_utf8_lossy(data(bytes, root[3])?).into_owned();
    let mut samplings = Vec::new();
    let t = data(bytes, root[4])?;
    let mut cur = Cur { b: t, at: 0 };
    while cur.at < t.len() {
        let max = cur.u32()?;
        let per = f64::from_le_bytes(cur.take(8)?.try_into().unwrap_or([0; 8]));
        let count = cur.u32()? as usize;
        if count == 0 {
            return Err("a time sampling stores no times".into());
        }
        let mut times = Vec::new();
        for _ in 0..count {
            times.push(f64::from_le_bytes(cur.take(8)?.try_into().unwrap_or([0; 8])));
        }
        samplings.push(Sampling { max, per, times });
    }
    let raw = data(bytes, root[5])?;
    let mut indexed = Vec::new();
    let mut cur = Cur { b: raw, at: 0 };
    while cur.at < raw.len() {
        let l = cur.take(1)?[0] as usize;
        indexed.push(cur.text(l)?);
    }
    let mut budget = MAX_NODES;
    let top = object(bytes, root[2], ("ABC".into(), app_meta.clone(), 0), &indexed, 0, &mut budget)?;
    Ok(Archive { bytes, frozen, version, root_offset, root_children: root.len(), file_version, library_version, app_meta, samplings, indexed, indexed_raw_len: raw.len(), top })
}

impl Property {
    pub fn find(&self, name: &str) -> Option<&Property> {
        self.subs.iter().find(|p| p.name == name)
    }

    pub fn child_index(&self, i: u32) -> usize {
        let (f, l) = (self.first, self.last);
        if self.constant || i < f {
            0
        } else if i >= l {
            (l.saturating_sub(f) as usize).saturating_add(1)
        } else {
            ((i - f) as usize).saturating_add(1)
        }
    }

    pub fn sample<'a>(&self, ar: &Archive<'a>, i: u32) -> Result<Sample<'a>, String> {
        if self.kind == PropType::Compound || i >= self.n {
            return Err(format!("property {} has no sample {i}", self.name));
        }
        let k = self.child_index(i);
        let (di, dims) = if self.kind == PropType::Array { (2 * k, self.entries.get(2 * k + 1).copied().unwrap_or(0)) } else { (k, 0) };
        let entry = *self.entries.get(di).ok_or_else(|| format!("property {} lacks child {di}", self.name))?;
        let blob = data(ar.bytes, entry)?;
        if blob.is_empty() {
            return Ok(Sample { key: None, payload: blob, dims_entry: dims });
        }
        if blob.len() < 16 {
            return Err(format!("property {} sample blob is {} bytes", self.name, blob.len()));
        }
        let mut key = [0u8; 16];
        key.copy_from_slice(&blob[..16]);
        Ok(Sample { key: Some(key), payload: &blob[16..], dims_entry: dims })
    }
}

impl Object {
    pub fn prop(&self, name: &str) -> Option<&Property> {
        self.props.iter().find(|p| p.name == name)
    }
}

impl Archive<'_> {
    pub fn sample_time(&self, ts: u32, i: u32) -> Option<f64> {
        let s = self.samplings.get(ts as usize)?;
        if s.times.len() == 1 {
            Some(s.times[0] + s.per * f64::from(i))
        } else {
            let n = s.times.len() as u32;
            Some(s.times[(i % n) as usize] + s.per * f64::from(i / n))
        }
    }
}
