use super::*;

#[test]
fn is_context_window_overflow_error_matches_known_provider_phrases() {
    assert!(is_context_window_overflow_error(&anyhow::anyhow!("maximum context length reached")));
    assert!(is_context_window_overflow_error(&anyhow::anyhow!("too many tokens")));
    assert!(!is_context_window_overflow_error(&anyhow::anyhow!("ordinary failure")));
}
