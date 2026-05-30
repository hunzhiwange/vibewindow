use super::attachments::{TelegramAttachmentKind, is_http_url};

#[test]
fn media_url_detection_allows_only_http_and_https() {
    assert!(is_http_url("http://example.test/file"));
    assert!(is_http_url("https://example.test/file"));
    assert!(!is_http_url("file:///tmp/file"));
    assert_eq!(TelegramAttachmentKind::from_marker("video"), Some(TelegramAttachmentKind::Video));
}
