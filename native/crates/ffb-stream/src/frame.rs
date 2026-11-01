pub const MAGIC: [u8; 4] = *b"FFBS";
pub const VERSION: u16 = 1;
pub const HEADER_LEN: usize = 44;
pub const KIND_TOPOLOGY: u16 = 1;
pub const KIND_POSITIONS: u16 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub version: u16,
    pub kind: u16,
    pub frame: u32,
    pub object: u32,
    pub vertex_count: u32,
    pub origin: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Positions(Vec<f32>),
    Topology { indices: Vec<u32>, panel_ids: Vec<u16> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub header: Header,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    TooShort,
    BadMagic,
    BadVersion(u16),
    BadKind(u16),
    BadLength,
    IndexOutOfRange,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::TooShort => f.write_str("frame shorter than its header"),
            DecodeError::BadMagic => f.write_str("frame magic is not FFBS"),
            DecodeError::BadVersion(v) => write!(f, "unsupported frame version {v}"),
            DecodeError::BadKind(k) => write!(f, "unknown frame kind {k}"),
            DecodeError::BadLength => f.write_str("frame body length does not match its header"),
            DecodeError::IndexOutOfRange => f.write_str("triangle index is not below the vertex count"),
        }
    }
}

impl std::error::Error for DecodeError {}

fn put_header(out: &mut Vec<u8>, kind: u16, frame: u32, object: u32, count: u32, origin: [f32; 3], scale: [f32; 3]) {
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&frame.to_le_bytes());
    out.extend_from_slice(&object.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    for v in origin.into_iter().chain(scale) {
        out.extend_from_slice(&v.to_le_bytes());
    }
}

pub fn encode_positions(frame: u32, object: u32, positions: &[f32]) -> Vec<u8> {
    let count = positions.len() / 3;
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for v in positions.chunks_exact(3) {
        for a in 0..3 {
            if v[a].is_finite() {
                lo[a] = lo[a].min(v[a]);
                hi[a] = hi[a].max(v[a]);
            }
        }
    }
    let mut scale = [1.0f32; 3];
    for a in 0..3 {
        if !lo[a].is_finite() {
            lo[a] = 0.0;
            hi[a] = 0.0;
        }
        let extent = hi[a] - lo[a];
        scale[a] = if extent > 0.0 { extent } else { 1.0 };
    }
    let mut out = Vec::with_capacity(HEADER_LEN + count * 6);
    put_header(&mut out, KIND_POSITIONS, frame, object, count as u32, lo, scale);
    for v in positions.chunks_exact(3) {
        for a in 0..3 {
            let t = if v[a].is_finite() { ((v[a] - lo[a]) / scale[a]).clamp(0.0, 1.0) } else { 0.0 };
            out.extend_from_slice(&((t * 65535.0).round() as u16).to_le_bytes());
        }
    }
    out
}

pub fn encode_topology(frame: u32, object: u32, vertex_count: u32, indices: &[u32], panel_ids: &[u16]) -> Vec<u8> {
    let tris = indices.len() / 3;
    let mut out = Vec::with_capacity(HEADER_LEN + 4 + tris * 12 + tris * 2 + 2);
    put_header(&mut out, KIND_TOPOLOGY, frame, object, vertex_count, [0.0; 3], [0.0; 3]);
    out.extend_from_slice(&(tris as u32).to_le_bytes());
    for i in &indices[..tris * 3] {
        out.extend_from_slice(&i.to_le_bytes());
    }
    for t in 0..tris {
        out.extend_from_slice(&panel_ids.get(t).copied().unwrap_or(0).to_le_bytes());
    }
    out
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn f32_at(b: &[u8], at: usize) -> f32 {
    f32::from_bits(u32_at(b, at))
}

pub fn decode(bytes: &[u8]) -> Result<Frame, DecodeError> {
    if bytes.len() < HEADER_LEN {
        return Err(DecodeError::TooShort);
    }
    if bytes[0..4] != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let version = u16_at(bytes, 4);
    if version != VERSION {
        return Err(DecodeError::BadVersion(version));
    }
    let kind = u16_at(bytes, 6);
    let header = Header {
        version,
        kind,
        frame: u32_at(bytes, 8),
        object: u32_at(bytes, 12),
        vertex_count: u32_at(bytes, 16),
        origin: [f32_at(bytes, 20), f32_at(bytes, 24), f32_at(bytes, 28)],
        scale: [f32_at(bytes, 32), f32_at(bytes, 36), f32_at(bytes, 40)],
    };
    let body = &bytes[HEADER_LEN..];
    match kind {
        KIND_POSITIONS => {
            let n = header.vertex_count as usize;
            if body.len() != n.checked_mul(6).ok_or(DecodeError::BadLength)? {
                return Err(DecodeError::BadLength);
            }
            let mut positions = Vec::with_capacity(n * 3);
            for (k, c) in body.chunks_exact(2).enumerate() {
                let a = k % 3;
                let q = u16::from_le_bytes([c[0], c[1]]) as f32;
                positions.push(header.origin[a] + q / 65535.0 * header.scale[a]);
            }
            Ok(Frame { header, body: Body::Positions(positions) })
        }
        KIND_TOPOLOGY => {
            if body.len() < 4 {
                return Err(DecodeError::BadLength);
            }
            let tris = u32_at(body, 0) as usize;
            let need = tris.checked_mul(14).and_then(|v| v.checked_add(4)).ok_or(DecodeError::BadLength)?;
            if body.len() != need {
                return Err(DecodeError::BadLength);
            }
            let indices: Vec<u32> = (0..tris * 3).map(|k| u32_at(body, 4 + k * 4)).collect();
            if indices.iter().any(|&i| i >= header.vertex_count) {
                return Err(DecodeError::IndexOutOfRange);
            }
            let base = 4 + tris * 12;
            let panel_ids = (0..tris).map(|t| u16_at(body, base + t * 2)).collect();
            Ok(Frame { header, body: Body::Topology { indices, panel_ids } })
        }
        other => Err(DecodeError::BadKind(other)),
    }
}
