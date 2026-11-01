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
    let Session { garment, mesh, bodies, .. } = session;
    let assembled = assemble(garment, mesh, bodies).map_err(|e| CommandError::new("invalid_geometry", e))?;
    let mut c = lock(&core);
    c.wind = garment.wind.clone();
    c.install(assembled);
    Ok(())
}

pub(crate) fn ensure(session: &mut Session) -> Result<(), CommandError> {
    build(session, false)
}

pub(crate) fn rebuild(session: &mut Session) -> Result<(), CommandError> {
    build(session, true)
}
