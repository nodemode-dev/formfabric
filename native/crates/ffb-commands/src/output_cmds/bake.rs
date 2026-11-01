use ffb_core::{Bvh, Mesh};
use ffb_io::export::uv::{self, Island};
use ffb_io::export::{png, vertex_normals};
use glam::Vec3;
use serde_json::{json, Value};

use super::scene::{bad, collect, effective_islands, number, tile_count, PanelGeo};
use crate::{args, Command, CommandError, Group, Session, EXTERNAL};

const MAX_WORK: f64 = 6.0e8;

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "maps": {"type": "array", "items": {"type": "string", "enum": ["normal", "ao", "displacement"]}, "description": "Maps to bake. Default all three."},
            "size": {"type": "integer", "minimum": 64, "maximum": 4096, "default": 1024, "description": "Pixels per side of every tile."},
            "path": {"type": "string", "description": "Absolute path ending in .png. Files are written as <name>_<map>.png, with _<udim> added when the layout uses more than one tile."},
            "samples": {"type": "integer", "minimum": 1, "maximum": 128, "default": 12, "description": "Ambient occlusion rays per pixel."},
            "distance": {"type": "number", "minimum": 1, "maximum": 5000, "default": 150, "unit": "mm", "description": "Longest occlusion ray."},
            "smooth": {"type": "integer", "minimum": 0, "maximum": 50, "default": 6, "description": "Laplacian smoothing passes that build the low frequency base surface the normal and displacement maps are measured against."},
            "padding": {"type": "integer", "minimum": 0, "maximum": 32, "default": 4, "description": "Pixels to extend islands past their borders to hide seams."},
            "bodies": {"type": "boolean", "default": true, "description": "Let visible bodies occlude the cloth in the ambient occlusion map."}
        },
        "required": ["path"],
        "additionalProperties": false
    })
}

struct Panel {
    uv: Vec<[f32; 2]>,
    pos: Vec<Vec3>,
    normal: Vec<Vec3>,
    base_normal: Vec<Vec3>,
    tangent: Vec<Vec3>,
    disp: Vec<f32>,
    indices: Vec<u32>,
}

fn to_vec(p: &[[f32; 3]]) -> Vec<Vec3> {
    p.iter().map(|v| Vec3::from(*v)).collect()
}

fn smooth(positions: &[[f32; 3]], indices: &[u32], passes: usize) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut neighbours: Vec<Vec<u32>> = vec![Vec::new(); n];
    for t in indices.chunks_exact(3) {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            if !neighbours[a as usize].contains(&b) {
                neighbours[a as usize].push(b);
            }
            if !neighbours[b as usize].contains(&a) {
                neighbours[b as usize].push(a);
            }
        }
    }
    let mut cur = to_vec(positions);
    for _ in 0..passes {
        let next: Vec<Vec3> = (0..n)
            .map(|i| {
                if neighbours[i].is_empty() {
                    return cur[i];
                }
                let avg = neighbours[i].iter().map(|j| cur[*j as usize]).sum::<Vec3>() / neighbours[i].len() as f32;
                cur[i].lerp(avg, 0.5)
            })
            .collect();
        cur = next;
    }
    cur.into_iter().map(|v| v.to_array()).collect()
}

