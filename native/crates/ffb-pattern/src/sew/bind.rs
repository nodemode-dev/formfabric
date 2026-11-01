use std::collections::{BTreeMap, HashMap, HashSet};

use crate::geom::{dist, lerp, V2};
use crate::{path, Garment, Panel, Seam, SeamEnd};

const SNAP: f64 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexRef {
    pub panel: u32,
    pub vertex: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Pair { seam: u32, a: VertexRef, b: VertexRef, rest_length: f64, stiffness: f64 },
    Edge { seam: u32, a: VertexRef, b0: VertexRef, b1: VertexRef, w: f64, rest_length: f64, stiffness: f64 },
}

#[derive(Debug, Clone, Copy)]
pub struct MeshView<'a> {
    pub positions: &'a [[f64; 2]],
    pub boundary: &'a [u32],
    pub boundary_param: &'a [(usize, f64)],
    pub inner_edges: &'a [(u32, Vec<u32>)],
}

struct Source {
    panel: u32,
    verts: Vec<u32>,
    u: Vec<f64>,
    pos: Vec<V2>,
}

impl Source {
    fn s_of(&self, u: f64) -> f64 {
        let n = self.u.len();
        let i = self.u.partition_point(|x| *x <= u).saturating_sub(1).min(n - 2);
        let span = self.u[i + 1] - self.u[i];
        let f = if span > 1e-12 { ((u - self.u[i]) / span).clamp(0.0, 1.0) } else { 0.0 };
        if f < SNAP {
            i as f64
        } else if f > 1.0 - SNAP {
            (i + 1) as f64
        } else {
            i as f64 + f
        }
    }

    fn pos_at(&self, s: f64) -> V2 {
        let i = (s.floor() as usize).min(self.pos.len() - 1);
        let w = s - i as f64;
        if w < SNAP || i + 1 >= self.pos.len() {
            self.pos[i]
        } else {
            lerp(self.pos[i], self.pos[i + 1], w)
        }
    }
}

fn ring_source(panel: &Panel, mesh: &MeshView) -> Option<Source> {
    let nseg = path::seg_count(panel.outline.len(), true);
    if mesh.boundary.len() < 3 || mesh.boundary_param.len() != mesh.boundary.len() || nseg == 0 {
        return None;
    }
    let mut verts = mesh.boundary.to_vec();
    verts.push(mesh.boundary[0]);
    let mut u: Vec<f64> = mesh.boundary_param.iter().map(|(s, t)| *s as f64 + t).collect();
    u.push(nseg as f64);
    let pos = verts.iter().map(|v| mesh.positions.get(*v as usize).copied()).collect::<Option<Vec<_>>>()?;
    Some(Source { panel: panel.id, verts, u, pos })
}

fn inner_source(panel: &Panel, inner: u32, mesh: &MeshView) -> Option<Source> {
    let shape = panel.inner.iter().find(|i| i.id == inner)?;
    let run = mesh.inner_edges.iter().filter(|(id, _)| *id == inner).map(|(_, v)| v).max_by_key(|v| v.len())?;
    if run.len() < 2 {
        return None;
    }
    let pos = run.iter().map(|v| mesh.positions.get(*v as usize).copied()).collect::<Option<Vec<_>>>()?;
    let mut top = 0.0f64;
    let u = pos
        .iter()
        .map(|p| {
            let near = path::nearest(&shape.points, shape.closed, *p);
            top = top.max(near.map_or(0.0, |n| n.seg as f64 + n.t));
            top
        })
        .collect();
    Some(Source { panel: panel.id, verts: run.clone(), u, pos })
}

#[derive(Clone, Copy)]
struct Pt {
    src: usize,
    s: f64,
    pos: V2,
}

struct Chain {
    pts: Vec<Pt>,
    cum: Vec<f64>,
}

enum Loc {
    Vertex(VertexRef),
    Edge(VertexRef, VertexRef, f64),
}

struct Ctx<'a> {
    garment: &'a Garment,
    meshes: &'a BTreeMap<u32, MeshView<'a>>,
    sources: Vec<Source>,
    index: HashMap<(u32, Option<u32>), Option<usize>>,
}

