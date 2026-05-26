use std::collections::HashMap;

use crate::app::agent::config::{AutonomyConfig, NonCliNaturalLanguageApprovalMode};
use crate::app::agent::security::AutonomyLevel;

use super::{ApprovalManager, ApprovalResponse};

#[test]
fn policy_applies_auto_approve_always_ask_and_session_allowlist() {
    let config = AutonomyConfig {
        level: AutonomyLevel::Supervised,
        auto_approve: vec!["file_read".to_string()],
        always_ask: vec!["shell".to_string()],
        ..AutonomyConfig::default()
    };
    let manager = ApprovalManager::from_config(&config);

    assert!(!manager.needs_approval("file_read"));
    assert!(manager.needs_approval("file_write"));

    manager.record_decision(
        "file_write",
        &serde_json::json!({"path": "a"}),
        ApprovalResponse::Always,
        "cli",
    );
    assert!(!manager.needs_approval("file_write"));
    assert!(manager.needs_approval("shell"));
}

#[test]
fn policy_normalizes_approvers_and_channel_modes() {
    let mut modes = HashMap::new();
    modes.insert(" Discord ".to_string(), NonCliNaturalLanguageApprovalMode::Disabled);
    let config = AutonomyConfig {
        non_cli_approval_approvers: vec![" alice ".to_string(), "".to_string()],
        non_cli_natural_language_approval_mode_by_channel: modes,
        ..AutonomyConfig::default()
    };
    let manager = ApprovalManager::from_config(&config);

    assert!(manager.non_cli_approval_approvers().contains("alice"));
    assert_eq!(
        manager.non_cli_natural_language_approval_mode_for_channel("DISCORD"),
        NonCliNaturalLanguageApprovalMode::Disabled
    );
}