fn prepare(geo: &PanelGeo, island: &Island, passes: usize) -> Panel {
    let normals = vertex_normals(&geo.pos3, &geo.indices);
    let base = smooth(&geo.pos3, &geo.indices, passes);
    let base_normals = vertex_normals(&base, &geo.indices);
    let pos = to_vec(&geo.pos3);
    let basev = to_vec(&base);
    let base_normal = to_vec(&base_normals);
    let uv = uv::island_uvs(&geo.aligned, island);
    let mut tangent = vec![Vec3::ZERO; pos.len()];
    for t in geo.indices.chunks_exact(3) {
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let e1 = pos[b] - pos[a];
        let e2 = pos[c] - pos[a];
        let (du1, dv1) = (uv[b][0] - uv[a][0], uv[b][1] - uv[a][1]);
        let (du2, dv2) = (uv[c][0] - uv[a][0], uv[c][1] - uv[a][1]);
        let det = du1 * dv2 - du2 * dv1;
        if det.abs() < 1e-20 {
            continue;
        }
        let tan = (e1 * dv2 - e2 * dv1) / det;
        for i in [a, b, c] {
            tangent[i] += tan;
        }
    }
    for (i, t) in tangent.iter_mut().enumerate() {
        let n = base_normal[i];
        let ortho = *t - n * n.dot(*t);
        *t = if ortho.length_squared() > 1e-18 { ortho.normalize() } else { n.any_orthonormal_vector() };
    }
    let disp = (0..pos.len()).map(|i| (pos[i] - basev[i]).dot(base_normal[i])).collect();
    Panel { uv, pos, normal: normals.iter().map(|v| Vec3::from(*v)).collect(), base_normal, tangent, disp, indices: geo.indices.clone() }
}

fn hash(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65536.0
}

fn hemisphere(k: u32, count: u32, jitter: f32, n: Vec3) -> Vec3 {
    let u = (k as f32 + 0.5) / count as f32;
    let bits = k.reverse_bits() as f32 / 4_294_967_296.0;
    let phi = (bits + jitter).fract() * std::f32::consts::TAU;
    let r = u.sqrt();
    let (x, y, z) = (r * phi.cos(), r * phi.sin(), (1.0 - u).max(0.0).sqrt());
    let t = n.any_orthonormal_vector();
    let b = n.cross(t);
    (t * x + b * y + n * z).normalize()
}

struct Layer {
    normal: Vec<u8>,
    ao: Vec<u8>,
    disp: Vec<f32>,
    covered: Vec<bool>,
}

