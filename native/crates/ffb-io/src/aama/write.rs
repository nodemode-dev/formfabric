#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use ffb_pattern::geom::*;
use ffb_pattern::{path, polygon};
use ffb_pattern::{Inner, Notch, NotchKind, Panel};

use super::offset::{self, Trace};
use super::piece::classify;
use super::{ExportOptions, ExportPanel, Exported, Flavour};
use crate::dxf::{Sink, Writer};

struct Prep<'a> {
    panel: &'a Panel,
    net: Trace,
    cut: Option<Trace>,
    widths: Vec<f64>,
    side: f64,
}

fn prepare<'a>(p: &'a Panel, opts: &ExportOptions) -> Result<Prep<'a>, String> {
    if p.outline.len() < 3 {
        return Err(format!("panel {} has no outline to export", p.name));
    }
    let limit = crate::dxf::MAX_COORD_MM;
    let within = |v: f64| v.is_finite() && v.abs() <= limit;
    if !p.position.iter().all(|v| within(*v)) || !p.outline.iter().all(|pt| pt.p.iter().all(|v| within(*v))) {
        return Err(format!("panel {} has a coordinate outside plus or minus {limit} mm or not a number", p.name));
    }
    let net = offset::trace(&p.outline, true, opts.tolerance);
    if net.pts.len() < 3 {
        return Err(format!("panel {} has a degenerate outline", p.name));
    }
    if net.pts.len() > crate::dxf::MAX_VERTS {
        return Err(format!("panel {} would write {} vertices; the limit is {}", p.name, net.pts.len(), crate::dxf::MAX_VERTS));
    }
    let mut widths = Vec::new();
    let mut cut = None;
    if let Some(sa) = p.seam_allowance.as_ref().filter(|_| opts.seam_allowance) {
        widths = (0..p.outline.len()).map(|i| sa.per_segment.get(i).copied().flatten().unwrap_or(sa.width)).collect();
        if widths.iter().any(|w| w.abs() > 1e-9) {
            let spec = ffb_pattern::offset::Spec { widths: widths.clone(), corners: sa.corners, miter_limit: ffb_pattern::offset::MITER_LIMIT };
            let traced = match ffb_pattern::offset::offset_closed(&p.outline, &spec) {
                Ok(points) => offset::trace(&points, true, opts.tolerance),
                Err(_) => offset::offset(&net, &widths, sa.corners, opts.tolerance),
            };
            if traced.pts.len() >= 3 {
                cut = Some(traced);
            }
        }
    }
    let side = ffb_pattern::notch::outward_side(&p.outline);
    Ok(Prep { panel: p, net, cut, widths, side })
}

fn clean_text(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

fn unit_vec(v: V2, fallback: V2) -> V2 {
    unit(v).unwrap_or(fallback)
}

fn notch_frame(prep: &Prep, n: &Notch) -> Option<(V2, V2)> {
    if n.segment >= prep.panel.outline.len() {
        return None;
    }
    let seg = path::seg(&prep.panel.outline, n.segment);
    let t = n.t.clamp(0.0, 1.0);
    let on = seg.at(t);
    let tangent = unit_vec(seg.tangent(t), unit_vec(sub(seg.p[3], seg.p[0]), [1.0, 0.0]));
    let w = if prep.cut.is_some() { prep.widths.get(n.segment).copied().unwrap_or(0.0) } else { 0.0 };
    let base = add(on, mul([tangent[1], -tangent[0]], w * prep.side));
    Some((base, rotate(tangent, n.angle * prep.side)))
}

fn inner_label(inner: &Inner) -> Option<String> {
    let default_kind = classify(inner.points.len(), inner.closed);
    let needs = inner.role != ffb_pattern::InnerRole::Reference || inner.kind != default_kind || (inner.fold_angle - 180.0).abs() > 1e-9 || inner.fold_strength.abs() > 1e-9;
    needs.then(|| {
        let role = format!("{:?}", inner.role).to_ascii_lowercase();
        let kind = format!("{:?}", inner.kind).to_ascii_lowercase();
        format!("FF:role={role};kind={kind};angle={};strength={}", inner.fold_angle, inner.fold_strength)
    })
}

fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(not(target_arch = "wasm32"))]
fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

