use ffb_pattern::{InnerRole, Panel, Point};

use crate::build::MeshOptions;

pub struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv {
    pub fn bytes(&mut self, data: &[u8]) {
        for b in data {
            self.0 ^= u64::from(*b);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }

    pub fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.u64(if v == 0.0 { 0 } else { v.to_bits() });
    }

    pub fn pair(&mut self, v: [f64; 2]) {
        self.f64(v[0]);
        self.f64(v[1]);
    }

    pub fn finish(&self) -> u64 {
        self.0
    }
}

fn point(h: &mut Fnv, p: &Point) {
    h.pair(p.p);
    for handle in [p.h_in, p.h_out] {
        match handle {
            Some(v) => {
                h.u64(1);
                h.pair(v);
            }
            None => h.u64(0),
        }
    }
}

pub fn meshed(role: InnerRole) -> bool {
    matches!(role, InnerRole::Fold | InnerRole::Dart)
}

pub fn panel_key(panel: &Panel, options: &MeshOptions) -> u64 {
    let mut h = Fnv::default();
    h.f64(panel.particle_distance);
    h.u64(panel.outline.len() as u64);
    for p in &panel.outline {
        point(&mut h, p);
    }
    for inner in panel.inner.iter().filter(|i| meshed(i.role)) {
        h.u64(u64::from(inner.id));
        h.u64(u64::from(inner.closed));
        h.u64(inner.points.len() as u64);
        for p in &inner.points {
            point(&mut h, p);
        }
    }
    h.f64(options.min_angle);
    h.f64(options.jitter);
    h.f64(options.inset);
    for (segment, count) in &options.segment_counts {
        h.u64(*segment as u64);
        h.u64(*count as u64);
    }
    h.finish()
}
