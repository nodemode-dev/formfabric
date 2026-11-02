mod support;

use ffb_fabric::Physical;
use ffb_sim::{Cloth, Collider, Material, Params, PinPoint, Solver};
use support::{arg, arg_str, name};

fn scene(n: usize, hang: bool) -> (Cloth, Vec<Collider>) {
    let spacing = 1000.0 / (n - 1) as f32;
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    if hang {
        support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, -t, 0.0]);
        for v in [0, n - 1] {
            cloth.pins.push(PinPoint { vertex: v as u32, target: [v as f32 * spacing - 500.0, 0.0, 0.0] });
        }
        return (cloth, Vec::new());
    }
    let radius = 200.0;
    support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, radius + 40.0, t - 500.0]);
    (cloth, vec![Collider::Sphere { center: [0.0, 0.0, 0.0], radius }, Collider::Plane { point: [0.0, -radius, 0.0], normal: [0.0, 1.0, 0.0] }])
}

fn main() {
    let sizes: Vec<usize> = arg_str("sizes", "26,71").split(',').filter_map(|s| s.parse().ok()).collect();
    let frames = arg("frames", 60.0) as usize;
    let hang = arg_str("scene", "drape") == "hang";
    let gold = arg("gold", 0.0);
    let marks = arg("marks", 10.0) as usize;
    for m in support::methods() {
        let params: Params = support::params_for(m);
        for &n in &sizes {
            let (cloth, colliders) = scene(n, hang);
            let mut s = Solver::new(cloth, params);
            s.set_colliders(colliders);
            let mut ms = Vec::new();
            let mut its = 0u64;
            let mut snaps: Vec<Solver> = Vec::new();
            let cpu0 = support::cpu_ms();
            for f in 0..frames {
                let st = s.step();
                ms.push(st.ms);
                its += st.iterations as u64;
                if gold > 0.0 && (f + 1) % marks == 0 {
                    snaps.push(s.clone());
                }
            }
            if gold > 0.0 {
                let gp = Params { method: ffb_sim::Method::Implicit, substeps: 10, newton: 3, cg_iterations: 2000, cg_tolerance: 1.0e-6, ..params };
                let (cloth, colliders) = scene(n, hang);
                let mut g = Solver::new(cloth, gp);
                g.set_colliders(colliders);
                let mut line = String::from("  distance to gold implicit (10 substeps, newton 3, tol 1e-6):");
                let mut k = 0;
                for f in 0..frames {
                    g.step();
                    if (f + 1) % marks == 0 {
                        let (mean, max) = ffb_sim::compare_positions(&snaps[k], &g);
                        line.push_str(&format!(" f{} {:.2}/{:.2}", f + 1, mean, max));
                        k += 1;
                    }
                }
                println!("{line} mm (mean/max)");
            }
            let cpu = (support::cpu_ms() - cpu0) / frames as f64;
            let vp = s.vbd_profile();
            if m == ffb_sim::Method::Vbd {
                let f = frames as f64;
                println!("  cpu {:.2} ms/frame  vbd phases ms/frame: begin {:.2} hinge-eval {:.2} tri-eval {:.2} sweep {:.2} end(collide+self) {:.2}", cpu, vp[0] / f, vp[1] / f, vp[2] / f, vp[3] / f, vp[5] / f);
            } else {
                println!("  cpu {:.2} ms/frame", cpu);
            }
            let mean = ms.iter().sum::<f32>() / ms.len() as f32;
            let late: Vec<f32> = ms[ms.len() / 2..].to_vec();
            let late_mean = late.iter().sum::<f32>() / late.len() as f32;
            let mut sorted = ms.clone();
            sorted.sort_by(|a, b| a.total_cmp(b));
            let p = s.implicit_profile();
            println!(
                "{:8} {:6} vertices substeps {} : mean {:8.2} ms/frame  late {:8.2}  worst {:8.2}  its/frame {:7.1}  penetration {:.3}  strain +{:.4} -{:.4}  KE {:.2}  profile(ms/frame) assemble {:.2} pcg {:.2}",
                name(m),
                s.vertex_count(),
                params.substeps,
                mean,
                late_mean,
                sorted[sorted.len() - 1],
                its as f32 / frames as f32,
                s.max_penetration(),
                s.max_stretch(),
                s.max_compression(),
                s.kinetic_energy(),
                p[0] / frames as f64,
                p[1] / frames as f64
            );
        }
    }
}
