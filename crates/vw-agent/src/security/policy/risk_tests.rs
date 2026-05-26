use super::*;
use super::super::CommandRiskLevel;

#[test]
fn classifies_destructive_commands_as_high_risk() {
    assert!(matches!(classify_command_risk("rm -rf /"), CommandRiskLevel::High));
    assert!(matches!(classify_command_risk("cat README.md"), CommandRiskLevel::Low));
}