#[cfg(target_arch = "wasm32")]
fn now_secs() -> i64 {
    0
}

fn stamp() -> (String, String) {
    let secs = now_secs();
    let (y, m, d) = civil(secs.div_euclid(86_400));
    let rem = secs.rem_euclid(86_400);
    (format!("{d:02}-{m:02}-{y:04}"), format!("{:02}-{:02}", rem / 3600, rem % 3600 / 60))
}

fn block_name(name: &str, used: &mut Vec<String>) -> String {
    let mut base: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
    if base.is_empty() {
        base = "piece".into();
    }
    base.truncate(26);
    let mut candidate = base.clone();
    let mut n = 1;
    while used.contains(&candidate.to_ascii_uppercase()) {
        n += 1;
        candidate = format!("{base}_{n}");
    }
    used.push(candidate.to_ascii_uppercase());
    candidate
}

fn marks(sink: &mut Sink, trace: &Trace, k: f64) -> usize {
    for (p, turn) in trace.pts.iter().zip(&trace.turn) {
        sink.point(if *turn { "2" } else { "3" }, [p[0] * k, p[1] * k], 0.0, None, None);
    }
    trace.pts.len()
}

fn scaled(trace: &Trace, k: f64) -> Vec<V2> {
    trace.pts.iter().map(|p| [p[0] * k, p[1] * k]).collect()
}

fn aama_piece(sink: &mut Sink, prep: &Prep, category: &Option<String>, opts: &ExportOptions, out: &mut Exported) {
    let k = 1.0 / opts.unit_mm;
    let s = |p: V2| [p[0] * k, p[1] * k];
    let panel = prep.panel;
    let boundary = prep.cut.as_ref().unwrap_or(&prep.net);
    let height = 2.5 * k;
    let anchor = s(boundary.pts[0]);
    sink.text("1", anchor, height, &format!("Piece Name:{}", clean_text(&panel.name)));
    sink.text("1", anchor, height, if panel.mirror { "Quantity:0,1" } else { "Quantity:1,0" });
    if let Some(c) = category {
        sink.text("1", anchor, height, &format!("Category:{}", clean_text(c)));
    }
    sink.polyline("1", &scaled(boundary, k), true);
    out.vertices += marks(sink, boundary, k);
    if prep.cut.is_some() {
        sink.polyline("14", &scaled(&prep.net, k), true);
        out.vertices += marks(sink, &prep.net, k);
    }
    if opts.notches {
        for n in &panel.notches {
            let Some((base, dir)) = notch_frame(prep, n) else { continue };
            let angle = dir[1].atan2(dir[0]).to_degrees().rem_euclid(360.0);
            let (layer, width) = match n.kind {
                NotchKind::Slit => ("4", 0.0),
                NotchKind::V => ("4", n.width),
                NotchKind::T => ("80", n.width),
                NotchKind::U => ("83", n.width),
            };
            sink.point(layer, s(base), n.depth * k, Some(angle), Some(width * k));
            out.notches += 1;
        }
    }
    if opts.grain_line {
        if let Some(g) = panel.grain_line {
            sink.line("7", s(g[0]), s(g[1]));
            out.grain_lines += 1;
        }
    }
    if opts.inner_lines {
        for inner in panel.inner.iter().filter(|i| i.points.len() >= 2) {
            let tr = offset::trace(&inner.points, inner.closed, opts.tolerance);
            if tr.pts.len() < 2 {
                continue;
            }
            sink.polyline("8", &scaled(&tr, k), inner.closed);
            out.vertices += marks(sink, &tr, k);
            if opts.labels {
                if let Some(label) = inner_label(inner) {
                    sink.text("8", s(tr.pts[0]), height, &label);
                }
            }
            out.inner_lines += 1;
        }
    }
    if opts.labels {
        let c = polygon::centroid(&prep.net.pts);
        sink.text("15", s(c), (4.0 * k).max(height), &clean_text(&panel.name));
    }
}

