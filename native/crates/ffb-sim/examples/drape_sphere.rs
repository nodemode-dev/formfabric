mod support;

use ffb_fabric::Physical;
use ffb_sim::{compare_positions, reference_step, Cloth, Collider, Material, Params, Solver};

fn arg(name: &str, default: f32) -> f32 {
    let key = format!("--{name}=");
    std::env::args().find_map(|a| a.strip_prefix(&key).and_then(|v| v.parse().ok())).unwrap_or(default)
}

fn main() {
    let spacing = arg("spacing", 20.0);
    let n = (1000.0 / spacing).round() as usize + 1;
    let frames = arg("frames", 90.0) as usize;
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    let radius = arg("radius", 200.0);
    let height = radius + arg("drop", 40.0);
    let hang = arg("hang", 0.0) > 0.0;
    if hang {
        support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, -t, 0.0]);
        for v in [0, n - 1] {
            cloth.pins.push(ffb_sim::PinPoint { vertex: v as u32, target: [v as f32 * spacing - 500.0, 0.0, 0.0] });
        }
    } else {
        support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, height, t - 500.0]);
    }
    let mut params = Params { substeps: arg("substeps", 5.0) as u32, iterations: arg("iterations", 10.0) as u32, accel: arg("accel", 0.0), ..Params::default() };
    params.nonlinear = arg("nonlinear", 0.0) > 0.0;
    params.friction = arg("friction", params.friction);
    let colliders = if hang { Vec::new() } else { vec![Collider::Sphere { center: [0.0, 0.0, 0.0], radius }, Collider::Plane { point: [0.0, -radius, 0.0], normal: [0.0, 1.0, 0.0] }] };

    let mut vbd = Solver::new(cloth.clone(), params);
    vbd.set_colliders(colliders.clone());
    let mut cg = Solver::new(cloth, params);
    cg.set_colliders(colliders);
    println!("vertices {} triangles {} substeps {} iterations {} frames {}", vbd.vertex_count(), 2 * (n - 1) * (n - 1), params.substeps, params.iterations, frames);

    let every = arg("every", 15.0) as usize;
    let mut vbd_trace: std::collections::HashMap<usize, Solver> = std::collections::HashMap::new();
    let mut total = 0.0;
    let mut worst: f32 = 0.0;
    let mut last = ffb_sim::StepStats::default();
    for f in 0..frames {
        last = vbd.step();
        total += last.ms;
        worst = worst.max(last.ms);
        if f % every == every - 1 {
            vbd_trace.insert(f, vbd.clone());
            println!("vbd frame {:3}  {:7.2} ms  KE {:10.3}  vmax {:8.2}  contacts {:5}  penetration {:.3}  strain +{:.4} -{:.4}", f + 1, last.ms, vbd.kinetic_energy(), last.max_velocity, last.contacts, vbd.max_penetration(), vbd.max_stretch(), vbd.max_compression());
        }
    }
    println!("vbd: mean {:.2} ms/frame, worst {:.2} ms, final KE {:.4} uN.mm, max penetration {:.4} mm, strain +{:.4} -{:.4}, last contacts {}", total / frames as f32, worst, vbd.kinetic_energy(), vbd.max_penetration(), vbd.max_stretch(), vbd.max_compression(), last.contacts);

    if arg("ref", 1.0) <= 0.0 {
        return;
    }
    let mut total_ref = 0.0;
    let mut cg_its = 0;
    for f in 0..frames {
        let s = reference_step(&mut cg);
        total_ref += s.ms;
        cg_its += s.iterations;
        if arg("trace", 0.0) > 0.0 {
            println!("ref {} KE {:.1} vmax {:.1} contacts {}", f + 1, cg.kinetic_energy(), s.max_velocity, s.contacts);
        }
        if let Some(snap) = vbd_trace.get(&f) {
            let (m, x) = compare_positions(snap, &cg);
            println!("ref frame {:3}  {:7.2} ms  cg {:5}  KE {:10.3}  vmax {:8.2}  contacts {:5}  strain +{:.4} -{:.4}  distance to vbd mean {:.3} max {:.3}", f + 1, s.ms, s.iterations, cg.kinetic_energy(), s.max_velocity, s.contacts, cg.max_stretch(), cg.max_compression(), m, x);
        }
    }
    println!("reference: mean {:.2} ms/frame, cg iterations/frame {}, final KE {:.4}, max penetration {:.4} mm, strain +{:.4} -{:.4}", total_ref / frames as f32, cg_its / frames as u32, cg.kinetic_energy(), cg.max_penetration(), cg.max_stretch(), cg.max_compression());
    let (mean, max) = compare_positions(&vbd, &cg);
    println!("vbd vs reference after {} frames ({:.1} s): mean vertex distance {:.3} mm, max {:.3} mm", frames, frames as f32 * params.time_step, mean, max);
}
