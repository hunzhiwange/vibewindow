//! 覆盖 Markdown 工具消息处理的编辑、滚动和本地转换行为。

use super::{
    MarkdownToolMessage, apply_scroll_lines, extract_remote_image_urls, max_scroll_top_line,
    paste_snippet, refresh_preview, replace_markdown, update, visible_line_count,
};
use crate::app::{App, components::markdown_editor::MarkdownViewMode};
use iced::mouse;
use iced::widget::text_editor;
use std::path::PathBuf;

fn test_app() -> App {
    App::new().0
}

#[test]
fn extract_remote_image_urls_keeps_only_unique_http_urls() {
    let urls = extract_remote_image_urls(
        "![a](https://b.test/img.png) ![b](file:///tmp/a.png) ![c](http://a.test/x.png) ![d](https://b.test/img.png)",
    );

    assert_eq!(urls, vec!["http://a.test/x.png", "https://b.test/img.png"]);
}

#[test]
fn helpers_replace_paste_preview_and_scroll_with_bounds() {
    let mut app = test_app();

    replace_markdown(&mut app, "# title".to_string());
    assert_eq!(app.markdown_tool_editor.text(), "# title");

    paste_snippet(&mut app, "\nbody");
    assert!(app.markdown_tool_editor.text().contains("body"));

    app.markdown_tool_stream_enabled = true;
    app.markdown_tool_stream_chars = 1;
    refresh_preview(&mut app);
    assert_eq!(app.markdown_tool_stream_chars, 1);

    app.markdown_tool_editor = text_editor::Content::with_text("1\n2\n3\n4\n5");
    app.current_line_height = 10.0;
    app.markdown_tool_viewport_height = 20.0;
    assert_eq!(visible_line_count(&app), 2.0);
    assert_eq!(max_scroll_top_line(&app), 3.0);

    apply_scroll_lines(&mut app, 99);
    assert_eq!(app.markdown_tool_scroll_top_line, 3.0);
    apply_scroll_lines(&mut app, -99);
    assert_eq!(app.markdown_tool_scroll_top_line, 0.0);
}

#[test]
fn update_inserts_snippets_and_toggles_dialog_state() {
    let mut app = test_app();

    let _ = update(&mut app, MarkdownToolMessage::SetViewMode(MarkdownViewMode::Preview));
    assert_eq!(app.markdown_tool_view_mode, MarkdownViewMode::Preview);

    for message in [
        MarkdownToolMessage::InsertBold,
        MarkdownToolMessage::InsertItalic,
        MarkdownToolMessage::InsertStrike,
        MarkdownToolMessage::InsertHeading,
        MarkdownToolMessage::InsertQuote,
        MarkdownToolMessage::InsertCodeBlock,
        MarkdownToolMessage::InsertLink,
        MarkdownToolMessage::InsertTable,
    ] {
        let _ = update(&mut app, message);
    }

    let text = app.markdown_tool_editor.text();
    assert!(text.contains("**粗体**"));
    assert!(text.contains("*斜体*"));
    assert!(text.contains("~~删除线~~"));
    assert!(text.contains("### 标题"));
    assert!(text.contains("> 引用"));
    assert!(text.contains("```text"));
    assert!(text.contains("[链接文本](https://example.com)"));
    assert!(text.contains("| header1 | header2 |"));

    let _ = update(&mut app, MarkdownToolMessage::InsertImage);
    assert!(app.markdown_tool_show_image);
    let _ = update(
        &mut app,
        MarkdownToolMessage::ImageUrlChanged("https://example.com/a.png".to_string()),
    );
    let _ = update(&mut app, MarkdownToolMessage::InsertImageFromUrl);
    assert!(!app.markdown_tool_show_image);
    assert!(app.markdown_tool_editor.text().contains("![](https://example.com/a.png)"));
    assert_eq!(app.markdown_tool_notification.as_deref(), Some("已插入图片"));

    let _ = update(&mut app, MarkdownToolMessage::CloseImage);
    assert!(!app.markdown_tool_show_image);
}

