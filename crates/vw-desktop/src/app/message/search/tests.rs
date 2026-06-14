#![allow(unused_must_use)]
use super::*;

fn app() -> App {
    App::new().0
}

#[test]
fn input_changed_updates_text_overlay_and_refreshes_cache() {
    let mut app = app();

    update(&mut app, SearchMessage::InputChanged("  query  ".to_string()));
    assert_eq!(app.search_text, "  query  ");
    assert!(app.show_search_overlay);

    update(&mut app, SearchMessage::InputChanged("   ".to_string()));
    assert_eq!(app.search_text, "   ");
    assert!(!app.show_search_overlay);
}

#[test]
fn toggle_sets_overlay_visibility_directly() {
    let mut app = app();

    update(&mut app, SearchMessage::Toggle(true));
    assert!(app.show_search_overlay);

    update(&mut app, SearchMessage::Toggle(false));
    assert!(!app.show_search_overlay);
}

#[test]
fn select_file_sets_file_input_and_hides_overlay() {
    let mut app = app();
    app.show_search_overlay = true;

    update(&mut app, SearchMessage::SelectFile("/tmp/data.json".to_string()));

    assert_eq!(app.file_url_input, "/tmp/data.json");
    assert!(!app.show_search_overlay);
}

#[test]
fn select_session_without_known_project_prepares_active_session_state() {
    let mut app = app();
    app.show_search_overlay = true;
    app.active_session_view_state.updated_ms = 123;
    app.active_session_view_state.base_ready = true;
    app.active_session_view_state.ui_preparing = false;

    update(&mut app, SearchMessage::SelectSession("session-1".to_string()));

    assert!(!app.show_search_overlay);
    assert_eq!(app.active_session_id.as_deref(), Some("session-1"));
    assert_eq!(app.usage, crate::app::models::TokenUsage::default());
    assert_eq!(app.active_session_view_state.updated_ms, 0);
    assert!(app.active_session_view_state.ui_preparing);
    assert!(!app.active_session_view_state.base_ready);
}

#[test]
fn select_project_hides_overlay_before_returning_open_task() {
    let mut app = app();
    app.show_search_overlay = true;

    update(&mut app, SearchMessage::SelectProject("/tmp/project".to_string()));

    assert!(!app.show_search_overlay);
}
