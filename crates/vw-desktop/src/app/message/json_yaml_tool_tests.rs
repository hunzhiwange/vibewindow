use super::json_yaml_tool::{JsonYamlToolMessage, update};
use iced::mouse;
use iced::widget::text_editor;

#[test]
fn left_and_right_context_menus_open_and_close_independently() {
    let (mut app, _task) = crate::app::App::new();

    let _task = update(&mut app, JsonYamlToolMessage::LeftOpenContextMenu { x: 1.0, y: 2.0 });
    let _task = update(&mut app, JsonYamlToolMessage::RightOpenContextMenu { x: 3.0, y: 4.0 });

    assert!(app.json_yaml_left_context_menu_open);
    assert_eq!(app.json_yaml_left_context_menu_pos, Some((1.0, 2.0)));
    assert!(app.json_yaml_right_context_menu_open);
    assert_eq!(app.json_yaml_right_context_menu_pos, Some((3.0, 4.0)));

    let _task = update(&mut app, JsonYamlToolMessage::LeftCloseContextMenu);
    let _task = update(&mut app, JsonYamlToolMessage::RightCloseContextMenu);

    assert!(!app.json_yaml_left_context_menu_open);
    assert_eq!(app.json_yaml_left_context_menu_pos, None);
    assert!(!app.json_yaml_right_context_menu_open);
    assert_eq!(app.json_yaml_right_context_menu_pos, None);
}

#[test]
fn swap_exchanges_editors_and_resets_scroll() {
    let (mut app, _task) = crate::app::App::new();
    app.json_yaml_left_editor = text_editor::Content::with_text("{\"a\":1}");
    app.json_yaml_right_editor = text_editor::Content::with_text("a: 1");
    app.json_yaml_left_scroll_top_line = 3.0;
    app.json_yaml_left_scroll_remainder = 0.5;
    app.json_yaml_right_scroll_top_line = 4.0;
    app.json_yaml_right_scroll_remainder = 0.25;
    app.json_yaml_left_context_menu_open = true;
    app.json_yaml_right_context_menu_open = true;

    let _task = update(&mut app, JsonYamlToolMessage::Swap);

    assert_eq!(app.json_yaml_left_editor.text(), "a: 1");
    assert_eq!(app.json_yaml_right_editor.text(), "{\"a\":1}");
    assert_eq!(app.json_yaml_left_scroll_top_line, 0.0);
    assert_eq!(app.json_yaml_left_scroll_remainder, 0.0);
    assert_eq!(app.json_yaml_right_scroll_top_line, 0.0);
    assert_eq!(app.json_yaml_right_scroll_remainder, 0.0);
    assert!(!app.json_yaml_left_context_menu_open);
    assert!(!app.json_yaml_right_context_menu_open);
}

#[test]
fn clear_left_and_right_reset_only_target_editor() {
    let (mut app, _task) = crate::app::App::new();
    app.json_yaml_left_editor = text_editor::Content::with_text("left");
    app.json_yaml_right_editor = text_editor::Content::with_text("right");
    app.json_yaml_left_scroll_top_line = 2.0;
    app.json_yaml_right_scroll_top_line = 3.0;

    let _task = update(&mut app, JsonYamlToolMessage::ClearLeft);

    assert!(app.json_yaml_left_editor.text().is_empty());
    assert_eq!(app.json_yaml_right_editor.text(), "right");
    assert_eq!(app.json_yaml_left_scroll_top_line, 0.0);

    let _task = update(&mut app, JsonYamlToolMessage::ClearRight);

    assert!(app.json_yaml_right_editor.text().is_empty());
    assert_eq!(app.json_yaml_right_scroll_top_line, 0.0);
}

