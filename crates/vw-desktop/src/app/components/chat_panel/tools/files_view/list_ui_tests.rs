use super::list_ui::tool_file_hover_key;

#[test]
fn tool_file_hover_key_includes_indices_and_path() {
    assert_eq!(tool_file_hover_key(1, 2, "/tmp/a.rs"), "1:2:/tmp/a.rs");
}
