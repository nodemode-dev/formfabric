use ffb_pattern::trim::{param, span_path};
use ffb_pattern::{notch, path, polygon, Garment, InnerRole, Panel, SeamEnd, StyleKind};
use serde_json::{json, Value};

use super::draw::{Anchor, Drawing, P};
use super::measure;
use crate::{args, Command, CommandError, Group, Session, EXTERNAL};

const TOL: f64 = 0.1;
const TEETH_PITCH: f64 = 4.0;

fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

fn length(line: &[P]) -> f64 {
    line.windows(2).map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt()).sum()
}

fn resample(line: &[P], pitch: f64) -> Vec<(P, P)> {
    let total = length(line);
    let mut out = Vec::new();
    if total < 1.0e-6 {
        return out;
    }
    let count = (total / pitch).floor().max(1.0) as usize;
    let mut walked = 0.0;
    let mut seg = 0;
    for k in 0..=count {
        let target = total * k as f64 / count as f64;
        while seg + 1 < line.len() - 1 && walked + seg_len(line, seg) < target {
            walked += seg_len(line, seg);
            seg += 1;
        }
        let (a, b) = (line[seg], line[seg + 1]);
        let l = seg_len(line, seg).max(1.0e-9);
        let t = ((target - walked) / l).clamp(0.0, 1.0);
        let dir = [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
        out.push(([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t], dir));
    }
    out
}

fn seg_len(line: &[P], i: usize) -> f64 {
    ((line[i + 1][0] - line[i][0]).powi(2) + (line[i + 1][1] - line[i][1]).powi(2)).sqrt()
}

fn mid(line: &[P]) -> P {
    let half = length(line) * 0.5;
    let mut walked = 0.0;
    for i in 0..line.len() - 1 {
        let l = seg_len(line, i);
        if walked + l >= half && l > 1.0e-9 {
            let t = (half - walked) / l;
            return [line[i][0] + (line[i + 1][0] - line[i][0]) * t, line[i][1] + (line[i + 1][1] - line[i][1]) * t];
        }
        walked += l;
    }
    line[0]
}

fn world_path(garment: &Garment, ends: &[SeamEnd]) -> Option<Vec<P>> {
    let line = span_path(garment, ends)?;
    let first = ends.first()?.panel;
    let panel = garment.panels.iter().find(|p| p.id == first)?;
    let place = panel.placement();
    Some(line.into_iter().map(|p| place.to_world(p)).collect())
}

fn offset_inside(line: &[P], panel: &Panel, distance: f64) -> Vec<P> {
    let outline = panel.world_outline(TOL);
    let shifted = |sign: f64| -> Vec<P> {
        let mut out = Vec::with_capacity(line.len());
        for i in 0..line.len() {
            let (a, b) = (line[i.saturating_sub(1)], line[(i + 1).min(line.len() - 1)]);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let l = (dx * dx + dy * dy).sqrt().max(1.0e-9);
            out.push([line[i][0] - dy / l * distance * sign, line[i][1] + dx / l * distance * sign]);
        }
        out
    };
    let left = shifted(1.0);
    let inside = left.iter().filter(|p| polygon::contains(&outline, **p)).count();
    if inside * 2 >= left.len() {
        left
    } else {
        shifted(-1.0)
    }
}

fn arrow(d: &mut Drawing, a: P, b: P) {
    d.line(vec![a, b], false, 0.4, None, 0.0);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l = (dx * dx + dy * dy).sqrt().max(1.0e-9);
    let (ux, uy) = (dx / l, dy / l);
    for (p, s) in [(b, -1.0), (a, 1.0)] {
        let tip = p;
        let base = [tip[0] + ux * 6.0 * s, tip[1] + uy * 6.0 * s];
        d.line(vec![[base[0] - uy * 2.0, base[1] + ux * 2.0], tip, [base[0] + uy * 2.0, base[1] - ux * 2.0]], false, 0.4, None, 0.0);
    }
}

fn panel_drawing(panel: &Panel, d: &mut Drawing) {
    let place = panel.placement();
    let outline = panel.world_outline(TOL);
    if outline.len() < 3 {
        return;
    }
    d.line(outline.clone(), true, 0.7, None, 0.0);
    for inner in &panel.inner {
        let line: Vec<P> = path::flatten(&inner.points, inner.closed, TOL).into_iter().map(|p| place.to_world(p)).collect();
        match inner.role {
            InnerRole::Fold => d.line(line, inner.closed, 0.35, Some((5.0, 1.5)), 0.2),
            InnerRole::Dart => d.line(line, inner.closed, 0.4, None, 0.0),
            InnerRole::Reference => d.line(line, inner.closed, 0.25, Some((2.0, 2.0)), 0.5),
            InnerRole::Shape => d.line(line, inner.closed, 0.35, None, 0.25),
        }
    }
    for n in &panel.notches {
        if let Some((p, out)) = notch::locate(panel, n) {
            let a = place.to_world(p);
            let v = place.vec_to_world(out);
            let depth = n.depth.clamp(1.0, 20.0);
            d.line(vec![a, [a[0] - v[0] * depth, a[1] - v[1] * depth]], false, 0.5, None, 0.0);
        }
    }
    let centre = polygon::centroid(&outline);
    let (lo, hi) = outline.iter().fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])]));
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    match panel.grain_line {
        Some([a, b]) => arrow(d, place.to_world(a), place.to_world(b)),
        None => {
            let angle = (panel.rotation + if panel.mirror { 180.0 - panel.grain } else { panel.grain }).to_radians();
            let half = (w.max(h) * 0.25).clamp(10.0, 150.0);
            arrow(d, [centre[0] - angle.cos() * half, centre[1] - angle.sin() * half], [centre[0] + angle.cos() * half, centre[1] + angle.sin() * half]);
        }
    }
    let name = if panel.name.is_empty() { format!("Panel {}", panel.id) } else { panel.name.clone() };
    let size = (w.min(h) * 0.08).clamp(4.0, 10.0);
    d.text([centre[0], centre[1] + size * 1.5], size, name, Anchor::Middle);
    d.text([centre[0], centre[1] - size * 2.5], size * 0.7, format!("cut {}", if panel.mirror { "mirrored" } else { "1" }), Anchor::Middle);
    d.text([lo[0] + w * 0.5, lo[1] - 8.0], 4.0, format!("W {w:.1} mm"), Anchor::Middle);
    d.text([hi[0] + 3.0, lo[1] + h * 0.5], 4.0, format!("H {h:.1} mm"), Anchor::Start);
}

