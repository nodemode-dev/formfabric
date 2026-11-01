use std::collections::{BTreeMap, HashMap, HashSet};

use ffb_pattern::geom::{dist, V2};
use ffb_pattern::{path, Inner, Panel};
use spade::handles::{FixedFaceHandle, FixedVertexHandle, InnerTag};
use spade::{AngleLimit, ConstrainedDelaunayTriangulation, Point2, RefinementParameters, Triangulation};

use crate::grid::SegmentGrid;
use crate::hash::meshed;
use crate::resample::{arc_length, flatten_tolerance, natural_count, resample};
use crate::PanelMesh;

const MAX_ESTIMATED_VERTICES: f64 = 3_000_000.0;
const MAX_SEGMENT_COUNT: usize = 100_000;
const EQUILATERAL: f64 = 0.433_012_701_892_219_3;
const AREA_FACTOR: f64 = 1.6;

#[derive(Debug, Clone, PartialEq)]
pub struct MeshOptions {
    pub segment_counts: BTreeMap<usize, usize>,
    pub min_angle: f64,
    pub jitter: f64,
    pub inset: f64,
}

impl Default for MeshOptions {
    fn default() -> Self {
        MeshOptions { segment_counts: BTreeMap::new(), min_angle: 20.0, jitter: 0.0, inset: 0.65 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshError(pub String);

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MeshError {}

fn fail<T>(message: impl Into<String>) -> Result<T, MeshError> {
    Err(MeshError(message.into()))
}

#[derive(Clone, Copy)]
enum Node {
    Boundary(usize, u64),
    Free(V2),
    Dropped,
}

struct Chain {
    id: u32,
    closed: bool,
    nodes: Vec<Node>,
}

fn inside(poly: &[V2], q: V2) -> bool {
    let mut odd = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a[1] > q[1]) != (b[1] > q[1]) && q[0] < a[0] + (q[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            odd = !odd;
        }
    }
    odd
}

fn chain_points(inner: &Inner, d: f64, tol: f64) -> Vec<V2> {
    let segments = path::segments(&inner.points, inner.closed);
    let mut pts = Vec::new();
    for s in &segments {
        let n = natural_count(arc_length(s, tol), d);
        pts.extend(resample(s, n, tol).into_iter().map(|x| x.1));
    }
    if !inner.closed {
        if let Some(last) = segments.last() {
            pts.push(last.p[3]);
        }
    }
    pts
}

fn classify(panel: &Panel, samples: &mut [Vec<(f64, V2)>], forced: &BTreeMap<usize, usize>, poly: &[V2], q: V2, d: f64) -> Node {
    let snap = 0.05_f64.max(0.01 * d);
    let Some(near) = path::nearest(&panel.outline, true, q) else { return Node::Dropped };
    if near.dist > snap {
        return if inside(poly, q) { Node::Free(q) } else { Node::Dropped };
    }
    let (mut seg, mut t) = (near.seg, near.t);
    if t > 1.0 - 1e-6 {
        seg = (seg + 1) % samples.len();
        t = 0.0;
    }
    let list = &mut samples[seg];
    if list.is_empty() {
        return Node::Dropped;
    }
    let (best, gap) = list.iter().enumerate().map(|(i, s)| (i, dist(s.1, q))).min_by(|a, b| a.1.total_cmp(&b.1)).unwrap_or((0, f64::MAX));
    if forced.contains_key(&seg) || gap < 0.35 * d {
        return Node::Boundary(seg, list[best].0.to_bits());
    }
    let at = list.partition_point(|s| s.0 < t);
    list.insert(at, (t, near.point));
    Node::Boundary(seg, t.to_bits())
}

pub fn build_panel(panel: &Panel, options: &MeshOptions) -> Result<PanelMesh, MeshError> {
    let d = panel.particle_distance;
    if !d.is_finite() || d <= 0.0 {
        return fail("particle distance must be a positive number");
    }
    if panel.outline.len() < 3 {
        return fail("the outline needs at least 3 points");
    }
    let tol = flatten_tolerance(d);
    let segments = path::segments(&panel.outline, true);
    let Some(bb) = path::bounds(&panel.outline, true) else { return fail("the outline has no extent") };
    let extent = [bb[1][0] - bb[0][0], bb[1][1] - bb[0][1]];
    if !(extent[0].is_finite() && extent[1].is_finite()) || extent[0] * extent[1] / (EQUILATERAL * d * d) > MAX_ESTIMATED_VERTICES * 2.0 {
        return fail("the particle distance is too small for this panel");
    }

    let mut samples: Vec<Vec<(f64, V2)>> = segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let length = arc_length(s, tol);
            if length < 1e-6 {
                return Vec::new();
            }
            let n = options.segment_counts.get(&i).copied().unwrap_or_else(|| natural_count(length, d));
            resample(s, n.clamp(1, MAX_SEGMENT_COUNT), tol)
        })
        .collect();

    let poly0: Vec<V2> = samples.iter().flatten().map(|s| s.1).collect();
    if poly0.len() < 3 {
        return fail("the outline resamples to fewer than 3 vertices");
    }

