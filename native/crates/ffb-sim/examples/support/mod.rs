#![allow(dead_code)]

use ffb_sim::{Cloth, Method, Params};

pub fn arg(name: &str, default: f32) -> f32 {
    let key = format!("--{name}=");
    std::env::args().find_map(|a| a.strip_prefix(&key).and_then(|v| v.parse().ok())).unwrap_or(default)
}

pub fn arg_str(name: &str, default: &str) -> String {
    let key = format!("--{name}=");
    std::env::args().find_map(|a| a.strip_prefix(&key).map(str::to_string)).unwrap_or_else(|| default.to_string())
}

pub fn methods() -> Vec<Method> {
    match arg_str("method", "both").as_str() {
        "vbd" => vec![Method::Vbd],
        "implicit" => vec![Method::Implicit],
        _ => vec![Method::Vbd, Method::Implicit],
    }
}

pub fn name(m: Method) -> &'static str {
    match m {
        Method::Vbd => "vbd",
        Method::Implicit => "implicit",
    }
}

pub fn params_for(m: Method) -> Params {
    let base = Params::preset(&arg_str("preset", "normal"), m).unwrap_or(Params { method: m, ..Params::default() });
    Params {
        substeps: arg("substeps", arg("isubsteps", base.substeps as f32)) as u32,
        iterations: arg("iterations", base.iterations as f32) as u32,
        newton: arg("newton", base.newton as f32) as u32,
        cg_iterations: arg("cg", base.cg_iterations as f32) as u32,
        cg_tolerance: arg("tol", base.cg_tolerance),
        ..base
    }
}

pub struct Grid {
    pub nx: usize,
    pub first: u32,
}

impl Grid {
    pub fn index(&self, i: usize, j: usize) -> u32 {
        self.first + (j * self.nx + i) as u32
    }
}

pub fn add_grid(cloth: &mut Cloth, nx: usize, ny: usize, spacing: f32, panel: u32, place: &dyn Fn(f32, f32) -> [f32; 3]) -> Grid {
    let first = cloth.positions.len() as u32;
    for j in 0..ny {
        for i in 0..nx {
            let (s, t) = (i as f32 * spacing, j as f32 * spacing);
            cloth.positions.push(place(s, t));
            cloth.rest.push([s, t]);
            cloth.vertex_panel.push(panel);
        }
    }
    let grid = Grid { nx, first };
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            let (a, b, c, d) = (grid.index(i, j), grid.index(i + 1, j), grid.index(i + 1, j + 1), grid.index(i, j + 1));
            if (i + j) % 2 == 0 {
                cloth.triangles.push([a, b, c]);
                cloth.triangles.push([a, c, d]);
            } else {
                cloth.triangles.push([a, b, d]);
                cloth.triangles.push([b, c, d]);
            }
            cloth.triangle_material.push(0);
            cloth.triangle_material.push(0);
            cloth.triangle_grain.push(0.0);
            cloth.triangle_grain.push(0.0);
        }
    }
    grid
}
