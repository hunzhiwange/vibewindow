use super::*;
use crate::tools::shell::ast::parse_command;
use crate::tools::shell::security::{SecurityValidator, Severity};

fn findings(command: &str) -> Vec<crate::tools::shell::security::SecurityFinding> {
    HeredocValidator.validate(&parse_command(command))
}

#[test]
fn name_is_stable() {
    assert_eq!(HeredocValidator.name(), "heredoc");
}

#[test]
fn blocks_representative_risky_input() {
    let findings = findings("cat <<EOF");
    assert!(findings.iter().any(|finding| finding.severity == Severity::Block));
}

#[test]
fn allows_representative_plain_input() {
    assert!(findings("cat <<'EOF'").is_empty());
}
