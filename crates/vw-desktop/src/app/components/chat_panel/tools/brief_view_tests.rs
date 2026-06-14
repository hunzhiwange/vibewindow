use super::brief_view::tool_brief_view;
use super::brief_view::{compact_attachment_path, format_attachment_size, format_sent_at};
use crate::app::App;

#[test]
fn compact_attachment_path_keeps_file_name_for_long_paths() {
    assert_eq!(compact_attachment_path("/tmp/project/src/main.rs"), "src/main.rs");
    assert_eq!(compact_attachment_path("README.md"), "README.md");
}

#[test]
fn format_attachment_size_uses_human_units() {
    assert_eq!(format_attachment_size(512), "512 B");
    assert_eq!(format_attachment_size(2048), "2.0 KB");
    assert_eq!(format_attachment_size(2 * 1024 * 1024), "2.0 MB");
}

#[test]
fn format_sent_at_rejects_empty_values() {
    assert_eq!(format_sent_at(""), None);
    assert!(format_sent_at("2026-06-12T08:30:00Z").is_some());
}

#[test]
fn brief_view_rejects_malformed_error_and_empty_payloads() {
    let app = App::new().0;

    assert!(tool_brief_view(&app, 0, 0, "tool bash\n{}").is_none());
    assert!(tool_brief_view(&app, 0, 0, "tool brief\nnot-json").is_none());
    assert!(
        tool_brief_view(
            &app,
            0,
            0,
            r#"tool brief
{"status":"error","result":{"data":{"message":"failed"}}}"#
        )
        .is_none()
    );
    assert!(
        tool_brief_view(
            &app,
            0,
            0,
            r#"tool brief
{"result":{"data":{"message":"","attachments":[]}}}"#
        )
        .is_none()
    );
}

#[test]
fn brief_view_renders_message_and_attachments() {
    let mut app = App::new().0;
    app.chat_tool_hovered_idx = Some((2_u64 << 32) | 1);
    let visible = r#"tool brief
{"result":{"data":{"message":"sent","status":"proactive","sentAt":"2026-06-12T08:30:00Z","attachments":[{"path":"/tmp/project/a.png","size":2048,"isImage":true},{"path":"/tmp/project/readme.md","size":12}]}}}"#;

    assert!(tool_brief_view(&app, 2, 1, visible).is_some());
}