fn plain_piece(sink: &mut Sink, prep: &Prep, opts: &ExportOptions, out: &mut Exported) {
    let k = 1.0 / opts.unit_mm;
    let panel = prep.panel;
    let place = panel.placement();
    let w = |p: V2| {
        let q = place.to_world(p);
        [q[0] * k, q[1] * k]
    };
    let boundary = prep.cut.as_ref().unwrap_or(&prep.net);
    let to_world = |t: &Trace| -> Vec<V2> { t.pts.iter().map(|p| w(*p)).collect() };
    sink.polyline("CUT", &to_world(boundary), true);
    out.vertices += boundary.pts.len();
    if prep.cut.is_some() {
        sink.polyline("SEW", &to_world(&prep.net), true);
        out.vertices += prep.net.pts.len();
    }
    if opts.notches {
        for n in &panel.notches {
            let Some((base, dir)) = notch_frame(prep, n) else { continue };
            let tip = add(base, mul(dir, n.depth));
            let layer = match n.kind {
                NotchKind::Slit => "NOTCH",
                NotchKind::V => "NOTCH_V",
                NotchKind::T => "NOTCH_T",
                NotchKind::U => "NOTCH_U",
            };
            sink.line(layer, w(base), w(tip));
            out.notches += 1;
        }
    }
    if opts.grain_line {
        if let Some(g) = panel.grain_line {
            sink.line("GRAIN", w(g[0]), w(g[1]));
            out.grain_lines += 1;
        }
    }
    if opts.inner_lines {
        for inner in panel.inner.iter().filter(|i| i.points.len() >= 2) {
            let tr = offset::trace(&inner.points, inner.closed, opts.tolerance);
            if tr.pts.len() < 2 {
                continue;
            }
            sink.polyline("INTERNAL", &to_world(&tr), inner.closed);
            out.vertices += tr.pts.len();
            out.inner_lines += 1;
        }
    }
    if opts.labels {
        let c = polygon::centroid(&prep.net.pts);
        sink.text("NAME", w(c), (4.0 * k).max(0.1), &clean_text(&panel.name));
    }
}

pub fn export(panels: &[ExportPanel], opts: &ExportOptions) -> Result<Exported, String> {
    if panels.is_empty() {
        return Err("there are no panels to export".into());
    }
    if opts.unit_mm <= 0.0 {
        return Err("the unit size must be positive".into());
    }
    let imperial = (opts.unit_mm - 25.4).abs() < 1e-9;
    if opts.flavour == Flavour::Aama && !imperial && (opts.unit_mm - 1.0).abs() > 1e-9 {
        return Err("AAMA files are written in millimetres or inches only".into());
    }
    let code = if imperial {
        1
    } else if (opts.unit_mm - 10.0).abs() < 1e-9 {
        5
    } else {
        4
    };
    let decimals = if opts.flavour == Flavour::Aama { if imperial { 4 } else { 2 } } else { 4 };
    let mut writer = Writer::new(decimals, code);
    let mut out = Exported::default();
    let mut used = Vec::new();
    let k = 1.0 / opts.unit_mm;
    if opts.flavour == Flavour::Aama {
        let (date, time) = stamp();
        let mut style = writer.sink();
        let lines = [
            format!("Style Name:{}", clean_text(&opts.style)),
            format!("Creation Date:{date}"),
            format!("Creation Time:{time}"),
            "Author:formfabric".to_string(),
            "Sample Size:Base".to_string(),
            format!("Units:{}", if imperial { "ENGLISH" } else { "METRIC" }),
            "Standard Version:ASTM D6673".to_string(),
        ];
        for (i, line) in lines.iter().enumerate() {
            style.text("1", [0.0, -(i as f64) * 4.0 * k], 2.5 * k, line);
        }
        writer.entities = style;
    }
    for item in panels {
        let prep = prepare(item.panel, opts)?;
        match opts.flavour {
            Flavour::Aama => {
                let mut sink = writer.sink();
                aama_piece(&mut sink, &prep, &item.category, opts, &mut out);
                let name = block_name(&item.panel.name, &mut used);
                writer.add_block(&name, sink);
            }
            Flavour::Plain => {
                let mut sink = std::mem::replace(&mut writer.entities, Sink::new(decimals));
                plain_piece(&mut sink, &prep, opts, &mut out);
                writer.entities = sink;
            }
        }
        out.panels += 1;
    }
    out.text = writer.finish();
    Ok(out)
}
