use crate::Body;

#[derive(Debug, Clone, PartialEq)]
pub struct BodyChange {
    pub id: u32,
    pub index: usize,
    pub before: Option<Box<Body>>,
    pub after: Option<Box<Body>>,
    pub light: bool,
}

pub fn light(body: &Body) -> Body {
    Body {
        positions: Vec::new(),
        normals: Vec::new(),
        triangles: Vec::new(),
        joints: Vec::new(),
        weights: Vec::new(),
        ..body.clone()
    }
}

fn restore_light(target: &mut Body, snap: &Body) {
    target.name = snap.name.clone();
    target.source = snap.source.clone();
    target.license = snap.license.clone();
    target.pose = snap.pose.clone();
    target.measurements = snap.measurements.clone();
    target.volumes = snap.volumes.clone();
    target.points = snap.points.clone();
    target.skin_offset = snap.skin_offset;
    target.friction = snap.friction;
    target.visible = snap.visible;
}

pub fn apply(bodies: &mut Vec<Body>, changes: &[BodyChange], forward: bool) {
    let mut run = |c: &BodyChange| {
        let (from, to) = if forward { (&c.before, &c.after) } else { (&c.after, &c.before) };
        match (from, to) {
            (None, Some(b)) => {
                let at = c.index.min(bodies.len());
                bodies.insert(at, (**b).clone());
            }
            (Some(_), None) => bodies.retain(|b| b.id != c.id),
            (Some(_), Some(b)) => {
                if let Some(t) = bodies.iter_mut().find(|t| t.id == c.id) {
                    if c.light {
                        restore_light(t, b);
                    } else {
                        *t = (**b).clone();
                    }
                }
            }
            (None, None) => {}
        }
    };
    if forward {
        changes.iter().for_each(&mut run);
    } else {
        changes.iter().rev().for_each(&mut run);
    }
}
