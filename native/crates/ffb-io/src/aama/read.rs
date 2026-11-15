use std::collections::HashMap;

use ffb_pattern::geom::*;
use ffb_pattern::{path, NotchKind, Panel};

use super::conv::poly_points;
use super::piece::{self, Piece, RawNotch};
use super::{Flavour, Imported, ImportOptions};
use crate::dxf::{self, Block, Document, Entity, Poly, Shape, Vert};

const JOIN: f64 = 0.02;
const MATCH: f64 = 0.05;
const TOUCH: f64 = 0.05;
const FACET_EDGE: f64 = 40.0;
const FACET_TURN: f64 = 0.26;

struct Grid {
    cells: HashMap<(i64, i64), Vec<V2>>,
}

impl Grid {
    fn new(points: &[V2]) -> Grid {
        let mut cells: HashMap<(i64, i64), Vec<V2>> = HashMap::new();
        for p in points {
            cells.entry(Grid::key(*p)).or_default().push(*p);
        }
        Grid { cells }
    }

    fn key(p: V2) -> (i64, i64) {
        ((p[0] / MATCH).floor() as i64, (p[1] / MATCH).floor() as i64)
    }

    fn has(&self, p: V2) -> bool {
        let (kx, ky) = Grid::key(p);
        (-1..=1).any(|dx| (-1..=1).any(|dy| self.cells.get(&(kx + dx, ky + dy)).is_some_and(|v| v.iter().any(|q| dist(*q, p) <= MATCH))))
    }
}

fn reversed(p: &Poly) -> Poly {
    let n = p.verts.len();
    let verts = (0..n)
        .map(|k| {
            let v = &p.verts[n - 1 - k];
            Vert { p: v.p, curve: v.curve, bulge: if k + 1 < n { -p.verts[n - 2 - k].bulge } else { 0.0 } }
        })
        .collect();
    Poly { verts, closed: p.closed }
}

fn ends(p: &Poly) -> (V2, V2) {
    (p.verts[0].p, p.verts[p.verts.len() - 1].p)
}

fn chain(polys: Vec<Poly>) -> Vec<Poly> {
    let mut done = Vec::new();
    let mut open = Vec::new();
    for p in polys {
        if p.verts.len() < 2 {
            continue;
        }
        if p.closed {
            done.push(p);
        } else {
            open.push(p);
        }
    }
    while let Some(mut cur) = open.pop() {
        loop {
            let (first, last) = ends(&cur);
            if cur.verts.len() >= 3 && dist(first, last) < JOIN {
                cur.verts.pop();
                cur.closed = true;
                break;
            }
            let found = open.iter().position(|o| {
                let (a, b) = ends(o);
                dist(a, last) < JOIN || dist(b, last) < JOIN || dist(b, first) < JOIN || dist(a, first) < JOIN
            });
            let Some(j) = found else { break };
            let o = open.swap_remove(j);
            let (a, b) = ends(&o);
            if dist(a, last) < JOIN {
                cur.verts.pop();
                cur.verts.extend(o.verts);
            } else if dist(b, last) < JOIN {
                cur.verts.pop();
                cur.verts.extend(reversed(&o).verts);
            } else {
                let mut head = if dist(b, first) < JOIN { o } else { reversed(&o) };
                head.verts.pop();
                head.verts.extend(cur.verts);
                cur = head;
            }
        }
        done.push(cur);
    }
    done
}

fn area(p: &Poly) -> f64 {
    let pts: Vec<V2> = p.verts.iter().map(|v| v.p).collect();
    if pts.len() < 3 {
        0.0
    } else {
        ffb_pattern::polygon::signed_area(&pts).abs()
    }
}

fn largest_closed(polys: Vec<Poly>) -> (Option<Poly>, Vec<Poly>) {
    let mut best: Option<Poly> = None;
    let mut rest = Vec::new();
    for p in chain(polys) {
        if !p.closed {
            rest.push(p);
        } else if best.as_ref().is_none_or(|b| area(&p) > area(b)) {
            if let Some(old) = best.replace(p) {
                rest.push(old);
            }
        } else {
            rest.push(p);
        }
    }
    (best, rest)
}