    let mut chains: Vec<Chain> = Vec::new();
    for inner in panel.inner.iter().filter(|i| meshed(i.role)) {
        let pts = chain_points(inner, d, tol);
        let min = if inner.closed { 3 } else { 2 };
        if pts.len() < min {
            continue;
        }
        let nodes = pts.into_iter().map(|q| classify(panel, &mut samples, &options.segment_counts, &poly0, q, d)).collect();
        chains.push(Chain { id: inner.id, closed: inner.closed, nodes });
    }

    let mut verts: Vec<V2> = Vec::new();
    let mut boundary_param: Vec<(usize, f64)> = Vec::new();
    let mut boundary_index: HashMap<(usize, u64), u32> = HashMap::new();
    let mut seen: HashMap<(i64, i64), u32> = HashMap::new();
    let quant = |p: V2| ((p[0] * 1e4).round() as i64, (p[1] * 1e4).round() as i64);
    for (i, list) in samples.iter().enumerate() {
        for (t, p) in list {
            if seen.contains_key(&quant(*p)) {
                return fail("the outline has coincident vertices");
            }
            let id = verts.len() as u32;
            seen.insert(quant(*p), id);
            boundary_index.insert((i, t.to_bits()), id);
            boundary_param.push((i, *t));
            verts.push(*p);
        }
    }
    let nb = verts.len();

    let mut runs: Vec<(u32, Vec<u32>)> = Vec::new();
    let mut edge_set: HashSet<(u32, u32)> = HashSet::new();
    let loop_pairs: HashSet<(u32, u32)> = (0..nb).map(|i| order(i as u32, ((i + 1) % nb) as u32)).collect();
    for chain in &chains {
        let mut ids: Vec<Option<u32>> = Vec::with_capacity(chain.nodes.len());
        for node in &chain.nodes {
            ids.push(match node {
                Node::Dropped => None,
                Node::Boundary(seg, bits) => boundary_index.get(&(*seg, *bits)).copied(),
                Node::Free(p) => Some(*seen.entry(quant(*p)).or_insert_with(|| {
                    verts.push(*p);
                    verts.len() as u32 - 1
                })),
            });
        }
        let whole = chain.closed && ids.iter().all(Option::is_some);
        let mut run: Vec<u32> = Vec::new();
        let mut flush = |run: &mut Vec<u32>| {
            if run.len() >= 2 {
                runs.push((chain.id, std::mem::take(run)));
            }
            run.clear();
        };
        for id in &ids {
            match id {
                Some(v) if run.last() != Some(v) => run.push(*v),
                Some(_) => {}
                None => flush(&mut run),
            }
        }
        if whole {
            if let Some(first) = run.first().copied() {
                run.push(first);
            }
        }
        flush(&mut run);
    }
    for (_, run) in &runs {
        for w in run.windows(2) {
            let pair = order(w[0], w[1]);
            if w[0] != w[1] && !loop_pairs.contains(&pair) {
                edge_set.insert(pair);
            }
        }
    }

    let mut guard = SegmentGrid::new(bb[0], bb[1], d, options.inset * d);
    for i in 0..nb {
        guard.insert(verts[i], verts[(i + 1) % nb]);
    }
    for (a, b) in &edge_set {
        guard.insert(verts[*a as usize], verts[*b as usize]);
    }

    let lattice = lattice_points(&verts[..nb], bb, d, options.jitter, &guard);

