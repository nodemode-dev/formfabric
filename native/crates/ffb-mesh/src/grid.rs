use ffb_pattern::geom::{closest_on_segment, dist, V2};

pub struct SegmentGrid {
    origin: V2,
    cell: f64,
    nx: usize,
    ny: usize,
    reach: f64,
    cells: Vec<Vec<u32>>,
    segments: Vec<(V2, V2)>,
}

impl SegmentGrid {
    pub fn new(min: V2, max: V2, cell: f64, reach: f64) -> Self {
        let nx = (((max[0] - min[0]) / cell).ceil() as usize + 3).max(1);
        let ny = (((max[1] - min[1]) / cell).ceil() as usize + 3).max(1);
        SegmentGrid { origin: [min[0] - cell, min[1] - cell], cell, nx, ny, reach, cells: vec![Vec::new(); nx * ny], segments: Vec::new() }
    }

    fn index(&self, v: f64, axis: usize) -> usize {
        let n = if axis == 0 { self.nx } else { self.ny };
        (((v - self.origin[axis]) / self.cell).floor().max(0.0) as usize).min(n - 1)
    }

    pub fn insert(&mut self, a: V2, b: V2) {
        let id = self.segments.len() as u32;
        self.segments.push((a, b));
        let x0 = self.index(a[0].min(b[0]) - self.reach, 0);
        let x1 = self.index(a[0].max(b[0]) + self.reach, 0);
        let y0 = self.index(a[1].min(b[1]) - self.reach, 1);
        let y1 = self.index(a[1].max(b[1]) + self.reach, 1);
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.cells[y * self.nx + x].push(id);
            }
        }
    }

    pub fn within_reach(&self, p: V2) -> bool {
        let cell = self.index(p[1], 1) * self.nx + self.index(p[0], 0);
        self.cells[cell].iter().any(|id| {
            let (a, b) = self.segments[*id as usize];
            let (_, q) = closest_on_segment(p, a, b);
            dist(p, q) < self.reach
        })
    }
}
