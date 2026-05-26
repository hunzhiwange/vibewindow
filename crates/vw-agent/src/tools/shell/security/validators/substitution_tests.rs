use super::substitution::SubstitutionValidator;
use crate::tools::shell::ast::parse_command;
use crate::tools::shell::security::{SecurityValidator, Severity};

#[test]
fn name_is_stable() {
    assert_eq!(SubstitutionValidator::new(true).name(), "substitution");
}

#[test]
fn strict_mode_blocks_substitution() {
    let findings = SubstitutionValidator::new(true).validate(&parse_command("echo $(date)"));

    assert_eq!(findings[0].severity, Severity::Block);
}

#[test]
fn relaxed_mode_warns_for_substitution() {
    let findings = SubstitutionValidator::new(false).validate(&parse_command("cat <(echo ok)"));

    assert_eq!(findings[0].severity, Severity::Warn);
}
