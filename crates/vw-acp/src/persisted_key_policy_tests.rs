use super::*;
use serde_json::json;

#[test]
fn persisted_key_policy_allows_snake_case_and_zed_tags() {
    let value = json!({
        "session_id": "s1",
        "messages": [{"Agent": {"tool_results": [{"CamelCaseOutput": "allowed under tag"}]}}],
        "agent_capabilities": {"CamelCaseOpaque": true}
    });

    assert!(assert_persisted_key_policy(&value).is_ok());
}

#[test]
fn persisted_key_policy_reports_nested_violations() {
    let violations = find_persisted_key_policy_violations(&json!({
        "badKey": true,
        "nested": {"alsoBad": 1}
    }));

    assert_eq!(violations, vec!["badKey", "nested.alsoBad"]);
}

#[test]
fn snake_case_key_rejects_empty_and_uppercase_starts() {
    assert!(is_snake_case_key("valid_key_1"));
    assert!(!is_snake_case_key(""));
    assert!(!is_snake_case_key("Bad"));
}
