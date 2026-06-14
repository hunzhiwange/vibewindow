use super::*;

// Tests for plan6 task 850.
const SOURCE: &str = include_str!("new.rs");

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn new_tests_keeps_planned_coverage_targets() {
    for name in ["new"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn new_initializes_safe_empty_session_state() {
    let (app, _task) = App::new();

    assert_eq!(app.screen, Screen::Home);
    assert_eq!(app.project_path, None);
    assert_eq!(app.project_id, None);
    assert_eq!(app.active_session_id, None);
    assert!(app.chat.is_empty());
    assert!(app.chat_message_ids.is_empty());
    assert!(app.sessions.is_empty());
    assert!(!app.is_requesting);
    assert!(app.queue.is_empty());
    assert!(app.chat_auto_scroll);
    assert_eq!(app.active_tab_id.as_deref(), Some("home"));
    assert_eq!(app.open_tabs.len(), 1);
    assert_eq!(app.open_tabs[0].id, "home");
    assert_eq!(app.open_tabs[0].screen, Screen::Home);
    assert_eq!(app.open_tabs[0].project_path, None);
}

#[test]
fn new_keeps_runtime_and_tool_defaults_consistent() {
    let (app, _task) = App::new();

    let empty_runtime = app
        .session_runtime_states
        .get("__empty__")
        .expect("new app should seed an empty runtime state");
    assert_eq!(empty_runtime.model, app.model);
    assert_eq!(empty_runtime.auto_model, app.auto_model);
    assert_eq!(empty_runtime.acp_agent, app.acp_agent);

    assert_eq!(app.pwd_length_input, "12");
    assert_eq!(app.pwd_count_input, "1");
    assert!(app.pwd_digits);
    assert!(app.pwd_lowercase);
    assert!(app.pwd_uppercase);
    assert!(app.pwd_special);
    assert_eq!(app.qr_size, 256);
    assert_eq!(app.qr_size_input, "256");
    assert_eq!(app.color_hex_input, "#000000ff");
}

#[test]
fn new_initializes_layout_panels_and_overlay_state() {
    let (app, _task) = App::new();

    assert_eq!(app.split_ratio, 0.6);
    assert!(!app.dragging_split);
    assert_eq!(app.window_size, (1200.0, 800.0));
    assert!(app.file_manager_width >= 180.0);
    assert!(app.file_manager_width <= 600.0);
    assert!(!app.show_model_popover);
    assert!(!app.show_file_popover);
    assert!(!app.show_usage_popover);
    assert!(app.chat_context_menu_target.is_none());
    assert!(app.input_context_menu_pos.is_none());
    assert!(app.tool_detail_dialog.is_none());
}

#[test]
fn new_initializes_project_edit_and_new_session_state() {
    let (app, _task) = App::new();

    assert!(app.project_sessions.is_empty());
    assert!(app.project_sessions_loading.is_empty());
    assert!(app.project_session_load_counts.is_empty());
    assert!(app.new_session_picker_project.is_none());
    assert!(app.new_session_picker_options.is_empty());
    assert!(app.new_session_worktree_name.is_empty());
    assert!(app.new_session_confirm_delete_directory.is_none());
    assert!(app.project_edit_path.is_none());
    assert_eq!(app.project_edit_tab, crate::app::state::ProjectEditTab::General);
    assert!(app.project_edit_name.is_empty());
    assert!(app.project_edit_start_script.is_empty());
}

#[test]
fn new_initializes_notifications_and_task_board_state() {
    let (app, _task) = App::new();

    assert!(app.notifications.is_empty());
    assert!(!app.notifications_expanded);
    assert_eq!(app.next_notification_id, 0);
    assert!(app.notification_editors.is_empty());
    assert!(app.active_toast.is_none());
    assert_eq!(app.next_toast_id, 0);
    assert!(!app.show_task_board);
    assert!(!app.task_board_loading);
}

#[test]
fn new_initializes_timestamp_state_from_current_time() {
    let (app, _task) = App::new();

    let seconds = app.ts_now_unix_sec.parse::<i64>().expect("seconds timestamp should be numeric");
    let millis =
        app.ts_now_unix_ms.parse::<u128>().expect("millisecond timestamp should be numeric");

    assert!(seconds > 0);
    assert!(millis >= seconds as u128 * 1000);
    assert_eq!(app.ts_input_ts, "");
    assert_eq!(app.ts_time_output, "");
    assert_eq!(app.ts_time_input, "");
    assert_eq!(app.ts_ts_output_sec, "");
    assert_eq!(app.ts_ts_output_ms, "");
    assert_eq!(app.ts_notification, None);
    assert!(app.ts_auto);
}
