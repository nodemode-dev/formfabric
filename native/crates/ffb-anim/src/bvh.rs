use ffb_body::math::*;

use crate::clip::{Clip, ClipJoint, MAX_FRAMES, MAX_JOINTS, MAX_VALUES};

pub const MAX_BYTES: usize = 32 << 20;
const MAX_DEPTH: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Channel {
    Tx,
    Ty,
    Tz,
    Rx,
    Ry,
    Rz,
}

struct Node {
    name: String,
    parent: Option<usize>,
    offset: V3,
    channels: Vec<Channel>,
    end_site: bool,
}

pub struct Options {
    pub units_mm: Option<f32>,
    pub up: Q,
    pub name: String,
}

fn channel(s: &str) -> Option<Channel> {
    Some(match s.to_ascii_lowercase().as_str() {
        "xposition" => Channel::Tx,
        "yposition" => Channel::Ty,
        "zposition" => Channel::Tz,
        "xrotation" => Channel::Rx,
        "yrotation" => Channel::Ry,
        "zrotation" => Channel::Rz,
        _ => return None,
    })
}

struct Tokens<'a> {
    list: Vec<&'a str>,
    at: usize,
}

impl<'a> Tokens<'a> {
    fn next(&mut self) -> Result<&'a str, String> {
        let t = self.list.get(self.at).copied().ok_or("the BVH file ends early")?;
        self.at += 1;
        Ok(t)
    }

    fn peek(&self) -> Option<&'a str> {
        self.list.get(self.at).copied()
    }

    fn expect(&mut self, word: &str) -> Result<(), String> {
        let t = self.next()?;
        if t.eq_ignore_ascii_case(word) {
            Ok(())
        } else {
            Err(format!("the BVH file has {t} where {word} was expected"))
        }
    }

    fn number(&mut self) -> Result<f32, String> {
        let t = self.next()?;
        let x: f32 = t.parse().map_err(|_| format!("the BVH file has {t} where a number was expected"))?;
        if x.is_finite() {
            Ok(x)
        } else {
            Err("the BVH file holds a non-finite number".into())
        }
    }

    fn count(&mut self, limit: usize) -> Result<usize, String> {
        let t = self.next()?;
        let x: usize = t.parse().map_err(|_| format!("the BVH file has {t} where a count was expected"))?;
        if x > limit {
            return Err(format!("the BVH file declares {x}; the limit is {limit}"));
        }
        Ok(x)
    }
}

fn euler(channels: &[Channel], values: &[f32]) -> Q {
    let mut q = IDENTITY;
    for (c, v) in channels.iter().zip(values) {
        let a = v.to_radians();
        let step = match c {
            Channel::Rx => qaxis([1.0, 0.0, 0.0], a),
            Channel::Ry => qaxis([0.0, 1.0, 0.0], a),
            Channel::Rz => qaxis([0.0, 0.0, 1.0], a),
            _ => continue,
        };
        q = qmul(q, step);
    }
    qnorm(q)
}

