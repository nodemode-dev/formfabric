use serde::{Deserialize, Serialize};

pub const MAGIC: &[u8; 4] = b"FFAC";
pub const MAX_FRAMES: u32 = 100_000;
pub const MAX_HEADER: usize = 64 << 20;
pub const MAX_BYTES: usize = 480 << 20;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelSpec {
    pub id: u32,
    pub vertices: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BodySpec {
    pub id: u32,
    pub joints: Vec<String>,
    pub morph_to: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FrameStat {
    pub frame: u32,
    pub penetration_mm: f32,
    pub self_intersections: u32,
    pub max_displacement_mm: f32,
    pub seam_gap_mm: f32,
    pub max_velocity_mm_s: f32,
    pub ms: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Header {
    pub name: String,
    pub fps: f32,
    pub start: u32,
    pub frames: u32,
    pub quantized: bool,
    pub panels: Vec<PanelSpec>,
    pub bodies: Vec<BodySpec>,
    pub stats: Vec<FrameStat>,
}

#[derive(Debug, Clone, Default)]
pub struct BodyFrame {
    pub id: u32,
    pub offset: [f32; 3],
    pub morph: f32,
    pub quats: Vec<[f32; 4]>,
}

impl Header {
    pub fn frame_size(&self) -> usize {
        self.checked_frame_size().unwrap_or(usize::MAX)
    }

    pub fn checked_frame_size(&self) -> Option<usize> {
        let mut size = 0usize;
        for p in &self.panels {
            let n = p.vertices as usize;
            let s = if self.quantized { n.checked_mul(6)?.checked_add(24)? } else { n.checked_mul(12)? };
            size = size.checked_add(s)?;
        }
        for b in &self.bodies {
            size = size.checked_add(b.joints.len().checked_mul(16)?.checked_add(16)?)?;
        }
        Some(size)
    }
}

pub struct Writer {
    header: Header,
    payload: Vec<u8>,
}

impl Writer {
    pub fn new(header: Header) -> Writer {
        Writer { header, payload: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.payload.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    pub fn push(&mut self, panels: &[Vec<f32>], bodies: &[BodyFrame], stat: FrameStat) -> Result<(), String> {
        if panels.len() != self.header.panels.len() || bodies.len() != self.header.bodies.len() {
            return Err("the frame does not match the cache layout".into());
        }
        let before = self.payload.len();
        for (p, spec) in panels.iter().zip(&self.header.panels) {
            if p.len() != spec.vertices as usize * 3 {
                return Err("a panel frame has the wrong vertex count".into());
            }
            if self.header.quantized {
                let mut lo = [f32::MAX; 3];
                let mut hi = [f32::MIN; 3];
                for v in p.as_chunks::<3>().0 {
                    for k in 0..3 {
                        lo[k] = lo[k].min(v[k]);
                        hi[k] = hi[k].max(v[k]);
                    }
                }
                if p.is_empty() {
                    lo = [0.0; 3];
                    hi = [0.0; 3];
                }
                let ext = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
                for x in lo.iter().chain(ext.iter()) {
                    self.payload.extend_from_slice(&x.to_le_bytes());
                }
                for v in p.as_chunks::<3>().0 {
                    for k in 0..3 {
                        let u = if ext[k] > 1e-9 { ((v[k] - lo[k]) / ext[k] * 65535.0).round().clamp(0.0, 65535.0) as u16 } else { 0 };
                        self.payload.extend_from_slice(&u.to_le_bytes());
                    }
                }
            } else {
                for x in p {
                    self.payload.extend_from_slice(&x.to_le_bytes());
                }
            }
        }
        for (b, spec) in bodies.iter().zip(&self.header.bodies) {
            if b.quats.len() != spec.joints.len() {
                return Err("a body frame has the wrong joint count".into());
            }
            for x in b.offset.iter().chain(std::iter::once(&b.morph)) {
                self.payload.extend_from_slice(&x.to_le_bytes());
            }
            for q in &b.quats {
                for x in q {
                    self.payload.extend_from_slice(&x.to_le_bytes());
                }
            }
        }
        if self.payload.len() - before != self.header.frame_size() {
            return Err("internal cache layout mismatch".into());
        }
        if self.payload.len() > MAX_BYTES {
            return Err("the cache is larger than 1.5 GB; record fewer frames or use quantized storage".into());
        }
        self.header.stats.push(stat);
        self.header.frames += 1;
        Ok(())
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn finish(self) -> Result<Vec<u8>, String> {
        let json = serde_json::to_vec(&self.header).map_err(|e| e.to_string())?;
        let mut out = Vec::with_capacity(12 + json.len() + self.payload.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&u16::from(self.header.quantized).to_le_bytes());
        out.extend_from_slice(&(json.len() as u32).to_le_bytes());
        out.extend_from_slice(&json);
        out.extend_from_slice(&self.payload);
        Ok(out)
    }
}

pub struct Cache<'a> {
    pub header: Header,
    payload: &'a [u8],
    frame_size: usize,
}

fn f32_at(b: &[u8], at: usize) -> f32 {
    f32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl<'a> Cache<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Cache<'a>, String> {
        if bytes.len() < 12 || &bytes[..4] != MAGIC {
            return Err("not an animation cache entry".into());
        }
        if u16::from_le_bytes([bytes[4], bytes[5]]) != 1 {
            return Err("unsupported animation cache version".into());
        }
        let hl = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        if hl > MAX_HEADER || 12 + hl > bytes.len() {
            return Err("the animation cache header is malformed".into());
        }
        let header: Header = serde_json::from_slice(&bytes[12..12 + hl]).map_err(|e| format!("the animation cache header is malformed: {e}"))?;
        if header.frames == 0 || header.frames > MAX_FRAMES || header.panels.len() > 4096 || header.bodies.len() > 256 {
            return Err("the animation cache is over the limits".into());
        }
        if header.stats.len() != header.frames as usize || header.bodies.iter().any(|b| b.joints.len() > 4096) {
            return Err("the animation cache header is inconsistent".into());
        }
        let frame_size = header.checked_frame_size().ok_or("the animation cache layout is too large")?;
        let payload = &bytes[12 + hl..];
        if frame_size.checked_mul(header.frames as usize) != Some(payload.len()) {
            return Err("the animation cache payload does not match its header".into());
        }
        Ok(Cache { header, payload, frame_size })
    }

    pub fn panel_frame(&self, frame: u32, index: usize) -> Option<Vec<f32>> {
        if frame >= self.header.frames {
            return None;
        }
        let mut at = frame as usize * self.frame_size;
        for (k, p) in self.header.panels.iter().enumerate() {
            let n = p.vertices as usize;
            let size = if self.header.quantized { 24 + n * 6 } else { n * 12 };
            if k == index {
                let b = &self.payload[at..at + size];
                let mut out = Vec::with_capacity(n * 3);
                if self.header.quantized {
                    let lo = [f32_at(b, 0), f32_at(b, 4), f32_at(b, 8)];
                    let ext = [f32_at(b, 12), f32_at(b, 16), f32_at(b, 20)];
                    for v in 0..n {
                        for c in 0..3 {
                            let o = 24 + (v * 3 + c) * 2;
                            let u = u16::from_le_bytes([b[o], b[o + 1]]);
                            out.push(lo[c] + f32::from(u) / 65535.0 * ext[c]);
                        }
                    }
                } else {
                    for i in 0..n * 3 {
                        out.push(f32_at(b, i * 4));
                    }
                }
                return Some(out);
            }
            at += size;
        }
        None
    }

    pub fn body_frame(&self, frame: u32, index: usize) -> Option<BodyFrame> {
        if frame >= self.header.frames {
            return None;
        }
        let mut at = frame as usize * self.frame_size;
        for p in &self.header.panels {
            let n = p.vertices as usize;
            at += if self.header.quantized { 24 + n * 6 } else { n * 12 };
        }
        for (k, spec) in self.header.bodies.iter().enumerate() {
            let size = 16 + spec.joints.len() * 16;
            if k == index {
                let b = &self.payload[at..at + size];
                let quats = (0..spec.joints.len()).map(|j| [f32_at(b, 16 + j * 16), f32_at(b, 20 + j * 16), f32_at(b, 24 + j * 16), f32_at(b, 28 + j * 16)]).collect();
                return Some(BodyFrame { id: spec.id, offset: [f32_at(b, 0), f32_at(b, 4), f32_at(b, 8)], morph: f32_at(b, 12), quats });
            }
            at += size;
        }
        None
    }
}

pub fn read_header(bytes: &[u8]) -> Result<Header, String> {
    Cache::parse(bytes).map(|c| c.header)
}

fn same_layout(a: &Header, b: &Header) -> bool {
    a.panels == b.panels && a.bodies == b.bodies && a.quantized == b.quantized
}

fn encode(header: &Header, payload: &[u8]) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(header).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(12 + json.len() + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&u16::from(header.quantized).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&json);
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn slice(bytes: &[u8], from: u32, count: u32, new_start: u32) -> Result<Vec<u8>, String> {
    let cache = Cache::parse(bytes)?;
    if count == 0 || from.checked_add(count).is_none_or(|e| e > cache.header.frames) {
        return Err("the frame range is outside the cache".into());
    }
    let mut header = cache.header.clone();
    header.start = new_start;
    header.frames = count;
    header.stats = cache.header.stats[from as usize..(from + count) as usize].to_vec();
    let a = from as usize * cache.frame_size;
    let b = (from + count) as usize * cache.frame_size;
    encode(&header, &cache.payload[a..b])
}

pub struct Composed {
    pub bytes: Vec<u8>,
    pub start: u32,
    pub frames: u32,
}

struct Pulled {
    panels: Vec<Vec<f32>>,
    bodies: Vec<BodyFrame>,
    stat: FrameStat,
}

fn pull(c: &Cache, rel: u32) -> Option<Pulled> {
    let panels = (0..c.header.panels.len()).map(|i| c.panel_frame(rel, i)).collect::<Option<Vec<_>>>()?;
    let bodies = (0..c.header.bodies.len()).map(|i| c.body_frame(rel, i)).collect::<Option<Vec<_>>>()?;
    Some(Pulled { panels, bodies, stat: c.header.stats.get(rel as usize).cloned().unwrap_or_default() })
}

fn blend(a: &Pulled, b: &Pulled, t: f32) -> Pulled {
    let mix = |x: f32, y: f32| x + (y - x) * t;
    let panels = a.panels.iter().zip(&b.panels).map(|(p, q)| p.iter().zip(q).map(|(x, y)| mix(*x, *y)).collect()).collect();
    let bodies = a
        .bodies
        .iter()
        .zip(&b.bodies)
        .map(|(p, q)| {
            let quats = p
                .quats
                .iter()
                .zip(&q.quats)
                .map(|(x, y)| {
                    let sign = if x.iter().zip(y).map(|(m, n)| m * n).sum::<f32>() < 0.0 { -1.0 } else { 1.0 };
                    let mut o = [0.0f32; 4];
                    for k in 0..4 {
                        o[k] = mix(x[k], y[k] * sign);
                    }
                    let n = o.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-9);
                    o.map(|v| v / n)
                })
                .collect();
            BodyFrame { id: p.id, offset: [mix(p.offset[0], q.offset[0]), mix(p.offset[1], q.offset[1]), mix(p.offset[2], q.offset[2])], morph: mix(p.morph, q.morph), quats }
        })
        .collect();
    Pulled { panels, bodies, stat: a.stat.clone() }
}

pub fn compose(layers: &[(u32, &[u8])]) -> Result<Composed, String> {
    let caches: Vec<(u32, Cache)> = layers.iter().map(|(s, b)| Cache::parse(b).map(|c| (*s, c))).collect::<Result<_, _>>()?;
    let Some((_, top)) = caches.last() else { return Err("nothing to merge".into()) };
    if caches.iter().any(|(_, c)| !same_layout(&c.header, &top.header)) {
        return Err("the layers were recorded with different panels or bodies and cannot be merged".into());
    }
    let first = caches.iter().map(|(s, _)| *s).min().unwrap_or(0);
    let mut last = 0u32;
    for (s, c) in &caches {
        if *s > MAX_FRAMES {
            return Err("a layer starts beyond the frame limit".into());
        }
        last = last.max(s.checked_add(c.header.frames.saturating_sub(1)).ok_or("a layer range overflows")?);
    }
    let total = u64::from(last - first) + 1;
    if total > u64::from(MAX_FRAMES) {
        return Err(format!("the merged layer would span {total} frames, over the limit of {MAX_FRAMES}; move the layers closer first"));
    }
    if total.saturating_mul(top.header.frame_size() as u64) > MAX_BYTES as u64 {
        return Err("the merged layer would be larger than the cache size limit; move the layers closer or merge fewer frames".into());
    }
    let at = |f: u32| -> Option<Pulled> {
        caches.iter().rev().find(|(s, c)| f >= *s && f - *s < c.header.frames).and_then(|(s, c)| pull(c, f - *s))
    };
    let mut header = top.header.clone();
    header.start = first;
    header.frames = 0;
    header.stats = Vec::new();
    let mut writer = Writer::new(header);
    let mut prev: Option<(u32, Pulled)> = None;
    let mut next: Option<(u32, Pulled)> = None;
    for f in first..=last {
        let mut one = match at(f) {
            Some(p) => p,
            None => {
                if next.as_ref().is_none_or(|(n, _)| *n <= f) {
                    next = (f + 1..=last).find_map(|n| at(n).map(|x| (n, x)));
                }
                match (&prev, &next) {
                    (Some((p, a)), Some((n, b))) => blend(a, b, (f - p) as f32 / (n - p) as f32),
                    _ => return Err("a frame could not be resolved".into()),
                }
            }
        };
        one.stat.frame = f;
        writer.push(&one.panels, &one.bodies, one.stat.clone())?;
        prev = Some((f, one));
    }
    let frames = writer.header().frames;
    Ok(Composed { bytes: writer.finish()?, start: first, frames })
}
