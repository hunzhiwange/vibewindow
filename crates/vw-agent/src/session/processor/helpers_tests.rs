use super::*;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn allowed_tool_ids_include_aliases_and_request_filter_intersects_base_set() {
    let base = allowed_tool_ids(None);
    assert!(!base.is_empty());

    let selected = base.iter().next().expect("at least one registered tool").clone();
    let filtered = allowed_tool_ids_for_request(
        None,
        &json!({"allowed_tools": [selected.clone(), "missing-tool", "  ", 42]}),
    );
    assert_eq!(filtered, HashSet::from([selected]));

    assert_eq!(allowed_tool_ids_for_request(None, &json!({})), base);
    assert_eq!(allowed_tool_ids_for_request(None, &json!({"allowed_tools": ["  ", 42]})), base);
}

#[test]
fn response_preview_trims_scrubs_credentials_and_truncates() {
    let secret = "abcdefghijklmnopqrstuvwxyz";
    let preview = response_preview(&format!("  token={secret}  "));

    assert_eq!(preview, "token=abcd*[REDACTED]");
    assert!(!preview.contains(secret));

    let long_preview = response_preview(&"x".repeat(300));
    assert!(long_preview.len() < 300);
    assert!(long_preview.ends_with("..."));
}

#[test]
fn tool_call_preview_sanitizes_large_or_sensitive_arguments() {
    let preview = tool_call_preview(
        "file_write",
        r#"{"path":"/tmp/example","content":"full file content should be omitted"}"#,
    );

    assert!(preview.starts_with("file_write("));
    assert!(preview.contains("<omitted"));
    assert!(!preview.contains("full file content should be omitted"));
}

#[test]
fn acp_detection_accepts_test_flag_or_non_empty_agent() {
    assert!(is_acp_request(&json!({"acp_test": true})));
    assert!(is_acp_request(&json!({"acp_agent": "copilot"})));
    assert!(!is_acp_request(&json!({"acp_test": false, "acp_agent": "  "})));
    assert!(!is_acp_request(&json!({})));
}

#[test]
fn structured_tool_calls_run_locally_only_outside_acp() {
    assert!(!should_execute_structured_tool_calls_locally(&json!({"acp_test": true})));
    assert!(should_execute_structured_tool_calls_locally(&json!({"acp_test": false})));
}
