use crate::Session;

pub(crate) fn after_change(session: &mut Session, command: &str) {
    crate::pattern_tools_cmds::after_change(session, command);
    crate::sew_cmds::after_change(session, command);
    crate::body_cmds::after_change(session, command);
    crate::place_cmds::after_change(session, command);
    crate::mesh_cmds::after_change(session, command);
    crate::sim_cmds::after_change(session, command);
}
