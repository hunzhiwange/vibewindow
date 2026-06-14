//! 覆盖 JSON 差异工具的消息处理行为，保护格式化和差异状态更新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::*;
use iced::mouse;
use iced::widget::text_editor;

#[test]
fn prettify_json_text_formats_pretty_output() {
    let formatted = prettify_json_text(r#"{"name":"alice","roles":["admin","dev"]}"#)
        .expect("json should format");

    assert_eq!(
        formatted,
        "{\n  \"name\": \"alice\",\n  \"roles\": [\n    \"admin\",\n    \"dev\"\n  ]\n}"
    );
}

#[test]
fn compare_json_documents_marks_changed_and_missing_fields() {
    let diffs = compare_json_documents(
        r#"{"name":"","enabled":true,"profile":{"city":"Shanghai"}}"#,
        r#"{"enabled":false,"profile":{"street":"Huaihai Rd"}}"#,
    )
    .expect("json should compare");

    assert_eq!(
        diffs,
        vec![
            JsonDiffEntry {
                path: "enabled".to_string(),
                left: Some("true".to_string()),
                right: Some("false".to_string()),
            },
            JsonDiffEntry { path: "name".to_string(), left: Some(String::new()), right: None },
            JsonDiffEntry {
                path: "profile.city".to_string(),
                left: Some("Shanghai".to_string()),
                right: None,
            },
            JsonDiffEntry {
                path: "profile.street".to_string(),
                left: None,
                right: Some("Huaihai Rd".to_string()),
            },
        ]
    );
}

#[test]
fn compare_json_documents_reports_side_specific_parse_error() {
    let error = compare_json_documents("{", r#"{"name":"ok"}"#).expect_err("left side should fail");

    assert!(error.contains("左侧 JSON 解析失败"));
}

#[test]
fn compare_json_documents_reports_array_differences() {
    let diffs = compare_json_documents(
        r#"{"items":[{"id":1},{"id":2},3]}"#,
        r#"{"items":[{"id":1},{"id":4},3,5]}"#,
    )
    .expect("json should compare");

    assert_eq!(
        diffs,
        vec![
            JsonDiffEntry {
                path: "items[1].id".to_string(),
                left: Some("2".to_string()),
                right: Some("4".to_string()),
            },
            JsonDiffEntry {
                path: "items[3]".to_string(),
                left: None,
                right: Some("5".to_string()),
            },
        ]
    );
}

#[test]
fn stringify_formats_nested_values() {
    let value = serde_json::json!({"a":[1, true]});

    assert_eq!(stringify(&value), "{\n  \"a\": [\n    1,\n    true\n  ]\n}");
    assert_eq!(stringify(&serde_json::json!("plain")), "plain");
}

#[test]
fn context_menus_close_other_side_when_opened() {
    let (mut app, _task) = crate::app::App::new();

    let _task = update(&mut app, JsonDiffToolMessage::LeftOpenContextMenu { x: 1.0, y: 2.0 });
    let _task = update(&mut app, JsonDiffToolMessage::RightOpenContextMenu { x: 3.0, y: 4.0 });

    assert!(!app.json_diff_left_context_menu_open);
    assert_eq!(app.json_diff_left_context_menu_pos, None);
    assert!(app.json_diff_right_context_menu_open);
    assert_eq!(app.json_diff_right_context_menu_pos, Some((3.0, 4.0)));

    let _task = update(&mut app, JsonDiffToolMessage::RightCloseContextMenu);

    assert!(!app.json_diff_right_context_menu_open);
}

#[test]
fn compare_finished_success_and_error_update_results_and_notifications() {
    let (mut app, _task) = crate::app::App::new();
    app.json_diff_loading = true;

    let _task = update(
        &mut app,
        JsonDiffToolMessage::CompareFinished(Ok(vec![JsonDiffEntry {
            path: "a".to_string(),
            left: Some("1".to_string()),
            right: Some("2".to_string()),
        }])),
    );

    assert!(!app.json_diff_loading);
    assert_eq!(app.json_diff_results.len(), 1);
    assert_eq!(app.json_diff_notification.as_deref(), Some("共发现 1 处差异"));
    assert!(!app.json_diff_notification_is_error);

    let _task = update(&mut app, JsonDiffToolMessage::CompareFinished(Err("bad json".to_string())));

    assert!(app.json_diff_results.is_empty());
    assert_eq!(app.json_diff_notification.as_deref(), Some("bad json"));
    assert!(app.json_diff_notification_is_error);
}

#[test]
fn format_finished_messages_replace_target_editors_or_report_errors() {
    let (mut app, _task) = crate::app::App::new();
    app.json_diff_loading = true;
    app.json_diff_left_scroll_top_line = 4.0;

    let _task = update(
        &mut app,
        JsonDiffToolMessage::FormatLeftFinished(Ok("{\n  \"a\": 1\n}".to_string())),
    );

    assert!(!app.json_diff_loading);
    assert_eq!(app.json_diff_left_editor.text(), "{\n  \"a\": 1\n}");
    assert_eq!(app.json_diff_left_scroll_top_line, 0.0);
    assert_eq!(app.json_diff_notification.as_deref(), Some("左侧已格式化"));

    app.json_diff_loading = true;
    app.json_diff_right_scroll_top_line = 3.0;
    let _task = update(
        &mut app,
        JsonDiffToolMessage::FormatRightFinished(Ok("{\n  \"b\": 2\n}".to_string())),
    );

    assert!(!app.json_diff_loading);
    assert_eq!(app.json_diff_right_editor.text(), "{\n  \"b\": 2\n}");
    assert_eq!(app.json_diff_right_scroll_top_line, 0.0);
    assert_eq!(app.json_diff_notification.as_deref(), Some("右侧已格式化"));

    let _task =
        update(&mut app, JsonDiffToolMessage::FormatLeftFinished(Err("invalid".to_string())));

    assert_eq!(app.json_diff_notification.as_deref(), Some("invalid"));
    assert!(app.json_diff_notification_is_error);
}

#[test]
fn format_both_finished_replaces_both_or_reports_error() {
    let (mut app, _task) = crate::app::App::new();
    app.json_diff_loading = true;
    app.json_diff_left_scroll_top_line = 2.0;
    app.json_diff_right_scroll_top_line = 3.0;

    let _task = update(
        &mut app,
        JsonDiffToolMessage::FormatBothFinished(Ok((
            "{\n  \"left\": true\n}".to_string(),
            "{\n  \"right\": true\n}".to_string(),
        ))),
    );

    assert!(!app.json_diff_loading);
    assert_eq!(app.json_diff_left_editor.text(), "{\n  \"left\": true\n}");
    assert_eq!(app.json_diff_right_editor.text(), "{\n  \"right\": true\n}");
    assert_eq!(app.json_diff_left_scroll_top_line, 0.0);
    assert_eq!(app.json_diff_right_scroll_top_line, 0.0);
    assert_eq!(app.json_diff_notification.as_deref(), Some("左右两侧已格式化"));

    let _task =
        update(&mut app, JsonDiffToolMessage::FormatBothFinished(Err("both failed".to_string())));

    assert_eq!(app.json_diff_notification.as_deref(), Some("both failed"));
    assert!(app.json_diff_notification_is_error);
}

#[test]
fn swap_clear_copy_and_insert_messages_update_state() {
    let (mut app, _task) = crate::app::App::new();
    app.json_diff_left_editor = text_editor::Content::with_text("left");
    app.json_diff_right_editor = text_editor::Content::with_text("right");
    app.json_diff_left_scroll_top_line = 2.0;
    app.json_diff_right_scroll_top_line = 3.0;

    let _task = update(&mut app, JsonDiffToolMessage::Swap);

    assert_eq!(app.json_diff_left_editor.text(), "right");
    assert_eq!(app.json_diff_right_editor.text(), "left");
    assert_eq!(app.json_diff_notification.as_deref(), Some("已交换左右内容"));

    let _task = update(&mut app, JsonDiffToolMessage::ClearLeft);
    assert!(app.json_diff_left_editor.text().is_empty());
    assert_eq!(app.json_diff_notification.as_deref(), Some("已清空左侧"));

    let _task = update(&mut app, JsonDiffToolMessage::ClearRight);
    assert!(app.json_diff_right_editor.text().is_empty());
    assert_eq!(app.json_diff_notification.as_deref(), Some("已清空右侧"));

    let _task = update(&mut app, JsonDiffToolMessage::InsertPair("L".into(), "R".into()));
    assert_eq!(app.json_diff_left_editor.text(), "L");
    assert_eq!(app.json_diff_right_editor.text(), "R");

    let _task = update(&mut app, JsonDiffToolMessage::InsertLeft("LL".into()));
    let _task = update(&mut app, JsonDiffToolMessage::InsertRight("RR".into()));

    assert_eq!(app.json_diff_left_editor.text(), "LL");
    assert_eq!(app.json_diff_right_editor.text(), "RR");

    let _task = update(&mut app, JsonDiffToolMessage::CopyLeft);
    assert_eq!(app.json_diff_notification.as_deref(), Some("已复制左侧"));
    let _task = update(&mut app, JsonDiffToolMessage::CopyRight);
    assert_eq!(app.json_diff_notification.as_deref(), Some("已复制右侧"));

    let _task = update(&mut app, JsonDiffToolMessage::ClearNotification);
    assert!(app.json_diff_notification.is_none());
    assert!(!app.json_diff_notification_is_error);
}

#[test]
fn scroll_messages_clamp_left_and_right_top_lines() {
    let (mut app, _task) = crate::app::App::new();
    app.current_line_height = 10.0;
    let text = "1\n2\n3\n4\n5\n6\n7\n8\n9";
    app.json_diff_left_editor = text_editor::Content::with_text(text);
    app.json_diff_right_editor = text_editor::Content::with_text(text);

    let _task = update(
        &mut app,
        JsonDiffToolMessage::LeftEditorWheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -24.0 },
            viewport_height: 30.0,
        },
    );
    let _task = update(
        &mut app,
        JsonDiffToolMessage::RightEditorWheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
            viewport_height: 30.0,
        },
    );

    assert_eq!(app.json_diff_left_scroll_top_line, 2.0);
    assert_eq!(app.json_diff_left_scroll_remainder, 0.4);
    assert_eq!(app.json_diff_right_scroll_top_line, 2.0);
    assert_eq!(app.json_diff_right_scroll_remainder, 0.5);

    let _task = update(
        &mut app,
        JsonDiffToolMessage::LeftScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );
    let _task = update(
        &mut app,
        JsonDiffToolMessage::RightScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );

    assert_eq!(app.json_diff_left_scroll_top_line, 6.0);
    assert_eq!(app.json_diff_right_scroll_top_line, 6.0);
}

#[test]
fn async_operation_messages_mark_loading() {
    let (mut app, _task) = crate::app::App::new();
    app.json_diff_left_editor = text_editor::Content::with_text("{\"a\":1}");
    app.json_diff_right_editor = text_editor::Content::with_text("{\"a\":2}");

    for message in [
        JsonDiffToolMessage::Compare,
        JsonDiffToolMessage::FormatLeft,
        JsonDiffToolMessage::FormatRight,
        JsonDiffToolMessage::FormatBoth,
    ] {
        app.json_diff_loading = false;
        let _task = update(&mut app, message);
        assert!(app.json_diff_loading);
    }
}
