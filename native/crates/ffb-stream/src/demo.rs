use std::thread;
use std::time::Duration;

use crate::Publisher;

const SIDE: usize = 32;
const PITCH: f32 = 20.0;
pub const DEMO_OBJECT: u32 = 1;

pub fn grid_topology() -> (Vec<u32>, Vec<u16>) {
    let mut indices = Vec::with_capacity((SIDE - 1) * (SIDE - 1) * 6);
    let mut panels = Vec::with_capacity((SIDE - 1) * (SIDE - 1) * 2);
    for j in 0..SIDE - 1 {
        for i in 0..SIDE - 1 {
            let a = (j * SIDE + i) as u32;
            let b = a + 1;
            let c = a + SIDE as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
            let panel = if i < (SIDE - 1) / 2 { 0 } else { 1 };
            panels.extend_from_slice(&[panel, panel]);
        }
    }
    (indices, panels)
}

pub fn grid_positions(t: f32) -> Vec<f32> {
    let mut out = Vec::with_capacity(SIDE * SIDE * 3);
    for j in 0..SIDE {
        for i in 0..SIDE {
            let x = i as f32 * PITCH;
            let z = j as f32 * PITCH;
            let r = ((i as f32 - 15.5).powi(2) + (j as f32 - 15.5).powi(2)).sqrt();
            let y = 40.0 * (r * 0.5 - t * 4.0).sin() / (1.0 + r * 0.15);
            out.extend_from_slice(&[x, y, z]);
        }
    }
    out
}

pub fn spawn(publisher: Publisher, frames: u32) {
    let _ = thread::Builder::new().name("ffb-stream-demo".into()).spawn(move || {
        let (indices, panels) = grid_topology();
        publisher.publish_topology(DEMO_OBJECT, &indices, &panels);
        for n in 0..frames {
            publisher.publish_positions(DEMO_OBJECT, &grid_positions(n as f32 / 30.0));
            thread::sleep(Duration::from_millis(33));
        }
    });
}
