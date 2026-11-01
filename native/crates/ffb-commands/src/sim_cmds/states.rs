use ffb_sim::{Cloth, PinPoint, SeamPair};

pub(crate) const HOLD_RATE: f32 = 2000.0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Anchor {
    pub vertex: u32,
    pub point: [f32; 3],
    pub length: f32,
    pub stiffness: f32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Extras {
    pub frozen: Vec<(usize, usize)>,
    pub hold: Vec<(usize, usize)>,
    pub anchors: Vec<Anchor>,
}

fn vertex_mass(cloth: &Cloth) -> Vec<f32> {
    let mut mass = vec![0.0f32; cloth.positions.len()];
    for (ti, t) in cloth.triangles.iter().enumerate() {
        let r = t.map(|i| cloth.rest.get(i as usize).copied().unwrap_or([0.0, 0.0]));
        let area = 0.5 * ((r[1][0] - r[0][0]) * (r[2][1] - r[0][1]) - (r[2][0] - r[0][0]) * (r[1][1] - r[0][1])).abs();
        let material = cloth.materials.get(cloth.triangle_material.get(ti).copied().unwrap_or(0) as usize).copied().unwrap_or_default();
        let density = if material.density > 0.0 { material.density } else { ffb_sim::material::DEFAULT_DENSITY };
        for &i in t {
            if let Some(m) = mass.get_mut(i as usize) {
                *m += area / 3.0 * density;
            }
        }
    }
    mass
}

pub(crate) fn finish(cloth: &mut Cloth, extras: &Extras) -> usize {
    let solid = cloth.positions.len();
    let mut pinned = vec![false; solid];
    for p in &cloth.pins {
        if let Some(slot) = pinned.get_mut(p.vertex as usize) {
            *slot = true;
        }
    }
    for &(start, count) in &extras.frozen {
        for (v, slot) in pinned.iter_mut().enumerate().take((start + count).min(solid)).skip(start) {
            if !*slot {
                *slot = true;
                cloth.pins.push(PinPoint { vertex: v as u32, target: cloth.positions[v] });
            }
        }
    }
    if !extras.hold.is_empty() {
        let mass = vertex_mass(cloth);
        for &(start, count) in &extras.hold {
            for v in start..(start + count).min(solid) {
                if pinned[v] {
                    continue;
                }
                let at = cloth.positions[v];
                let anchor = cloth.positions.len() as u32;
                cloth.positions.push(at);
                cloth.pins.push(PinPoint { vertex: anchor, target: at });
                cloth.seams.push(SeamPair { a: v as u32, b: anchor, rest_length: 0.0, stiffness: (HOLD_RATE * mass[v]).max(1.0e-6) });
            }
        }
    }
    for a in &extras.anchors {
        if a.vertex as usize >= solid {
            continue;
        }
        let anchor = cloth.positions.len() as u32;
        cloth.positions.push(a.point);
        cloth.pins.push(PinPoint { vertex: anchor, target: a.point });
        cloth.seams.push(SeamPair { a: a.vertex, b: anchor, rest_length: a.length, stiffness: a.stiffness });
    }
    solid
}
