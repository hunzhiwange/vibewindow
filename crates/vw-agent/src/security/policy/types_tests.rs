use super::*;

#[test]
fn display_formats_command_risk_levels() {
    assert_eq!(CommandRiskLevel::Low.to_string(), "low");
    assert_eq!(CommandRiskLevel::Medium.to_string(), "medium");
    assert_eq!(CommandRiskLevel::High.to_string(), "high");
}

