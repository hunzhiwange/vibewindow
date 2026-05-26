use super::*;
use serde_json::json;

#[test]
fn wildcard_match_respects_simple_globs() {
    assert!(wildcard_match("shell.read", "shell.*"));
    assert!(wildcard_match("abc", "a*c"));
    assert!(!wildcard_match("abc", "a*d"));
}

#[test]
fn evaluate_defaults_to_ask_without_matching_rule() {
    let ruleset = from_config(&json!({"shell.rm": "deny", "shell.read": "ask"}));
    assert_eq!(evaluate("shell.rm", "anything", &[ruleset.clone()]).action, Action::Deny);
    assert_eq!(evaluate("shell.read", "anything", &[ruleset]).action, Action::Ask);
}
