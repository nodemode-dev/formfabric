mod support;

use ffb_fabric::Physical;
use ffb_sim::{Cloth, Collider, Material, Params, Solver};
use support::{arg, arg_str};

fn two_sheets(n: usize, spacing: f32, low_y: f32, high: &dyn Fn(f32, f32) -> f32, layers: (i32, i32)) -> (Cloth, usize) {
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    let half = spacing * (n - 1) as f32 * 0.5;
    support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [t - half, low_y, s - half]);
    let count = cloth.positions.len();
    support::add_grid(&mut cloth, n, n, spacing, 1, &|s, t| [t - half, high(t - half, s - half), s - half]);
    cloth.vertex_layer = (0..cloth.positions.len()).map(|i| if i < count { layers.0 } else { layers.1 }).collect();
    (cloth, count)
}

fn params(method: ffb_sim::Method) -> Params {
    let mut p = support::params_for(method);
    p.self_collision = true;
    p.collision_thickness = arg("thick", 1.0);
    p
}

fn stack(method: ffb_sim::Method) {
    let (cloth, count) = two_sheets(26, 20.0, 30.0, &|_, _| 60.0, (0, 1));
    let mut s = Solver::new(cloth, params(method));
    s.set_colliders(vec![Collider::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] }]);
    let frames = arg("frames", 150.0) as usize;
    let mut total = 0.0;
    for _ in 0..frames {
        total += s.step().ms;
    }
    let p = s.positions();
    let gaps: Vec<f32> = (0..count).map(|i| p[count + i][1] - p[i][1]).collect();
    let min = gaps.iter().copied().fold(f32::MAX, f32::min);
    let mean = gaps.iter().sum::<f32>() / gaps.len() as f32;
    let crossed = gaps.iter().filter(|g| **g < 0.0).count();
    println!("stack {:?}: {frames} frames, {:.1} ms/frame, upper minus lower: min {min:.3} mm mean {mean:.3} mm, crossed {crossed}, intersecting pairs {}, KE {:.2}", method, total / frames as f32, s.self_intersections().len(), s.kinetic_energy());
}

fn untangle(method: ffb_sim::Method) {
    let dome = |x: f32, z: f32| 30.0 + 6.0 * (-(x * x + z * z) / (2.0 * 70.0 * 70.0)).exp();
    let (cloth, _) = two_sheets(26, 20.0, 33.0, &dome, (0, 0));
    let mut s = Solver::new(cloth, params(method));
    let before = s.self_intersections().len();
    let left = s.untangle(arg("steps", 20.0) as u32);
    println!("untangle {:?}: intersecting pairs {before} -> {} after {} steps", method, s.self_intersections().len().min(left.max(s.self_intersections().len())), arg("steps", 20.0));
}

fn main() {
    let what = arg_str("scene", "both");
    for m in support::methods() {
        if what != "untangle" {
            stack(m);
        }
        if what != "stack" {
            untangle(m);
        }
    }
}
