use super::sort_acp_agent_names;

#[test]
fn acp_settings_message_tests_are_wired() {
    assert!(module_path!().contains("acp_tests"));
}

#[test]
fn sort_acp_agent_names_keeps_codex_first_then_lexicographic_order() {
    let sorted = sort_acp_agent_names(vec![
        "opencode".to_string(),
        "claude".to_string(),
        "codex".to_string(),
        "gemini".to_string(),
    ]);

    assert_eq!(sorted, vec!["codex", "claude", "gemini", "opencode"]);
}

#[test]
fn sort_acp_agent_names_handles_empty_and_missing_codex_lists() {
    assert!(sort_acp_agent_names(Vec::new()).is_empty());

    let sorted = sort_acp_agent_names(vec![
        "zeta".to_string(),
        "alpha".to_string(),
        "openclaw".to_string(),
    ]);

    assert_eq!(sorted, vec!["alpha", "openclaw", "zeta"]);
}

#[test]
fn sort_acp_agent_names_is_stable_for_duplicate_names() {
    let sorted = sort_acp_agent_names(vec![
        "codex".to_string(),
        "codex".to_string(),
        "agent".to_string(),
    ]);

    assert_eq!(sorted, vec!["codex", "codex", "agent"]);
}