fn scaled(e: &Entity, k: f64) -> Entity {
    let s = |p: V2| [p[0] * k, p[1] * k];
    let shape = match &e.shape {
        Shape::Poly(p) => Shape::Poly(Poly { verts: p.verts.iter().map(|v| Vert { p: s(v.p), bulge: v.bulge, curve: v.curve }).collect(), closed: p.closed }),
        Shape::Point { p, z, angle, thickness } => Shape::Point { p: s(*p), z: z * k, angle: *angle, thickness: thickness.map(|t| t * k) },
        Shape::Text { p, height, text } => Shape::Text { p: s(*p), height: height * k, text: text.clone() },
        other => other.clone(),
    };
    Entity { layer: e.layer.clone(), shape }
}

fn layer_no(layer: &str) -> Option<i32> {
    layer.trim().parse().ok()
}

fn tagged<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let (head, rest) = text.split_once(':')?;
    head.trim().eq_ignore_ascii_case(key).then(|| rest.trim())
}

fn style_scale(doc: &Document) -> Option<f64> {
    doc.entities.iter().find_map(|e| match &e.shape {
        Shape::Text { text, .. } => tagged(text, "Units").map(|u| if u.eq_ignore_ascii_case("ENGLISH") { 25.4 } else { 1.0 }),
        _ => None,
    })
}

fn insunits_scale(doc: &Document) -> Option<f64> {
    doc.insunits.and_then(|u| match u {
        1 => Some(25.4),
        2 => Some(304.8),
        4 => Some(1.0),
        5 => Some(10.0),
        6 => Some(1000.0),
        _ => None,
    })
}

fn detect(doc: &Document) -> Flavour {
    let in_blocks = doc.blocks.iter().any(|b| b.entities.iter().any(|e| matches!(&e.shape, Shape::Text { text, .. } if tagged(text, "Piece Name").is_some())));
    let in_style = doc.entities.iter().any(|e| matches!(&e.shape, Shape::Text { text, .. } if tagged(text, "Style Name").is_some() || tagged(text, "Standard Version").is_some()));
    if in_blocks || in_style {
        Flavour::Aama
    } else {
        Flavour::Plain
    }
}

fn notch_kind(layer: i32, width: Option<f64>) -> NotchKind {
    match layer {
        80 => NotchKind::T,
        81 | 83 => NotchKind::U,
        82 => NotchKind::V,
        _ => {
            if width.is_some_and(|w| w > 1e-9) {
                NotchKind::V
            } else {
                NotchKind::Slit
            }
        }
    }
}

struct BlockInfo {
    piece: Piece,
    size: Option<String>,
}

