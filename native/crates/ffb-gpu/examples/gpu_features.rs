use ffb_fabric::Physical;
use ffb_gpu::AnyStepper;
use ffb_sim::{Cloth, Collider, Material, Method, Params, PinPoint, SeamPair, Solver};

fn arg(name: &str, default: f32) -> f32 {
    let key = format!("--{name}=");
    std::env::args().find_map(|a| a.strip_prefix(&key).and_then(|v| v.parse().ok())).unwrap_or(default)
}

fn add_grid(cloth: &mut Cloth, nx: usize, ny: usize, spacing: f32, layer: i32, panel: u32, place: &dyn Fn(f32, f32) -> [f32; 3]) -> usize {
    let base = cloth.positions.len();
    for j in 0..ny {
        for i in 0..nx {
            let (s, t) = (i as f32 * spacing, j as f32 * spacing);
            cloth.positions.push(place(s, t));
            cloth.rest.push([s, t]);
            cloth.vertex_panel.push(panel);
            cloth.vertex_layer.push(layer);
        }
    }
    let idx = |i: usize, j: usize| (base + j * nx + i) as u32;
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
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
    base
}

fn new_cloth() -> Cloth {
    Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() }
}

fn dist(a: &[[f32; 3]], b: &[[f32; 3]]) -> (f32, f32) {
    let mut sum = 0.0f64;
    let mut max = 0.0f32;
    for (p, q) in a.iter().zip(b) {
        let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        sum += d as f64;
        max = max.max(d);
    }
    ((sum / a.len().max(1) as f64) as f32, max)
}

