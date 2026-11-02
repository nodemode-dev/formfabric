mod support;

use ffb_fabric::Physical;
use ffb_sim::{Cloth, Collider, Material, Params, PinPoint, Solver};
use support::{arg, arg_str, methods, name};

struct Stat {
    mean_u: f64,
    min_u: f64,
    mean_v: f64,
    min_v: f64,
    area: f64,
}

fn measure(cloth: &Cloth, x: &[[f32; 3]], nx: usize, ny: usize) -> Stat {
    let d = |a: usize, b: usize| {
        let p = x[a];
        let q = x[b];
        (((p[0] - q[0]) as f64).powi(2) + ((p[1] - q[1]) as f64).powi(2) + ((p[2] - q[2]) as f64).powi(2)).sqrt()
    };
    let rd = |a: usize, b: usize| {
        let p = cloth.rest[a];
        let q = cloth.rest[b];
        (((p[0] - q[0]) as f64).powi(2) + ((p[1] - q[1]) as f64).powi(2)).sqrt()
    };
    let (mut su, mut nu, mut mnu, mut sv, mut nv, mut mnv) = (0.0, 0.0, f64::MAX, 0.0, 0.0, f64::MAX);
    for j in 0..ny {
        for i in 0..nx {
            let a = j * nx + i;
            if i + 1 < nx {
                let r = d(a, a + 1) / rd(a, a + 1);
                su += r;
                nu += 1.0;
                mnu = mnu.min(r);
            }
            if j + 1 < ny {
                let r = d(a, a + nx) / rd(a, a + nx);
                sv += r;
                nv += 1.0;
                mnv = mnv.min(r);
            }
        }
    }
    let (mut a3, mut a2) = (0.0, 0.0);
    for t in &cloth.triangles {
        let p: Vec<[f64; 3]> = t.iter().map(|&i| [x[i as usize][0] as f64, x[i as usize][1] as f64, x[i as usize][2] as f64]).collect();
        let u = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let v = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let c = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        a3 += 0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        let r: Vec<[f32; 2]> = t.iter().map(|&i| cloth.rest[i as usize]).collect();
        a2 += 0.5 * (((r[1][0] - r[0][0]) * (r[2][1] - r[0][1]) - (r[2][0] - r[0][0]) * (r[1][1] - r[0][1])) as f64).abs();
    }
    Stat { mean_u: su / nu, min_u: mnu, mean_v: sv / nv, min_v: mnv, area: a3 / a2 }
}

fn main() {
    let spacing = arg("spacing", 20.0);
    let n = (1000.0 / spacing).round() as usize + 1;
    let frames = arg("frames", 90.0) as usize;
    let every = arg("every", 15.0) as usize;
    let scene = arg_str("scene", "hang");
    let fabric = arg_str("fabric", "default");
    let physical = if fabric == "default" { Physical::default() } else { ffb_fabric::presets::find(&fabric).map(|p| p.physical).unwrap_or_default() };
    let radius = arg("radius", 200.0);
    let mut cloth = Cloth { materials: vec![Material::from_physical(&physical)], ..Cloth::default() };
    let colliders = match scene.as_str() {
        "hang" => {
            support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, -t, 0.0]);
            for v in 0..n {
                cloth.pins.push(PinPoint { vertex: v as u32, target: [v as f32 * spacing - 500.0, 0.0, 0.0] });
            }
            Vec::new()
        }
        "sphere" => {
            support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, radius + 40.0, t - 500.0]);
            vec![Collider::Sphere { center: [0.0, 0.0, 0.0], radius }, Collider::Plane { point: [0.0, -radius, 0.0], normal: [0.0, 1.0, 0.0] }]
        }
        "tube" => {
            let gap = arg("gap", 60.0);
            let circ = 1000.0 + gap;
            let r = circ / std::f32::consts::TAU;
            support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| {
                let a = s / circ * std::f32::consts::TAU;
                [r * a.cos(), 300.0 - t, r * a.sin()]
            });
            for j in 0..n {
                cloth.seams.push(ffb_sim::SeamPair { a: (j * n) as u32, b: (j * n + n - 1) as u32, rest_length: 0.0, stiffness: 0.0 });
            }
            vec![Collider::Plane { point: [0.0, -300.0, 0.0], normal: [0.0, 1.0, 0.0] }]
        }
        _ => {
            support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, 1.0, t - 500.0]);
            vec![Collider::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] }]
        }
    };
    for m in methods() {
        let mut params: Params = support::params_for(m);
        if scene == "ground" && arg("gravity", 1.0) <= 0.0 {
            params.gravity = [0.0, 0.0, 0.0];
        }
        let mut s = Solver::new(cloth.clone(), params);
        s.set_colliders(colliders.clone());
        let st = measure(&cloth, s.positions(), n, n);
        println!("{} {} {} {} v {} start area {:.4}", scene, name(m), arg_str("preset", "normal"), fabric, cloth.positions.len(), st.area);
        let close = arg("close", 0.0) as usize;
        if close > 0 {
            s.push_out(0.0);
        }
        for f in 0..frames {
            if close > 0 && f < close {
                let u = f as f32 / close as f32;
                let ramp = ((u - 0.6) / 0.4).clamp(0.0, 1.0);
                let damping = params.air_damping.max(6.0) * (1.0 - ramp) + params.air_damping * ramp;
                s.set_params(Params { gravity: params.gravity.map(|g| g * (0.1 + 0.9 * ramp)), air_damping: damping, ..params });
                let up = (u / 0.35).clamp(0.0, 1.0);
                let down = ((u - 0.7) / 0.3).clamp(0.0, 1.0);
                s.scale_links(f64::from(1.0 + 24.0 * up * (1.0 - down)));
            } else if close > 0 && f == close {
                s.set_params(params);
                s.scale_links(1.0);
            }
            let stats = s.step();
            if f % every == every - 1 {
                let st = measure(&cloth, s.positions(), n, n);
                println!("  f{:3} weft mean {:.4} min {:.4}  warp mean {:.4} min {:.4}  area {:.4}  pen {:.3}  its {}  ms {:.1}", f + 1, st.mean_u, st.min_u, st.mean_v, st.min_v, st.area, s.max_penetration(), stats.iterations, stats.ms);
            }
        }
    }
}
