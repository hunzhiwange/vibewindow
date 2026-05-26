use super::*;

#[test]
fn default_policy_is_supervised_and_workspace_only() {
    let policy = SecurityPolicy::default();
    assert!(matches!(policy.autonomy, AutonomyLevel::Supervised));
    assert!(policy.workspace_only);
    assert!(policy.block_high_risk_commands);
}