pub fn parse(bytes: &[u8], options: &Options) -> Result<Clip, String> {
    if bytes.len() > MAX_BYTES {
        return Err(format!("the BVH file is larger than {} MB", MAX_BYTES >> 20));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "the BVH file is not text")?;
    let mut t = Tokens { list: text.split_whitespace().collect(), at: 0 };
    t.expect("HIERARCHY")?;
    let mut nodes: Vec<Node> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut pending: Option<(Option<usize>, bool)> = None;
    let mut first = true;
    loop {
        let word = t.next()?;
        let lower = word.to_ascii_lowercase();
        match lower.as_str() {
            "root" | "joint" => {
                if nodes.len() >= MAX_JOINTS {
                    return Err(format!("the BVH file has more than {MAX_JOINTS} joints"));
                }
                if lower == "root" && !first {
                    return Err("the BVH file has more than one ROOT".into());
                }
                first = false;
                let name = t.next()?.to_string();
                let parent = stack.last().copied();
                nodes.push(Node { name, parent, offset: [0.0; 3], channels: Vec::new(), end_site: false });
                pending = Some((Some(nodes.len() - 1), false));
            }
            "end" => {
                t.expect("Site")?;
                if nodes.len() >= 4 * MAX_JOINTS {
                    return Err("the BVH file has too many End Site nodes".into());
                }
                pending = Some((None, true));
            }
            "{" => {
                if stack.len() >= MAX_DEPTH {
                    return Err("the BVH hierarchy is too deep".into());
                }
                match pending.take() {
                    Some((Some(i), _)) => stack.push(i),
                    Some((None, true)) => {
                        let parent = stack.last().copied();
                        nodes.push(Node { name: String::new(), parent, offset: [0.0; 3], channels: Vec::new(), end_site: true });
                        stack.push(nodes.len() - 1);
                    }
                    _ => return Err("the BVH file has an unexpected brace".into()),
                }
            }
            "}" => {
                stack.pop().ok_or("the BVH file has an unbalanced brace")?;
                if stack.is_empty() {
                    break;
                }
            }
            "offset" => {
                let o = [t.number()?, t.number()?, t.number()?];
                let i = *stack.last().ok_or("OFFSET outside a joint")?;
                nodes[i].offset = o;
            }
            "channels" => {
                let n = t.count(6)?;
                let i = *stack.last().ok_or("CHANNELS outside a joint")?;
                if !nodes[i].channels.is_empty() || nodes[i].end_site {
                    return Err("the BVH file has a second CHANNELS line on one joint".into());
                }
                for _ in 0..n {
                    let name = t.next()?;
                    nodes[i].channels.push(channel(name).ok_or_else(|| format!("the BVH file has an unknown channel {name}"))?);
                }
            }
            other => return Err(format!("the BVH file has {other} where a joint was expected")),
        }
    }
    t.expect("MOTION")?;
    t.expect("Frames:")?;
    let frames = t.count(MAX_FRAMES)?;
    t.expect("Frame")?;
    t.expect("Time:")?;
    let dt = t.number()?;
    if !(dt > 0.0 && dt <= 10.0) {
        return Err("the BVH frame time must be between 0 and 10 seconds".into());
    }
    let real: Vec<usize> = (0..nodes.len()).filter(|i| !nodes[*i].end_site).collect();
    if real.is_empty() {
        return Err("the BVH file has no joints".into());
    }
    let per_frame: usize = real.iter().map(|i| nodes[*i].channels.len()).sum();
    if frames == 0 {
        return Err("the BVH file has no frames".into());
    }
    if per_frame.saturating_mul(frames) > MAX_VALUES || real.len().saturating_mul(frames) > MAX_VALUES / 7 {
        return Err("the BVH motion is larger than the import limit".into());
    }
    let mut index = vec![usize::MAX; nodes.len()];
    for (k, i) in real.iter().enumerate() {
        index[*i] = k;
    }
    let n = real.len();
    let skeleton_parent = |i: usize| -> Option<usize> {
        let mut p = nodes[i].parent;
        while let Some(x) = p {
            if !nodes[x].end_site {
                return Some(index[x]);
            }
            p = nodes[x].parent;
        }
        None
    };
    let mut rest_world = vec![[0.0f32; 3]; n];
    for (k, i) in real.iter().enumerate() {
        rest_world[k] = match skeleton_parent(*i) {
            Some(p) => add(rest_world[p], nodes[*i].offset),
            None => nodes[*i].offset,
        };
    }
    let (lo, hi) = rest_world.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
    let extent = hi - lo;
    let unit = options.units_mm.unwrap_or(if extent < 4.0 && extent > 0.0 {
        1000.0
    } else if extent < 400.0 {
        10.0
    } else {
        1.0
    });
    let up = qnorm(options.up);
    let mut joints = Vec::with_capacity(n);
    for i in real.iter() {
        let parent = skeleton_parent(*i);
        let mut offset = scale(nodes[*i].offset, unit);
        if parent.is_none() {
            offset = qrot(up, offset);
        }
        joints.push(ClipJoint { name: nodes[*i].name.clone(), parent, rest_pos: offset, rest_rot: if parent.is_none() { up } else { IDENTITY } });
    }
    let mut rot = Vec::with_capacity(frames * n);
    let mut pos = Vec::with_capacity(frames * n);
    for _ in 0..frames {
        for (k, i) in real.iter().enumerate() {
            let node = &nodes[*i];
            let mut values = [0.0f32; 6];
            for v in values.iter_mut().take(node.channels.len()) {
                *v = t.number()?;
            }
            let values = &values[..node.channels.len()];
            let mut p = joints[k].rest_pos;
            let mut offs = node.offset;
            let mut moved = false;
            for (c, v) in node.channels.iter().zip(values) {
                match c {
                    Channel::Tx => {
                        offs[0] = *v;
                        moved = true;
                    }
                    Channel::Ty => {
                        offs[1] = *v;
                        moved = true;
                    }
                    Channel::Tz => {
                        offs[2] = *v;
                        moved = true;
                    }
                    _ => {}
                }
            }
            if moved {
                let o = scale(offs, unit);
                p = if joints[k].parent.is_none() { qrot(up, o) } else { o };
            }
            let mut q = euler(&node.channels, values);
            if joints[k].parent.is_none() {
                q = qnorm(qmul(up, q));
            }
            rot.push(q);
            pos.push(p);
        }
    }
    if t.peek().is_some() {
        return Err("the BVH file has more values than its frame count".into());
    }
    let clip = Clip { name: options.name.clone(), fps: 1.0 / dt, joints, frames, rot, pos }.order_parents_first();
    clip.check()?;
    Ok(clip)
}
