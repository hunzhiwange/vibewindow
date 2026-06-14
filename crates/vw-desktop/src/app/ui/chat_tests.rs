#[test]
fn format_position_keeps_file_line_and_column() {
    assert_eq!(super::format_position("src/main.rs", 7, 3), "文件:src/main.rs 行:7 列:3");
    assert_eq!(
        super::format_selection_positions("src/main.rs", 1, 2, 3, 4),
        "@src/main.rs:1:2-3:4"
    );
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
    assert_eq!(
        visible,
        "before
after"
    );
    assert!(!open);
}

#[test]
fn append_line_adds_separator_only_when_content_exists() {
    let mut editor = iced::widget::text_editor::Content::new();
    super::append_line(&mut editor, "first");
    assert_eq!(editor.text(), "first");

    super::append_line(&mut editor, "second");
    assert_eq!(editor.text(), "first\nsecond");
}

#[test]
fn insert_at_cursor_pastes_text_without_replacing_existing_content() {
    let mut editor = iced::widget::text_editor::Content::with_text("hello");
    super::insert_at_cursor(&mut editor, " world");
    assert_eq!(editor.text(), "hello world");
}

#[test]
fn split_think_reports_open_think_when_close_tag_is_missing() {
    let (thinks, visible, open) = super::split_think("visible <think>still thinking");
    assert_eq!(thinks, vec!["still thinking".to_string()]);
    assert_eq!(visible, "visible ");
    assert!(open);
}

#[test]
fn split_think_keeps_tool_block_visible_when_inside_closed_think() {
    let raw = r#"before<think>secret
tool call
{"ok":true}
</think>after"#;
    let (thinks, visible, open) = super::split_think(raw);
    assert_eq!(thinks, vec!["secret\n".to_string()]);
    assert_eq!(visible, "beforeafter");
    assert!(!open);
}

#[test]
fn split_think_ignores_tags_with_non_whitespace_suffixes() {
    let (thinks, visible, open) = super::split_think("a <thinking>nope</thinking> b");
    assert!(thinks.is_empty());
    assert_eq!(visible, "a <thinking>nope</thinking> b");
    assert!(!open);
}

#[test]
fn split_think_normalizes_repeated_blank_lines_outside_code_fences() {
    let raw = "a  \n\n\n```rust\nfn main() { }  \n```\n\n b\t\n";
    let (_thinks, visible, _open) = super::split_think(raw);
    assert_eq!(visible, "a\n```rust\nfn main() { }  \n```\n\n b");
}
