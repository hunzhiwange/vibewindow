use super::*;
use axum::http::HeaderValue;

#[test]
fn cors_origin_allows_localhost_and_configured_origin() {
    assert!(cors_origin_allowed(&HeaderValue::from_static("http://localhost:3000"), &[]));
    assert!(cors_origin_allowed(
        &HeaderValue::from_static("https://example.com"),
        &["https://example.com".to_string()]
    ));
}

#[test]
fn cors_origin_rejects_unlisted_remote_origin() {
    assert!(!cors_origin_allowed(&HeaderValue::from_static("https://evil.example"), &[]));
}
