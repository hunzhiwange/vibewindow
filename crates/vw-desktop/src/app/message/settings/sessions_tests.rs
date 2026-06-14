use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn session_delete_and_copy_update_sync_state() {
    let mut app = app();
    app.active_session_id = Some("missing".to_string());
    app.sessions.clear();
    app.chat.push(crate::app::models::ChatMessage {
        role: vw_shared::session::ui_types::ChatRole::User,
        content: "hello".to_string(),
        think_timing: Vec::new(),
    });
    app.usage.input_tokens = 12;
    let _ = update(&mut app, SettingsMessage::SessionDelete("missing".to_string()));
    assert_eq!(app.active_session_id, None);
    assert!(app.chat.is_empty());
    assert_eq!(app.usage.input_tokens, 0);

    app.active_session_id = Some("active".to_string());
    app.session_runtime_states.insert("old".to_string(), Default::default());
    let _ = update(&mut app, SettingsMessage::SessionDelete("old".to_string()));
    assert_eq!(app.active_session_id.as_deref(), Some("active"));
    assert!(!app.session_runtime_states.contains_key("old"));
    let _ = update(&mut app, SettingsMessage::SessionCopy("active".to_string()));
    assert_eq!(app.active_session_id.as_deref(), Some("active"));
}
