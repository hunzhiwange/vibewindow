use super::*;
use crate::app::agent::skills::audit::report::SkillAuditReport;

#[test]
fn markdown_audit_flags_unsafe_links() {
    let dir = tempfile::tempdir().expect("temp dir");
    let markdown = dir.path().join("SKILL.md");
    std::fs::write(
        &markdown,
        "[outside](/etc/passwd)\n[script](run.sh)\n[remote](https://example.com/guide.md)",
    )
    .unwrap();
    let mut report = SkillAuditReport::default();

    audit_markdown_file(dir.path(), &markdown, &mut report).unwrap();
    let summary = report.summary();
    assert!(summary.contains("absolute markdown link"));
    assert!(summary.contains("script files"));
    assert!(summary.contains("remote markdown links"));
}

#[test]
fn cross_skill_references_are_detected() {
    assert!(is_cross_skill_reference("../shared/SKILL.md"));
    assert!(!is_cross_skill_reference("docs/guide.md"));
}
