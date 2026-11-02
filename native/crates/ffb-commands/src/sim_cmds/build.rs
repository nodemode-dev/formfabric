use super::assemble::assemble;
use super::runtime::lock;
use ffb_sim::Params;
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
    if session.sim.auto_method {
        let want = if assembled.cloth.seams.is_empty() { ffb_sim::Method::Implicit } else { ffb_sim::Method::Vbd };
        let mut shared = session.sim.params.lock().unwrap_or_else(|e| e.into_inner());
        if shared.method != want {
            let moved = Params { method: want, ..*shared };
            *shared = if session.sim.preset == "custom" { moved } else { super::adapter::preset(&session.sim.preset, moved).unwrap_or(moved) };
        }
    }
    let mut c = lock(&core);
    c.wind = garment.wind.clone();
    c.install(assembled);
    Ok(())
}

pub(crate) fn keyed_rebuilder(core: std::sync::Arc<std::sync::Mutex<super::runtime::Core>>, forced: std::collections::HashMap<u32, std::collections::BTreeMap<usize, usize>>) -> impl FnMut(&ffb_pattern::Garment, &[ffb_body::Body]) -> Result<(), String> + Send {
    let mut mesh = crate::mesh_cmds::MeshState { forced, ..Default::default() };
    move |garment, bodies| {
        let assembled = assemble(garment, &mut mesh, bodies)?;
        let mut c = lock(&core);
        c.wind = garment.wind.clone();
        c.install_keep(assembled);
        Ok(())
    }
}

pub(crate) fn ensure(session: &mut Session) -> Result<(), CommandError> {
    build(session, false)
}

pub(crate) fn rebuild(session: &mut Session) -> Result<(), CommandError> {
    build(session, true)
}
