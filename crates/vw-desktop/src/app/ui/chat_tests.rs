#[test]
fn format_position_keeps_file_line_and_column() {
    assert_eq!(super::format_position("src/main.rs", 7, 3), "文件:src/main.rs 行:7 列:3");
    assert_eq!(super::format_selection_positions("src/main.rs", 1, 2, 3, 4), "@src/main.rs:1:2-3:4");
}

#[test]
fn split_think_removes_closed_think_and_reports_visible_text() {
    let (thinks, visible, open) = super::split_think("hello<think>secret</think> world");
    assert_eq!(thinks, vec!["secret".to_string()]);
    assert_eq!(visible, "hello world");
    assert!(!open);
}

#[test]
fn split_think_strips_valid_tool_json_blocks_from_visible_text() {
    let raw = r#"before
tool call
{"ok":true}
after"#;
    let (thinks, visible, open) = super::split_think(raw);
    assert!(thinks.is_empty());
    assert_eq!(visible, "before
after");
    assert!(!open);
}
