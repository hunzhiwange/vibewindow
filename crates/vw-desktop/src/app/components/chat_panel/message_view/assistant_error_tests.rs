use super::assistant_error::extract_url_from_error_message;

#[test]
fn extract_url_from_error_message_trims_common_wrappers() {
    assert_eq!(
        extract_url_from_error_message("request failed (https://api.example.test/v1)"),
        Some("https://api.example.test/v1".to_string())
    );
    assert_eq!(
        extract_url_from_error_message("\"http://localhost:11434/api\" timed out"),
        Some("http://localhost:11434/api".to_string())
    );
}

#[test]
fn extract_url_from_error_message_returns_none_without_url() {
    assert_eq!(extract_url_from_error_message("network unavailable"), None);
}
