mod support;

use ffb_fabric::Physical;
use ffb_sim::{Cloth, Collider, Material, Params, SeamPair, Solver};

fn arg(name: &str, default: f32) -> f32 {
    let key = format!("--{name}=");
    std::env::args().find_map(|a| a.strip_prefix(&key).and_then(|v| v.parse().ok())).unwrap_or(default)
}

fn main() {
    let spacing = 20.0f32;
    let (nx, ny) = ((400.0 / spacing) as usize + 1, (600.0 / spacing) as usize + 1);
    let r = 400.0 / std::f32::consts::PI;
    let frames = arg("frames", 90.0) as usize;
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    let top = support::add_grid(&mut cloth, nx, ny, spacing, 0, &|s, t| {
        let phi = std::f32::consts::PI * s / 400.0;
        [t - 300.0, r * phi.sin(), r * phi.cos()]
    });
    let bottom = support::add_grid(&mut cloth, nx, ny, spacing, 1, &|s, t| {
        let phi = std::f32::consts::PI + std::f32::consts::PI * s / 400.0;
        [t - 300.0, r * phi.sin(), r * phi.cos()]
    });
    for j in 0..ny {
        cloth.seams.push(SeamPair { a: top.index(0, j), b: bottom.index(nx - 1, j), rest_length: 0.0, stiffness: 0.0 });
        cloth.seams.push(SeamPair { a: top.index(nx - 1, j), b: bottom.index(0, j), rest_length: 0.0, stiffness: 0.0 });
    }
    let seams = cloth.seams.clone();
    let params = Params { substeps: arg("substeps", 5.0) as u32, iterations: arg("iterations", 10.0) as u32, ..Params::default() };
    let capsule_radius = 100.0;
    let colliders = vec![Collider::Capsule { a: [-400.0, 0.0, 0.0], b: [400.0, 0.0, 0.0], radius: capsule_radius }, Collider::Plane { point: [0.0, -400.0, 0.0], normal: [0.0, 1.0, 0.0] }];
    let mut solver = Solver::new(cloth, params);
    solver.set_colliders(colliders);
    println!("vertices {} seams {} substeps {} iterations {} frames {}", solver.vertex_count(), seams.len(), params.substeps, params.iterations, frames);

    let mut total = 0.0;
    for f in 0..frames {
        let stats = solver.step();
        total += stats.ms;
        if f % 15 == 14 {
            let p = solver.positions();
            let gap = seams.iter().map(|s| {
                let (a, b) = (p[s.a as usize], p[s.b as usize]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            }).fold(0.0f32, f32::max);
            println!("frame {:3}  {:7.2} ms  KE {:10.3}  vmax {:8.2}  contacts {:4}  penetration {:.3}  max seam gap {:.3} mm", f + 1, stats.ms, solver.kinetic_energy(), stats.max_velocity, stats.contacts, solver.max_penetration(), gap);
        }
    }
    println!("tube: mean {:.2} ms/frame, final KE {:.3} uN.mm, max penetration {:.4} mm", total / frames as f32, solver.kinetic_energy(), solver.max_penetration());
}
