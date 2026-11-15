use ffb_core::{IdGen, Units};
use ffb_io::aama::{self, Flavour, Imported, ImportOptions};
use ffb_pattern::geom::*;
use serde_json::{json, Value};

use crate::{args, Command, CommandError, Edit, Group, Session, EXTERNAL};

const GAP: f64 = 50.0;
const ROW_WIDTH: f64 = 3000.0;

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Path of the DXF file to read. ASCII DXF, release 12 or later."},
            "units": {"type": "string", "enum": ["mm", "cm", "in"], "description": "Unit of the numbers in the file. Omit to detect it: AAMA files declare Units METRIC or ENGLISH, other files may carry the header variable INSUNITS, and millimetres is the fallback."},
            "flavour": {"type": "string", "enum": ["auto", "aama", "plain"], "default": "auto", "description": "aama reads one pattern piece per block with the ASTM D6673 layer numbers (1 cut line, 2 turn points, 3 curve points, 4 notches, 7 grain line, 8 internal lines, 14 sew line). plain reads closed outlines from lines, polylines, arcs, circles, splines and inserted blocks. auto picks aama when the file carries Piece Name or Style Name text."},
            "mode": {"type": "string", "enum": ["add", "as_new"], "default": "add", "description": "add appends the pieces to the pattern to the right of what is there. as_new replaces all panels of the pattern with the imported ones. Either is one undo step."},
            "fit_tolerance": {"type": "number", "minimum": 0.005, "maximum": 1.0, "default": 0.05, "description": "Largest distance in mm between a point of the file and the Bezier curve fitted through the curve points."},
            "smooth_facets": {"type": "boolean", "default": true, "description": "Plain files only. Treat runs of short polyline edges that turn by less than 15 degrees at each vertex as curves, so a curve that another program flattened comes back as a few Bezier segments instead of many corners."},
            "use_sew_line": {"type": "boolean", "default": true, "description": "When a piece has a closed sew line (layer 14) inside its cut line (layer 1), make the sew line the panel outline and turn the gap into the seam allowance."}
        },
        "required": ["path"],
        "additionalProperties": false
    })
}

