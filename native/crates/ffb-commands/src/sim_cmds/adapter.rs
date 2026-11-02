use ffb_fabric::Physical;
use ffb_sim::{compare_positions, reference_step, Cloth, Collider, Material, Method, Params};

#[cfg(feature = "gpu")]
pub(crate) use ffb_gpu::AnyStepper as Stepper;
#[cfg(not(feature = "gpu"))]
pub(crate) use ffb_sim::Solver as Stepper;
pub(crate) use ffb_sim::StepStats as Stats;

#[cfg(feature = "gpu")]
pub(crate) fn new_stepper(cloth: Cloth, params: Params, want_gpu: bool) -> (Stepper, String) {
    Stepper::new(cloth, params, want_gpu)
}

#[cfg(not(feature = "gpu"))]
pub(crate) fn new_stepper(cloth: Cloth, params: Params, want_gpu: bool) -> (Stepper, String) {
    let note = if want_gpu { "gpu is not built into this engine; running on the cpu".to_string() } else { String::new() };
    (Stepper::new(cloth, params), note)
}

#[cfg(feature = "gpu")]
pub(crate) fn accepts(stepper: &Stepper, params: &Params) -> bool {
    stepper.accepts(params)
}

#[cfg(not(feature = "gpu"))]
pub(crate) fn accepts(_stepper: &Stepper, _params: &Params) -> bool {
    true
}

#[cfg(feature = "gpu")]
pub(crate) fn self_hits(stepper: &Stepper) -> u32 {
    stepper.self_intersections().map_or(0, |v| v.len() as u32)
}

#[cfg(not(feature = "gpu"))]
pub(crate) fn self_hits(stepper: &Stepper) -> u32 {
    stepper.self_intersections().len() as u32
}

#[cfg(feature = "gpu")]
pub(crate) fn is_gpu(stepper: &Stepper) -> bool {
    stepper.is_gpu()
}

#[cfg(not(feature = "gpu"))]
pub(crate) fn is_gpu(_stepper: &Stepper) -> bool {
    false
}

pub(crate) fn material(p: &Physical) -> Material {
    Material::from_physical(p)
}

pub(crate) fn body_colliders(garment: &ffb_pattern::Garment, bodies: &[ffb_body::Body]) -> Vec<Collider> {
    bodies.iter().filter(|b| b.visible && !b.triangles.is_empty()).map(|b| crate::edit3d_cmds::fit_shell_collider(garment, b)).collect()
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

#[cfg(feature = "gpu")]
pub(crate) fn gpu_available() -> bool {
    ffb_gpu::available()
}

#[cfg(not(feature = "gpu"))]
pub(crate) fn gpu_available() -> bool {
    false
}

pub(crate) fn preset(name: &str, base: Params) -> Option<Params> {
    let made = Params::preset(name, base.method)?;
    let light = name == "normal";
    Some(Params {
        self_collision: made.self_collision || light,
        collision_thickness: if light { 1.0 } else { made.collision_thickness },
        time_step: base.time_step,
        gravity: base.gravity,
        air_damping: base.air_damping,
        skin_offset: base.skin_offset,
        friction: base.friction,
        close_frames: base.close_frames,
        close_gap: base.close_gap,
        seam_boost: base.seam_boost,
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
    let mut all = crate::trim_cmds::stiffen_bands(garment, meshes);
    for (id, factors) in crate::edit3d_cmds::styleline_stiffen(garment, meshes) {
        match all.get_mut(&id) {
            Some(have) => {
                if have.len() < factors.len() {
                    have.resize(factors.len(), 1.0);
                }
                for (h, f) in have.iter_mut().zip(factors.iter()) {
                    *h = h.max(*f);
                }
            }
            None => {
                all.insert(id, factors);
            }
        }
    }
    all.into_iter().collect()
}

pub(crate) fn style_folds(garment: &ffb_pattern::Garment, meshes: &std::collections::BTreeMap<u32, ffb_mesh::PanelMesh>, starts: &std::collections::BTreeMap<u32, u32>) -> Vec<ffb_sim::Fold> {
    crate::edit3d_cmds::styleline_folds(garment, meshes, starts)
}

pub(crate) fn fit_signature(garment: &ffb_pattern::Garment) -> u64 {
    crate::edit3d_cmds::fit_and_style_signature(garment)
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
    let mut main = ffb_sim::Solver::new(cloth.clone(), params);
    main.set_colliders(colliders.to_vec());
    let mut reference = ffb_sim::Solver::new(cloth.clone(), params);
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
