use super::auth::extract_bearer_token;
use axum::http::{header, HeaderMap, HeaderValue};

#[test]
fn extract_bearer_token_requires_bearer_prefix() {
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer token-a"));

    assert_eq!(extract_bearer_token(&headers), Some("token-a"));

    headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic token-a"));
    assert_eq!(extract_bearer_token(&headers), None);
}
