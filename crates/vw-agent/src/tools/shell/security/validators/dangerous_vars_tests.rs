use super::*;
use crate::tools::shell::ast::parse_command;
use crate::tools::shell::security::{SecurityValidator, Severity};

fn findings(command: &str) -> Vec<crate::tools::shell::security::SecurityFinding> {
    DangerousVarsValidator.validate(&parse_command(command))
}

#[test]
fn name_is_stable() {
    assert_eq!(DangerousVarsValidator.name(), "dangerous_vars");
}

#[test]
fn blocks_representative_risky_input() {
    let findings = findings("echo $BASH_EXECUTION_STRING");
    assert!(findings.iter().any(|finding| finding.severity == Severity::Block));
}

#[test]
fn allows_representative_plain_input() {
    assert!(findings("echo $HOME").is_empty());
}
