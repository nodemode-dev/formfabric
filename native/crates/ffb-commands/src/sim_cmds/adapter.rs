use ffb_fabric::Physical;
use ffb_sim::{compare_positions, reference_step, Cloth, Collider, Material, Params};

pub(crate) use ffb_sim::{Solver as Stepper, StepStats as Stats};

pub(crate) fn material(p: &Physical) -> Material {
    Material::from_physical(p)
}

pub(crate) fn body_colliders(bodies: &[ffb_body::Body]) -> Vec<Collider> {
    bodies.iter().filter(|b| !b.triangles.is_empty()).map(ffb_body::collider).collect()
}

pub(crate) fn floor() -> Collider {
    Collider::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] }
}

pub(crate) struct Compared {
    pub mean: f32,
    pub max: f32,
    pub main_ms: f32,
    pub reference_ms: f32,
}

pub(crate) fn compare(cloth: &Cloth, colliders: &[Collider], params: Params, frames: u32) -> Compared {
    let mut main = Stepper::new(cloth.clone(), params);
    main.set_colliders(colliders.to_vec());
    let mut reference = Stepper::new(cloth.clone(), params);
    reference.set_colliders(colliders.to_vec());
    let (mut main_ms, mut reference_ms) = (0.0, 0.0);
    for _ in 0..frames {
        main_ms += main.step().ms;
        reference_ms += reference_step(&mut reference).ms;
    }
    let (mean, max) = compare_positions(&main, &reference);
    Compared { mean, max, main_ms, reference_ms }
}
