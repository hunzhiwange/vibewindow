use super::text::{
    normalize_display_text, strip_internal_tool_trace, truncate_chars, truncate_lines_middle,
};

#[test]
fn truncate_chars_and_lines_keep_boundaries() {
    assert_eq!(truncate_chars("abcdef", 3), "abc…");
    assert_eq!(truncate_lines_middle("a\nb\nc\nd", 2, 10), "a\n…\nd");
}

#[test]
fn normalize_display_text_collapses_excess_blank_lines() {
    assert_eq!(normalize_display_text("a\n\n\nb").as_ref(), "a\n\nb");
}

#[test]
fn normalize_display_text_preserves_code_fences() {
    let text = "head\n\n```\nline  \n\n```\n\nfoot";

    assert_eq!(normalize_display_text(text).as_ref(), "head\n\n```\nline  \n\n```\n\nfoot");
}

#[test]
fn strip_internal_tool_trace_removes_compact_tool_lines() {
    let stripped = strip_internal_tool_trace("tool bash(command=\"ls\")\nvisible");

    assert!(stripped.contains("visible"));
    assert!(!stripped.contains("tool bash"));
}

#[test]
fn strip_internal_tool_trace_drops_verbose_tool_leaks() {
    let stripped = strip_internal_tool_trace("Called the read tool\nmetadata\nvisible output");

    assert_eq!(stripped, "visible output");
}
