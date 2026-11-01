use std::fmt::Write;

pub(crate) type P = [f64; 2];

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Anchor {
    Start,
    Middle,
}

#[derive(Debug, Clone)]
pub(crate) enum Prim {
    Line { pts: Vec<P>, closed: bool, width: f64, dash: Option<(f64, f64)>, gray: f64 },
    Circle { c: P, r: f64, width: f64, fill: bool },
    Text { at: P, size: f64, text: String, anchor: Anchor },
}

#[derive(Debug, Default)]
pub(crate) struct Drawing {
    pub prims: Vec<Prim>,
}

const MARGIN: f64 = 15.0;
const MAX_PRIMS: usize = 400_000;

fn text_width(text: &str, size: f64) -> f64 {
    text.chars().count() as f64 * size * 0.55
}

impl Drawing {
    pub fn line(&mut self, pts: Vec<P>, closed: bool, width: f64, dash: Option<(f64, f64)>, gray: f64) {
        if pts.len() >= 2 && self.prims.len() < MAX_PRIMS {
            self.prims.push(Prim::Line { pts, closed, width, dash, gray });
        }
    }

    pub fn circle(&mut self, c: P, r: f64, width: f64, fill: bool) {
        if self.prims.len() < MAX_PRIMS {
            self.prims.push(Prim::Circle { c, r, width, fill });
        }
    }

    pub fn text(&mut self, at: P, size: f64, text: impl Into<String>, anchor: Anchor) {
        if self.prims.len() < MAX_PRIMS {
            self.prims.push(Prim::Text { at, size, text: text.into(), anchor });
        }
    }

    pub fn bounds(&self) -> Option<(P, P)> {
        let mut lo = [f64::MAX; 2];
        let mut hi = [f64::MIN; 2];
        let mut grow = |p: P| {
            for a in 0..2 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        };
        for prim in &self.prims {
            match prim {
                Prim::Line { pts, .. } => pts.iter().for_each(|p| grow(*p)),
                Prim::Circle { c, r, .. } => {
                    grow([c[0] - r, c[1] - r]);
                    grow([c[0] + r, c[1] + r]);
                }
                Prim::Text { at, size, text, anchor } => {
                    let w = text_width(text, *size);
                    let x0 = if *anchor == Anchor::Middle { at[0] - w * 0.5 } else { at[0] };
                    grow([x0, at[1] - size * 0.3]);
                    grow([x0 + w, at[1] + size]);
                }
            }
        }
        (lo[0] <= hi[0] && lo[1] <= hi[1]).then_some((lo, hi))
    }

    fn frame(&self) -> (P, f64, f64) {
        let (lo, hi) = self.bounds().unwrap_or(([0.0, 0.0], [100.0, 100.0]));
        ([lo[0] - MARGIN, hi[1] + MARGIN], hi[0] - lo[0] + 2.0 * MARGIN, hi[1] - lo[1] + 2.0 * MARGIN)
    }

    pub fn size(&self) -> (f64, f64) {
        let (_, w, h) = self.frame();
        (w, h)
    }

