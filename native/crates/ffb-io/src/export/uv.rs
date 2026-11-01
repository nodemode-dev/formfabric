pub const DEFAULT_PADDING: f32 = 0.004;
pub const MAX_TILES: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Island {
    pub panel: u32,
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: f32,
    pub tile: u32,
}

pub fn grain_rotation(grain_degrees: f64) -> f64 {
    90.0 - grain_degrees
}

pub fn rotate(points: &[[f64; 2]], degrees: f64) -> Vec<[f64; 2]> {
    let (s, c) = degrees.to_radians().sin_cos();
    points.iter().map(|p| [p[0] * c - p[1] * s, p[0] * s + p[1] * c]).collect()
}

pub fn bounds(points: &[[f64; 2]]) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::MAX; 2];
    let mut hi = [f64::MIN; 2];
    for p in points {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    if lo[0] > hi[0] {
        ([0.0; 2], [0.0; 2])
    } else {
        (lo, hi)
    }
}

pub fn aligned(points: &[[f64; 2]], grain_degrees: f64) -> Vec<[f64; 2]> {
    rotate(points, grain_rotation(grain_degrees))
}

pub fn island_size(aligned_points: &[[f64; 2]], rotation: f32) -> [f64; 2] {
    let r = rotate(aligned_points, f64::from(rotation));
    let (lo, hi) = bounds(&r);
    [hi[0] - lo[0], hi[1] - lo[1]]
}

pub fn island_uvs(aligned_points: &[[f64; 2]], island: &Island) -> Vec<[f32; 2]> {
    let r = rotate(aligned_points, f64::from(island.rotation));
    let (lo, _) = bounds(&r);
    let s = f64::from(island.scale);
    r.iter().map(|p| [((p[0] - lo[0]) * s) as f32 + island.offset[0] + island.tile as f32, ((p[1] - lo[1]) * s) as f32 + island.offset[1]]).collect()
}

pub fn tiled_uvs(aligned_points: &[[f64; 2]], map_scale: [f64; 2]) -> Vec<[f32; 2]> {
    let sx = if map_scale[0] > 1e-9 { map_scale[0] } else { 100.0 };
    let sy = if map_scale[1] > 1e-9 { map_scale[1] } else { 100.0 };
    aligned_points.iter().map(|p| [(p[0] / sx) as f32, (p[1] / sy) as f32]).collect()
}

struct Packed {
    places: Vec<(f32, f32, u32)>,
}

fn shelf(sizes: &[[f64; 2]], scale: f64, padding: f64, tiles: u32) -> Option<Packed> {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by(|a, b| sizes[*b][1].total_cmp(&sizes[*a][1]).then(a.cmp(b)));
    let mut places = vec![(0.0f32, 0.0f32, 0u32); sizes.len()];
    let (mut tile, mut x, mut y, mut row_h) = (0u32, padding, padding, 0.0f64);
    for i in order {
        let (w, h) = (sizes[i][0] * scale, sizes[i][1] * scale);
        if w + 2.0 * padding > 1.0 + 1e-9 || h + 2.0 * padding > 1.0 + 1e-9 {
            return None;
        }
        if x + w + padding > 1.0 + 1e-9 {
            x = padding;
            y += row_h + padding;
            row_h = 0.0;
        }
        if y + h + padding > 1.0 + 1e-9 {
            tile += 1;
            x = padding;
            y = padding;
            row_h = 0.0;
            if tile >= tiles {
                return None;
            }
        }
        places[i] = (x as f32, y as f32, tile);
        x += w + padding;
        row_h = row_h.max(h);
    }
    Some(Packed { places })
}

pub fn pack(panels: &[(u32, [f64; 2])], rotations: &[f32], padding: f32, tiles: u32) -> Vec<Island> {
    let padding = f64::from(if padding > 0.0 { padding } else { DEFAULT_PADDING }).min(0.1);
    let tiles = tiles.clamp(1, MAX_TILES);
    let sizes: Vec<[f64; 2]> = panels.iter().map(|p| p.1).collect();
    if sizes.is_empty() {
        return Vec::new();
    }
    let (mut lo, mut hi) = (1e-9f64, 1.0f64);
    while shelf(&sizes, hi, padding, tiles).is_some() && hi < 1e6 {
        hi *= 2.0;
    }
    for _ in 0..48 {
        let mid = (lo * hi).sqrt();
        if shelf(&sizes, mid, padding, tiles).is_some() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let best = shelf(&sizes, lo, padding, tiles).unwrap_or(Packed { places: vec![(0.0, 0.0, 0); sizes.len()] });
    panels
        .iter()
        .enumerate()
        .map(|(i, p)| Island { panel: p.0, offset: [best.places[i].0, best.places[i].1], rotation: rotations.get(i).copied().unwrap_or(0.0), scale: lo as f32, tile: best.places[i].2 })
        .collect()
}