fn dilate(layer: &mut Layer, size: usize, passes: usize) {
    for _ in 0..passes {
        let covered = layer.covered.clone();
        let (normal, ao, disp) = (layer.normal.clone(), layer.ao.clone(), layer.disp.clone());
        for y in 0..size {
            for x in 0..size {
                let i = y * size + x;
                if covered[i] {
                    continue;
                }
                'search: for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= size as i32 || ny >= size as i32 {
                        continue;
                    }
                    let j = ny as usize * size + nx as usize;
                    if covered[j] {
                        layer.normal[i * 3..i * 3 + 3].copy_from_slice(&normal[j * 3..j * 3 + 3]);
                        layer.ao[i] = ao[j];
                        layer.disp[i] = disp[j];
                        layer.covered[i] = true;
                        break 'search;
                    }
                }
            }
        }
    }
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::file_path(a, "path", Some("png"))?;
    let maps: Vec<String> = match a.get("maps").and_then(Value::as_array) {
        None => vec!["normal".into(), "ao".into(), "displacement".into()],
        Some(list) => list.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
    };
    if maps.is_empty() || maps.iter().any(|m| !["normal", "ao", "displacement"].contains(&m.as_str())) {
        return Err(bad("maps must list normal, ao or displacement"));
    }
    let size = number(a, "size", 64.0, 4096.0)?.map_or(1024, |x| x as usize);
    let samples = number(a, "samples", 1.0, 128.0)?.map_or(12, |x| x as u32);
    let distance = number(a, "distance", 1.0, 5000.0)?.map_or(150.0, |x| x as f32);
    let passes = number(a, "smooth", 0.0, 50.0)?.map_or(6, |x| x as usize);
    let padding = number(a, "padding", 0.0, 32.0)?.map_or(4, |x| x as usize);
    let with_bodies = a.get("bodies").and_then(Value::as_bool).unwrap_or(true);
    let geos = collect(s, None)?;
    let (islands, _) = effective_islands(&s.garment, &geos);
    let tiles = tile_count(&islands) as usize;
    let want_ao = maps.iter().any(|m| m == "ao");
    let total_triangles: usize = geos.iter().map(|g| g.indices.len() / 3).sum();
    if want_ao && (size * size * tiles) as f64 * f64::from(samples) > MAX_WORK {
        return Err(bad("size, tiles and samples ask for too many rays; lower the size or the samples"));
    }
    let panels: Vec<Panel> = geos.iter().zip(&islands).map(|(g, i)| prepare(g, i, passes)).collect();
    let (mut occluder, mut bvh) = (Mesh::default(), None);
    if want_ao {
        for g in &geos {
            let base = occluder.positions.len() as u32;
            occluder.positions.extend_from_slice(&g.pos3);
            occluder.indices.extend(g.indices.iter().map(|i| i + base));
        }
        if with_bodies {
            for b in s.bodies.iter().filter(|b| b.visible) {
                let base = occluder.positions.len() as u32;
                occluder.positions.extend_from_slice(&b.positions);
                occluder.indices.extend(b.triangles.iter().flat_map(|t| t.iter().map(|i| i + base)));
            }
        }
        bvh = Some(Bvh::build(&occluder));
    }
    let max_disp = panels.iter().flat_map(|p| p.disp.iter()).fold(0.0f32, |m, d| m.max(d.abs())).max(1e-3);
    let mut written = Vec::new();
    let location = std::path::Path::new(&path);
    let stem = location.file_stem().and_then(|x| x.to_str()).unwrap_or("bake").to_string();
    let dir = location.parent().ok_or_else(|| bad("path has no folder"))?;
    let mut covered_total = 0usize;
    for tile in 0..tiles {
        let mut layer = Layer { normal: vec![0; size * size * 3], ao: vec![255; size * size], disp: vec![0.0; size * size], covered: vec![false; size * size] };
        for i in 0..size * size {
            layer.normal[i * 3..i * 3 + 3].copy_from_slice(&[128, 128, 255]);
        }
        for (pi, panel) in panels.iter().enumerate() {
            if islands[pi].tile as usize != tile {
                continue;
            }
            for t in panel.indices.chunks_exact(3) {
                let ids = [t[0] as usize, t[1] as usize, t[2] as usize];
                let p: Vec<[f32; 2]> = ids.iter().map(|i| [(panel.uv[*i][0] - tile as f32) * size as f32, panel.uv[*i][1] * size as f32]).collect();
                let (min_x, max_x) = (p.iter().map(|q| q[0]).fold(f32::MAX, f32::min), p.iter().map(|q| q[0]).fold(f32::MIN, f32::max));
                let (min_y, max_y) = (p.iter().map(|q| q[1]).fold(f32::MAX, f32::min), p.iter().map(|q| q[1]).fold(f32::MIN, f32::max));
                let det = (p[1][1] - p[2][1]) * (p[0][0] - p[2][0]) + (p[2][0] - p[1][0]) * (p[0][1] - p[2][1]);
                if det.abs() < 1e-12 {
                    continue;
                }
                let (x0, x1) = ((min_x.floor().max(0.0)) as usize, (max_x.ceil().min(size as f32 - 1.0)).max(0.0) as usize);
                let (y0, y1) = ((min_y.floor().max(0.0)) as usize, (max_y.ceil().min(size as f32 - 1.0)).max(0.0) as usize);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
                        let l0 = ((p[1][1] - p[2][1]) * (cx - p[2][0]) + (p[2][0] - p[1][0]) * (cy - p[2][1])) / det;
                        let l1 = ((p[2][1] - p[0][1]) * (cx - p[2][0]) + (p[0][0] - p[2][0]) * (cy - p[2][1])) / det;
                        let l2 = 1.0 - l0 - l1;
                        if l0 < -1e-4 || l1 < -1e-4 || l2 < -1e-4 {
                            continue;
                        }
                        let idx = y * size + x;
                        if layer.covered[idx] {
                            continue;
                        }
                        let w = [l0, l1, l2];
                        let mix = |f: &dyn Fn(usize) -> Vec3| -> Vec3 { f(ids[0]) * w[0] + f(ids[1]) * w[1] + f(ids[2]) * w[2] };
                        let pos = mix(&|i| panel.pos[i]);
                        let n = mix(&|i| panel.normal[i]).normalize_or_zero();
                        let nb = mix(&|i| panel.base_normal[i]).normalize_or_zero();
                        let tg = mix(&|i| panel.tangent[i]);
                        let tg = (tg - nb * nb.dot(tg)).normalize_or_zero();
                        let bt = nb.cross(tg);
                        let ts = Vec3::new(n.dot(tg), n.dot(bt), n.dot(nb)).normalize_or_zero();
                        for k in 0..3 {
                            layer.normal[idx * 3 + k] = ((ts[k] * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
                        }
                        layer.disp[idx] = w[0] * panel.disp[ids[0]] + w[1] * panel.disp[ids[1]] + w[2] * panel.disp[ids[2]];
                        if let (true, Some(bvh)) = (want_ao, bvh.as_ref()) {
                            let origin = pos + n * 0.3;
                            let jitter = hash(x as u32 + (tile as u32) * 7919, y as u32);
                            let mut hit = 0u32;
                            for k in 0..samples {
                                let d = hemisphere(k, samples, jitter, n);
                                if bvh.raycast(&occluder, origin, d).is_some_and(|h| h.t < distance) {
                                    hit += 1;
                                }
                            }
                            layer.ao[idx] = ((1.0 - hit as f32 / samples as f32) * 255.0).round() as u8;
                        }
                        layer.covered[idx] = true;
                        covered_total += 1;
                    }
                }
            }
        }
        dilate(&mut layer, size, padding);
        let suffix = if tiles > 1 { format!("_{}", 1001 + tile) } else { String::new() };
        for map in &maps {
            let (channels, data): (u8, Vec<u8>) = match map.as_str() {
                "normal" => (3, layer.normal.clone()),
                "ao" => (1, layer.ao.clone()),
                _ => (1, layer.disp.iter().zip(&layer.covered).map(|(d, c)| if *c { ((0.5 + 0.5 * d / max_disp).clamp(0.0, 1.0) * 255.0).round() as u8 } else { 128 }).collect()),
            };
            let bytes = png::encode(size as u32, size as u32, channels, &data).map_err(|e| CommandError::new("export_failed", e))?;
            let file = dir.join(format!("{stem}_{map}{suffix}.png"));
            let text = file.to_str().ok_or_else(|| bad("the path is not valid text"))?.to_string();
            args::write_atomic(&text, &bytes)?;
            written.push(json!({"map": map, "tile": tile, "path": text, "bytes": bytes.len()}));
        }
    }
    let coverage = covered_total as f64 / (size * size * tiles) as f64;
    Ok(json!({
        "files": written,
        "size": size,
        "tiles": tiles,
        "triangles": total_triangles,
        "coverage": coverage,
        "displacement_range_mm": max_disp,
        "reference": "low frequency base surface made by Laplacian smoothing of the draped mesh; normal maps are tangent space against it, displacement is the signed offset along its normal (0.5 is zero)",
        "occluders": if want_ao { occluder.triangle_count() } else { 0 }
    }))
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "bake.run",
        mcp: "bake_run",
        group: Group::Export,
        label: "Bake Maps",
        description: "Bake texture maps in the UV layout (uv.layout) as 8 bit PNG: a tangent space normal map and a displacement map that carry the drape detail of the simulated mesh over a smoothed base surface, and an ambient occlusion map by ray casting through the BVH, with visible bodies as occluders. Writes <name>_<map>.png beside the path, one file per map and UDIM tile. Synchronous; the ray budget is capped.",
        keys: &[],
        hints: EXTERNAL,
        schema,
        run,
    }]
}
