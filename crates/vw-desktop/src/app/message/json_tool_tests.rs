use super::json_tool::{JsonToolMessage, update};
use iced::mouse;
use iced::widget::text_editor;

#[test]
fn open_close_and_clear_notification_update_ui_state() {
    let (mut app, _task) = crate::app::App::new();

    let _task = update(&mut app, JsonToolMessage::OpenContextMenu { x: 4.0, y: 8.0 });

    assert!(app.json_tool_context_menu_open);
    assert_eq!(app.json_tool_context_menu_pos, Some((4.0, 8.0)));

    app.json_tool_notification = Some("done".to_string());
    let _task = update(&mut app, JsonToolMessage::ClearNotification);

    assert!(app.json_tool_notification.is_none());

    let _task = update(&mut app, JsonToolMessage::CloseContextMenu);

    assert!(!app.json_tool_context_menu_open);
    assert_eq!(app.json_tool_context_menu_pos, None);
}

#[test]
fn content_updated_success_replaces_editor_and_resets_scroll() {
    let (mut app, _task) = crate::app::App::new();
    app.json_tool_loading = true;
    app.json_tool_context_menu_open = true;
    app.json_tool_scroll_top_line = 4.0;
    app.json_tool_scroll_remainder = 0.5;

    let _task =
        update(&mut app, JsonToolMessage::ContentUpdated(Some("{\"ok\":true}".to_string())));

    assert!(!app.json_tool_loading);
    assert_eq!(app.json_tool_editor.text(), "{\"ok\":true}");
    assert_eq!(app.json_tool_scroll_top_line, 0.0);
    assert_eq!(app.json_tool_scroll_remainder, 0.0);
    assert!(!app.json_tool_context_menu_open);
    assert_eq!(app.json_tool_notification.as_deref(), Some("操作成功"));
}

#[test]
fn content_updated_failure_sets_failure_notification() {
    let (mut app, _task) = crate::app::App::new();
    app.json_tool_loading = true;
    app.json_tool_context_menu_open = true;

    let _task = update(&mut app, JsonToolMessage::ContentUpdated(None));

    assert!(!app.json_tool_loading);
    assert!(!app.json_tool_context_menu_open);
    assert_eq!(app.json_tool_notification.as_deref(), Some("操作失败或格式错误"));
}

#[test]
fn clear_and_copy_set_success_notification() {
    let (mut app, _task) = crate::app::App::new();
    app.json_tool_editor = text_editor::Content::with_text("{\"a\":1}");
    app.json_tool_scroll_top_line = 3.0;
    app.json_tool_scroll_remainder = 0.25;

    let _task = update(&mut app, JsonToolMessage::Clear);

    assert!(app.json_tool_editor.text().is_empty());
    assert_eq!(app.json_tool_scroll_top_line, 0.0);
    assert_eq!(app.json_tool_scroll_remainder, 0.0);
    assert_eq!(app.json_tool_notification.as_deref(), Some("已清空"));

    let _task = update(&mut app, JsonToolMessage::Copy);

    assert_eq!(app.json_tool_notification.as_deref(), Some("已复制"));
}

#[test]
fn editor_scroll_messages_clamp_top_line_and_remainder() {
    let (mut app, _task) = crate::app::App::new();
    app.current_line_height = 10.0;
    app.json_tool_editor = text_editor::Content::with_text("1\n2\n3\n4\n5\n6\n7\n8");

    let _task = update(
        &mut app,
        JsonToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
            viewport_height: 30.0,
        },
    );

    assert_eq!(app.json_tool_viewport_height, 30.0);
    assert_eq!(app.json_tool_scroll_top_line, 2.0);
    assert_eq!(app.json_tool_scroll_remainder, 0.5);

    let _task = update(
        &mut app,
        JsonToolMessage::ScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );

    assert_eq!(app.json_tool_scroll_top_line, 5.0);
}

#[test]
fn async_operation_messages_mark_loading() {
    let (mut app, _task) = crate::app::App::new();
    app.json_tool_editor = text_editor::Content::with_text("{\"name\":\"张三\"}");

    for message in [
        JsonToolMessage::Format,
        JsonToolMessage::Compress,
        JsonToolMessage::Escape,
        JsonToolMessage::Unescape,
        JsonToolMessage::UnicodeToCn,
        JsonToolMessage::CnToUnicode,
        JsonToolMessage::ToGet,
    ] {
        app.json_tool_loading = false;
        let _task = update(&mut app, message);
        assert!(app.json_tool_loading);
    }
}

#[test]
fn toggle_remember_updates_config_state() {
    let (mut app, _task) = crate::app::App::new();
    app.json_tool_editor = text_editor::Content::with_text("{\"remember\":true}");

    let _task = update(&mut app, JsonToolMessage::ToggleRemember(true));

    assert!(app.json_tool_remember);

    let _task = update(&mut app, JsonToolMessage::ToggleRemember(false));

    assert!(!app.json_tool_remember);
}
