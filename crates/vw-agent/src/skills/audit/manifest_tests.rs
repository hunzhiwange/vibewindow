use super::*;
use crate::app::agent::skills::audit::report::SkillAuditReport;

#[test]
fn manifest_audit_flags_shell_chaining_and_missing_commands() {
    let dir = tempfile::tempdir().expect("temp dir");
    let manifest = dir.path().join("SKILL.toml");
    std::fs::write(
        &manifest,
        r#"
[[tools]]
kind = "shell"
command = "echo ok && rm file"

[[tools]]
kind = "shell"
"#,
    )
    .unwrap();
    let mut report = SkillAuditReport::default();

    audit_manifest_file(dir.path(), &manifest, &mut report).unwrap();

    assert!(report.summary().contains("shell chaining"));
    assert!(report.summary().contains("missing a command"));
}
