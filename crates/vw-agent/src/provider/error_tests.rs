use super::*;

#[test]
fn overflow_detection_matches_common_provider_messages() {
    assert!(is_overflow("context length exceeded for this model"));
    assert!(!is_overflow("temporary network error"));
}

#[test]
fn parses_stream_error_json_payload() {
    let parsed = parse_stream_error(r#"{"error":{"message":"too many tokens","type":"invalid_request_error"}}"#);
    assert!(parsed.is_some());
}