impl Ctx<'_> {
    fn source(&mut self, end: &SeamEnd) -> Option<usize> {
        let key = (end.panel, end.inner);
        if let Some(found) = self.index.get(&key) {
            return *found;
        }
        let built = self
            .garment
            .panels
            .iter()
            .find(|p| p.id == end.panel)
            .zip(self.meshes.get(&end.panel))
            .and_then(|(panel, mesh)| match end.inner {
                None => ring_source(panel, mesh),
                Some(id) => inner_source(panel, id, mesh),
            });
        let slot = built.map(|s| {
            self.sources.push(s);
            self.sources.len() - 1
        });
        self.index.insert(key, slot);
        slot
    }

    fn end_points(&mut self, end: &SeamEnd) -> Vec<Pt> {
        let Some(src) = self.source(end) else { return Vec::new() };
        let source = &self.sources[src];
        let (u0, u1) = (end.segment as f64 + end.t0.clamp(0.0, 1.0), end.segment as f64 + end.t1.clamp(0.0, 1.0));
        let reversed = u0 > u1;
        let (lo, hi) = if reversed { (u1, u0) } else { (u0, u1) };
        let (s_lo, s_hi) = (source.s_of(lo), source.s_of(hi));
        let mut ss = vec![s_lo];
        if s_hi - s_lo > SNAP {
            let first = (s_lo + SNAP).floor() as usize + 1;
            let last = (s_hi - SNAP).ceil() as usize;
            ss.extend((first..last).map(|j| j as f64));
            ss.push(s_hi);
        }
        let mut pts: Vec<Pt> = ss.into_iter().map(|s| Pt { src, s, pos: source.pos_at(s) }).collect();
        if reversed {
            pts.reverse();
        }
        pts
    }

    fn chain(&mut self, ends: &[SeamEnd], reverse: bool) -> Chain {
        let mut pts: Vec<Pt> = Vec::new();
        for end in ends {
            for p in self.end_points(end) {
                if pts.last().is_some_and(|q| q.src == p.src && (q.s - p.s).abs() < SNAP) {
                    continue;
                }
                pts.push(p);
            }
        }
        if reverse {
            pts.reverse();
        }
        let mut cum = Vec::with_capacity(pts.len());
        let mut total = 0.0;
        for (k, p) in pts.iter().enumerate() {
            if k > 0 && pts[k - 1].src == p.src && (pts[k - 1].s - p.s).abs() <= 1.0 + SNAP {
                total += dist(pts[k - 1].pos, p.pos);
            }
            cum.push(total);
        }
        Chain { pts, cum }
    }

    fn locate(&self, chain: &Chain, x: f64) -> Option<Loc> {
        let n = chain.pts.len();
        if n == 0 {
            return None;
        }
        if n == 1 {
            return Some(self.at(chain.pts[0].src, chain.pts[0].s));
        }
        let k = chain.cum.partition_point(|c| *c <= x).saturating_sub(1).min(n - 2);
        let (p, q) = (chain.pts[k], chain.pts[k + 1]);
        let span = chain.cum[k + 1] - chain.cum[k];
        let f = if span > 1e-12 { ((x - chain.cum[k]) / span).clamp(0.0, 1.0) } else { 0.0 };
        if p.src == q.src && (p.s - q.s).abs() <= 1.0 + SNAP {
            Some(self.at(p.src, p.s + (q.s - p.s) * f))
        } else if f < 0.5 {
            Some(self.at(p.src, p.s))
        } else {
            Some(self.at(q.src, q.s))
        }
    }

    fn at(&self, src: usize, s: f64) -> Loc {
        let source = &self.sources[src];
        let last = source.verts.len() - 1;
        let i = (s.floor() as usize).min(last);
        let w = s - i as f64;
        let vertex = |k: usize| VertexRef { panel: source.panel, vertex: source.verts[k.min(last)] };
        if w < SNAP || i >= last {
            Loc::Vertex(vertex(i))
        } else if w > 1.0 - SNAP {
            Loc::Vertex(vertex(i + 1))
        } else {
            Loc::Edge(vertex(i), vertex(i + 1), w)
        }
    }
}

fn is_vertex(p: &Pt) -> bool {
    (p.s - p.s.round()).abs() < SNAP
}

fn bind_seam(ctx: &mut Ctx, seam: &Seam, out: &mut Vec<Binding>) {
    let a = ctx.chain(&seam.a, false);
    let b = ctx.chain(&seam.b, seam.reversed);
    let (ta, tb) = (a.cum.last().copied().unwrap_or(0.0), b.cum.last().copied().unwrap_or(0.0));
    if a.pts.is_empty() || b.pts.is_empty() || ta < 1e-9 || tb < 1e-9 {
        return;
    }
    let mut seen: HashSet<(VertexRef, VertexRef)> = HashSet::new();
    let mut emit = |binding: Option<Binding>, out: &mut Vec<Binding>| {
        let Some(binding) = binding else { return };
        if let Binding::Pair { a, b, .. } = &binding {
            if a == b || !seen.insert((*a.min(b), *a.max(b))) {
                return;
            }
        }
        out.push(binding);
    };
    let make = |from: VertexRef, loc: Loc, flip: bool| -> Option<Binding> {
        let (rest_length, stiffness) = (0.0, seam.tension);
        match loc {
            Loc::Vertex(v) if v == from => None,
            Loc::Vertex(v) => Some(if flip {
                Binding::Pair { seam: seam.id, a: v, b: from, rest_length, stiffness }
            } else {
                Binding::Pair { seam: seam.id, a: from, b: v, rest_length, stiffness }
            }),
            Loc::Edge(b0, b1, w) => Some(Binding::Edge { seam: seam.id, a: from, b0, b1, w, rest_length, stiffness }),
        }
    };
    for (k, p) in a.pts.iter().enumerate().filter(|(_, p)| is_vertex(p)) {
        let Loc::Vertex(from) = ctx.at(p.src, p.s) else { continue };
        let x = a.cum[k] / ta * tb;
        if let Some(loc) = ctx.locate(&b, x) {
            emit(make(from, loc, false), out);
        }
    }
    for (k, p) in b.pts.iter().enumerate().filter(|(_, p)| is_vertex(p)) {
        let Loc::Vertex(from) = ctx.at(p.src, p.s) else { continue };
        let x = b.cum[k] / tb * ta;
        if let Some(loc) = ctx.locate(&a, x) {
            emit(make(from, loc, true), out);
        }
    }
}

pub fn seam_bindings(garment: &Garment, meshes: &BTreeMap<u32, MeshView>) -> Vec<Binding> {
    let mut ctx = Ctx { garment, meshes, sources: Vec::new(), index: HashMap::new() };
    let mut out = Vec::new();
    for seam in &garment.seams {
        bind_seam(&mut ctx, seam, &mut out);
    }
    out
}
