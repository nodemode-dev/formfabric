use ffb_io::export::{self, Built, Options, Thickness};

use super::scene::{collect, effective_islands, materials, source_objects};
use crate::{CommandError, Session};

pub(crate) struct LinkScene {
    pub built: Built,
    pub ids: Vec<u32>,
    pub counts: Vec<usize>,
    pub raw: Vec<[f32; 3]>,
    pub options: Options,
}

pub(crate) fn link_scene(s: &mut Session, stem: &str, thickness: bool, z_up: bool) -> Result<LinkScene, CommandError> {
    let geos = collect(s, None)?;
    let thick = thickness.then_some(Thickness { amount: s.garment.output.thickness.amount.max(1.0), sides: s.garment.output.thickness.sides });
    let (islands, _) = effective_islands(&s.garment, &geos);
    let mats = materials(&s.garment, &geos, thick.is_some());
    let objects = source_objects(&geos, &islands, &mats, "packed");
    let built = export::build(&objects, mats.materials.clone(), thick.as_ref()).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let ids = geos.iter().map(|g| g.id).collect();
    let counts = geos.iter().map(|g| g.pos3.len()).collect();
    let raw = geos.iter().flat_map(|g| g.pos3.iter().copied()).collect();
    let options = Options { stem: stem.to_string(), z_up, fps: s.garment.anim.fps, ..Options::default() };
    Ok(LinkScene { built, ids, counts, raw, options })
}
