use super::parse::{
    build_file_list_state, is_edit_like_tool, is_git_diff_tool, is_search_tool, parse_read_range,
    should_skip_files_view,
};

#[test]
fn tool_classification_is_explicit() {
    assert!(is_git_diff_tool("git_diff", ""));
    assert!(is_git_diff_tool("git_operations", r#"{"operation":"diff"}"#));
    assert!(should_skip_files_view("apply_patch", ""));
    assert!(is_edit_like_tool("file_edit"));
    assert!(is_search_tool("grep"));
}

#[test]
fn parse_read_range_formats_present_fields() {
    assert_eq!(
        parse_read_range("read", r#"{"offset":0,"limit":20}"#),
        Some("offset=1, limit=20".to_string())
    );
    assert_eq!(parse_read_range("bash", r#"{"offset":1}"#), None);
}

#[test]
fn build_file_list_state_filters_search_results() {
    let items = vec![
        ("src/main.rs".to_string(), "/tmp/src/main.rs".to_string()),
        ("README.md".to_string(), "/tmp/README.md".to_string()),
    ];

    let state = build_file_list_state(items, true, "main", 100);

    assert_eq!(state.display_count, 1);
    assert_eq!(state.items_for_display[0].0, "src/main.rs");
    assert_eq!(state.filter_query, "main");
}
