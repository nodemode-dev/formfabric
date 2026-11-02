use std::collections::{BTreeMap, BTreeSet};

use ffb_pattern::sew::{self, Binding, MeshView};
use serde_json::{json, Value};

use super::scene::bad;
use crate::{CommandError, QuadPanel, Session};

pub(super) enum Mode {
    Unweld,
    Weld,
    Selected(Vec<u32>),
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Unweld => "unweld",
            Mode::Weld => "weld",
            Mode::Selected(_) => "weld_selected",
        }
    }

    pub fn on(&self) -> bool {
        !matches!(self, Mode::Unweld)
    }
}

pub(super) fn mode(s: &Session, a: &Value) -> Result<Mode, CommandError> {
    match a.get("weld").filter(|v| !v.is_null()).and_then(Value::as_str).unwrap_or("unweld") {
        "unweld" => Ok(Mode::Unweld),
        "weld" => Ok(Mode::Weld),
        "weld_selected" => {
            let list = super::export::panel_filter(s, a)?.unwrap_or_default();
            if list.len() < 2 {
                return Err(bad("weld_selected needs panels with at least two panel ids"));
            }
            Ok(Mode::Selected(list))
        }
        other => Err(bad(format!("weld must be unweld, weld or weld_selected, got {other}"))),
    }
}

pub(super) struct Source<'a> {
    pub panel: u32,
    pub count: usize,
    pub quad: Option<&'a QuadPanel>,
}

pub(super) struct Grouping {
    pub groups: Vec<u32>,
    pub joined: usize,
    pub seams: usize,
    pub open: usize,
}

impl Grouping {
    pub fn report(&self, mode: &Mode, points: usize, vertices: usize, dropped: usize) -> Value {
        json!({"mode": mode.name(), "vertices_before": vertices, "points": points, "merged": vertices.saturating_sub(points), "vertex_joins": self.joined, "seams_welded": self.seams, "seams_not_matched": self.open, "degenerate_faces_dropped": dropped})
    }
}

fn find(parent: &mut [u32], mut x: u32) -> u32 {
    while parent[x as usize] != x {
        let up = parent[parent[x as usize] as usize];
        parent[x as usize] = up;
        x = up;
    }
    x
}

fn join(parent: &mut [u32], a: u32, b: u32) -> bool {
    let (ra, rb) = (find(parent, a), find(parent, b));
    if ra == rb {
        return false;
    }
    parent[ra.max(rb) as usize] = ra.min(rb);
    true
}

fn rim_line(q: &QuadPanel, segment: usize) -> Option<Vec<u32>> {
    let mut mine: Vec<(usize, u32)> = q.boundary.iter().filter(|r| r.1 == segment).map(|r| (r.2, r.0)).collect();
    if mine.is_empty() {
        return None;
    }
    mine.sort_unstable();
    let mut ids: Vec<u32> = mine.iter().map(|x| x.1).collect();
    let present: BTreeSet<usize> = q.boundary.iter().map(|r| r.1).collect();
    let next = present.range(segment + 1..).next().or_else(|| present.iter().next()).copied()?;
    let end = q.boundary.iter().find(|r| r.1 == next && r.2 == 0)?;
    ids.push(end.0);
    Some(ids)
}

pub(super) fn groups(s: &mut Session, sources: &[Source], mode: &Mode) -> Grouping {
    let total: usize = sources.iter().map(|x| x.count).sum();
    let mut parent: Vec<u32> = (0..total as u32).collect();
    let mut offsets: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
    let mut at = 0usize;
    for (i, src) in sources.iter().enumerate() {
        offsets.insert(src.panel, (at, i));
        at += src.count;
    }
    let chosen = |panel: u32| match mode {
        Mode::Selected(list) => list.contains(&panel),
        _ => true,
    };
    let (mut joined, mut seams, mut open) = (0usize, 0usize, 0usize);
    let tri_panels: Vec<u32> = sources.iter().filter(|x| x.quad.is_none()).map(|x| x.panel).collect();
    if !tri_panels.is_empty() {
        let mut held = Vec::new();
        for id in &tri_panels {
            let Some(panel) = s.garment.panels.iter().find(|p| p.id == *id) else { continue };
            if let Ok(built) = s.mesh.ensure(panel, false).built {
                held.push((*id, built));
            }
        }
        let views: BTreeMap<u32, MeshView> = held.iter().map(|(id, b)| (*id, MeshView { positions: &b.mesh.positions, boundary: &b.mesh.boundary, boundary_param: &b.mesh.boundary_param, inner_edges: &b.mesh.inner_edges })).collect();
        let mut welded_seams: BTreeSet<u32> = BTreeSet::new();
        for binding in sew::seam_bindings(&s.garment, &views) {
            match binding {
                Binding::Pair { seam, a, b, .. } => {
                    if s.garment.ripped.contains(&seam) || !chosen(a.panel) || !chosen(b.panel) {
                        continue;
                    }
                    let (Some((oa, _)), Some((ob, _))) = (offsets.get(&a.panel), offsets.get(&b.panel)) else { continue };
                    if join(&mut parent, (oa + a.vertex as usize) as u32, (ob + b.vertex as usize) as u32) {
                        joined += 1;
                    }
                    welded_seams.insert(seam);
                }
                Binding::Edge { seam, a, b0, .. } => {
                    if !s.garment.ripped.contains(&seam) && chosen(a.panel) && chosen(b0.panel) {
                        open += 1;
                    }
                }
            }
        }
        seams += welded_seams.len();
    }
    for seam in &s.garment.seams {
        if s.garment.ripped.contains(&seam.id) || seam.a.len() != 1 || seam.b.len() != 1 {
            continue;
        }
        let (ea, eb) = (&seam.a[0], &seam.b[0]);
        if ea.inner.is_some() || eb.inner.is_some() || !sew::is_full(ea) || !sew::is_full(eb) || !chosen(ea.panel) || !chosen(eb.panel) {
            continue;
        }
        let (Some(&(oa, ia)), Some(&(ob, ib))) = (offsets.get(&ea.panel), offsets.get(&eb.panel)) else { continue };
        let (Some(qa), Some(qb)) = (sources[ia].quad, sources[ib].quad) else { continue };
        let (Some(la), Some(mut lb)) = (rim_line(qa, ea.segment), rim_line(qb, eb.segment)) else {
            open += 1;
            continue;
        };
        if seam.reversed {
            lb.reverse();
        }
        if la.len() != lb.len() {
            open += 1;
            continue;
        }
        for (x, y) in la.iter().zip(&lb) {
            if join(&mut parent, (oa + *x as usize) as u32, (ob + *y as usize) as u32) {
                joined += 1;
            }
        }
        seams += 1;
    }
    let ids = (0..total as u32).map(|i| find(&mut parent, i)).collect();
    Grouping { groups: ids, joined, seams, open }
}