fn uv_sphere(center: [f32; 3], radius: f32, rings: usize, segs: usize) -> Collider {
    let mut positions = vec![[center[0], center[1] + radius, center[2]]];
    for r in 1..rings {
        let phi = std::f32::consts::PI * r as f32 / rings as f32;
        for s in 0..segs {
            let th = std::f32::consts::TAU * s as f32 / segs as f32;
            positions.push([center[0] + radius * phi.sin() * th.cos(), center[1] + radius * phi.cos(), center[2] + radius * phi.sin() * th.sin()]);
        }
    }
    positions.push([center[0], center[1] - radius, center[2]]);
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

struct Pair {
    cpu: Solver,
    gpu: AnyStepper,
    cpu_ms: f32,
    gpu_ms: f32,
}

impl Pair {
    fn new(cloth: Cloth, params: Params, colliders: Vec<Collider>) -> Pair {
        let mut cpu = Solver::new(cloth.clone(), params);
        cpu.set_colliders(colliders.clone());
        let (mut gpu, note) = AnyStepper::new(cloth, params, true);
        gpu.set_colliders(colliders);
        println!("  backend: {note}, {} vertices", gpu.vertex_count());
        Pair { cpu, gpu, cpu_ms: 0.0, gpu_ms: 0.0 }
    }

    fn step(&mut self) {
        self.cpu_ms += self.cpu.step().ms;
        self.gpu_ms += self.gpu.step().ms;
    }

    fn report(&self, label: &str, frames: usize) {
        let (mean, max) = dist(self.cpu.positions(), self.gpu.positions());
        println!("  {label}: gpu vs cpu mean {mean:.4} mm max {max:.4} mm; cpu {:.1} ms/frame, gpu {:.1} ms/frame", self.cpu_ms / frames as f32, self.gpu_ms / frames as f32);
    }
}

fn params() -> Params {
    Params { method: Method::Vbd, substeps: arg("substeps", 5.0) as u32, iterations: arg("iterations", 10.0) as u32, ..Params::default() }
}

fn wind(frames: usize) {
    println!("wind (external forces), hanging sheet");
    let n = arg("n", 31.0) as usize;
    let mut cloth = new_cloth();
    add_grid(&mut cloth, n, n, 25.0, 0, 0, &|s, t| [s - 375.0, -t, 0.0]);
    let pinned: Vec<usize> = if arg("curtain", 1.0) > 0.5 { (0..n).collect() } else { vec![0, n - 1] };
    for v in pinned {
        cloth.pins.push(PinPoint { vertex: v as u32, target: cloth.positions[v] });
    }
    let mut twin_cloth = cloth.clone();
    for (i, p) in twin_cloth.positions.iter_mut().enumerate() {
        p[2] += 0.001 * (((i * 37) % 11) as f32 - 5.0) / 5.0;
    }
    let mut twin = Solver::new(twin_cloth, params());
    let mut pair = Pair::new(cloth, params(), Vec::new());
    let force = arg("wind", 0.02);
    let f = vec![[0.0, 0.0, force]; n * n];
    pair.cpu.set_external_forces(&f);
    pair.gpu.set_external_forces(&f);
    twin.set_external_forces(&f);
    let start: Vec<[f32; 3]> = pair.cpu.positions().to_vec();
    let mean_z = |p: &[[f32; 3]]| p.iter().map(|q| q[2]).sum::<f32>() / (n * n) as f32;
    for k in 0..frames {
        pair.step();
        twin.step();
        if (k + 1) % 5 == 0 {
            println!("  frame {}: mean z cpu {:.2} gpu {:.2} cpu twin (1 micrometre start offset) {:.2} mm", k + 1, mean_z(pair.cpu.positions()), mean_z(pair.gpu.positions()), mean_z(twin.positions()));
        }
        if (k + 1) % 15 == 0 {
            pair.report(&format!("frame {}", k + 1), k + 1);
        }
    }
    let (mut cos_sum, mut cos_n, mut same_z, mut total) = (0.0f64, 0usize, 0usize, 0usize);
    for ((s, c), g) in start.iter().zip(pair.cpu.positions()).zip(pair.gpu.positions()) {
        let dc = [c[0] - s[0], c[1] - s[1], c[2] - s[2]];
        let dg = [g[0] - s[0], g[1] - s[1], g[2] - s[2]];
        let (lc, lg) = ((dc[0] * dc[0] + dc[1] * dc[1] + dc[2] * dc[2]).sqrt(), (dg[0] * dg[0] + dg[1] * dg[1] + dg[2] * dg[2]).sqrt());
        if lc > 0.5 && lg > 0.5 {
            cos_sum += ((dc[0] * dg[0] + dc[1] * dg[1] + dc[2] * dg[2]) / (lc * lg)) as f64;
            cos_n += 1;
        }
        if dc[2].abs() > 0.5 {
            total += 1;
            if dc[2] * dg[2] > 0.0 {
                same_z += 1;
            }
        }
    }
    println!("  displacement direction: mean cosine {:.3} over {cos_n} vertices; z sign agrees {same_z}/{total}", cos_sum / cos_n.max(1) as f64);
    println!("  mean z displacement cpu {:.2} gpu {:.2} mm", mean_z(pair.cpu.positions()), mean_z(pair.gpu.positions()));
}

fn seams(frames: usize) {
    println!("seams, two panels joined by pair links, hanging");
    let n = 16usize;
    let mut cloth = new_cloth();
    let a = add_grid(&mut cloth, n, n, 20.0, 0, 0, &|s, t| [s - 330.0, -t, 0.0]);
    let b = add_grid(&mut cloth, n, n, 20.0, 0, 1, &|s, t| [s + 30.0, -t, 8.0]);
    for j in 0..n {
        cloth.seams.push(SeamPair { a: (a + j * n + n - 1) as u32, b: (b + j * n) as u32, rest_length: 0.0, stiffness: 0.0 });
    }
    for v in [a, a + n - 1, b, b + n - 1] {
        cloth.pins.push(PinPoint { vertex: v as u32, target: cloth.positions[v] });
    }
    let mut pair = Pair::new(cloth, params(), Vec::new());
    for k in 0..frames {
        pair.step();
        if k == 0 {
            println!("  frame 1 seam gap cpu {:.3} gpu {:.3} mm", pair.cpu.max_link_gap(), pair.gpu.max_link_gap());
        }
    }
    println!("  seam gap after {frames} frames: cpu {:.3} gpu {:.3} mm, links {}", pair.cpu.max_link_gap(), pair.gpu.max_link_gap(), pair.gpu.link_count());
    pair.report("end", frames);
}

fn moving(frames: usize) {
    println!("sheet on a translating mesh sphere (per frame collider update), friction {}", arg("friction", 0.6));
    let n = 31usize;
    let mut cloth = new_cloth();
    add_grid(&mut cloth, n, n, 20.0, 0, 0, &|s, t| [s - 300.0, 140.0, t - 300.0]);
    let p = Params { friction: arg("friction", 0.6), ..params() };
    let sphere = |x: f32| vec![uv_sphere([x, 0.0, 0.0], 100.0, 24, 48), Collider::Plane { point: [0.0, -100.0, 0.0], normal: [0.0, 1.0, 0.0] }];
    let mut pair = Pair::new(cloth, p, sphere(0.0));
    let t0 = std::time::Instant::now();
    for k in 0..frames {
        let x = 0.5 * k as f32;
        pair.cpu.set_colliders(sphere(x));
        pair.gpu.set_colliders(sphere(x));
        pair.step();
        if (k + 1) % 15 == 0 {
            pair.report(&format!("frame {}", k + 1), k + 1);
        }
    }
    println!("  penetration cpu {:.4} gpu {:.4} mm; wall {:.1} s", pair.cpu.max_penetration(), pair.gpu.max_penetration(), t0.elapsed().as_secs_f32());
}

fn layer_depth(cloth: &Cloth, p: &[[f32; 3]]) -> f32 {
    let v3 = |q: [f32; 3]| glam::DVec3::new(f64::from(q[0]), f64::from(q[1]), f64::from(q[2]));
    let mut worst = 0.0f64;
    for (vi, q) in p.iter().enumerate() {
        let x = v3(*q);
        let lv = cloth.vertex_layer[vi];
        for t in &cloth.triangles {
            let lt = cloth.vertex_layer[t[0] as usize];
            if lv <= lt || t.contains(&(vi as u32)) {
                continue;
            }
            let (a, b, c) = (v3(p[t[0] as usize]), v3(p[t[1] as usize]), v3(p[t[2] as usize]));
            if (x - a).abs().max_element() > 70.0 {
                continue;
            }
            let (cp, _) = ffb_sim::bvh::closest_on_triangle(x, a, b, c);
            if (x - cp).length() > 8.0 {
                continue;
            }
            let bary = ffb_sim::bvh::barycentric(cp, a, b, c);
            if bary.iter().any(|w| *w <= 1.0e-6) {
                continue;
            }
            let tn = (b - a).cross(c - a).normalize_or_zero();
            worst = worst.max(-(x - cp).dot(tn));
        }
    }
    worst as f32
}

fn stack(frames: usize) {
    let flip = arg("flip", 0.0) > 0.5;
    println!("self contact, two layered sheets falling on a floor (layers.rs orientation{})", if flip { ", FLIPPED winding" } else { "" });
    let n = 26usize;
    let mut cloth = new_cloth();
    let place = |y: f32| move |s: f32, t: f32| if flip { [s - 250.0, y, t - 250.0] } else { [t - 250.0, y, s - 250.0] };
    let lo = add_grid(&mut cloth, n, n, 20.0, 0, 0, &place(arg("low", 30.0)));
    let hi = add_grid(&mut cloth, n, n, 20.0, 1, 1, &place(arg("high", 60.0)));
    let p = Params { self_collision: true, collision_thickness: arg("thick", 1.0), ..params() };
    let floor = vec![Collider::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] }];
    let mut pair = Pair::new(cloth.clone(), p, floor);
    let mut tester = Solver::new(cloth.clone(), p);
    let mut worst_g = 0.0f32;
    let mut worst_c = 0.0f32;
    for k in 0..frames {
        pair.step();
        let (dg, dc) = (layer_depth(&cloth, pair.gpu.positions()), layer_depth(&cloth, pair.cpu.positions()));
        worst_g = worst_g.max(dg);
        worst_c = worst_c.max(dc);
        if arg("trace", 0.0) > 0.5 {
            let m = n * n / 2 + n / 2;
            println!("  trace {}: centre y gpu lower {:.3} upper {:.3}; cpu lower {:.3} upper {:.3}", k + 1, pair.gpu.positions()[lo + m][1], pair.gpu.positions()[hi + m][1], pair.cpu.positions()[lo + m][1], pair.cpu.positions()[hi + m][1]);
        }
        if (k + 1) % 15 == 0 {
            let g = pair.gpu.positions();
            let mut worst = f32::MAX;
            let mut worst_cpu = f32::MAX;
            let c = pair.cpu.positions();
            for j in 0..n * n {
                worst = worst.min(g[hi + j][1] - g[lo + j][1]);
                worst_cpu = worst_cpu.min(c[hi + j][1] - c[lo + j][1]);
            }
            tester.set_positions(g);
            println!("  frame {}: upper minus lower min gpu {worst:.3} cpu {worst_cpu:.3} mm; interpenetration depth now gpu {dg:.3} cpu {dc:.3} mm; intersecting pairs gpu {}, cpu {}", k + 1, tester.self_intersections().len(), pair.cpu.self_intersections().len());
        }
    }
    println!("  worst interpenetration over {frames} frames: gpu {worst_g:.3} mm, cpu {worst_c:.3} mm");
    pair.report("end", frames);
}

