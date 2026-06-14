use super::list_ui::{
    build_file_list_column, fallback_edit_diff_payload, file_row_label, tool_file_hover_key,
};
use super::{FileListState, FilesViewContext};
use crate::app::{App, Message};
use std::collections::HashMap;

fn test_app() -> App {
    let mut app = App::new().0;
    app.project_path = Some("/tmp/vibe-window".to_string());
    app
}

fn state(items: Vec<(String, String)>, is_search: bool) -> FileListState {
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
        is_search,
    }
}

fn ctx<'a>(app: &'a App, tool_name: &str, input: &str, output: &str) -> FilesViewContext<'a> {
    FilesViewContext {
        app,
        msg_idx: 1,
        tool_idx: 2,
        visible: format!("tool {tool_name}\n{{}}"),
        tool_name: tool_name.to_string(),
        error_text: None,
        input: input.to_string(),
        output: output.to_string(),
        verb: "写入",
        is_error: false,
        is_running: false,
        is_edit_like: matches!(tool_name, "write" | "file_write" | "file_edit"),
        read_range: None,
    }
}

fn keep_column(column: iced::widget::Column<'_, Message>) {
    std::hint::black_box(column);
}

#[test]
fn tool_file_hover_key_includes_indices_and_path() {
    assert_eq!(tool_file_hover_key(1, 2, "/tmp/a.rs"), "1:2:/tmp/a.rs");
}

#[test]
fn file_row_label_uses_relative_path_for_edit_tools() {
    let app = test_app();
    let view_ctx = ctx(&app, "write", r#"{"path":"src/main.rs"}"#, "");

    assert_eq!(
        file_row_label(&app, &view_ctx, "display.rs", "/tmp/vibe-window/src/main.rs"),
        "src/main.rs"
    );
}

#[test]
fn file_row_label_uses_file_name_for_non_edit_tools() {
    let app = test_app();
    let mut view_ctx = ctx(&app, "read", r#"{"path":"src/main.rs"}"#, "");
    view_ctx.is_edit_like = false;

    assert_eq!(
        file_row_label(&app, &view_ctx, "src/main.rs", "/tmp/vibe-window/src/main.rs"),
        "main.rs"
    );
    assert_eq!(file_row_label(&app, &view_ctx, "README", "/tmp/vibe-window/README"), "README");
}

#[test]
fn fallback_edit_diff_payload_requires_edit_tool_matching_input_and_preview() {
    let app = test_app();
    let view_ctx = ctx(&app, "write", r#"{"path":"src/main.rs","content":"fn main() {}"}"#, "");

    let payload =
        fallback_edit_diff_payload(&app, &view_ctx, "src/main.rs", "/tmp/vibe-window/src/main.rs")
            .expect("write preview payload");

    assert_eq!(payload.0, "src/main.rs  写入内容");
    assert_eq!(payload.1, "src/main.rs");
    assert!(payload.2.contains("fn main"));
    assert!(
        fallback_edit_diff_payload(&app, &view_ctx, "src/lib.rs", "/tmp/vibe-window/src/lib.rs")
            .is_none()
    );
}

#[test]
fn build_file_list_column_handles_empty_search_and_plain_empty_states() {
    let app = test_app();
    let view_ctx = ctx(&app, "grep", "{}", "");
    let mut search_state = state(Vec::new(), true);
    search_state.filter_query = "missing".to_string();
    search_state.is_empty_filtered = true;

    keep_column(build_file_list_column(&app, &search_state, &view_ctx, &HashMap::new()));

    let plain_state = state(Vec::new(), false);
    keep_column(build_file_list_column(&app, &plain_state, &view_ctx, &HashMap::new()));
}

#[test]
fn build_file_list_column_handles_tail_and_middle_omissions() {
    let app = test_app();
    let view_ctx = ctx(&app, "read", r#"{"path":"src/main.rs"}"#, "");
    let mut render_state = state(
        vec![
            ("src/a.rs".to_string(), "/tmp/vibe-window/src/a.rs".to_string()),
            ("src/z.rs".to_string(), "/tmp/vibe-window/src/z.rs".to_string()),
        ],
        false,
    );
    render_state.truncated_middle = true;
    render_state.max_items = 2;
    render_state.tail_omitted = 3;

    keep_column(build_file_list_column(&app, &render_state, &view_ctx, &HashMap::new()));
}
