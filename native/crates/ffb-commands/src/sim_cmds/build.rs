use super::assemble::assemble;
use super::runtime::lock;
use crate::{CommandError, Session};

fn build(session: &mut Session, force: bool) -> Result<(), CommandError> {
    let core = session.sim.core.clone();
    {
        let mut c = lock(&core);
        c.sink = session.sink.clone();
        if !force && c.stepper.is_some() && !c.stale {
            return Ok(());
        }
    }
    let Session { garment, mesh, sim, bodies, .. } = session;
    let live: Vec<u32> = garment.panels.iter().map(|p| p.id).collect();
    sim.drape.forget_missing(&live);
    let assembled = assemble(garment, mesh, bodies, &sim.drape).map_err(|e| CommandError::new("invalid_geometry", e))?;
    lock(&core).install(assembled);
    Ok(())
}

pub(crate) fn ensure(session: &mut Session) -> Result<(), CommandError> {
    build(session, false)
}

pub(crate) fn rebuild(session: &mut Session) -> Result<(), CommandError> {
    build(session, true)
}
