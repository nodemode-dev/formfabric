use std::collections::HashMap;

use ffb_body::{math, skin, Body};

use crate::{FrameSink, Session};

pub const OBJECT_BASE: u32 = 1_000_000;
const HIDDEN: u64 = 1;

#[derive(Default)]
pub struct StreamState {
    sent: HashMap<u32, (u64, u64)>,
}

fn signatures(body: &Body) -> (u64, u64) {
    let mut t = math::FNV_START;
    t ^= body.triangles.len() as u64;
    for tri in &body.triangles {
        for i in tri {
            t ^= u64::from(*i);
            t = t.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let mut p = math::FNV_START;
    for v in &body.positions {
        math::hash_f32s(&mut p, v);
    }
    for (name, q) in &body.pose {
        for b in name.bytes() {
            p ^= u64::from(b);
            p = p.wrapping_mul(0x0000_0100_0000_01b3);
        }
        math::hash_f32s(&mut p, q);
    }
    (t.max(2), p)
}

fn flat(points: &[[f32; 3]]) -> Vec<f32> {
    points.iter().flat_map(|p| p.iter().copied()).collect()
}

fn sync(bodies: &[Body], state: &mut StreamState, sink: &dyn FrameSink, full: bool) {
    let gone: Vec<u32> = state.sent.keys().filter(|id| !bodies.iter().any(|b| b.id == **id)).copied().collect();
    for id in gone {
        sink.topology(OBJECT_BASE + id, &[], &[]);
        state.sent.remove(&id);
    }
    for body in bodies {
        let previous = state.sent.get(&body.id).copied();
        if !body.visible || body.triangles.is_empty() {
            if full || previous.map(|s| s.0) != Some(HIDDEN) {
                sink.topology(OBJECT_BASE + body.id, &[], &[]);
            }
            state.sent.insert(body.id, (HIDDEN, 0));
            continue;
        }
        let (topology, positions) = signatures(body);
        let new_topology = full || previous.map(|s| s.0) != Some(topology);
        if new_topology {
            let indices: Vec<u32> = body.triangles.iter().flat_map(|t| t.iter().copied()).collect();
            sink.topology(OBJECT_BASE + body.id, &indices, &vec![0u16; body.triangles.len()]);
        }
        if new_topology || previous.map(|s| s.1) != Some(positions) {
            sink.positions(OBJECT_BASE + body.id, &flat(&skin::posed_positions(body)));
        }
        state.sent.insert(body.id, (topology, positions));
    }
}

pub(super) fn on_change(session: &mut Session) {
    let Session { bodies, body_stream, sink, .. } = session;
    match sink.clone() {
        Some(sink) => sync(bodies, body_stream, sink.as_ref(), false),
        None => body_stream.sent.clear(),
    }
}

pub(super) fn full(session: &mut Session) {
    let Session { bodies, body_stream, sink, .. } = session;
    if let Some(sink) = sink.clone() {
        sync(bodies, body_stream, sink.as_ref(), true);
    }
}
