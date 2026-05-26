use super::utils::{parse_rfc3339, truncate_for_display};

#[test]
fn truncate_for_display_handles_unicode_boundaries() {
    assert_eq!(truncate_for_display("abcdef", 3), "abc…");
    assert_eq!(truncate_for_display("你好世界", 2), "你好…");
}

#[test]
fn parse_rfc3339_accepts_valid_timestamp_and_rejects_invalid_text() {
    assert!(parse_rfc3339("2026-05-24T00:00:00Z").is_some());
    assert!(parse_rfc3339("not-a-date").is_none());
}
