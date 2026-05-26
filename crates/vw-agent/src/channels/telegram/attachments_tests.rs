use super::attachments::{
    format_attachment_content, is_http_url, is_image_extension, sanitize_attachment_filename,
    sanitize_generated_extension, IncomingAttachmentKind, TelegramAttachmentKind,
};
use std::path::Path;

#[test]
fn attachment_kind_from_marker_accepts_document_aliases() {
    assert_eq!(TelegramAttachmentKind::from_marker(" photo "), Some(TelegramAttachmentKind::Image));
    assert_eq!(TelegramAttachmentKind::from_marker("file"), Some(TelegramAttachmentKind::Document));
    assert_eq!(TelegramAttachmentKind::from_marker("unknown"), None);
}

#[test]
fn image_extension_is_case_insensitive_and_rejects_text() {
    assert!(is_image_extension(Path::new("A.PNG")));
    assert!(!is_image_extension(Path::new("notes.md")));
}

#[test]
fn attachment_content_only_uses_image_marker_for_image_paths() {
    let image = format_attachment_content(
        IncomingAttachmentKind::Photo,
        "photo.jpg",
        Path::new("/tmp/photo.jpg"),
    );
    let text = format_attachment_content(
        IncomingAttachmentKind::Photo,
        "notes.md",
        Path::new("/tmp/notes.md"),
    );

    assert_eq!(image, "[IMAGE:/tmp/photo.jpg]");
    assert!(text.starts_with("[Document: notes.md]"));
}

#[test]
fn attachment_filename_sanitizer_rejects_directory_markers() {
    assert_eq!(sanitize_attachment_filename("../report.pdf").as_deref(), Some("report.pdf"));
    assert_eq!(sanitize_attachment_filename(".."), None);
    assert_eq!(sanitize_generated_extension("Jp*g!VeryLong").as_str(), "jpgveryl");
    assert!(is_http_url("https://example.test/file.png"));
    assert!(!is_http_url("ftp://example.test/file.png"));
}
