use super::*;

#[test]
fn ui_handlers_are_available() {
    let _ = session_ui_get;
    let _ = session_ui_save;
    let _ = session_ui_previews;
    let _ = session_ui_preview_meta;
    let _ = session_archived_get;
    let _ = session_archived_put;
    let _ = session_path_get;
    let _ = session_scope_get;
    let _ = session_scope_put;
    let _ = session_ui_get_any;
}
