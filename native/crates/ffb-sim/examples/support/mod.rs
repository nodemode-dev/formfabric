use ffb_sim::Cloth;

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
