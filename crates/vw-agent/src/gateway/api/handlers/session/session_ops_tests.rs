use super::*;

#[test]
fn session_handlers_are_available() {
    let _ = ui_session_list;
    let _ = ui_session_status;
    let _ = ui_session_get;
    let _ = ui_session_create;
    let _ = ui_session_delete;
    let _ = session_children;
    let _ = session_todo_get;
    let _ = session_todo_put;
    let _ = session_fork;
}