fn read_block(block: &Block, k: f64) -> BlockInfo {
    let ents: Vec<Entity> = block.entities.iter().map(|e| scaled(e, k)).collect();
    let mut name = None;
    let mut size = None;
    let mut turn = Vec::new();
    let mut curve = Vec::new();
    let mut layers: HashMap<i32, Vec<Poly>> = HashMap::new();
    let mut labels: Vec<(V2, String)> = Vec::new();
    let mut notches = Vec::new();
    for e in &ents {
        let no = layer_no(&e.layer);
        match &e.shape {
            Shape::Text { p, text, .. } => {
                if let Some(v) = tagged(text, "Piece Name") {
                    name = Some(v.to_string());
                } else if let Some(v) = tagged(text, "Size Name") {
                    size = Some(v.to_string());
                } else if text.trim_start().starts_with("FF:") && no == Some(8) {
                    labels.push((*p, text.trim().to_string()));
                }
            }
            Shape::Point { p, z, angle, thickness } => match no {
                Some(2) => turn.push(*p),
                Some(3) => curve.push(*p),
                Some(l @ (4 | 80 | 81 | 82 | 83)) => notches.push(RawNotch { p: *p, kind: notch_kind(l, *thickness), angle: *angle, depth: z.abs(), width: *thickness }),
                _ => {}
            },
            Shape::Poly(poly) => {
                if let Some(l) = no {
                    layers.entry(l).or_default().push(poly.clone());
                }
            }
            Shape::Insert { .. } => {}
        }
    }
    let (curve_grid, turn_grid) = (Grid::new(&curve), Grid::new(&turn));
    let mark = |polys: Vec<Poly>| -> Vec<Poly> {
        polys
            .into_iter()
            .map(|mut p| {
                for v in p.verts.iter_mut() {
                    v.curve = curve_grid.has(v.p) && !turn_grid.has(v.p);
                }
                p
            })
            .collect()
    };
    let mut piece = Piece { name: name.unwrap_or_else(|| block.name.clone()), notches, ..Piece::default() };
    let (cut, _) = largest_closed(mark(layers.remove(&1).unwrap_or_default()));
    piece.cut = cut;
    let (sew, open_sew) = largest_closed(mark(layers.remove(&14).unwrap_or_default()));
    piece.sew = sew;
    for p in open_sew {
        piece.inner.push((p, None));
    }
    let attach = |p: &Poly| labels.iter().find(|(at, _)| dist(*at, p.verts[0].p) <= MATCH).map(|(_, t)| t.clone());
    for p in mark(layers.remove(&8).unwrap_or_default()) {
        if p.verts.len() >= 2 {
            let label = attach(&p);
            piece.inner.push((p, label));
        }
    }
    for p in mark(layers.remove(&11).unwrap_or_default()) {
        if p.verts.len() >= 2 {
            let label = attach(&p).or_else(|| Some("FF:role=shape".to_string()));
            piece.inner.push((p, label));
        }
    }
    piece.grain = layers.remove(&7).and_then(|ps| ps.into_iter().find(|p| p.verts.len() >= 2)).map(|p| [p.verts[0].p, p.verts[p.verts.len() - 1].p]);
    BlockInfo { piece, size }
}

fn aama(doc: &Document, opts: &ImportOptions, scale: f64) -> Imported {
    let sample = doc.entities.iter().find_map(|e| match &e.shape {
        Shape::Text { text, .. } => tagged(text, "Sample Size").map(str::to_string),
        _ => None,
    });
    let mut notes = Vec::new();
    let mut chosen: Vec<BlockInfo> = Vec::new();
    let mut skipped_sizes = 0;
    for block in &doc.blocks {
        if block.name.starts_with('*') {
            continue;
        }
        let info = read_block(block, scale);
        if info.piece.cut.is_none() && info.piece.sew.is_none() {
            continue;
        }
        match chosen.iter().position(|c| c.piece.name == info.piece.name) {
            None => chosen.push(info),
            Some(i) => {
                skipped_sizes += 1;
                let is_sample = sample.is_some() && info.size == sample;
                let held_sample = sample.is_some() && chosen[i].size == sample;
                if is_sample && !held_sample {
                    chosen[i] = info;
                }
            }
        }
    }
    if skipped_sizes > 0 {
        notes.push(format!("{skipped_sizes} graded size blocks were skipped, only the sample size of each piece is imported"));
    }
    let mut panels = Vec::new();
    for info in chosen {
        match piece::build(info.piece, opts.fit_tolerance, opts.sew_allowance, &mut notes) {
            Ok(p) => panels.push(p),
            Err(e) => notes.push(e),
        }
    }
    Imported { panels, flavour: Flavour::Aama, scale, keep_positions: false, notes }
}

enum Hint {
    Sew,
    Grain,
    Notch(NotchKind),
    Internal,
    Name,
    Other,
}

fn hint_of(layer: &str) -> Hint {
    let up = layer.to_ascii_uppercase();
    if up.contains("SEW") {
        Hint::Sew
    } else if up.contains("GRAIN") {
        Hint::Grain
    } else if up.starts_with("NOTCH") {
        Hint::Notch(match up.rsplit('_').next() {
            Some("V") => NotchKind::V,
            Some("T") => NotchKind::T,
            Some("U") => NotchKind::U,
            _ => NotchKind::Slit,
        })
    } else if up.contains("INTERNAL") || up.contains("INNER") {
        Hint::Internal
    } else if up.contains("NAME") {
        Hint::Name
    } else {
        Hint::Other
    }
}

