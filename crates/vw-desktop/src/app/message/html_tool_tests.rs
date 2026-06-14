//! 覆盖 HTML 工具消息处理的行为，确保输入、预览和结果状态符合预期。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::html_tool::{HtmlToolMessage, beautify_html, compress_html, update};
use iced::mouse;
use iced::widget::text_editor;

#[test]
fn beautify_html_preserves_textarea_whitespace() {
    let input = "<textarea>  line 1\n    line 2\n</textarea>";
    let expected = "<textarea>\n  line 1\n    line 2\n</textarea>\n";

    assert_eq!(beautify_html(input).as_deref(), Some(expected));
}

#[test]
fn beautify_html_indents_script_content() {
    let input = "<script>const answer = 42;\nconsole.log(answer);</script>";
    let expected = "<script>\n    const answer = 42;\n    console.log(answer);\n</script>\n";

    assert_eq!(beautify_html(input).as_deref(), Some(expected));
}

#[test]
fn compress_html_preserves_pre_content() {
    let input = "<pre>  keep\n    spacing</pre>";
    let expected = "<pre>  keep\n    spacing</pre>";

    assert_eq!(compress_html(input).as_deref(), Some(expected));
}

#[test]
fn beautify_html_handles_doctype_comments_void_tags_and_text() {
    let input = "<!doctype html><div>Hello   world<br><img src=\"a>b\"><!-- ok --></div>";
    let expected = "<!doctype html>\n<div>\n    Hello world\n    <br>\n    <img src=\"a>b\">\n    <!-- ok -->\n</div>\n";

    assert_eq!(beautify_html(input).as_deref(), Some(expected));
}

#[test]
fn html_formatters_return_none_for_blank_input() {
    assert_eq!(beautify_html("   \n\t"), None);
    assert_eq!(compress_html("   \n\t"), None);
}

#[test]
fn compress_html_collapses_inter_tag_text() {
    let input = "<div>  alpha\n beta  </div><span> gamma </span>";

    assert_eq!(compress_html(input).as_deref(), Some("<div>alpha beta</div><span>gamma</span>"));
}

#[test]
fn open_close_and_clear_notification_update_ui_state() {
    let (mut app, _task) = crate::app::App::new();

    let _task = update(&mut app, HtmlToolMessage::OpenContextMenu { x: 12.0, y: 34.0 });

    assert!(app.html_tool_context_menu_open);
    assert_eq!(app.html_tool_context_menu_pos, Some((12.0, 34.0)));

    app.html_tool_notification = Some("done".to_string());
    let _task = update(&mut app, HtmlToolMessage::ClearNotification);

    assert!(app.html_tool_notification.is_none());

    let _task = update(&mut app, HtmlToolMessage::CloseContextMenu);

    assert!(!app.html_tool_context_menu_open);
    assert_eq!(app.html_tool_context_menu_pos, None);
}

#[test]
fn content_updated_success_replaces_editor_and_resets_scroll() {
    let (mut app, _task) = crate::app::App::new();
    app.html_tool_loading = true;
    app.html_tool_context_menu_open = true;
    app.html_tool_scroll_top_line = 5.0;
    app.html_tool_scroll_remainder = 0.75;

    let _task = update(&mut app, HtmlToolMessage::ContentUpdated(Some("<p>ok</p>".to_string())));

    assert!(!app.html_tool_loading);
    assert_eq!(app.html_tool_editor.text(), "<p>ok</p>");
    assert_eq!(app.html_tool_scroll_top_line, 0.0);
    assert_eq!(app.html_tool_scroll_remainder, 0.0);
    assert!(!app.html_tool_context_menu_open);
    assert_eq!(app.html_tool_notification.as_deref(), Some("操作成功"));
}

#[test]
fn content_updated_failure_sets_failure_notification() {
    let (mut app, _task) = crate::app::App::new();
    app.html_tool_loading = true;
    app.html_tool_context_menu_open = true;

    let _task = update(&mut app, HtmlToolMessage::ContentUpdated(None));

    assert!(!app.html_tool_loading);
    assert!(!app.html_tool_context_menu_open);
    assert_eq!(app.html_tool_notification.as_deref(), Some("操作失败或格式错误"));
}

#[test]
fn clear_and_copy_set_success_notification() {
    let (mut app, _task) = crate::app::App::new();
    app.html_tool_editor = text_editor::Content::with_text("<div>value</div>");
    app.html_tool_scroll_top_line = 4.0;
    app.html_tool_scroll_remainder = 0.4;

    let _task = update(&mut app, HtmlToolMessage::Clear);

    assert!(app.html_tool_editor.text().is_empty());
    assert_eq!(app.html_tool_scroll_top_line, 0.0);
    assert_eq!(app.html_tool_scroll_remainder, 0.0);
    assert_eq!(app.html_tool_notification.as_deref(), Some("已清空"));

    let _task = update(&mut app, HtmlToolMessage::Copy);

    assert_eq!(app.html_tool_notification.as_deref(), Some("已复制"));
}

#[test]
fn editor_scroll_messages_clamp_top_line_and_remainder() {
    let (mut app, _task) = crate::app::App::new();
    app.current_line_height = 10.0;
    app.html_tool_editor = text_editor::Content::with_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");

    let _task = update(
        &mut app,
        HtmlToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -25.0 },
            viewport_height: 30.0,
        },
    );

    assert_eq!(app.html_tool_viewport_height, 30.0);
    assert_eq!(app.html_tool_scroll_top_line, 2.0);
    assert_eq!(app.html_tool_scroll_remainder, 0.5);

    let _task = update(
        &mut app,
        HtmlToolMessage::ScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );

    assert_eq!(app.html_tool_scroll_top_line, 7.0);
}

#[test]
fn toggle_remember_updates_config_state() {
    let (mut app, _task) = crate::app::App::new();
    app.html_tool_editor = text_editor::Content::with_text("<p>remember</p>");

    let _task = update(&mut app, HtmlToolMessage::ToggleRemember(true));

    assert!(app.html_tool_remember);

    let _task = update(&mut app, HtmlToolMessage::ToggleRemember(false));

    assert!(!app.html_tool_remember);
}
