use std::collections::BTreeSet;
use std::f64::consts::TAU;

pub type V2 = [f64; 2];

#[derive(Debug, Clone, PartialEq)]
pub struct Vert {
    pub p: V2,
    pub bulge: f64,
    pub curve: bool,
}

impl Vert {
    pub fn at(p: V2) -> Vert {
        Vert { p, bulge: 0.0, curve: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Poly {
    pub verts: Vec<Vert>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Poly(Poly),
    Point { p: V2, z: f64, angle: Option<f64>, thickness: Option<f64> },
    Text { p: V2, height: f64, text: String },
    Insert { block: String, p: V2, scale: V2, rotation: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub layer: String,
    pub shape: Shape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub name: String,
    pub base: V2,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub insunits: Option<i32>,
    pub blocks: Vec<Block>,
    pub entities: Vec<Entity>,
}

struct Record {
    kind: String,
    codes: Vec<(i32, String)>,
}

fn tokenize(text: &str) -> Result<Vec<(i32, String)>, String> {
    let mut lines = text.lines();
    let mut out = Vec::new();
    let mut number = 0usize;
    while let Some(code) = lines.next() {
        number += 1;
        let Some(value) = lines.next() else { break };
        number += 1;
        let code = code.trim();
        if code.is_empty() && out.is_empty() {
            continue;
        }
        let parsed = code.parse::<i32>().map_err(|_| format!("line {} is not a DXF group code", number - 1))?;
        out.push((parsed, value.trim().to_string()));
    }
    Ok(out)
}

fn records(pairs: &[(i32, String)]) -> Vec<Record> {
    let mut out: Vec<Record> = Vec::new();
    for (code, value) in pairs {
        if *code == 0 {
            out.push(Record { kind: value.to_ascii_uppercase(), codes: Vec::new() });
        } else if let Some(last) = out.last_mut() {
            last.codes.push((*code, value.clone()));
        }
    }
    out
}

fn num(codes: &[(i32, String)], code: i32) -> Option<f64> {
    codes.iter().find(|(c, _)| *c == code).and_then(|(_, v)| v.parse().ok())
}

fn text_of(codes: &[(i32, String)], code: i32) -> Option<&str> {
    codes.iter().find(|(c, _)| *c == code).map(|(_, v)| v.as_str())
}

fn layer_of(codes: &[(i32, String)]) -> String {
    text_of(codes, 8).unwrap_or("0").to_string()
}

fn xy(codes: &[(i32, String)], x: i32, y: i32) -> V2 {
    [num(codes, x).unwrap_or(0.0), num(codes, y).unwrap_or(0.0)]
}

pub fn bulge_for_sweep(sweep: f64) -> f64 {
    (sweep / 4.0).tan()
}

fn arc_poly(c: V2, r: f64, a0: f64, a1: f64) -> Poly {
    let mut sweep = (a1 - a0).rem_euclid(360.0).to_radians();
    if sweep < 1e-9 {
        sweep = TAU;
    }
    let (s, e) = (a0.to_radians(), a0.to_radians() + sweep);
    let at = |a: f64| [c[0] + r * a.cos(), c[1] + r * a.sin()];
    let halves = if sweep > TAU - 1e-6 { 2 } else { 1 };
    if halves == 2 {
        let mid = s + sweep / 2.0;
        return Poly { verts: vec![Vert { p: at(s), bulge: bulge_for_sweep(sweep / 2.0), curve: false }, Vert { p: at(mid), bulge: bulge_for_sweep(sweep / 2.0), curve: false }], closed: true };
    }
    Poly { verts: vec![Vert { p: at(s), bulge: bulge_for_sweep(sweep), curve: false }, Vert::at(at(e))], closed: false }
}

fn lw_poly(codes: &[(i32, String)]) -> Poly {
    let mut verts: Vec<Vert> = Vec::new();
    let mut flags = 0i32;
    for (code, value) in codes {
        match code {
            70 => flags = value.parse().unwrap_or(0),
            10 => verts.push(Vert::at([value.parse().unwrap_or(0.0), 0.0])),
            20 => {
                if let Some(v) = verts.last_mut() {
                    v.p[1] = value.parse().unwrap_or(0.0);
                }
            }
            42 => {
                if let Some(v) = verts.last_mut() {
                    v.bulge = value.parse().unwrap_or(0.0);
                }
            }
            _ => {}
        }
    }
    Poly { verts, closed: flags & 1 == 1 }
}

fn basis_point(degree: usize, knots: &[f64], ctrl: &[V2], weights: &[f64], u: f64) -> V2 {
    let n = ctrl.len();
    let mut k = degree;
    while k + 1 < n && u >= knots[k + 1] {
        k += 1;
    }
    let mut d: Vec<[f64; 3]> = (0..=degree)
        .map(|j| {
            let i = j + k - degree;
            let w = weights.get(i).copied().unwrap_or(1.0);
            [ctrl[i][0] * w, ctrl[i][1] * w, w]
        })
        .collect();
    for r in 1..=degree {
        for j in (r..=degree).rev() {
            let lo = knots[j + k - degree];
            let hi = knots[j + 1 + k - r];
            let a = if (hi - lo).abs() < 1e-12 { 0.0 } else { (u - lo) / (hi - lo) };
            let (prev, cur) = (d[j - 1], d[j]);
            d[j] = [(1.0 - a) * prev[0] + a * cur[0], (1.0 - a) * prev[1] + a * cur[1], (1.0 - a) * prev[2] + a * cur[2]];
        }
    }
    let h = d[degree];
    if h[2].abs() < 1e-12 {
        [h[0], h[1]]
    } else {
        [h[0] / h[2], h[1] / h[2]]
    }
}

fn spline_poly(codes: &[(i32, String)]) -> Option<Poly> {
    let flags = num(codes, 70).unwrap_or(0.0) as i32;
    let degree = num(codes, 71).unwrap_or(3.0) as usize;
    let knots: Vec<f64> = codes.iter().filter(|(c, _)| *c == 40).filter_map(|(_, v)| v.parse().ok()).collect();
    let weights: Vec<f64> = codes.iter().filter(|(c, _)| *c == 41).filter_map(|(_, v)| v.parse().ok()).collect();
    let collect = |cx: i32, cy: i32| -> Vec<V2> {
        let xs: Vec<f64> = codes.iter().filter(|(c, _)| *c == cx).filter_map(|(_, v)| v.parse().ok()).collect();
        let ys: Vec<f64> = codes.iter().filter(|(c, _)| *c == cy).filter_map(|(_, v)| v.parse().ok()).collect();
        xs.into_iter().zip(ys).map(|(x, y)| [x, y]).collect()
    };
    let ctrl = collect(10, 20);
    let fit = collect(11, 21);
    let closed = flags & 1 == 1;
    let finish = |pts: Vec<V2>, curve: bool| -> Poly {
        let n = pts.len();
        let verts = pts.into_iter().enumerate().map(|(i, p)| Vert { p, bulge: 0.0, curve: curve && (closed || (i > 0 && i + 1 < n)) }).collect();
        Poly { verts, closed }
    };
    if ctrl.len() > degree && degree >= 1 && knots.len() == ctrl.len() + degree + 1 {
        if degree == 1 {
            return Some(finish(ctrl, false));
        }
        let (lo, hi) = (knots[degree], knots[ctrl.len()]);
        if hi <= lo {
            return None;
        }
        let spans = (degree..ctrl.len()).filter(|i| knots[i + 1] > knots[*i]).count().max(1);
        let steps = spans * 16;
        let mut pts: Vec<V2> = (0..=steps)
            .map(|i| {
                let u = (lo + (hi - lo) * i as f64 / steps as f64).min(hi - (hi - lo) * 1e-12);
                basis_point(degree, &knots, &ctrl, &weights, u)
            })
            .collect();
        if let Some(last) = pts.last_mut() {
            *last = basis_point(degree, &knots, &ctrl, &weights, hi - (hi - lo) * 1e-12);
        }
        if closed && pts.len() > 1 && dist(pts[0], pts[pts.len() - 1]) < 1e-6 {
            pts.pop();
        }
        return Some(finish(pts, true));
    }
    if fit.len() >= 2 {
        return Some(finish(fit, true));
    }
    if ctrl.len() >= 2 {
        return Some(finish(ctrl, false));
    }
    None
}

fn dist(a: V2, b: V2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn entities_from(recs: &[Record]) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < recs.len() {
        let rec = &recs[i];
        let c = &rec.codes;
        let layer = layer_of(c);
        i += 1;
        let shape = match rec.kind.as_str() {
            "LINE" => Some(Shape::Poly(Poly { verts: vec![Vert::at(xy(c, 10, 20)), Vert::at(xy(c, 11, 21))], closed: false })),
            "LWPOLYLINE" => Some(Shape::Poly(lw_poly(c))),
            "POLYLINE" => {
                let flags = num(c, 70).unwrap_or(0.0) as i32;
                let mut verts = Vec::new();
                while i < recs.len() && recs[i].kind != "SEQEND" {
                    if recs[i].kind == "VERTEX" {
                        let vc = &recs[i].codes;
                        let vflags = num(vc, 70).unwrap_or(0.0) as i32;
                        if vflags & 16 == 0 {
                            verts.push(Vert { p: xy(vc, 10, 20), bulge: num(vc, 42).unwrap_or(0.0), curve: false });
                        }
                    } else if !matches!(recs[i].kind.as_str(), "ATTRIB") {
                        break;
                    }
                    i += 1;
                }
                if i < recs.len() && recs[i].kind == "SEQEND" {
                    i += 1;
                }
                (flags & (16 | 64) == 0).then_some(Shape::Poly(Poly { verts, closed: flags & 1 == 1 }))
            }
            "ARC" => {
                let r = num(c, 40).unwrap_or(0.0);
                (r > 0.0).then(|| Shape::Poly(arc_poly(xy(c, 10, 20), r, num(c, 50).unwrap_or(0.0), num(c, 51).unwrap_or(360.0))))
            }
            "CIRCLE" => {
                let r = num(c, 40).unwrap_or(0.0);
                (r > 0.0).then(|| Shape::Poly(arc_poly(xy(c, 10, 20), r, 0.0, 360.0)))
            }
            "POINT" => Some(Shape::Point { p: xy(c, 10, 20), z: num(c, 30).unwrap_or(0.0), angle: num(c, 50), thickness: num(c, 39) }),
            "TEXT" => text_of(c, 1).map(|t| Shape::Text { p: xy(c, 10, 20), height: num(c, 40).unwrap_or(0.0), text: t.to_string() }),
            "MTEXT" => {
                let mut t = String::new();
                for (code, v) in c {
                    if *code == 3 || *code == 1 {
                        t.push_str(v);
                    }
                }
                Some(Shape::Text { p: xy(c, 10, 20), height: num(c, 40).unwrap_or(0.0), text: t })
            }
            "INSERT" => {
                if num(c, 66).unwrap_or(0.0) as i32 == 1 {
                    while i < recs.len() && recs[i].kind != "SEQEND" {
                        i += 1;
                    }
                    if i < recs.len() {
                        i += 1;
                    }
                }
                text_of(c, 2).map(|name| Shape::Insert { block: name.to_string(), p: xy(c, 10, 20), scale: [num(c, 41).unwrap_or(1.0), num(c, 42).unwrap_or(1.0)], rotation: num(c, 50).unwrap_or(0.0) })
            }
            "SPLINE" => spline_poly(c).map(Shape::Poly),
            _ => None,
        };
        if let Some(shape) = shape {
            out.push(Entity { layer, shape });
        }
    }
    out
}

fn sections(pairs: &[(i32, String)]) -> Vec<(String, &[(i32, String)])> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < pairs.len() {
        if pairs[i].0 == 0 && pairs[i].1.eq_ignore_ascii_case("SECTION") && pairs[i + 1].0 == 2 {
            let name = pairs[i + 1].1.to_ascii_uppercase();
            let start = i + 2;
            let mut end = start;
            while end < pairs.len() && !(pairs[end].0 == 0 && pairs[end].1.eq_ignore_ascii_case("ENDSEC")) {
                end += 1;
            }
            out.push((name, &pairs[start..end]));
            i = end;
        }
        i += 1;
    }
    out
}

fn blocks_from(recs: &[Record]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < recs.len() {
        if recs[i].kind == "BLOCK" {
            let name = text_of(&recs[i].codes, 2).unwrap_or("").to_string();
            let base = xy(&recs[i].codes, 10, 20);
            let start = i + 1;
            let mut end = start;
            while end < recs.len() && recs[end].kind != "ENDBLK" {
                end += 1;
            }
            out.push(Block { name, base, entities: entities_from(&recs[start..end]) });
            i = end;
        }
        i += 1;
    }
    out
}

pub fn parse(text: &str) -> Result<Document, String> {
    let pairs = tokenize(text)?;
    let mut doc = Document::default();
    let mut found = false;
    for (name, body) in sections(&pairs) {
        found = true;
        match name.as_str() {
            "HEADER" => {
                for w in body.windows(2) {
                    if w[0].0 == 9 && w[0].1.eq_ignore_ascii_case("$INSUNITS") && w[1].0 == 70 {
                        doc.insunits = w[1].1.parse().ok();
                    }
                }
            }
            "BLOCKS" => doc.blocks = blocks_from(&records(body)),
            "ENTITIES" => doc.entities = entities_from(&records(body)),
            _ => {}
        }
    }
    if !found {
        return Err("no SECTION found, this is not an ASCII DXF file".into());
    }
    Ok(doc)
}

fn affine(p: V2, scale: V2, rotation: f64, to: V2) -> V2 {
    let (s, c) = rotation.to_radians().sin_cos();
    let q = [p[0] * scale[0], p[1] * scale[1]];
    [to[0] + q[0] * c - q[1] * s, to[1] + q[0] * s + q[1] * c]
}

fn transformed(e: &Entity, scale: V2, rotation: f64, to: V2) -> Entity {
    let flip = scale[0] * scale[1] < 0.0;
    let shape = match &e.shape {
        Shape::Poly(p) => Shape::Poly(Poly {
            verts: p.verts.iter().map(|v| Vert { p: affine(v.p, scale, rotation, to), bulge: if flip { -v.bulge } else { v.bulge }, curve: v.curve }).collect(),
            closed: p.closed,
        }),
        Shape::Point { p, z, angle, thickness } => Shape::Point { p: affine(*p, scale, rotation, to), z: *z, angle: angle.map(|a| a + rotation), thickness: *thickness },
        Shape::Text { p, height, text } => Shape::Text { p: affine(*p, scale, rotation, to), height: *height, text: text.clone() },
        other => other.clone(),
    };
    Entity { layer: e.layer.clone(), shape }
}

impl Document {
    pub fn block(&self, name: &str) -> Option<&Block> {
        self.blocks.iter().find(|b| b.name.eq_ignore_ascii_case(name))
    }

    pub fn expand(&self, entities: &[Entity], depth: u32, out: &mut Vec<Entity>) {
        for e in entities {
            match &e.shape {
                Shape::Insert { block, p, scale, rotation } if depth < 8 => {
                    if let Some(b) = self.block(block) {
                        let moved: Vec<Entity> = b.entities.iter().map(|x| transformed(x, *scale, *rotation, [0.0, 0.0])).collect();
                        let base = affine(b.base, *scale, *rotation, [0.0, 0.0]);
                        let placed: Vec<Entity> = moved.iter().map(|x| transformed(x, [1.0, 1.0], 0.0, [p[0] - base[0], p[1] - base[1]])).collect();
                        self.expand(&placed, depth + 1, out);
                    }
                }
                Shape::Insert { .. } => {}
                _ => out.push(e.clone()),
            }
        }
    }

    pub fn flattened(&self) -> Vec<Entity> {
        let mut out = Vec::new();
        self.expand(&self.entities, 0, &mut out);
        out
    }
}

pub struct Sink {
    buf: String,
    decimals: usize,
    layers: BTreeSet<String>,
}

impl Sink {
    pub fn new(decimals: usize) -> Sink {
        Sink { buf: String::new(), decimals, layers: BTreeSet::new() }
    }

    fn pair(&mut self, code: i32, value: &str) {
        self.buf.push_str(&format!("{code}\n{value}\n"));
    }

    fn number(&mut self, code: i32, v: f64) {
        let mut s = format!("{:.*}", self.decimals, v);
        if s.starts_with('-') && s[1..].chars().all(|c| c == '0' || c == '.') {
            s.remove(0);
        }
        self.pair(code, &s);
    }

    fn head(&mut self, kind: &str, layer: &str) {
        self.layers.insert(layer.to_string());
        self.pair(0, kind);
        self.pair(8, layer);
    }

    pub fn polyline(&mut self, layer: &str, pts: &[V2], closed: bool) {
        if pts.len() < 2 {
            return;
        }
        self.head("POLYLINE", layer);
        self.pair(66, "1");
        self.pair(70, if closed { "1" } else { "0" });
        self.number(10, 0.0);
        self.number(20, 0.0);
        self.number(30, 0.0);
        for p in pts {
            self.head("VERTEX", layer);
            self.number(10, p[0]);
            self.number(20, p[1]);
            self.number(30, 0.0);
        }
        self.head("SEQEND", layer);
    }

    pub fn line(&mut self, layer: &str, a: V2, b: V2) {
        self.head("LINE", layer);
        self.number(10, a[0]);
        self.number(20, a[1]);
        self.number(30, 0.0);
        self.number(11, b[0]);
        self.number(21, b[1]);
        self.number(31, 0.0);
    }

    pub fn point(&mut self, layer: &str, p: V2, z: f64, angle: Option<f64>, thickness: Option<f64>) {
        self.head("POINT", layer);
        if let Some(t) = thickness {
            self.number(39, t);
        }
        self.number(10, p[0]);
        self.number(20, p[1]);
        self.number(30, z);
        if let Some(a) = angle {
            self.number(50, a);
        }
    }

    pub fn text(&mut self, layer: &str, p: V2, height: f64, text: &str) {
        self.head("TEXT", layer);
        self.number(10, p[0]);
        self.number(20, p[1]);
        self.number(30, 0.0);
        self.number(40, height);
        self.pair(1, text);
    }
}

pub struct Writer {
    pub units: i32,
    decimals: usize,
    blocks: Vec<(String, Sink)>,
    pub entities: Sink,
}

impl Writer {
    pub fn new(decimals: usize, units: i32) -> Writer {
        Writer { units, decimals, blocks: Vec::new(), entities: Sink::new(decimals) }
    }

    pub fn add_block(&mut self, name: &str, sink: Sink) {
        self.blocks.push((name.to_string(), sink));
    }

    pub fn sink(&self) -> Sink {
        Sink::new(self.decimals)
    }

    pub fn finish(self) -> String {
        let mut layers: BTreeSet<String> = self.entities.layers.clone();
        for (_, s) in &self.blocks {
            layers.extend(s.layers.iter().cloned());
        }
        layers.insert("0".into());
        let mut out = String::new();
        let mut put = |code: i32, value: &str| out.push_str(&format!("{code}\n{value}\n"));
        put(0, "SECTION");
        put(2, "HEADER");
        put(9, "$ACADVER");
        put(1, "AC1009");
        put(9, "$INSUNITS");
        put(70, &self.units.to_string());
        put(0, "ENDSEC");
        put(0, "SECTION");
        put(2, "TABLES");
        put(0, "TABLE");
        put(2, "LTYPE");
        put(70, "1");
        put(0, "LTYPE");
        put(2, "CONTINUOUS");
        put(70, "0");
        put(3, "Solid line");
        put(72, "65");
        put(73, "0");
        put(40, "0.0");
        put(0, "ENDTAB");
        put(0, "TABLE");
        put(2, "LAYER");
        put(70, &layers.len().to_string());
        for name in &layers {
            put(0, "LAYER");
            put(2, name);
            put(70, "0");
            put(62, "7");
            put(6, "CONTINUOUS");
        }
        put(0, "ENDTAB");
        put(0, "ENDSEC");
        put(0, "SECTION");
        put(2, "BLOCKS");
        let mut body = String::new();
        for (name, sink) in &self.blocks {
            body.push_str(&format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0.0\n20\n0.0\n30\n0.0\n3\n{name}\n1\n\n"));
            body.push_str(&sink.buf);
            body.push_str("0\nENDBLK\n8\n0\n");
        }
        out.push_str(&body);
        out.push_str("0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");
        out.push_str(&self.entities.buf);
        out.push_str("0\nENDSEC\n0\nEOF\n");
        out
    }
}
