use ffb_core::Units;
use ffb_io::aama::{self, ExportOptions, ExportPanel, Flavour};
use serde_json::{json, Value};

use crate::{args, Command, CommandError, Group, Session, EXTERNAL};

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Path of the DXF file to write."},
            "panels": {"type": "array", "items": {"type": "integer"}, "description": "Panel ids to export. Omit to export every panel."},
            "flavour": {"type": "string", "enum": ["aama", "plain"], "default": "aama", "description": "aama writes one block per piece with the ASTM D6673 layer numbers and system text. plain writes one drawing in the layout positions on named layers CUT, SEW, NOTCH, GRAIN, INTERNAL and NAME."},
            "seam_allowance": {"type": "boolean", "default": true, "description": "Write the panel outline plus its seam allowance as the cut line (layer 1) and the outline itself as the sew line (layer 14). When off, or when a panel has no allowance, the outline is the cut line."},
            "notches": {"type": "boolean", "default": true, "description": "Write notches."},
            "grain_line": {"type": "boolean", "default": true, "description": "Write grain lines (layer 7)."},
            "inner_lines": {"type": "boolean", "default": true, "description": "Write internal lines and shapes (layer 8). A text label at the start of each line keeps its role, fold angle and fold strength."},
            "internal_labels": {"type": "boolean", "default": true, "description": "Write the annotation text with the piece name (layer 15) and the role labels of internal lines."},
            "units": {"type": "string", "enum": ["mm", "cm", "in"], "default": "mm", "description": "Unit of the numbers in the file. AAMA files accept mm and in."},
            "tolerance": {"type": "number", "minimum": 0.005, "maximum": 5.0, "default": 0.1, "description": "Largest distance in mm between a curve and the polyline written for it."}
        },
        "required": ["path"],
        "additionalProperties": false
    })
}

fn flag(a: &Value, key: &str) -> bool {
    a.get(key).and_then(Value::as_bool).unwrap_or(true)
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let path = args::string(a, "path")?.to_string();
    let mut opts = ExportOptions {
        flavour: if a.get("flavour").and_then(Value::as_str) == Some("plain") { Flavour::Plain } else { Flavour::Aama },
        seam_allowance: flag(a, "seam_allowance"),
        notches: flag(a, "notches"),
        grain_line: flag(a, "grain_line"),
        inner_lines: flag(a, "inner_lines"),
        labels: flag(a, "internal_labels"),
        ..ExportOptions::default()
    };
    if let Some(u) = a.get("units").and_then(Value::as_str) {
        opts.unit_mm = Units::parse(u).ok_or_else(|| CommandError::new("bad_args", format!("unknown units {u}")))?.mm_per_unit();
    }
    if let Some(t) = a.get("tolerance").and_then(Value::as_f64) {
        if !(0.005..=5.0).contains(&t) {
            return Err(CommandError::new("bad_args", "tolerance must be between 0.005 and 5 mm"));
        }
        opts.tolerance = t;
    }
    opts.style = std::path::Path::new(&path).file_stem().and_then(|x| x.to_str()).unwrap_or("formfabric").to_string();

    let chosen: Vec<usize> = match a.get("panels").and_then(Value::as_array) {
        None => (0..s.garment.panels.len()).collect(),
        Some(list) => {
            let mut out = Vec::new();
            for v in list {
                let id = v.as_u64().ok_or_else(|| CommandError::new("bad_args", "panels must be integer ids"))? as u32;
                let index = s.garment.index_of(id).ok_or_else(|| CommandError::new("not_found", format!("no panel with id {id}")))?;
                if !out.contains(&index) {
                    out.push(index);
                }
            }
            out
        }
    };
    if chosen.is_empty() {
        return Err(CommandError::new("bad_args", "the pattern has no panels to export"));
    }
    let items: Vec<ExportPanel> = chosen
        .iter()
        .map(|i| {
            let panel = &s.garment.panels[*i];
            let category = panel.fabric.and_then(|f| s.garment.fabrics.iter().find(|x| x.id == f)).map(|f| f.name.clone());
            ExportPanel { panel, category }
        })
        .collect();
    let done = aama::export(&items, &opts).map_err(|m| CommandError::new("invalid_geometry", m))?;
    std::fs::write(&path, done.text.as_bytes()).map_err(|e| CommandError::new("io", format!("could not write {path}: {e}")))?;
    Ok(json!({
        "path": path, "flavour": opts.flavour.name(), "panels": done.panels, "notches": done.notches,
        "inner_lines": done.inner_lines, "grain_lines": done.grain_lines, "vertices": done.vertices, "bytes": done.text.len()
    }))
}

pub(super) fn command() -> Command {
    Command {
        id: "exchange.export_dxf",
        mcp: "exchange_export_dxf",
        group: Group::File,
        label: "Export DXF",
        description: "Write panels to an ASCII DXF file (release 12 entities). The aama flavour follows ASTM D6673: one block per piece with Piece Name, Quantity and Category text, the cut line on layer 1 (outline plus seam allowance when there is one) and the outline as sew line on layer 14, curves flattened to polylines within the tolerance with turn points on layer 2 and curve points on layer 3, notches on layers 4, 80 and 83, grain line on layer 7, internal lines on layer 8 and an annotation on layer 15. The plain flavour writes world-space polylines on named layers. Importing the aama file again restores outline, notches, grain line, internal lines and seam allowance width.",
        keys: &[],
        hints: EXTERNAL,
        schema,
        run,
    }
}
