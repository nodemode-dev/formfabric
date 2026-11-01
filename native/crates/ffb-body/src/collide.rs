use ffb_sim::Collider;

use crate::skin;
use crate::Body;

pub fn collider(body: &Body) -> Collider {
    Collider::Mesh { positions: skin::posed_positions(body), triangles: body.triangles.clone() }
}
