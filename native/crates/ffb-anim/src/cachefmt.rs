use serde::{Deserialize, Serialize};

pub const MAGIC: &[u8; 4] = b"FFAC";
pub const MAX_FRAMES: u32 = 100_000;
pub const MAX_HEADER: usize = 64 << 20;
pub const MAX_BYTES: usize = 3 << 29;

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
        let mut size = 0usize;
        for p in &self.panels {
            let n = p.vertices as usize;
            size += if self.quantized { 24 + n * 6 } else { n * 12 };
        }
        for b in &self.bodies {
            size += 16 + b.joints.len() * 16;
        }
        size
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
                for v in p.chunks_exact(3) {
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
                for v in p.chunks_exact(3) {
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
        if header.frames > MAX_FRAMES || header.panels.len() > 4096 || header.bodies.len() > 256 {
            return Err("the animation cache is over the limits".into());
        }
        let frame_size = header.frame_size();
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
