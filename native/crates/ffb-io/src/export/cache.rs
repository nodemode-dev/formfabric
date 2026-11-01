use super::{Animation, File, Options};

pub fn mdd(a: &Animation, o: &Options) -> Vec<File> {
    let frames = a.frames.len();
    let points = a.frames.first().map_or(0, Vec::len);
    let fps = if o.fps > 0.0 { o.fps } else { 30.0 };
    let mut out = Vec::with_capacity(8 + frames * 4 + frames * points * 12);
    out.extend_from_slice(&(frames as i32).to_be_bytes());
    out.extend_from_slice(&(points as i32).to_be_bytes());
    for k in 0..frames {
        out.extend_from_slice(&(k as f32 / fps).to_be_bytes());
    }
    for f in &a.frames {
        for p in f {
            for c in p {
                out.extend_from_slice(&c.to_be_bytes());
            }
        }
    }
    vec![File { name: format!("{}.mdd", o.stem), bytes: out }]
}

pub fn pc2(a: &Animation, o: &Options) -> Vec<File> {
    let frames = a.frames.len();
    let points = a.frames.first().map_or(0, Vec::len);
    let mut out = Vec::with_capacity(32 + frames * points * 12);
    out.extend_from_slice(b"POINTCACHE2\0");
    out.extend_from_slice(&1i32.to_le_bytes());
    out.extend_from_slice(&(points as i32).to_le_bytes());
    out.extend_from_slice(&(o.start_frame as f32).to_le_bytes());
    out.extend_from_slice(&1.0f32.to_le_bytes());
    out.extend_from_slice(&(frames as i32).to_le_bytes());
    for f in &a.frames {
        for p in f {
            for c in p {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
    }
    vec![File { name: format!("{}.pc2", o.stem), bytes: out }]
}