    pub fn svg(&self, title: &str) -> String {
        let (origin, w, h) = self.frame();
        let map = |p: P| (p[0] - origin[0], origin[1] - p[1]);
        let mut out = String::with_capacity(64 * 1024);
        let _ = writeln!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        let _ = writeln!(out, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">");
        let _ = writeln!(out, "<title>{}</title>", escape(title));
        let _ = writeln!(out, "<rect x=\"0\" y=\"0\" width=\"{w:.2}\" height=\"{h:.2}\" fill=\"#ffffff\"/>");
        for prim in &self.prims {
            match prim {
                Prim::Line { pts, closed, width, dash, gray } => {
                    let tag = if *closed { "polygon" } else { "polyline" };
                    let list: Vec<String> = pts.iter().map(|p| map(*p)).map(|(x, y)| format!("{x:.2},{y:.2}")).collect();
                    let dash = dash.map(|(a, b)| format!(" stroke-dasharray=\"{a:.2} {b:.2}\"")).unwrap_or_default();
                    let g = (gray.clamp(0.0, 1.0) * 255.0).round() as u8;
                    let _ = writeln!(out, "<{tag} points=\"{}\" fill=\"none\" stroke=\"#{g:02x}{g:02x}{g:02x}\" stroke-width=\"{width:.2}\" stroke-linejoin=\"round\" stroke-linecap=\"round\"{dash}/>", list.join(" "));
                }
                Prim::Circle { c, r, width, fill } => {
                    let (x, y) = map(*c);
                    let fill = if *fill { "#000000" } else { "none" };
                    let _ = writeln!(out, "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{r:.2}\" fill=\"{fill}\" stroke=\"#000000\" stroke-width=\"{width:.2}\"/>");
                }
                Prim::Text { at, size, text, anchor } => {
                    let (x, y) = map(*at);
                    let anchor = if *anchor == Anchor::Middle { "middle" } else { "start" };
                    let _ = writeln!(out, "<text x=\"{x:.2}\" y=\"{y:.2}\" font-family=\"Helvetica, Arial, sans-serif\" font-size=\"{size:.2}\" text-anchor=\"{anchor}\" fill=\"#000000\">{}</text>", escape(text));
                }
            }
        }
        out.push_str("</svg>\n");
        out
    }

    pub fn pdf(&self) -> Vec<u8> {
        let (origin, w, h) = self.frame();
        let k = 72.0 / 25.4;
        let map = |p: P| ((p[0] - origin[0]) * k, (p[1] - (origin[1] - h)) * k);
        let mut c = String::with_capacity(64 * 1024);
        for prim in &self.prims {
            match prim {
                Prim::Line { pts, closed, width, dash, gray } => {
                    let d = dash.map(|(a, b)| format!("[{:.2} {:.2}] 0 d", a * k, b * k)).unwrap_or_else(|| "[] 0 d".into());
                    let _ = writeln!(c, "{:.3} w {d} {:.3} G", width * k, gray.clamp(0.0, 1.0));
                    for (i, p) in pts.iter().enumerate() {
                        let (x, y) = map(*p);
                        let _ = writeln!(c, "{x:.2} {y:.2} {}", if i == 0 { "m" } else { "l" });
                    }
                    c.push_str(if *closed { "s\n" } else { "S\n" });
                }
                Prim::Circle { c: centre, r, width, fill } => {
                    let (x, y) = map(*centre);
                    let r = r * k;
                    let m = 0.552_284_75 * r;
                    let _ = writeln!(c, "{:.3} w [] 0 d 0 G 0 g", width * k);
                    let _ = writeln!(c, "{:.2} {:.2} m", x + r, y);
                    let _ = writeln!(c, "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c", x + r, y + m, x + m, y + r, x, y + r);
                    let _ = writeln!(c, "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c", x - m, y + r, x - r, y + m, x - r, y);
                    let _ = writeln!(c, "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c", x - r, y - m, x - m, y - r, x, y - r);
                    let _ = writeln!(c, "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c", x + m, y - r, x + r, y - m, x + r, y);
                    c.push_str(if *fill { "f\n" } else { "S\n" });
                }
                Prim::Text { at, size, text, anchor } => {
                    let (mut x, y) = map(*at);
                    if *anchor == Anchor::Middle {
                        x -= text_width(text, *size) * k * 0.5;
                    }
                    let _ = writeln!(c, "0 g BT /F1 {:.2} Tf {x:.2} {y:.2} Td ({}) Tj ET", size * k, pdf_text(text));
                }
            }
        }
        let mut out: Vec<u8> = Vec::new();
        let mut offsets: Vec<usize> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");
        let mut object = |out: &mut Vec<u8>, body: &str| {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", offsets.len()).as_bytes());
        };
        object(&mut out, "<< /Type /Catalog /Pages 2 0 R >>");
        object(&mut out, "<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
        object(&mut out, &format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>", w * k, h * k));
        object(&mut out, &format!("<< /Length {} >>\nstream\n{c}endstream", c.len()));
        object(&mut out, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>");
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
        for o in &offsets {
            out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1).as_bytes());
        out
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn pdf_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' | '(' | ')' => {
                out.push('\\');
                out.push(ch);
            }
            c if (' '..='~').contains(&c) => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}