fn seams(g: &Garment, d: &mut Drawing) {
    for seam in &g.seams {
        for (side, ends) in [("a", &seam.a), ("b", &seam.b)] {
            for end in ends.iter() {
                let Some(line) = world_path(g, std::slice::from_ref(end)) else { continue };
                if line.len() < 2 {
                    continue;
                }
                d.line(line.clone(), false, 1.1, None, 0.45);
                let at = mid(&line);
                d.text([at[0] + 2.0, at[1] + 2.0], 3.5, format!("S{}{side} {:.1}", seam.id, length(&line)), Anchor::Start);
            }
        }
    }
}

fn stitches(g: &Garment, d: &mut Drawing) {
    for stitch in &g.stitches {
        let Some(style) = g.styles.iter().find(|s| s.id == stitch.style).filter(|s| s.kind == StyleKind::Topstitch) else { continue };
        let Some(panel) = stitch.spans.first().and_then(|e| g.panels.iter().find(|p| p.id == e.panel)) else { continue };
        let Some(line) = world_path(g, &stitch.spans) else { continue };
        let count = param(style, "count", 1.0).clamp(1.0, 4.0) as usize;
        let base = if stitch.offset.abs() > 1.0e-6 { stitch.offset.abs() } else { param(style, "offset", 0.0).abs().max(3.0) };
        for k in 0..count {
            d.line(offset_inside(&line, panel, base + k as f64 * 2.5), false, 0.35, Some((2.0, 1.5)), 0.0);
        }
    }
}

fn zippers(g: &Garment, d: &mut Drawing) {
    for zip in &g.zippers {
        let width = g.styles.iter().find(|s| s.id == zip.style).map_or(10.0, |s| param(s, "width", 10.0)).clamp(2.0, 40.0);
        for ends in [&zip.a, &zip.b] {
            let Some(line) = world_path(g, ends) else { continue };
            d.line(line.clone(), false, 0.5, None, 0.0);
            for (k, (p, dir)) in resample(&line, TEETH_PITCH).into_iter().enumerate() {
                let s = if k % 2 == 0 { 1.0 } else { -1.0 };
                let (nx, ny) = (-dir[1], dir[0]);
                let reach = width * 0.25;
                d.line(vec![[p[0] + nx * reach * s * 0.2, p[1] + ny * reach * s * 0.2], [p[0] + nx * reach * s, p[1] + ny * reach * s]], false, 0.3, None, 0.0);
            }
        }
        if let Some(line) = world_path(g, &zip.a) {
            let at = mid(&line);
            d.text([at[0] + width * 0.5, at[1]], 3.5, format!("ZIP {} mm", length(&line).round()), Anchor::Start);
        }
    }
}

