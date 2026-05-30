use super::git_diff_view::{
    append_preview_gap, append_preview_line, is_git_diff_tool, parse_git_diff_previews,
};
use serde_json::json;

#[test]
fn append_preview_helpers_preserve_line_boundaries() {
    let mut buf = String::new();
    append_preview_line(&mut buf, "first");
    append_preview_gap(&mut buf);
    append_preview_line(&mut buf, "last");

    assert!(buf.contains("first"));
    assert!(buf.contains("last"));
}

#[test]
fn git_operations_diff_is_treated_as_git_diff_tool() {
    assert!(is_git_diff_tool("git_diff", ""));
    assert!(is_git_diff_tool("git_operations", r#"{"operation":"diff"}"#));
    assert!(!is_git_diff_tool("git_operations", r#"{"operation":"status"}"#));
}

#[test]
fn parse_git_diff_previews_returns_none_without_diff_data() {
    assert!(parse_git_diff_previews("", &json!({})).is_none());
}
