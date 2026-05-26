use super::*;

#[test]
fn report_summary_joins_findings_and_clean_tracks_empty() {
    let clean = SkillAuditReport::default();
    assert!(clean.is_clean());
    assert_eq!(clean.summary(), "");

    let report = SkillAuditReport {
        files_scanned: 2,
        findings: vec!["first".to_string(), "second".to_string()],
    };
    assert!(!report.is_clean());
    assert_eq!(report.summary(), "first; second");
}

#[test]
fn markdown_link_helpers_are_conservative() {
    assert!(is_cross_skill_reference("../other/SKILL.md"));
    assert!(looks_like_absolute_path("~/secret.md"));
    assert_eq!(strip_query_and_fragment("docs/readme.md?x=1#top"), "docs/readme.md");
    assert_eq!(url_scheme("https://example.com"), Some("https"));
    assert_eq!(url_scheme("not a scheme:thing"), None);
}
