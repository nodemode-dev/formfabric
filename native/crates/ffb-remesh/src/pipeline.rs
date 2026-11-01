use ffb_pattern::grain;
use ffb_pattern::Panel;

use crate::boundary::work_mesh;
use crate::cleanup::{collapse_diagonals, pure_quads, remove_valence2, singularities, smooth, Poly};
use crate::extract::extract;
use crate::fields::{adjacency, orientation, positions, FieldInput};
use crate::types::{Params, QuadMesh, RemeshError, Report, Rim, V2};

pub fn remesh_panel(panel: &Panel, params: &Params) -> Result<QuadMesh, RemeshError> {
    let work = work_mesh(panel, params)?;
    let mesh = &work.mesh;
    let n = mesh.positions.len();
    let nbr = adjacency(n, &mesh.triangles);
    let grain_angle = grain::effective(panel).map_or(std::f64::consts::FRAC_PI_2, |(line, _)| grain::angle_of(line).to_radians());
    let input = FieldInput { pos: &mesh.positions, nbr: &nbr, rim: &mesh.boundary, guides: &params.guides, grain: grain_angle, grain_weight: params.grain_weight, h: work.h };
    let mut fields = orientation(&input);
    let root = work.loop_ids.first().copied().unwrap_or(0) as usize;
    let lattice_positions = positions(&mesh.positions, &nbr, &mut fields, params.edge, root);
    let loop_pts: Vec<V2> = work.loop_ids.iter().map(|v| mesh.positions[*v as usize]).collect();
    let lattice = extract(&mesh.positions, &nbr, &fields.theta, &lattice_positions, params.edge, &loop_pts);
    let assembled = crate::band::assemble(&loop_pts, &lattice.sites, &lattice.quads);

    let nb = loop_pts.len();
    let mut faces: Vec<Vec<u32>> = assembled.quads.iter().map(|q| q.to_vec()).collect();
    faces.extend(assembled.tris.iter().map(|t| t.to_vec()));
    let mut rim_flags = vec![false; assembled.positions.len()];
    for flag in rim_flags.iter_mut().take(nb) {
        *flag = true;
    }
    let mut poly = Poly { positions: assembled.positions, faces, rim: rim_flags };
    let mut report = Report {
        fine_vertices: n,
        lattice_quads: assembled.lattice_quads,
        band_quads: assembled.band_quads,
        band_tris: assembled.band_tris,
        field_iterations: fields.field_iterations,
        position_iterations: fields.position_iterations,
        ..Report::default()
    };
    report.singularities_found = singularities(&poly);
    if params.cleanup {
        report.valence2_removed = remove_valence2(&mut poly);
        report.diagonal_collapses = collapse_diagonals(&mut poly);
        smooth(&mut poly, 10);
    }
    report.singularities_after = singularities(&poly);

    let mut rim: Vec<Rim> = (0..nb).map(|i| Rim { vertex: i as u32, segment: work.rim[i].0, index: work.rim[i].1, count: work.rim[i].2 }).collect();
    if params.pure_quads {
        let (split, mid) = pure_quads(&poly);
        let mut next: Vec<Rim> = Vec::with_capacity(nb * 2);
        for i in 0..nb {
            let (a, b) = (rim[i].vertex, rim[(i + 1) % nb].vertex);
            next.push(Rim { vertex: a, segment: rim[i].segment, index: rim[i].index * 2, count: rim[i].count * 2 });
            if let Some(m) = mid.get(&(a.min(b), a.max(b))) {
                next.push(Rim { vertex: *m, segment: rim[i].segment, index: rim[i].index * 2 + 1, count: rim[i].count * 2 });
            }
        }
        poly = split;
        rim = next;
    }

    let mut remap = vec![u32::MAX; poly.positions.len()];
    let mut out = QuadMesh { panel: panel.id, edge: params.edge, report, ..QuadMesh::default() };
    let mut take = |v: u32, out: &mut QuadMesh, positions: &[V2]| -> u32 {
        if remap[v as usize] == u32::MAX {
            remap[v as usize] = out.positions.len() as u32;
            out.positions.push(positions[v as usize]);
        }
        remap[v as usize]
    };
    for r in &rim {
        take(r.vertex, &mut out, &poly.positions);
    }
    for f in &poly.faces {
        let ids: Vec<u32> = f.iter().map(|v| take(*v, &mut out, &poly.positions)).collect();
        match ids.len() {
            4 => out.quads.push([ids[0], ids[1], ids[2], ids[3]]),
            3 => out.tris.push([ids[0], ids[1], ids[2]]),
            _ => {}
        }
    }
    out.rim = rim.iter().map(|r| Rim { vertex: remap[r.vertex as usize], ..*r }).collect();
    Ok(out)
}
