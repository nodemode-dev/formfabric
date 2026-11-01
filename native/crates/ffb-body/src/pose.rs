use std::collections::BTreeMap;

use crate::math::*;
use crate::measure;
use crate::skin;
use crate::Body;

pub const NAMES: [&str; 3] = ["rest", "A", "T"];
const A_ANGLE_DEGREES: f32 = 30.0;

pub fn named(body: &Body, name: &str) -> Result<BTreeMap<String, Q>, String> {
    let key = name.trim().to_lowercase();
    if key == "rest" || key == "bind" {
        return Ok(BTreeMap::new());
    }
    let tee = match key.as_str() {
        "t" => true,
        "a" => false,
        _ => return Err(format!("no pose named {name}; the poses are rest, A and T")),
    };
    let bind = skin::bind_frames(body);
    let chains = measure::arm_chains(&body.joints, &bind.pos);
    if chains.is_empty() {
        return Err("this body has no arm joints to pose".into());
    }
    let angle = A_ANGLE_DEGREES.to_radians();
    let mut out = BTreeMap::new();
    for c in chains {
        let dir = sub(bind.pos[c.elbow], bind.pos[c.shoulder]);
        let sx = if bind.pos[c.shoulder][0] >= 0.0 { 1.0 } else { -1.0 };
        let target = if tee { [sx, 0.0, 0.0] } else { [sx * angle.sin(), -angle.cos(), 0.0] };
        let world = qarc(dir, target);
        let rb = bind.rot[c.shoulder];
        let local = qnorm(qmul(qconj(rb), qmul(world, rb)));
        if (local[3].abs() - 1.0).abs() > 1e-6 {
            out.insert(body.joints[c.shoulder].name.clone(), local);
        }
    }
    Ok(out)
}
