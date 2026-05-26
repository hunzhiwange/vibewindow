use super::*;

#[test]
fn truncate_output_preserves_short_text_and_marks_truncation() {
    assert_eq!(truncate_output("short", 10), "short");
    assert_eq!(truncate_output("abcdef", 3), "abc... (truncated)");
}

#[test]
fn truncate_output_respects_char_boundaries() {
    assert_eq!(truncate_output("你好世界", 2), "你好... (truncated)");
}
