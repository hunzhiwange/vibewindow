use std::str::FromStr;

#[test]
fn parses_autonomy_levels_and_identity_defaults() {
    assert_eq!(super::AutonomyLevel::from_str("readonly").unwrap(), super::AutonomyLevel::ReadOnly);
    assert_eq!(
        super::AutonomyLevel::from_str("read_only").unwrap(),
        super::AutonomyLevel::ReadOnly
    );
    assert_eq!(
        super::AutonomyLevel::from_str("supervised").unwrap(),
        super::AutonomyLevel::Supervised
    );
    assert!(super::AutonomyLevel::from_str("invalid").is_err());

    let identity = super::IdentityConfig::default();
    assert_eq!(identity.format, "openclaw");
    assert!(identity.aieos_path.is_none());
}

#[test]
fn autonomy_defaults_include_common_safety_guards() {
    let autonomy = super::AutonomyConfig::default();
    assert_eq!(autonomy.level, super::AutonomyLevel::Supervised);
    assert!(autonomy.workspace_only);
    assert!(autonomy.require_approval_for_medium_risk);
    assert!(autonomy.block_high_risk_commands);
    assert_eq!(autonomy.shell_redirect_policy, super::ShellRedirectPolicy::Block);
    assert!(autonomy.auto_approve.contains(&"file_read".to_string()));
    assert!(autonomy.non_cli_excluded_tools.contains(&"bash".to_string()));
}
