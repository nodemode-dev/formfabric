use ffb_fabric::Physical;
use ffb_sim::{compare_positions, reference_step, Cloth, Collider, Material, Method, Params};

pub(crate) use ffb_sim::{Solver as Stepper, StepStats as Stats};

pub(crate) fn material(p: &Physical) -> Material {
    Material::from_physical(p)
}

pub(crate) fn body_colliders(bodies: &[ffb_body::Body]) -> Vec<Collider> {
    bodies.iter().filter(|b| b.visible && !b.triangles.is_empty()).map(ffb_body::collider).collect()
}

pub(crate) fn mesh_collider(positions: Vec<[f32; 3]>, triangles: &[[u32; 3]]) -> Collider {
    Collider::Mesh { positions, triangles: triangles.to_vec() }
}

pub(crate) fn stiffened(m: &Material, factor: f32) -> Material {
    let k = if factor.is_finite() && factor > 0.0 { factor } else { 1.0 };
    Material {
        stretch_weft: m.stretch_weft * k,
        stretch_warp: m.stretch_warp * k,
        shear: m.shear * k,
        bending_weft: m.bending_weft * k,
        bending_warp: m.bending_warp * k,
        bending_bias: m.bending_bias * k,
        ..*m
    }
}

pub(crate) fn set_external(stepper: &mut Stepper, forces: &[[f32; 3]]) {
    stepper.set_external_forces(forces);
}

pub(crate) fn method_name(p: &Params) -> &'static str {
    match p.method {
        Method::Vbd => "vbd",
        Method::Implicit => "implicit",
    }
}

pub(crate) fn set_method(p: &mut Params, name: &str) -> bool {
    match name {
        "vbd" => p.method = Method::Vbd,
        "implicit" => p.method = Method::Implicit,
        _ => return false,
    }
    true
}

pub(crate) fn gpu_available() -> bool {
    false
}

pub(crate) fn preset(name: &str, base: Params) -> Option<Params> {
    let made = Params::preset(name, base.method)?;
    Some(Params {
        time_step: base.time_step,
        gravity: base.gravity,
        air_damping: base.air_damping,
        skin_offset: base.skin_offset,
        friction: base.friction,
        ..made
    })
}

pub(crate) fn trim_bindings(garment: &ffb_pattern::Garment, meshes: &std::collections::BTreeMap<u32, ffb_mesh::PanelMesh>) -> Vec<ffb_pattern::sew::Binding> {
    use ffb_pattern::sew::{Binding, VertexRef};
    use ffb_pattern::trim::Barycentric;
    let found = crate::trim_cmds::trim_bindings(garment, meshes);
    let mut out = found.zippers;
    let heaviest = |b: &Barycentric| {
        let k = (0..3).max_by(|x, y| b.weights[*x].total_cmp(&b.weights[*y])).unwrap_or(0);
        VertexRef { panel: b.panel, vertex: b.vertices[k] }
    };
    for f in &found.fastenings {
        out.push(Binding::Pair { seam: 0, a: heaviest(&f.a), b: heaviest(&f.b), rest_length: f.rest_length, stiffness: 0.0 });
    }
    out
}

pub(crate) fn stiffen_bands(garment: &ffb_pattern::Garment, meshes: &std::collections::BTreeMap<u32, ffb_mesh::PanelMesh>) -> Vec<(u32, Vec<f32>)> {
    crate::trim_cmds::stiffen_bands(garment, meshes).into_iter().collect()
}

pub(crate) fn floor() -> Collider {
    Collider::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] }
}

pub(crate) struct Compared {
    pub mean: f32,
    pub max: f32,
    pub main_ms: f32,
    pub reference_ms: f32,
    pub frames: u32,
}

pub(crate) fn compare(cloth: &Cloth, colliders: &[Collider], params: Params, frames: u32, over: &dyn Fn() -> bool) -> Compared {
    let mut main = Stepper::new(cloth.clone(), params);
    main.set_colliders(colliders.to_vec());
    let mut reference = Stepper::new(cloth.clone(), params);
    reference.set_colliders(colliders.to_vec());
    let (mut main_ms, mut reference_ms) = (0.0, 0.0);
    let mut done = 0;
    for _ in 0..frames {
        if over() {
            break;
        }
        main_ms += main.step().ms;
        reference_ms += reference_step(&mut reference).ms;
        done += 1;
    }
    let (mean, max) = compare_positions(&main, &reference);
    Compared { mean, max, main_ms, reference_ms, frames: done }
}