    let free_end = verts.len();
    let centre = [0.5 * (bb[0][0] + bb[1][0]), 0.5 * (bb[0][1] + bb[1][1])];
    let reach = 3.0 * extent[0].max(extent[1]).max(d) + 10.0 * d;
    let mut points: Vec<Point2<f64>> = verts.iter().map(|p| Point2::new(p[0], p[1])).collect();
    for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        points.push(Point2::new(centre[0] + sx * reach, centre[1] + sy * reach));
    }
    points.extend(lattice.iter().map(|p| Point2::new(p[0], p[1])));

    let mut edges: Vec<[usize; 2]> = (0..nb).map(|i| [i, (i + 1) % nb]).collect();
    edges.extend(edge_set.iter().map(|(a, b)| [*a as usize, *b as usize]));

    let mut conflicts: Vec<[usize; 2]> = Vec::new();
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::try_bulk_load_cdt(points, edges, |e| conflicts.push(e))
        .map_err(|e| MeshError(format!("triangulation failed: {e}")))?;
    if conflicts.iter().any(|c| c[0] < nb && c[1] < nb && loop_pairs.contains(&order(c[0] as u32, c[1] as u32))) {
        return fail("the outline crosses itself");
    }
    if cdt.num_vertices() != free_end + 4 + lattice.len() {
        return fail("the triangulation merged coincident vertices");
    }

    let max_area = EQUILATERAL * d * d * AREA_FACTOR;
    let limit = (lattice.len() + free_end) * 2 + 1000;
    let parameters = RefinementParameters::<f64>::new()
        .with_angle_limit(AngleLimit::from_deg(options.min_angle.clamp(0.0, 33.0)))
        .with_max_allowed_area(max_area)
        .with_max_additional_vertices(limit)
        .keep_constraint_edges()
        .exclude_outer_faces(true);
    cdt.refine(parameters);

    let mut outside = vec![false; cdt.num_all_faces()];
    let mut stack: Vec<FixedFaceHandle<InnerTag>> = Vec::new();
    for edge in cdt.convex_hull() {
        for side in [edge.face(), edge.rev().face()] {
            if let Some(f) = side.as_inner() {
                if !outside[f.fix().index()] {
                    outside[f.fix().index()] = true;
                    stack.push(f.fix());
                }
            }
        }
    }
    while let Some(f) = stack.pop() {
        for edge in cdt.face(f).adjacent_edges() {
            let pair = order(edge.from().fix().index() as u32, edge.to().fix().index() as u32);
            if loop_pairs.contains(&pair) {
                continue;
            }
            if let Some(n) = edge.rev().face().as_inner() {
                if !outside[n.fix().index()] {
                    outside[n.fix().index()] = true;
                    stack.push(n.fix());
                }
            }
        }
    }

    let mut used = vec![false; cdt.num_vertices()];
    let mut raw: Vec<[usize; 3]> = Vec::new();
    for face in cdt.inner_faces() {
        if outside[face.fix().index()] {
            continue;
        }
        let v = face.vertices();
        let tri = [v[0].fix().index(), v[1].fix().index(), v[2].fix().index()];
        for i in tri {
            used[i] = true;
        }
        raw.push(tri);
    }
    let mut remap = vec![u32::MAX; used.len()];
    let mut positions: Vec<V2> = Vec::new();
    for (i, v) in cdt.vertices().enumerate() {
        if used[i] {
            remap[i] = positions.len() as u32;
            let p = v.position();
            positions.push([p.x, p.y]);
        }
    }
    if (0..nb).any(|i| remap[i] != i as u32) {
        return fail("a boundary vertex is not part of the mesh");
    }
    let triangles: Vec<[u32; 3]> = raw.iter().map(|t| [remap[t[0]], remap[t[1]], remap[t[2]]]).collect();

    let mut inner_edges: Vec<(u32, Vec<u32>)> = Vec::new();
    for (id, run) in runs {
        let mut piece: Vec<u32> = Vec::new();
        for (k, v) in run.iter().enumerate() {
            if k > 0 {
                let linked = loop_pairs.contains(&order(run[k - 1], *v))
                    || cdt.exists_constraint(FixedVertexHandle::from_index(run[k - 1] as usize), FixedVertexHandle::from_index(*v as usize));
                if !linked {
                    if piece.len() >= 2 {
                        inner_edges.push((id, piece.iter().map(|i| remap[*i as usize]).collect()));
                    }
                    piece.clear();
                }
            }
            piece.push(*v);
        }
        if piece.len() >= 2 {
            inner_edges.push((id, piece.iter().map(|i| remap[*i as usize]).collect()));
        }
    }

    Ok(PanelMesh { panel: panel.id, positions, triangles, boundary: (0..nb as u32).collect(), boundary_param, inner_edges })
}

fn order(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn hash01(a: i64, b: i64, salt: u64) -> f64 {
    let mut h = crate::hash::Fnv::default();
    h.u64(a as u64);
    h.u64(b as u64);
    h.u64(salt);
    (h.finish() >> 11) as f64 / (1u64 << 53) as f64
}

fn lattice_points(boundary: &[V2], bb: [V2; 2], d: f64, jitter: f64, guard: &SegmentGrid) -> Vec<V2> {
    let row_height = d * 0.866_025_403_784_438_6;
    let mut out = Vec::new();
    let rows = ((bb[1][1] - bb[0][1]) / row_height).ceil() as i64;
    let n = boundary.len();
    let mut xs: Vec<f64> = Vec::new();
    for r in 0..=rows {
        let y = bb[0][1] + row_height * (r as f64 + 0.5);
        if y > bb[1][1] {
            break;
        }
        xs.clear();
        for i in 0..n {
            let (a, b) = (boundary[i], boundary[(i + 1) % n]);
            if (a[1] > y) != (b[1] > y) {
                xs.push(a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]));
            }
        }
        xs.sort_by(f64::total_cmp);
        let shift = if r % 2 == 0 { 0.0 } else { 0.5 * d };
        for pair in xs.chunks_exact(2) {
            let c0 = ((pair[0] - bb[0][0] - shift) / d).ceil() as i64;
            let c1 = ((pair[1] - bb[0][0] - shift) / d).floor() as i64;
            for c in c0..=c1 {
                let mut p = [bb[0][0] + shift + c as f64 * d, y];
                if jitter > 0.0 {
                    p[0] += (hash01(r, c, 1) - 0.5) * jitter * d;
                    p[1] += (hash01(r, c, 2) - 0.5) * jitter * d;
                }
                if !guard.within_reach(p) {
                    out.push(p);
                }
            }
        }
    }
    out
}
