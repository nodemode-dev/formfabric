mod support;

use ffb_body::import::{read, Options, Units};
use ffb_fabric::Physical;
use ffb_sim::collide::Prepared;
use ffb_sim::{Cloth, Collider, Material, Method, SeamPair, Solver};
use support::{arg, arg_str, name};

fn capsule_stack() -> (Vec<Collider>, f32) {
    let torso = Collider::Capsule { a: [0.0, 1000.0, 0.0], b: [0.0, 1300.0, 0.0], radius: 150.0 };
    let hips = Collider::Capsule { a: [0.0, 820.0, 0.0], b: [0.0, 960.0, 0.0], radius: 175.0 };
    (vec![torso, hips, Collider::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] }], 1700.0)
}

fn load() -> (Vec<Collider>, f32, Vec<[f32; 3]>) {
    let path = arg_str("body", "/home/claude/ff/formfabric/assets/bodies/mpfb2-female/body.glb");
    if path == "capsules" {
        let (c, h) = capsule_stack();
        return (c, h, Vec::new());
    }
    let options = Options { units: Units::Auto, up: ffb_body::math::IDENTITY, ground: true, name: None };
    match read(&path, &options) {
        Ok(body) => {
            let mut top = f32::MIN;
            let mut bottom = f32::MAX;
            for p in &body.positions {
                top = top.max(p[1]);
                bottom = bottom.min(p[1]);
            }
            println!("body {} : {} vertices {} triangles, height {:.0} mm (y {:.0} to {:.0})", body.name, body.positions.len(), body.triangles.len(), top - bottom, bottom, top);
            let colliders = vec![ffb_body::collider(&body), Collider::Plane { point: [0.0, bottom, 0.0], normal: [0.0, 1.0, 0.0] }];
            (colliders, top - bottom, body.positions.clone())
        }
        Err(e) => {
            println!("could not load {path}: {e}; using a capsule stack");
            let (c, h) = capsule_stack();
            (c, h, Vec::new())
        }
    }
}

fn axis_z(points: &[[f32; 3]], y: f32) -> f32 {
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for p in points {
        if (p[1] - y).abs() < 10.0 && p[0].abs() < 90.0 {
            lo = lo.min(p[2]);
            hi = hi.max(p[2]);
        }
    }
    if lo > hi {
        0.0
    } else {
        0.5 * (lo + hi)
    }
}

