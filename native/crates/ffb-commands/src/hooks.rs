use crate::Session;

pub(crate) fn after_change(session: &mut Session, command: &str) {
    crate::pattern_tools_cmds::after_change(session, command);
    crate::mesh_cmds::after_change(session, command);
}
