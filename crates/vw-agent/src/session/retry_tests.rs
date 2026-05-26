use super::{delay, header_value_case_insensitive, parse_json_message, pow_u64};
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
}
