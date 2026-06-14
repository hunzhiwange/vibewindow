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
fn persisted_key_policy_descends_short_output_and_result_paths() {
    let violations = find_persisted_key_policy_violations(&json!({
        "output": {
            "badKey": true
        },
        "result": {
            "alsoBad": true
        }
    }));

    assert_eq!(violations, vec!["output.badKey", "result.alsoBad"]);
}

#[test]
fn persisted_key_policy_only_skips_exact_tool_result_payload_paths() {
    let violations = find_persisted_key_policy_violations(&json!({
        "messages": {
            "Agent": {
                "tool_results": {
                    "tool-1": {
                        "output": {
                            "camelCaseAllowed": true
                        },
                        "result": {
                            "alsoCamelCaseAllowed": true
                        },
                        "nested": {
                            "output": {
                                "badKey": true
                            }
                        }
                    }
                }
            }
        },
        "other": {
            "Agent": {
                "tool_results": {
                    "tool-2": {
                        "output": {
                            "wrongPrefixBadKey": true
                        }
                    }
                }
            }
        }
    }));

    assert_eq!(
        violations,
        vec![
            "messages.Agent.tool_results.tool-1.nested.output.badKey",
            "other.Agent.tool_results.tool-2",
            "other.Agent.tool_results.tool-2.output.wrongPrefixBadKey",
        ]
    );
}

#[test]
fn snake_case_key_rejects_empty_and_uppercase_starts() {
    assert!(is_snake_case_key("valid_key_1"));
    assert!(!is_snake_case_key(""));
    assert!(!is_snake_case_key("Bad"));
}

#[test]
fn tool_result_field_path_helper_rejects_short_paths() {
    let path = vec!["messages".to_string(), "Agent".to_string()];

    assert!(!matches_tool_result_field_path(&path));
}