struct Loop {
    poly: Poly,
    flat: Vec<V2>,
    area: f64,
}

fn inside_loop(l: &Loop, p: V2) -> bool {
    ffb_pattern::polygon::contains(&l.flat, p) || {
        let n = l.flat.len();
        (0..n).any(|i| dist(p, closest_on_segment(p, l.flat[i], l.flat[(i + 1) % n]).1) <= TOUCH)
    }
}

fn owner(loops: &[Loop], pts: &[V2]) -> Option<usize> {
    (0..loops.len()).filter(|i| pts.iter().all(|p| inside_loop(&loops[*i], *p))).min_by(|a, b| loops[*a].area.total_cmp(&loops[*b].area))
}

fn boundary_gap(l: &Loop, p: V2) -> f64 {
    let n = l.flat.len();
    (0..n).map(|i| dist(p, closest_on_segment(p, l.flat[i], l.flat[(i + 1) % n]).1)).fold(f64::MAX, f64::min)
}

fn smooth_facets(p: &mut Poly) {
    let n = p.verts.len();
    if n < 4 {
        return;
    }
    let pts: Vec<V2> = p.verts.iter().map(|v| v.p).collect();
    for i in 0..n {
        if !p.closed && (i == 0 || i + 1 == n) {
            continue;
        }
        let before = (i + n - 1) % n;
        let (d0, d1) = (sub(pts[i], pts[before]), sub(pts[(i + 1) % n], pts[i]));
        let (l0, l1) = (len(d0), len(d1));
        if l0 < EPS || l1 < EPS || l0 > FACET_EDGE || l1 > FACET_EDGE || p.verts[i].bulge != 0.0 || p.verts[before].bulge != 0.0 {
            continue;
        }
        if cross(d0, d1).atan2(dot(d0, d1)).abs() < FACET_TURN {
            p.verts[i].curve = true;
        }
    }
}

