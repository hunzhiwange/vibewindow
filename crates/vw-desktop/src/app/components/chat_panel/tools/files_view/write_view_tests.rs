use super::FileListState;
use super::write_view::write_tool_summary;

fn file_list_state(items: Vec<(String, String)>) -> FileListState {
    FileListState {
        total_items: items.len(),
        display_count: items.len(),
        items_for_display: items,
        truncated_middle: false,
        middle_omitted: 0,
        tail_omitted: 0,
        filter_query: String::new(),
        is_empty_filtered: false,
        max_items: 100,
        is_search: false,
    }
}

#[test]
fn write_tool_summary_uses_first_file_name_and_total_count() {
    let state = file_list_state(vec![
        ("src/main.rs".to_string(), "/tmp/src/main.rs".to_string()),
        ("src/lib.rs".to_string(), "/tmp/src/lib.rs".to_string()),
    ]);

    assert_eq!(write_tool_summary(&state), Some("main.rs 等 2 个文件".to_string()));
}

#[test]
fn write_tool_summary_uses_single_file_name() {
    let state = file_list_state(vec![("src/main.rs".to_string(), "/tmp/src/main.rs".to_string())]);

    assert_eq!(write_tool_summary(&state), Some("main.rs".to_string()));
}

#[test]
fn write_tool_summary_falls_back_to_file_count_for_blank_names() {
    let state = file_list_state(vec![("".to_string(), "/tmp/src/main.rs".to_string())]);

    assert_eq!(write_tool_summary(&state), Some("1 个文件".to_string()));
}

#[test]
fn write_tool_summary_returns_none_for_empty_state() {
    assert_eq!(write_tool_summary(&file_list_state(Vec::new())), None);
}
