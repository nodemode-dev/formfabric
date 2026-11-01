use ffb_body::math::*;

pub const MAX_JOINTS: usize = 1024;
pub const MAX_FRAMES: usize = 100_000;
pub const MAX_VALUES: usize = 40_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct ClipJoint {
    pub name: String,
    pub parent: Option<usize>,
    pub rest_pos: V3,
    pub rest_rot: Q,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Clip {
    pub name: String,
    pub fps: f32,
    pub joints: Vec<ClipJoint>,
    pub frames: usize,
    pub rot: Vec<Q>,
    pub pos: Vec<V3>,
}

#[derive(Debug, Clone, Default)]
pub struct Sample {
    pub rot: Vec<Q>,
    pub pos: Vec<V3>,
}

pub fn world(joints: &[ClipJoint], rot: &[Q], pos: &[V3]) -> (Vec<V3>, Vec<Q>) {
    let n = joints.len();
    let mut wp = vec![[0.0f32; 3]; n];
    let mut wr = vec![IDENTITY; n];
    for i in 0..n {
        match joints[i].parent {
            Some(p) if p < i => {
                wp[i] = add(wp[p], qrot(wr[p], pos[i]));
                wr[i] = qnorm(qmul(wr[p], rot[i]));
            }
            _ => {
                wp[i] = pos[i];
                wr[i] = qnorm(rot[i]);
            }
        }
    }
    (wp, wr)
}

impl Clip {
    pub fn duration(&self) -> f32 {
        if self.frames < 2 || self.fps <= 0.0 {
            0.0
        } else {
            (self.frames - 1) as f32 / self.fps
        }
    }

    pub fn rest(&self) -> Sample {
        Sample { rot: self.joints.iter().map(|j| j.rest_rot).collect(), pos: self.joints.iter().map(|j| j.rest_pos).collect() }
    }

    pub fn sample(&self, time: f32, looped: bool) -> Sample {
        let n = self.joints.len();
        if self.frames == 0 || n == 0 {
            return self.rest();
        }
        let last = (self.frames - 1) as f32;
        let mut f = time * self.fps;
        if !f.is_finite() {
            f = 0.0;
        }
        if looped && last > 0.0 {
            f = f.rem_euclid(last + 1.0);
            if f > last {
                f = last;
            }
        }
        let f = f.clamp(0.0, last);
        let a = f.floor() as usize;
        let b = (a + 1).min(self.frames - 1);
        let t = f - a as f32;
        let mut out = Sample { rot: Vec::with_capacity(n), pos: Vec::with_capacity(n) };
        for j in 0..n {
            let (ra, rb) = (self.rot[a * n + j], self.rot[b * n + j]);
            out.rot.push(crate::interp::slerp(ra, rb, t));
            out.pos.push(lerp(self.pos[a * n + j], self.pos[b * n + j], t));
        }
        out
    }

    pub fn order_parents_first(mut self) -> Clip {
        let n = self.joints.len();
        let mut order: Vec<usize> = Vec::with_capacity(n);
        let mut placed = vec![false; n];
        for _ in 0..=n {
            if order.len() == n {
                break;
            }
            for i in 0..n {
                if placed[i] {
                    continue;
                }
                let ok = match self.joints[i].parent {
                    None => true,
                    Some(p) => p >= n || p == i || placed[p],
                };
                if ok {
                    placed[i] = true;
                    order.push(i);
                }
            }
        }
        if order.iter().enumerate().all(|(k, i)| k == *i) {
            return self;
        }
        let mut new_index = vec![0usize; n];
        for (k, i) in order.iter().enumerate() {
            new_index[*i] = k;
        }
        let joints: Vec<ClipJoint> = order
            .iter()
            .map(|i| {
                let j = &self.joints[*i];
                ClipJoint { name: j.name.clone(), parent: j.parent.filter(|p| *p < n && p != i).map(|p| new_index[p]), rest_pos: j.rest_pos, rest_rot: j.rest_rot }
            })
            .collect();
        let mut rot = Vec::with_capacity(self.rot.len());
        let mut pos = Vec::with_capacity(self.pos.len());
        for f in 0..self.frames {
            for i in &order {
                rot.push(self.rot[f * n + i]);
                pos.push(self.pos[f * n + i]);
            }
        }
        self.joints = joints;
        self.rot = rot;
        self.pos = pos;
        self
    }

    pub fn check(&self) -> Result<(), String> {
        let n = self.joints.len();
        if n == 0 || n > MAX_JOINTS {
            return Err(format!("the motion has {n} joints; the limit is {MAX_JOINTS}"));
        }
        if self.frames == 0 || self.frames > MAX_FRAMES {
            return Err(format!("the motion has {} frames; the limit is {MAX_FRAMES}", self.frames));
        }
        if n.saturating_mul(self.frames) > MAX_VALUES / 7 {
            return Err("the motion is larger than the import limit".into());
        }
        if self.rot.len() != n * self.frames || self.pos.len() != n * self.frames {
            return Err("the motion data does not match its joint and frame counts".into());
        }
        if !(self.fps.is_finite() && self.fps > 0.0 && self.fps <= 1000.0) {
            return Err("the motion frame rate must be between 0 and 1000".into());
        }
        let bad = self.rot.iter().any(|q| q.iter().any(|x| !x.is_finite())) || self.pos.iter().any(|p| p.iter().any(|x| !x.is_finite()));
        if bad {
            return Err("the motion holds non-finite numbers".into());
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64 + self.rot.len() * 28);
        out.extend_from_slice(b"FFMC");
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend_from_slice(&self.fps.to_le_bytes());
        out.extend_from_slice(&(self.joints.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.frames as u32).to_le_bytes());
        let name = self.name.as_bytes();
        out.extend_from_slice(&(name.len() as u32).to_le_bytes());
        out.extend_from_slice(name);
        for j in &self.joints {
            let nm = j.name.as_bytes();
            out.extend_from_slice(&(nm.len() as u32).to_le_bytes());
            out.extend_from_slice(nm);
            out.extend_from_slice(&j.parent.map_or(-1i32, |p| p as i32).to_le_bytes());
            for x in j.rest_pos.iter().chain(j.rest_rot.iter()) {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
        for q in &self.rot {
            for x in q {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
        for p in &self.pos {
            for x in p {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Clip, String> {
        let mut r = Reader { bytes, at: 0 };
        if r.take(4)? != b"FFMC" {
            return Err("not a motion entry".into());
        }
        if r.u32()? != 1 {
            return Err("unsupported motion entry version".into());
        }
        let fps = r.f32()?;
        let n = r.u32()? as usize;
        let frames = r.u32()? as usize;
        if n == 0 || n > MAX_JOINTS || frames == 0 || frames > MAX_FRAMES || n.saturating_mul(frames) > MAX_VALUES / 7 {
            return Err("the motion entry is over the limits".into());
        }
        let name = r.string()?;
        let mut joints = Vec::with_capacity(n);
        for _ in 0..n {
            let nm = r.string()?;
            let parent = r.i32()?;
            let rest_pos = [r.f32()?, r.f32()?, r.f32()?];
            let rest_rot = [r.f32()?, r.f32()?, r.f32()?, r.f32()?];
            joints.push(ClipJoint { name: nm, parent: usize::try_from(parent).ok(), rest_pos, rest_rot });
        }
        let mut rot = Vec::with_capacity(n * frames);
        for _ in 0..n * frames {
            rot.push([r.f32()?, r.f32()?, r.f32()?, r.f32()?]);
        }
        let mut pos = Vec::with_capacity(n * frames);
        for _ in 0..n * frames {
            pos.push([r.f32()?, r.f32()?, r.f32()?]);
        }
        let clip = Clip { name, fps, joints, frames, rot, pos };
        clip.check()?;
        Ok(clip)
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.bytes.len()).ok_or("the entry is truncated")?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn i32(&mut self) -> Result<i32, String> {
        self.u32().map(|x| x as i32)
    }

    fn f32(&mut self) -> Result<f32, String> {
        self.u32().map(f32::from_bits)
    }

    fn string(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        if n > 4096 {
            return Err("a name in the entry is too long".into());
        }
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }
}
