use super::{
    RETRY_INITIAL_DELAY, RETRY_MAX_DELAY_NO_HEADERS, delay, header_value_case_insensitive,
    parse_json_message, pow_u64, retryable,
};
use crate::app::agent::session::message::AssistantError;
use std::collections::HashMap;

#[test]
fn pow_u64_saturates_on_overflow() {
    assert_eq!(pow_u64(2, 3), 8);
    assert_eq!(pow_u64(u64::MAX, 2), u64::MAX);
}

#[test]
fn delay_prefers_retry_after_headers_case_insensitively() {
    let mut headers = HashMap::new();
    headers.insert("Retry-After-MS".to_string(), "12.4".to_string());
    let error = AssistantError::APIError {
        message: "rate limited".to_string(),
        status_code: Some(429),
        response_body: None,
        response_headers: Some(headers),
        metadata: None,
        is_retryable: true,
    };

    assert_eq!(delay(1, Some(&error)), 13);
}

#[test]
fn header_lookup_and_json_parse_reject_empty_values() {
    let mut headers = HashMap::new();
    headers.insert("Retry-After".to_string(), "2".to_string());

    assert_eq!(header_value_case_insensitive(&headers, "retry-after"), Some("2"));
    assert!(parse_json_message("").is_none());
    assert!(parse_json_message("{\"type\":\"error\"}").is_some());
    assert!(parse_json_message("not json").is_none());
}

#[test]
fn delay_uses_retry_after_seconds_and_exponential_fallbacks() {
    let mut headers = HashMap::new();
    headers.insert("retry-after".to_string(), "1.25".to_string());
    let with_seconds = AssistantError::APIError {
        message: "rate limited".to_string(),
        status_code: Some(429),
        response_body: None,
        response_headers: Some(headers),
        metadata: None,
        is_retryable: true,
    };
    assert_eq!(delay(0, Some(&with_seconds)), 1250);

    let invalid_headers = AssistantError::APIError {
        message: "retry".to_string(),
        status_code: Some(503),
        response_body: None,
        response_headers: Some(HashMap::from([
            ("retry-after-ms".to_string(), "-1".to_string()),
            ("retry-after".to_string(), "nan".to_string()),
        ])),
        metadata: None,
        is_retryable: true,
    };
    assert_eq!(delay(3, Some(&invalid_headers)), RETRY_INITIAL_DELAY * 4);

    assert_eq!(delay(3, None), RETRY_INITIAL_DELAY * 4);
    assert_eq!(delay(99, None), RETRY_MAX_DELAY_NO_HEADERS);
}

#[test]
fn retryable_handles_api_error_branches() {
    let non_retryable = AssistantError::APIError {
        message: "bad request".to_string(),
        status_code: Some(400),
        response_body: None,
        response_headers: None,
        metadata: None,
        is_retryable: false,
    };
    assert!(retryable(&non_retryable).is_none());

    let overloaded = AssistantError::APIError {
        message: "Provider Overloaded".to_string(),
        status_code: Some(529),
        response_body: None,
        response_headers: None,
        metadata: None,
        is_retryable: true,
    };
    assert_eq!(retryable(&overloaded).as_deref(), Some("Provider is overloaded"));

    let plain = AssistantError::APIError {
        message: "temporary".to_string(),
        status_code: Some(500),
        response_body: None,
        response_headers: None,
        metadata: None,
        is_retryable: true,
    };
    assert_eq!(retryable(&plain).as_deref(), Some("temporary"));
}

#[test]
fn retryable_parses_structured_json_errors() {
    let too_many = AssistantError::Unknown {
        message: r#"{"type":"error","error":{"type":"too_many_requests"}}"#.to_string(),
    };
    assert_eq!(retryable(&too_many).as_deref(), Some("Too Many Requests"));

    let exhausted = AssistantError::MessageAbortedError {
        message: r#"{"code":"RESOURCE_EXHAUSTED"}"#.to_string(),
    };
    assert_eq!(retryable(&exhausted).as_deref(), Some("Provider is overloaded"));

    let rate = AssistantError::ProviderAuthError {
        provider_id: "p".to_string(),
        message: r#"{"type":"error","error":{"code":"hard_rate_limit"}}"#.to_string(),
    };
    assert_eq!(retryable(&rate).as_deref(), Some("Rate Limited"));

    let other_json = AssistantError::Unknown { message: r#"{"message":"try later"}"#.to_string() };
    assert_eq!(retryable(&other_json).as_deref(), Some(r#"{"message":"try later"}"#));
}

#[test]
fn retryable_rejects_non_retryable_error_shapes() {
    assert!(
        retryable(&AssistantError::ContextOverflowError {
            message: "too long".to_string(),
            response_body: None,
        })
        .is_none()
    );
    assert!(retryable(&AssistantError::MessageOutputLengthError).is_none());
    assert!(retryable(&AssistantError::Unknown { message: "plain".to_string() }).is_none());
    assert!(retryable(&AssistantError::Unknown { message: "[]".to_string() }).is_none());
}