#[test]
fn update_handles_local_image_html_stream_and_remote_result() {
    let mut app = test_app();

    let _ = update(
        &mut app,
        MarkdownToolMessage::ImagePicked(Some(PathBuf::from("/tmp/my image.png"))),
    );
    assert!(app.markdown_tool_editor.text().contains("![my image](</tmp/my image.png>)"));
    assert_eq!(app.markdown_tool_notification.as_deref(), Some("已插入图片"));

    let _ = update(&mut app, MarkdownToolMessage::OpenHtml2Md);
    assert!(app.markdown_tool_show_html2md);
    let _ = update(
        &mut app,
        MarkdownToolMessage::HtmlEditorAction(text_editor::Action::Edit(text_editor::Edit::Paste(
            std::sync::Arc::new("<h1>Hello</h1>".to_string()),
        ))),
    );
    let _ = update(&mut app, MarkdownToolMessage::ConvertHtmlToMarkdown);
    assert!(!app.markdown_tool_show_html2md);
    assert!(app.markdown_tool_editor.text().contains("Hello"));
    assert_eq!(app.markdown_tool_notification.as_deref(), Some("已转换"));

    let _ = update(&mut app, MarkdownToolMessage::ToggleStream(true));
    assert!(app.markdown_tool_stream_enabled);
    assert_eq!(app.markdown_tool_stream_chars, 0);
    let _ = update(&mut app, MarkdownToolMessage::StreamTick);
    assert!(app.markdown_tool_stream_chars > 0);
    let _ = update(&mut app, MarkdownToolMessage::ToggleStream(false));
    assert!(!app.markdown_tool_stream_enabled);
    assert_eq!(app.markdown_tool_stream_chars, usize::MAX);

    app.markdown_tool_remote_images_loading.insert("https://img".to_string());
    let _ = update(
        &mut app,
        MarkdownToolMessage::RemoteImageLoaded("https://img".to_string(), Ok(vec![1, 2, 3])),
    );
    assert!(!app.markdown_tool_remote_images_loading.contains("https://img"));
    assert!(app.markdown_tool_remote_images.contains_key("https://img"));

    let _ = update(&mut app, MarkdownToolMessage::ClearNotification);
    assert!(app.markdown_tool_notification.is_none());
}

#[test]
fn update_editor_scroll_context_menu_and_clear_are_local() {
    let mut app = test_app();
    app.markdown_tool_editor = text_editor::Content::with_text("1\n2\n3\n4\n5\n6");
    app.current_line_height = 10.0;

    let _ = update(&mut app, MarkdownToolMessage::OpenContextMenu { x: 4.0, y: 5.0 });
    assert!(app.markdown_tool_context_menu_open);
    assert_eq!(app.markdown_tool_context_menu_pos, Some((4.0, 5.0)));

    let _ = update(
        &mut app,
        MarkdownToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -30.0 },
            viewport_height: 20.0,
        },
    );
    assert!(!app.markdown_tool_context_menu_open);
    assert!(app.markdown_tool_scroll_top_line > 0.0);

    let _ = update(
        &mut app,
        MarkdownToolMessage::ScrollbarChanged { top_line: 99.0, viewport_height: 20.0 },
    );
    assert_eq!(app.markdown_tool_scroll_top_line, max_scroll_top_line(&app));

    let _ = update(
        &mut app,
        MarkdownToolMessage::EditorAction(text_editor::Action::Scroll { lines: -99 }),
    );
    assert_eq!(app.markdown_tool_scroll_top_line, 0.0);

    let _ = update(&mut app, MarkdownToolMessage::Clear);
    assert!(app.markdown_tool_editor.text().is_empty());

    let _ = update(&mut app, MarkdownToolMessage::CloseHtml2Md);
    assert!(!app.markdown_tool_show_html2md);
}