#[test]
fn output_updated_success_replaces_right_editor_and_notifies() {
    let (mut app, _task) = crate::app::App::new();
    app.json_yaml_loading = true;
    app.json_yaml_right_context_menu_open = true;
    app.json_yaml_right_scroll_top_line = 5.0;
    app.json_yaml_right_scroll_remainder = 0.75;

    let _task =
        update(&mut app, JsonYamlToolMessage::OutputUpdated(Some("{\n  \"a\": 1\n}".into())));

    assert!(!app.json_yaml_loading);
    assert_eq!(app.json_yaml_right_editor.text(), "{\n  \"a\": 1\n}");
    assert_eq!(app.json_yaml_right_scroll_top_line, 0.0);
    assert_eq!(app.json_yaml_right_scroll_remainder, 0.0);
    assert!(!app.json_yaml_right_context_menu_open);
    assert_eq!(app.json_yaml_notification.as_deref(), Some("转换成功"));
}

#[test]
fn output_updated_failure_keeps_content_and_notifies() {
    let (mut app, _task) = crate::app::App::new();
    app.json_yaml_loading = true;
    app.json_yaml_right_editor = text_editor::Content::with_text("old");

    let _task = update(&mut app, JsonYamlToolMessage::OutputUpdated(None));

    assert!(!app.json_yaml_loading);
    assert_eq!(app.json_yaml_right_editor.text(), "old");
    assert_eq!(app.json_yaml_notification.as_deref(), Some("转换失败或格式错误"));
}

#[test]
fn clear_notification_and_copy_messages_update_notification() {
    let (mut app, _task) = crate::app::App::new();
    app.json_yaml_left_editor = text_editor::Content::with_text("left");
    app.json_yaml_right_editor = text_editor::Content::with_text("right");

    let _task = update(&mut app, JsonYamlToolMessage::CopyLeft);

    assert_eq!(app.json_yaml_notification.as_deref(), Some("已复制左侧"));

    let _task = update(&mut app, JsonYamlToolMessage::CopyRight);

    assert_eq!(app.json_yaml_notification.as_deref(), Some("已复制右侧"));

    let _task = update(&mut app, JsonYamlToolMessage::ClearNotification);

    assert!(app.json_yaml_notification.is_none());
}

#[test]
fn scroll_messages_clamp_left_and_right_top_lines() {
    let (mut app, _task) = crate::app::App::new();
    app.current_line_height = 10.0;
    let text = "1\n2\n3\n4\n5\n6\n7\n8\n9";
    app.json_yaml_left_editor = text_editor::Content::with_text(text);
    app.json_yaml_right_editor = text_editor::Content::with_text(text);

    let _task = update(
        &mut app,
        JsonYamlToolMessage::LeftEditorWheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -24.0 },
            viewport_height: 30.0,
        },
    );
    let _task = update(
        &mut app,
        JsonYamlToolMessage::RightEditorWheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
            viewport_height: 30.0,
        },
    );

    assert_eq!(app.json_yaml_left_scroll_top_line, 2.0);
    assert_eq!(app.json_yaml_left_scroll_remainder, 0.4);
    assert_eq!(app.json_yaml_right_scroll_top_line, 2.0);
    assert_eq!(app.json_yaml_right_scroll_remainder, 0.5);

    let _task = update(
        &mut app,
        JsonYamlToolMessage::LeftScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );
    let _task = update(
        &mut app,
        JsonYamlToolMessage::RightScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );

    assert_eq!(app.json_yaml_left_scroll_top_line, 6.0);
    assert_eq!(app.json_yaml_right_scroll_top_line, 6.0);
}

#[test]
fn conversion_messages_mark_loading_and_close_menus() {
    let (mut app, _task) = crate::app::App::new();
    app.json_yaml_left_editor = text_editor::Content::with_text("{\"a\":1}");
    app.json_yaml_left_context_menu_open = true;
    app.json_yaml_right_context_menu_open = true;

    let _task = update(&mut app, JsonYamlToolMessage::JsonToYaml);

    assert!(app.json_yaml_loading);
    assert!(!app.json_yaml_left_context_menu_open);
    assert!(!app.json_yaml_right_context_menu_open);

    app.json_yaml_loading = false;
    app.json_yaml_left_context_menu_open = true;
    app.json_yaml_right_context_menu_open = true;

    let _task = update(&mut app, JsonYamlToolMessage::YamlToJson);

    assert!(app.json_yaml_loading);
    assert!(!app.json_yaml_left_context_menu_open);
    assert!(!app.json_yaml_right_context_menu_open);
}
