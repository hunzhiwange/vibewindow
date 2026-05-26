use super::*;

#[test]
fn guard_action_parses_known_values_with_warn_fallback() {
    assert_eq!(GuardAction::from_str("block"), GuardAction::Block);
    assert_eq!(GuardAction::from_str("sanitize"), GuardAction::Sanitize);
    assert_eq!(GuardAction::from_str("unknown"), GuardAction::Warn);
}

#[test]
fn prompt_guard_allows_benign_text() {
    let result = PromptGuard::default().scan("summarize this project README");
    assert!(matches!(result, GuardResult::Safe));
}