fn hardware(g: &Garment, d: &mut Drawing) {
    for item in &g.hardware {
        let Some(style) = g.styles.iter().find(|s| s.id == item.style) else { continue };
        let Some(panel) = g.panels.iter().find(|p| p.id == item.anchor.panel) else { continue };
        let at = panel.placement().to_world(item.anchor.point);
        match style.kind {
            StyleKind::Button => {
                let r = param(style, "width", 12.0).clamp(2.0, 60.0) * 0.5;
                d.circle(at, r, 0.5, false);
                d.circle(at, r * 0.75, 0.25, false);
                for (dx, dy) in [(-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2)] {
                    d.circle([at[0] + r * dx, at[1] + r * dy], 0.6, 0.2, true);
                }
            }
            StyleKind::Buttonhole => {
                let len = param(style, "length", 14.0).clamp(2.0, 80.0);
                let angle = (panel.rotation + item.rotation).to_radians();
                let (ux, uy) = (angle.cos(), angle.sin());
                let a = [at[0] - ux * len * 0.5, at[1] - uy * len * 0.5];
                let b = [at[0] + ux * len * 0.5, at[1] + uy * len * 0.5];
                d.line(vec![a, b], false, 1.0, None, 0.0);
                d.line(vec![[a[0] - uy * 1.2, a[1] + ux * 1.2], [a[0] + uy * 1.2, a[1] - ux * 1.2]], false, 0.5, None, 0.0);
                d.line(vec![[b[0] - uy * 1.2, b[1] + ux * 1.2], [b[0] + uy * 1.2, b[1] - ux * 1.2]], false, 0.5, None, 0.0);
            }
            _ => {}
        }
    }
}

fn summary_lines(g: &Garment, panels: usize) -> Vec<String> {
    let seams = g.seams.len();
    let fabrics = g.fabrics.len();
    vec![
        "formfabric schematic".to_string(),
        format!("{panels} panels, {seams} seams, {} stitch lines, {} fastenings, {} zippers, {fabrics} fabrics, units mm", g.stitches.len(), g.hardware.len(), g.zippers.len()),
    ]
}

pub(crate) fn drawing(s: &mut Session) -> Result<(Drawing, usize), CommandError> {
    let mut d = Drawing::default();
    let g = &s.garment;
    let shown: Vec<&Panel> = g.panels.iter().filter(|p| !p.state.disabled && p.outline.len() >= 3).collect();
    if shown.is_empty() {
        return Err(CommandError::new("invalid_geometry", "no panel has an outline to draw"));
    }
    for panel in &shown {
        panel_drawing(panel, &mut d);
    }
    seams(g, &mut d);
    stitches(g, &mut d);
    zippers(g, &mut d);
    hardware(g, &mut d);
    let bounds = d.bounds().unwrap_or(([0.0; 2], [0.0; 2]));
    let top = bounds.1[1] + 12.0;
    let summary = summary_lines(g, shown.len());
    let count = shown.len();
    for (i, line) in summary.iter().enumerate() {
        d.text([bounds.0[0], top + (summary.len() - i) as f64 * 7.0], if i == 0 { 6.0 } else { 4.0 }, line.clone(), Anchor::Start);
    }
    if let Ok(list) = measure::live(s) {
        for (i, m) in list.iter().enumerate() {
            let value = m.get("value").and_then(Value::as_f64).map_or("n/a".to_string(), |v| format!("{v:.1} mm"));
            let name = m.get("name").and_then(Value::as_str).unwrap_or("measure");
            let kind = m.get("kind").and_then(Value::as_str).unwrap_or("");
            d.text([bounds.0[0], bounds.0[1] - 20.0 - i as f64 * 6.0], 4.0, format!("{name} ({kind}): {value}"), Anchor::Start);
        }
    }
    Ok((d, count))
}

fn export_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Absolute output path ending in .svg or .pdf (matching format)."},
            "format": {"type": "string", "enum": ["svg", "pdf"], "default": "svg"}
        },
        "required": ["path"],
        "additionalProperties": false
    })
}

fn export(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let format = a.get("format").and_then(Value::as_str).unwrap_or("svg");
    if format != "svg" && format != "pdf" {
        return Err(bad("format must be svg or pdf"));
    }
    let path = args::file_path(a, "path", Some(format))?;
    let (d, panels) = drawing(s)?;
    let bytes = if format == "svg" { d.svg("formfabric schematic").into_bytes() } else { d.pdf() };
    args::write_atomic(&path, &bytes)?;
    let (w, h) = d.size();
    Ok(json!({"path": path, "format": format, "bytes": bytes.len(), "panels": panels, "width_mm": (w * 10.0).round() / 10.0, "height_mm": (h * 10.0).round() / 10.0, "primitives": d.prims.len()}))
}

pub(crate) fn commands() -> Vec<Command> {
    vec![Command {
        id: "schematic.export",
        mcp: "schematic_export",
        group: Group::Export,
        label: "Export Schematic",
        description: "Write a flat technical drawing of the garment as SVG or PDF (vector, 1 unit = 1 mm): panel outlines, internal lines (folds dashed), notches, grain lines, seams with ids and lengths, topstitch dashes, zippers, buttons and buttonholes from the trims, panel dimensions and the 3D measures. The file is replaced atomically.",
        keys: &[],
        hints: EXTERNAL,
        schema: export_schema,
        run: export,
    }]
}