fn radius(prep: &[Prepared], c: [f64; 3], dir: [f64; 3]) -> Option<f64> {
    let sd = |r: f64| -> f64 {
        let p = glam::DVec3::new(c[0] + dir[0] * r, c[1] + dir[1] * r, c[2] + dir[2] * r);
        prep.iter().filter_map(|q| q.signed_distance(p, 1.0e9)).map(|(d, _)| d).fold(f64::MAX, f64::min)
    };
    if sd(0.0) >= 0.0 {
        return None;
    }
    let mut r = 0.0;
    while r < 450.0 {
        r += 2.0;
        if sd(r) >= 0.0 {
            let (mut lo, mut hi) = (r - 2.0, r);
            for _ in 0..20 {
                let mid = 0.5 * (lo + hi);
                if sd(mid) < 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            return Some(0.5 * (lo + hi));
        }
    }
    None
}

fn build(prep: &[Prepared], points: &[[f32; 3]], height: f32, spacing: f32, gap: f32) -> Option<(Cloth, usize)> {
    let (y0, y1) = (arg("y0", 0.60) * height, arg("y1", 0.67) * height);
    let rows = ((y1 - y0) / spacing).round() as usize + 1;
    let dy = (y1 - y0) / (rows - 1) as f32;
    let cols = (std::f32::consts::PI * 150.0 / spacing).round() as usize + 1;
    let place = |theta: f32, y: f32| -> Option<[f32; 3]> {
        let cz = axis_z(points, y);
        let dir = [theta.sin() as f64, 0.0, theta.cos() as f64];
        let c = [0.0, y as f64, cz as f64];
        let r = radius(prep, c, dir)? as f32 + gap;
        Some([dir[0] as f32 * r, y, cz + dir[2] as f32 * r])
    };
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    let mut panels: Vec<Vec<[f32; 3]>> = Vec::new();
    for side in 0..2 {
        let mut grid = Vec::new();
        for j in 0..rows {
            for i in 0..cols {
                let theta = (-90.0 + 180.0 * i as f32 / (cols - 1) as f32 + 180.0 * side as f32).to_radians();
                grid.push(place(theta, y0 + dy * j as f32)?);
            }
        }
        panels.push(grid);
    }
    let mut total_arc = 0.0f32;
    for j in 0..rows {
        for i in 1..cols {
            let (p, q) = (panels[0][j * cols + i - 1], panels[0][j * cols + i]);
            total_arc += ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        }
    }
    let ds = total_arc / rows as f32 / (cols - 1) as f32;
    let mut firsts = Vec::new();
    for (side, grid) in panels.iter().enumerate() {
        let g = support::add_grid(&mut cloth, cols, rows, 1.0, side as u32, &|_, _| [0.0; 3]);
        firsts.push(g.first);
        for j in 0..rows {
            for i in 0..cols {
                let v = (g.first as usize) + j * cols + i;
                cloth.positions[v] = grid[j * cols + i];
                cloth.rest[v] = [i as f32 * ds, j as f32 * dy];
            }
        }
    }
    if arg("debug", 0.0) > 0.0 {
        for j in 0..rows {
            let row = &panels[0][j * cols..(j + 1) * cols];
            println!("row {j} y {:.0}: left {:?} mid {:?} right {:?} arc {:.0}", row[0][1], row[0].map(|v| v as i32), row[cols / 2].map(|v| v as i32), row[cols - 1].map(|v| v as i32), cloth.rest[firsts[0] as usize + j * cols + cols - 1][0]);
        }
    }
    let idx = |side: usize, i: usize, j: usize| firsts[side] + (j * cols + i) as u32;
    for j in 0..rows {
        cloth.seams.push(SeamPair { a: idx(0, cols - 1, j), b: idx(1, 0, j), rest_length: 0.0, stiffness: 0.0 });
        cloth.seams.push(SeamPair { a: idx(0, 0, j), b: idx(1, cols - 1, j), rest_length: 0.0, stiffness: 0.0 });
    }
    Some((cloth, cols))
}

fn run(method: Method, cloth: &Cloth, colliders: &[Collider], frames: usize) {
    let params = support::params_for(method);
    let mut s = Solver::new(cloth.clone(), params);
    s.set_colliders(colliders.to_vec());
    let seams = cloth.seams.clone();
    println!("{} vertices {} substeps {} newton {} tol {} frames {}", name(method), s.vertex_count(), params.substeps, params.newton, params.cg_tolerance, frames);
    let mut total = 0.0;
    let mut worst_pen: f32 = 0.0;
    let mut worst_v: f32 = 0.0;
    let start_y = cloth.positions.iter().map(|p| p[1]).sum::<f32>() / cloth.positions.len() as f32;
    println!("  placed shape: strain +{:.4} -{:.4}, penetration {:.3}", s.max_stretch(), s.max_compression(), s.max_penetration());
    for f in 0..frames {
        let st = s.step();
        total += st.ms;
        worst_pen = worst_pen.max(s.max_penetration());
        worst_v = worst_v.max(st.max_velocity);
        if f % 15 == 14 {
            let p = s.positions();
            let gap = seams
                .iter()
                .map(|q| {
                    let (a, b) = (p[q.a as usize], p[q.b as usize]);
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
                })
                .fold(0.0f32, f32::max);
            let mean_y = p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32;
            println!("  frame {:3}  {:7.2} ms  KE {:10.3}  vmax {:8.2}  contacts {:4}  penetration {:.3}  seam gap {:.3}  drop {:.1} mm", f + 1, st.ms, s.kinetic_energy(), st.max_velocity, st.contacts, s.max_penetration(), gap, start_y - mean_y);
        }
    }
    println!(
        "{} bodice: mean {:.2} ms/frame, final KE {:.3}, worst penetration {:.4} mm, max velocity {:.1} mm/s, strain +{:.4} -{:.4}",
        name(method),
        total / frames as f32,
        s.kinetic_energy(),
        worst_pen,
        worst_v,
        s.max_stretch(),
        s.max_compression()
    );
}

fn main() {
    let (colliders, height, points) = load();
    let prep: Vec<Prepared> = colliders.iter().map(Prepared::from_collider).collect();
    let spacing = arg("spacing", 14.0);
    let Some((cloth, cols)) = build(&prep, &points, height, spacing, arg("gap", 15.0)) else {
        println!("could not place the bodice on this body");
        return;
    };
    println!("bodice: 2 panels, {} columns, {} vertices, {} seams", cols, cloth.positions.len(), cloth.seams.len());
    let frames = arg("frames", 150.0) as usize;
    for m in support::methods() {
        run(m, &cloth, &colliders, frames);
    }
}
