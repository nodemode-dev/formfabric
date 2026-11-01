use ffb_fabric::lab::{self, LabInput};
use ffb_fabric::presets;
use serde_json::{json, Value};

use super::common::*;
use super::schema as sc;
use crate::{Command, CommandError, Group, Session, WRITE};

fn num(unit: &str, lo: f64, hi: f64, description: &str) -> Value {
    json!({"type": "number", "unit": unit, "minimum": lo, "maximum": hi, "description": description})
}

fn directions(inner: fn(&str) -> Value, label: &str) -> Value {
    sc::obj(json!({"warp": inner(label), "weft": inner(label), "bias": inner(label)}), &[])
}

fn length(label: &str) -> Value {
    num("cm", 0.1, 50.0, &format!("Cantilever bending length c ({label}): half the overhang at which the strip tip reaches 41.5 degrees below horizontal."))
}

fn tensile(label: &str) -> Value {
    sc::obj(
        json!({
            "force": num("N", 0.001, 10_000.0, &format!("Force measured on the strip ({label}).")),
            "strain": num("percent", 0.01, 100.0, "Strain reached at that force."),
            "width": num("mm", 1.0, 500.0, "Strip width, default 50.")
        }),
        &["force", "strain"],
    )
}

fn schema() -> Value {
    let mut tensile_dirs = directions(tensile, "tensile strip");
    if let Some(props) = tensile_dirs["properties"].as_object_mut() {
        props.remove("bias");
    }
    sc::obj(
        json!({
            "fabric": sc::fabric_id("Fabric whose current values fill the fields not measured, and the target of apply."),
            "apply": {"type": "boolean", "default": false, "description": "Write the computed values into the fabric as one undo step."},
            "density": num("g/m2", 1.0, 3000.0, "Areal density of the cloth."),
            "thickness": num("mm", 0.01, 10.0, "Thickness of the cloth."),
            "bending_length": directions(length, "cantilever test"),
            "tensile": tensile_dirs,
            "shear": sc::obj(
                json!({
                    "angle": num("deg", 0.1, 60.0, "Shear angle reached in the test."),
                    "force": num("N", 0.0001, 10_000.0, "Force at that angle."),
                    "length": num("mm", 1.0, 1000.0, "Length of the frame side the force acts along, default 100.")
                }),
                &["angle", "force"],
            ),
            "bias_extension": num("percent", 0.01, 100.0, "Bias extension EB5 at a load of 5 gf/cm (4.903 N/m) on a 45 degree strip; gives shear stiffness 123 / EB5 when no shear test is given."),
            "friction": num("ratio", 0.0, 1.0, "Measured friction coefficient."),
            "damping": num("ratio", 0.0, 1.0, "Internal damping to use."),
            "buckling_ratio": num("ratio", 0.0, 1.0, "Buckling ratio to use."),
            "buckling_stiffness": num("ratio", 0.0, 1.0, "Buckling stiffness to use.")
        }),
        &[],
    )
}

fn run(s: &mut Session, a: &Value) -> Result<Value, CommandError> {
    let fabric = opt_id(a, "fabric")?;
    let apply = a.get("apply").and_then(Value::as_bool).unwrap_or(false);
    if apply && fabric.is_none() {
        return Err(bad("apply needs the fabric to write to"));
    }
    let mut inputs = a.clone();
    if let Some(map) = inputs.as_object_mut() {
        map.remove("fabric");
        map.remove("apply");
    }
    let input: LabInput = serde_json::from_value(inputs).map_err(|e| bad(format!("measurements are not valid: {e}")))?;
    let base = match fabric {
        Some(id) => fabric_of(&s.garment, id)?.physical.clone(),
        None => presets::default_preset().physical,
    };
    let result = lab::run(&input, &base).map_err(bad)?;
    if result.computed.is_empty() {
        return Err(bad("give at least one measurement"));
    }
    let mut out = serde_json::to_value(&result).map_err(|e| CommandError::new("io", e.to_string()))?;
    if apply {
        let id = fabric.unwrap_or(0);
        let physical = result.physical.clone();
        let applied = mutate(s, "Apply Fabric Lab", |g| {
            let i = fabric_index(g, id)?;
            g.fabrics[i].physical = physical;
            Ok(json!({}))
        })?;
        out["applied"] = json!(true);
        out["changed"] = applied["changed"].clone();
    } else {
        out["applied"] = json!(false);
        out["changed"] = json!(false);
    }
    Ok(out)
}

pub(super) fn commands() -> Vec<Command> {
    vec![Command {
        id: "fabric.lab",
        mcp: "fabric_lab",
        group: Group::Fabric,
        label: "Fabric Lab",
        description: "Turn bench measurements (areal density, thickness, cantilever bending lengths, tensile strip results, shear test) into physical properties, with the formula and source keys behind every value. Fields not measured keep the fabric's values. With apply the result is written to the fabric.",
        keys: &[],
        hints: WRITE,
        schema,
        run,
    }]
}
