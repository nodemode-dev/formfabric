use ffb_fabric::Physical;
use ffb_gpu::GpuSolver;
use ffb_sim::{Cloth, Collider, Material, Method, Params, PinPoint, Solver};

fn arg(name: &str, default: f32) -> f32 {
    let key = format!("--{name}=");
    std::env::args().find_map(|a| a.strip_prefix(&key).and_then(|v| v.parse().ok())).unwrap_or(default)
}

fn grid(spacing: f32, place: &dyn Fn(f32, f32) -> [f32; 3]) -> (Cloth, usize) {
    let n = (1000.0 / spacing).round() as usize + 1;
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    for j in 0..n {
        for i in 0..n {
            let (s, t) = (i as f32 * spacing, j as f32 * spacing);
            cloth.positions.push(place(s, t));
            cloth.rest.push([s, t]);
            cloth.vertex_panel.push(0);
        }
    }
    let idx = |i: usize, j: usize| (j * n + i) as u32;
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1));
            if (i + j) % 2 == 0 {
                cloth.triangles.push([a, b, c]);
                cloth.triangles.push([a, c, d]);
            } else {
                cloth.triangles.push([a, b, d]);
                cloth.triangles.push([b, c, d]);
            }
            cloth.triangle_material.extend([0, 0]);
            cloth.triangle_grain.extend([0.0, 0.0]);
        }
    }
    (cloth, n)
}

fn uv_sphere(radius: f32, rings: usize, segs: usize) -> Collider {
    let mut positions = vec![[0.0, radius, 0.0]];
    for r in 1..rings {
        let phi = std::f32::consts::PI * r as f32 / rings as f32;
        for s in 0..segs {
            let th = std::f32::consts::TAU * s as f32 / segs as f32;
            positions.push([radius * phi.sin() * th.cos(), radius * phi.cos(), radius * phi.sin() * th.sin()]);
        }
    }
    positions.push([0.0, -radius, 0.0]);
    let bottom = (positions.len() - 1) as u32;
    let ring = |r: usize, s: usize| (1 + (r - 1) * segs + s % segs) as u32;
    let mut triangles = Vec::new();
    for s in 0..segs {
        triangles.push([0, ring(1, s + 1), ring(1, s)]);
        triangles.push([bottom, ring(rings - 1, s), ring(rings - 1, s + 1)]);
    }
    for r in 1..rings - 1 {
        for s in 0..segs {
            let (a, b, c, d) = (ring(r, s), ring(r, s + 1), ring(r + 1, s + 1), ring(r + 1, s));
            triangles.push([a, b, c]);
            triangles.push([a, c, d]);
        }
    }
    Collider::Mesh { positions, triangles }
}

fn distance(a: &[[f32; 3]], b: &[[f32; 3]]) -> (f32, f32) {
    let mut sum = 0.0f64;
    let mut max = 0.0f32;
    for (p, q) in a.iter().zip(b) {
        let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        sum += d as f64;
        max = max.max(d);
    }
    ((sum / a.len() as f64) as f32, max)
}

fn run(name: &str, cloth: Cloth, colliders: Vec<Collider>, params: Params, frames: usize) {
    let mut cpu = Solver::new(cloth.clone(), params);
    cpu.set_colliders(colliders.clone());
    let eps = arg("perturb", 0.0);
    let mut twin = cloth.clone();
    for (i, p) in twin.positions.iter_mut().enumerate() {
        p[1] += eps * (((i * 37) % 11) as f32 - 5.0) / 5.0;
    }
    let mut cpu2 = Solver::new(twin, params);
    cpu2.set_colliders(colliders.clone());
    let mut gpu = match GpuSolver::new(cloth, params) {
        Ok(g) => g,
        Err(e) => {
            println!("{name}: gpu unavailable: {e}");
            return;
        }
    };
    gpu.set_colliders(colliders);
    println!("{name}: {} vertices, {} colours, adapter {}", gpu.vertex_count(), gpu.color_count(), gpu.adapter_name());
    let (mut cpu_ms, mut gpu_ms) = (0.0, 0.0);
    let every = arg("every", 15.0) as usize;
    for f in 0..frames {
        let cs = cpu.step();
        cpu_ms += cs.ms;
        if eps > 0.0 {
            cpu2.step();
        }
        let s = gpu.step();
        gpu_ms += s.ms;
        if (f + 1) % every == 0 {
            let (mean, max) = distance(cpu.positions(), gpu.positions());
            let (pm, px) = distance(cpu.positions(), cpu2.positions());
            println!("  frame {:3} ({:.2} s)  gpu vs cpu mean {:.4} mm  max {:.4} mm  gpu vmax {:.1} cpu vmax {:.1}  contacts {}  cpu vs perturbed cpu ({eps} mm) mean {:.4} max {:.4}", f + 1, (f + 1) as f32 * params.time_step, mean, max, s.max_velocity, cs.max_velocity, s.contacts, pm, px);
        }
    }
    println!("  cpu {:.1} ms/frame, gpu {:.1} ms/frame (software device)", cpu_ms / frames as f32, gpu_ms / frames as f32);
}

fn main() {
    println!("adapter: {}", ffb_gpu::adapter_info());
    let spacing = arg("spacing", 20.0);
    let frames = arg("frames", 60.0) as usize;
    let params = Params { method: Method::Vbd, substeps: arg("substeps", 5.0) as u32, iterations: arg("iterations", 10.0) as u32, ..Params::default() };
    let which = arg("scene", 0.0) as u32;
    if which == 0 || which == 1 {
        let (mut cloth, n) = grid(spacing, &|s, t| [s - 500.0, -t, 0.0]);
        for v in [0, n - 1] {
            cloth.pins.push(PinPoint { vertex: v as u32, target: [v as f32 * spacing - 500.0, 0.0, 0.0] });
        }
        run("hanging square", cloth, Vec::new(), params, frames);
    }
    if which == 0 || which == 2 {
        let radius = 200.0;
        let (cloth, _) = grid(spacing, &|s, t| [s - 500.0, radius + 40.0, t - 500.0]);
        let colliders = vec![Collider::Sphere { center: [0.0, 0.0, 0.0], radius }, Collider::Plane { point: [0.0, -radius, 0.0], normal: [0.0, 1.0, 0.0] }];
        run("sphere drape", cloth, colliders, params, frames);
    }
    if which == 0 || which == 3 {
        let radius = 200.0;
        let (cloth, _) = grid(spacing, &|s, t| [s - 500.0, radius + 40.0, t - 500.0]);
        let colliders = vec![uv_sphere(radius, 48, 96), Collider::Plane { point: [0.0, -radius, 0.0], normal: [0.0, 1.0, 0.0] }];
        run("sphere drape (mesh collider)", cloth, colliders, params, frames);
    }
}