fn flag(a: &Value, key: &str, default: bool) -> bool {
    a.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn existing_bounds(s: &Session) -> Option<Bounds> {
    s.garment.panels.iter().filter_map(|p| p.world_bounds()).reduce(union)
}

fn imported_bounds(imp: &Imported) -> Option<Bounds> {
    imp.panels.iter().filter_map(|p| p.world_bounds()).reduce(union)
}

fn lay_out(imp: &mut Imported, existing: Option<Bounds>) {
    if imp.keep_positions {
        let (Some(ex), Some(b)) = (existing, imported_bounds(imp)) else { return };
        let (dx, dy) = (ex[1][0] + GAP - b[0][0], ex[0][1] - b[0][1]);
        for p in imp.panels.iter_mut() {
            p.position = [p.position[0] + dx, p.position[1] + dy];
        }
        return;
    }
    let start = existing.map_or(0.0, |e| e[1][0] + GAP);
    let (mut x, mut y, mut row) = (start, existing.map_or(0.0, |e| e[0][1]), 0.0f64);
    for p in imp.panels.iter_mut() {
        let Some(b) = p.world_bounds() else { continue };
        let (w, h) = (b[1][0] - b[0][0], b[1][1] - b[0][1]);
        if x > start && x - start + w > ROW_WIDTH {
            x = start;
            y += row + GAP;
            row = 0.0;
        }
        p.position = [p.position[0] + x - b[0][0], p.position[1] + y - b[0][1]];
        x += w + GAP;
        row = row.max(h);
    }
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::file_path(a, "path", None)?;
    let bytes = std::fs::read(&path).map_err(|e| CommandError::new(if e.kind() == std::io::ErrorKind::NotFound { "not_found" } else { "io" }, format!("could not read {path}: {e}")))?;
    let text = String::from_utf8_lossy(&bytes);
    let flavour = match a.get("flavour").and_then(Value::as_str) {
        Some("aama") => Some(Flavour::Aama),
        Some("plain") => Some(Flavour::Plain),
        _ => None,
    };
    let mut opts = ImportOptions { flavour, ..ImportOptions::default() };
    if let Some(u) = a.get("units").and_then(Value::as_str) {
        opts.scale = Some(Units::parse(u).ok_or_else(|| CommandError::new("bad_args", format!("unknown units {u}")))?.mm_per_unit());
    }
    if let Some(t) = a.get("fit_tolerance").and_then(Value::as_f64) {
        if !(0.005..=1.0).contains(&t) {
            return Err(CommandError::new("bad_args", "fit_tolerance must be between 0.005 and 1 mm"));
        }
        opts.fit_tolerance = t;
    }
    opts.sew_allowance = flag(a, "use_sew_line", true);
    opts.smooth_facets = flag(a, "smooth_facets", true);
    let as_new = a.get("mode").and_then(Value::as_str) == Some("as_new");
    let mut imp = aama::import(&text, &opts).map_err(|m| CommandError::new("open_failed", format!("{path}: {m}")))?;

    lay_out(&mut imp, if as_new { None } else { existing_bounds(s) });
    let used: Vec<u32> = s.garment.panels.iter().flat_map(|p| std::iter::once(p.id).chain(p.inner.iter().map(|i| i.id))).collect();
    let mut ids = IdGen::starting_after(used);
    for p in imp.panels.iter_mut() {
        p.id = ids.next_id();
        for i in p.inner.iter_mut() {
            i.id = ids.next_id();
        }
    }

    let report: Vec<Value> = imp
        .panels
        .iter()
        .map(|p| {
            json!({
                "id": p.id, "name": p.name, "points": p.outline.len(), "notches": p.notches.len(), "inner": p.inner.len(),
                "grain_line": p.grain_line.is_some(), "seam_allowance": p.seam_allowance.as_ref().map(|sa| sa.width),
                "position": p.position
            })
        })
        .collect();
    let before = s.garment.clone();
    if as_new {
        let mut after = before.clone();
        after.panels = imp.panels.clone();
        after.selection = Value::Object(Default::default());
        s.garment = after.clone();
        s.record(Edit::Garment { label: "Import DXF".into(), before: Box::new(before), after: Box::new(after) });
    } else {
        let base = s.garment.panels.len();
        let mut edits = Vec::new();
        for (i, p) in imp.panels.iter().enumerate() {
            s.garment.panels.push(p.clone());
            edits.push(Edit::Panel { label: "Import DXF".into(), index: base + i, before: None, after: Some(Box::new(p.clone())) });
        }
        if edits.len() == 1 {
            s.record(edits.remove(0));
        } else {
            s.record(Edit::Many { label: "Import DXF".into(), edits });
        }
    }
    Ok(json!({
        "path": path, "flavour": imp.flavour.name(), "mode": if as_new { "as_new" } else { "add" },
        "scale_to_mm": imp.scale, "imported": report.len(), "panels": report, "notes": imp.notes
    }))
}

pub(super) fn command() -> Command {
    Command {
        id: "exchange.import_dxf",
        mcp: "exchange_import_dxf",
        group: Group::File,
        label: "Import DXF",
        description: "Read a DXF pattern file and create panels from it as one undo step. AAMA files (ASTM D6673 layers) give panels with name, outline (curve points on layer 3 become smooth Bezier handles), notches (layers 4, 80 to 83), grain line (7), internal lines (8, role reference) and, when a sew line (14) lies inside the cut line (1), the sew line as outline with the gap as seam allowance. Plain DXF gives a panel per closed outline from lines, polylines, arcs, circles, splines and inserted blocks, with open lines inside it as internal lines. Reports what was skipped.",
        keys: &[],
        hints: EXTERNAL,
        schema,
        run,
    }
}
