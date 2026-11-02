use super::murmur;
use super::ogawa::{Writer, EMPTY_DATA};

pub const FILE_VERSION: i32 = 0;
pub const LIBRARY_VERSION: i32 = 10812;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pod {
    Bool = 0,
    U8 = 1,
    I8 = 2,
    U16 = 3,
    I16 = 4,
    U32 = 5,
    I32 = 6,
    U64 = 7,
    I64 = 8,
    F16 = 9,
    F32 = 10,
    F64 = 11,
}

impl Pod {
    pub fn size(self) -> usize {
        match self {
            Pod::Bool | Pod::U8 | Pod::I8 => 1,
            Pod::U16 | Pod::I16 | Pod::F16 => 2,
            Pod::U32 | Pod::I32 | Pod::F32 => 4,
            Pod::U64 | Pod::I64 | Pod::F64 => 8,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Kind {
    Compound(Vec<Prop>),
    Scalar,
    Array,
}

#[derive(Clone, Debug)]
pub struct Prop {
    pub name: String,
    pub meta: String,
    pub kind: Kind,
    pub pod: Pod,
    pub extent: u8,
    pub ts: u32,
    pub count: usize,
    pub payloads: Vec<Vec<u8>>,
}

impl Prop {
    pub fn compound(name: &str, meta: &str, children: Vec<Prop>) -> Prop {
        Prop { name: name.into(), meta: meta.into(), kind: Kind::Compound(children), pod: Pod::U8, extent: 1, ts: 0, count: 0, payloads: Vec::new() }
    }

    pub fn scalar(name: &str, meta: &str, pod: Pod, extent: u8, ts: u32, count: usize, payloads: Vec<Vec<u8>>) -> Prop {
        Prop { name: name.into(), meta: meta.into(), kind: Kind::Scalar, pod, extent, ts, count, payloads }
    }

    pub fn array(name: &str, meta: &str, pod: Pod, extent: u8, ts: u32, count: usize, payloads: Vec<Vec<u8>>) -> Prop {
        Prop { name: name.into(), meta: meta.into(), kind: Kind::Array, pod, extent, ts, count, payloads }
    }

    fn payload(&self, i: usize) -> &[u8] {
        if self.payloads.len() == 1 {
            &self.payloads[0]
        } else {
            &self.payloads[i]
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Obj {
    pub name: String,
    pub meta: String,
    pub props: Vec<Prop>,
    pub children: Vec<Obj>,
}

#[derive(Clone, Debug)]
pub struct Archive {
    pub application: String,
    samplings: Vec<(f64, f64)>,
    pub root: Vec<Obj>,
}

impl Archive {
    pub fn new(application: &str) -> Archive {
        Archive { application: application.into(), samplings: vec![(1.0, 0.0)], root: Vec::new() }
    }

    pub fn add_uniform_sampling(&mut self, seconds_per_sample: f64, start: f64) -> u32 {
        if let Some(i) = self.samplings.iter().position(|s| *s == (seconds_per_sample, start)) {
            return i as u32;
        }
        self.samplings.push((seconds_per_sample, start));
        (self.samplings.len() - 1) as u32
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut e = Emit { w: Writer::new(), meta: Vec::new(), max: vec![0; self.samplings.len()] };
        let top = e.object(&Obj { name: "ABC".into(), meta: String::new(), props: Vec::new(), children: self.root.clone() }, self.samplings.len())?;
        let file_version = e.w.data(&FILE_VERSION.to_le_bytes());
        let library_version = e.w.data(&LIBRARY_VERSION.to_le_bytes());
        if self.application.contains(';') || self.application.contains('=') {
            return Err("application name must not contain ; or =".into());
        }
        let app = if self.application.is_empty() { String::new() } else { format!("_ai_Application={}", self.application) };
        let app = e.w.data(app.as_bytes());
        let mut table = Vec::new();
        for (i, (per, start)) in self.samplings.iter().enumerate() {
            table.extend_from_slice(&e.max[i].to_le_bytes());
            table.extend_from_slice(&per.to_le_bytes());
            table.extend_from_slice(&1u32.to_le_bytes());
            table.extend_from_slice(&start.to_le_bytes());
        }
        let table = e.w.data(&table);
        let mut indexed = Vec::new();
        for m in &e.meta {
            indexed.push(m.len() as u8);
            indexed.extend_from_slice(m.as_bytes());
        }
        let indexed = e.w.data(&indexed);
        let root = e.w.group(&[file_version, library_version, top, app, table, indexed]);
        e.w.finish(root)
    }
}

struct Emit {
    w: Writer,
    meta: Vec<String>,
    max: Vec<u32>,
}

struct Done {
    entry: u64,
    n: u32,
    first: u32,
    last: u32,
    homogenous: bool,
    scalar_like: bool,
}

fn width(v: usize) -> u8 {
    if v < 256 {
        0
    } else if v < 65536 {
        1
    } else {
        2
    }
}

fn put(out: &mut Vec<u8>, v: usize, hint: u8) {
    match hint {
        0 => out.push(v as u8),
        1 => out.extend_from_slice(&(v as u16).to_le_bytes()),
        _ => out.extend_from_slice(&(v as u32).to_le_bytes()),
    }
}

fn check_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.contains('/') {
        return Err(format!("invalid Alembic name {name:?}"));
    }
    Ok(())
}

impl Emit {
    fn meta_index(&mut self, s: &str) -> u8 {
        if s.is_empty() {
            return 0;
        }
        if let Some(p) = self.meta.iter().position(|m| m == s) {
            return (p + 1) as u8;
        }
        if s.len() < 256 && self.meta.len() < 254 {
            self.meta.push(s.to_string());
            return self.meta.len() as u8;
        }
        255
    }

    fn blob(&mut self, payload: &[u8]) -> u64 {
        if payload.is_empty() {
            return EMPTY_DATA;
        }
        let mut v = Vec::with_capacity(16 + payload.len());
        v.extend_from_slice(&murmur::key(payload));
        v.extend_from_slice(payload);
        self.w.data(&v)
    }

    fn compound(&mut self, props: &[Prop], samplings: usize) -> Result<u64, String> {
        if props.is_empty() {
            return Ok(0);
        }
        let mut entries = Vec::with_capacity(props.len() + 1);
        let mut done = Vec::with_capacity(props.len());
        for p in props {
            check_name(&p.name)?;
            let d = self.prop(p, samplings)?;
            entries.push(d.entry);
            done.push(d);
        }
        let mut hdr = Vec::new();
        for (p, d) in props.iter().zip(&done) {
            self.header(p, d, &mut hdr);
        }
        entries.push(self.w.data(&hdr));
        Ok(self.w.group(&entries))
    }

    fn prop(&mut self, p: &Prop, samplings: usize) -> Result<Done, String> {
        if let Kind::Compound(children) = &p.kind {
            let entry = self.compound(children, samplings)?;
            return Ok(Done { entry, n: 0, first: 0, last: 0, homogenous: true, scalar_like: false });
        }
        let n = p.count;
        if n == 0 || !(p.payloads.len() == 1 || p.payloads.len() == n) {
            return Err(format!("property {} needs {n} samples", p.name));
        }
        if p.ts as usize >= samplings {
            return Err(format!("property {} uses an unknown time sampling", p.name));
        }
        let unit = p.pod.size() * usize::from(p.extent);
        let scalar = matches!(p.kind, Kind::Scalar);
        for i in 0..p.payloads.len() {
            let len = p.payloads[i].len();
            if len / unit * unit != len || (scalar && len != unit) {
                return Err(format!("property {} has a sample of {len} bytes for elements of {unit}", p.name));
            }
        }
        let (mut first, mut last) = (0usize, 0usize);
        if p.payloads.len() > 1 {
            for i in 1..n {
                if p.payload(i) != p.payload(i - 1) {
                    if first == 0 {
                        first = i;
                    }
                    last = i;
                }
            }
        }
        let zero = self.blob(p.payload(0));
        let mut stored = vec![zero];
        if !(first == 0 && last == 0) {
            let mut prev = zero;
            for i in first..=last {
                let entry = if p.payload(i) == p.payload(i - 1) { prev } else { self.blob(p.payload(i)) };
                stored.push(entry);
                prev = entry;
            }
        }
        let mut entries = Vec::with_capacity(stored.len() * 2);
        for s in stored {
            entries.push(s);
            if !scalar {
                entries.push(EMPTY_DATA);
            }
        }
        let entry = self.w.group(&entries);
        let constant = first == 0 && last == 0;
        let slot = &mut self.max[p.ts as usize];
        *slot = (*slot).max(if constant { 1 } else { n as u32 });
        let homogenous = p.payloads.iter().all(|x| x.len() == p.payloads[0].len());
        let scalar_like = !scalar && p.payloads.iter().all(|x| x.len() == unit);
        Ok(Done { entry, n: n as u32, first: first as u32, last: last as u32, homogenous, scalar_like })
    }

    fn header(&mut self, p: &Prop, d: &Done, out: &mut Vec<u8>) {
        let mi = self.meta_index(&p.meta);
        let inline = mi == 255;
        let mut big = p.name.len();
        if inline {
            big = big.max(p.meta.len());
        }
        let compound = matches!(p.kind, Kind::Compound(_));
        if !compound {
            big = big.max(d.n as usize).max(p.ts as usize);
        }
        let hint = width(big);
        let mut info: u32 = u32::from(hint) << 2 | u32::from(mi) << 20;
        let constant = d.first == 0 && d.last == 0;
        let explicit = !constant && !(d.first == 1 && d.last + 1 == d.n);
        if !compound {
            info |= match p.kind {
                Kind::Scalar => 1,
                _ if d.scalar_like => 3,
                _ => 2,
            };
            info |= (p.pod as u32) << 4;
            info |= u32::from(p.extent) << 12;
            if p.ts != 0 {
                info |= 0x100;
            }
            if explicit {
                info |= 0x200;
            }
            if d.homogenous {
                info |= 0x400;
            }
            if constant {
                info |= 0x800;
            }
        }
        out.extend_from_slice(&info.to_le_bytes());
        if !compound {
            put(out, d.n as usize, hint);
            if explicit {
                put(out, d.first as usize, hint);
                put(out, d.last as usize, hint);
            }
            if p.ts != 0 {
                put(out, p.ts as usize, hint);
            }
        }
        put(out, p.name.len(), hint);
        out.extend_from_slice(p.name.as_bytes());
        if inline {
            put(out, p.meta.len(), hint);
            out.extend_from_slice(p.meta.as_bytes());
        }
    }

    fn object(&mut self, o: &Obj, samplings: usize) -> Result<u64, String> {
        let mut child_entries = Vec::with_capacity(o.children.len());
        for c in &o.children {
            check_name(&c.name)?;
            child_entries.push(self.object(c, samplings)?);
        }
        let props = self.compound(&o.props, samplings)?;
        let mut hdr = Vec::new();
        for c in &o.children {
            hdr.extend_from_slice(&(c.name.len() as u32).to_le_bytes());
            hdr.extend_from_slice(c.name.as_bytes());
            let mi = self.meta_index(&c.meta);
            hdr.push(mi);
            if mi == 255 {
                hdr.extend_from_slice(&(c.meta.len() as u32).to_le_bytes());
                hdr.extend_from_slice(c.meta.as_bytes());
            }
        }
        hdr.extend_from_slice(&[0u8; 32]);
        let mut entries = Vec::with_capacity(child_entries.len() + 2);
        entries.push(props);
        entries.extend(child_entries);
        entries.push(self.w.data(&hdr));
        Ok(self.w.group(&entries))
    }
}
