use serde_json::json;
use std::collections::BTreeMap;

use super::{debug_redacted_value, redact_form_text, redact_string_map, redact_url_for_log};

#[test]
fn http_bridge_debug_redacts_url_query_credentials() {
    let url = redact_url_for_log(
        "https://user:pass@example.test/RestfulApi/goods?dhb_skey=abc123&page=1&token=secret",
    );

    assert!(!url.contains("user:pass"));
    assert!(url.contains("dhb_skey=%5BREDACTED%5D"));
    assert!(url.contains("token=%5BREDACTED%5D"));
    assert!(url.contains("page=1"));
}

#[test]
fn http_bridge_debug_redacts_headers_and_json_values() {
    let headers = BTreeMap::from([
        ("Authorization".to_string(), "Bearer secret".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
    ]);
    let redacted_headers = redact_string_map(&headers);

    assert_eq!(redacted_headers.get("Authorization").map(String::as_str), Some("[REDACTED]"));
    assert_eq!(redacted_headers.get("Content-Type").map(String::as_str), Some("application/json"));

    let value = json!({
        "params": {
            "dhb_skey": "abc123",
            "kw": "手机"
        }
    });
    let debug = debug_redacted_value(&value);

    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("abc123"));
    assert!(debug.contains("手机"));
}

#[test]
fn http_bridge_debug_redacts_form_body_values() {
    let body = redact_form_text("dhb_skey=abc123&page=1&password=secret");

    assert_eq!(body, "dhb_skey=[REDACTED]&page=1&password=[REDACTED]");
}