fn plain(doc: &Document, opts: &ImportOptions, scale: f64) -> Imported {
    let mut notes = Vec::new();
    let mut source = doc.flattened();
    if !source.iter().any(|e| matches!(e.shape, Shape::Poly(_))) && !doc.blocks.is_empty() {
        for b in &doc.blocks {
            source.extend(b.entities.iter().cloned());
        }
        notes.push("the file has no drawing in its entities section, the geometry of its blocks was used".to_string());
    }
    let ents: Vec<Entity> = source.iter().map(|e| scaled(e, scale)).collect();
    let mut bodies = Vec::new();
    let mut sews = Vec::new();
    let mut grains = Vec::new();
    let mut slits: Vec<(Poly, NotchKind)> = Vec::new();
    let mut internals = Vec::new();
    let mut names: Vec<(V2, String)> = Vec::new();
    for e in &ents {
        match (&e.shape, hint_of(&e.layer)) {
            (Shape::Poly(p), Hint::Sew) => sews.push(p.clone()),
            (Shape::Poly(p), Hint::Grain) => grains.push(p.clone()),
            (Shape::Poly(p), Hint::Notch(k)) => slits.push((p.clone(), k)),
            (Shape::Poly(p), Hint::Internal) => internals.push(p.clone()),
            (Shape::Poly(p), _) => bodies.push(p.clone()),
            (Shape::Text { p, text, .. }, Hint::Name) => names.push((*p, text.clone())),
            _ => {}
        }
    }
    let mut loops = Vec::new();
    let mut open = Vec::new();
    let relaxed = |mut p: Poly| -> Poly {
        if opts.smooth_facets {
            smooth_facets(&mut p);
        }
        p
    };
    for p in chain(bodies).into_iter().map(relaxed) {
        match (p.closed, poly_points(&p, opts.fit_tolerance)) {
            (true, Some((pts, true))) => {
                let flat = path::flatten(&pts, true, 0.05);
                let a = ffb_pattern::polygon::signed_area(&flat).abs();
                if a >= 1.0 {
                    loops.push(Loop { poly: p, flat, area: a });
                }
            }
            _ => open.push(p),
        }
    }
    let mut pieces: Vec<Piece> = loops.iter().enumerate().map(|(k, l)| Piece { name: format!("Piece {}", k + 1), cut: Some(l.poly.clone()), ..Piece::default() }).collect();
    for s in chain(sews).into_iter().map(relaxed) {
        let pts: Vec<V2> = s.verts.iter().map(|v| v.p).collect();
        match owner(&loops, &pts) {
            Some(i) if s.closed && pieces[i].sew.is_none() => pieces[i].sew = Some(s),
            Some(i) => pieces[i].inner.push((s, None)),
            None => notes.push("a sew line outside every closed outline was skipped".to_string()),
        }
    }
    for g in grains {
        if g.verts.len() < 2 {
            continue;
        }
        let ends = [g.verts[0].p, g.verts[g.verts.len() - 1].p];
        if let Some(i) = owner(&loops, &ends) {
            pieces[i].grain.get_or_insert(ends);
        }
    }
    for (n, kind) in slits {
        if n.verts.len() < 2 {
            continue;
        }
        let (a, b) = (n.verts[0].p, n.verts[n.verts.len() - 1].p);
        let gap = |i: &usize| boundary_gap(&loops[*i], a).min(boundary_gap(&loops[*i], b));
        let Some(i) = (0..loops.len()).filter(|i| inside_loop(&loops[*i], a) && inside_loop(&loops[*i], b)).min_by(|x, y| gap(x).total_cmp(&gap(y))) else { continue };
        let (base, tip) = if boundary_gap(&loops[i], a) <= boundary_gap(&loops[i], b) { (a, b) } else { (b, a) };
        let d = sub(tip, base);
        pieces[i].notches.push(RawNotch { p: base, kind, angle: Some(d[1].atan2(d[0]).to_degrees()), depth: len(d), width: None });
    }
    for p in internals.into_iter().chain(open).map(relaxed) {
        if p.verts.len() < 2 {
            continue;
        }
        let pts: Vec<V2> = p.verts.iter().map(|v| v.p).collect();
        match owner(&loops, &pts) {
            Some(i) => pieces[i].inner.push((p, None)),
            None => notes.push("an open line outside every closed outline was skipped".to_string()),
        }
    }
    for (at, text) in names {
        if let Some(i) = owner(&loops, &[at]) {
            pieces[i].name = text;
        }
    }
    let mut panels: Vec<Panel> = Vec::new();
    for p in pieces {
        match piece::build(p, opts.fit_tolerance, opts.sew_allowance, &mut notes) {
            Ok(panel) => panels.push(panel),
            Err(e) => notes.push(e),
        }
    }
    Imported { panels, flavour: Flavour::Plain, scale, keep_positions: true, notes }
}

pub fn import(text: &str, opts: &ImportOptions) -> Result<Imported, String> {
    let doc = dxf::parse(text)?;
    let flavour = opts.flavour.unwrap_or_else(|| detect(&doc));
    let scale = opts.scale.or_else(|| if flavour == Flavour::Aama { style_scale(&doc).or_else(|| insunits_scale(&doc)) } else { insunits_scale(&doc) }).unwrap_or(1.0);
    doc.check(scale, dxf::MAX_COORD_MM)?;
    dxf::check_entities(&doc.flattened(), scale, dxf::MAX_COORD_MM)?;
    let out = match flavour {
        Flavour::Aama => aama(&doc, opts, scale),
        Flavour::Plain => plain(&doc, opts, scale),
    };
    if out.panels.is_empty() {
        let why = out.notes.first().cloned().unwrap_or_else(|| "no closed outline was found".into());
        return Err(format!("nothing to import: {why}"));
    }
    Ok(out)
}
