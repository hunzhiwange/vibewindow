use super::*;

#[test]
fn is_user_allowed_supports_exact_match_and_wildcard() {
    assert!(is_user_allowed(&["alice".to_string()], "alice"));
    assert!(!is_user_allowed(&["alice".to_string()], "bob"));
    assert!(is_user_allowed(&["*".to_string()], "anyone"));
}

#[test]
fn is_group_sender_trigger_enabled_rejects_blank_sender() {
    assert!(!is_group_sender_trigger_enabled(&["*".to_string()], "  "));
    assert!(is_group_sender_trigger_enabled(&["sender-1".to_string()], " sender-1 "));
}