fn layered_drape(frames: usize) {
    println!("layered drape, two stacked sheets (layers 0 and 1) onto a sphere and floor");
    let n = 26usize;
    let mut cloth = new_cloth();
    let place = |y: f32| move |s: f32, t: f32| [t - 250.0, y, s - 250.0];
    add_grid(&mut cloth, n, n, 20.0, 0, 0, &place(arg("low", 130.0)));
    add_grid(&mut cloth, n, n, 20.0, 1, 1, &place(arg("high", 150.0)));
    let p = Params { self_collision: true, collision_thickness: arg("thick", 1.0), ..params() };
    let colliders = vec![Collider::Sphere { center: [0.0, 0.0, 0.0], radius: 100.0 }, Collider::Plane { point: [0.0, -100.0, 0.0], normal: [0.0, 1.0, 0.0] }];
    let mut pair = Pair::new(cloth.clone(), p, colliders);
    let (mut worst_g, mut worst_c) = (0.0f32, 0.0f32);
    for k in 0..frames {
        pair.step();
        let (dg, dc) = (layer_depth(&cloth, pair.gpu.positions()), layer_depth(&cloth, pair.cpu.positions()));
        worst_g = worst_g.max(dg);
        worst_c = worst_c.max(dc);
        if (k + 1) % 15 == 0 {
            println!("  frame {}: interpenetration depth now gpu {dg:.3} cpu {dc:.3} mm; sphere penetration gpu {:.3} cpu {:.3}", k + 1, pair.gpu.max_penetration(), pair.cpu.max_penetration());
            pair.report(&format!("frame {}", k + 1), k + 1);
        }
    }
    println!("  worst interpenetration over {frames} frames: gpu {worst_g:.3} mm, cpu {worst_c:.3} mm");
}

fn main() {
    println!("adapter: {}", ffb_gpu::adapter_info());
    let frames = arg("frames", 60.0) as usize;
    let which = arg("scene", 0.0) as u32;
    if which == 0 || which == 1 {
        wind(frames);
    }
    if which == 0 || which == 2 {
        seams(frames);
    }
    if which == 0 || which == 3 {
        moving(frames);
    }
    if which == 0 || which == 4 {
        stack(frames);
    }
    if which == 0 || which == 5 {
        layered_drape(frames);
    }
}
