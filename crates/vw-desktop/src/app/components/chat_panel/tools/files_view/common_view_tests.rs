use super::common_view::build_common_tool_view;
use super::{FileListState, FilesViewContext};
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn state(total_items: usize, is_search: bool) -> FileListState {
    FileListState {
        items_for_display: Vec::new(),
        total_items,
        display_count: total_items,
        truncated_middle: false,
        middle_omitted: 0,
        tail_omitted: 0,
        filter_query: String::new(),
        is_empty_filtered: total_items == 0,
        max_items: 100,
        is_search,
    }
}

fn ctx<'a>(app: &'a App, tool_name: &str, status: &str, output: &str) -> FilesViewContext<'a> {
    FilesViewContext {
        app,
        msg_idx: 0,
        tool_idx: 1,
        visible: format!(
            r#"tool {tool_name}
{{"status":"{status}","output":"{output}"}}"#
        ),
        tool_name: tool_name.to_string(),
        error_text: (status == "error").then(|| output.to_string()),
        input: "{}".to_string(),
        output: output.to_string(),
        verb: "读取",
        is_error: status == "error",
        is_running: status == "running",
        is_edit_like: false,
        read_range: None,
    }
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn common_view_test_module_is_linked() {
    assert_eq!("common_view", "common_view");
}

#[test]
fn build_common_tool_view_handles_running_and_empty_list() {
    let app = test_app();
    let view_ctx = ctx(&app, "read", "running", "");
    let list_column = iced::widget::column![];

    keep_element(build_common_tool_view(&view_ctx, &state(0, false), list_column));
}

#[test]
fn build_common_tool_view_handles_error_fallback_text() {
    let app = test_app();
    let view_ctx = ctx(&app, "read", "error", "permission denied");
    let list_column = iced::widget::column![];

    keep_element(build_common_tool_view(&view_ctx, &state(0, false), list_column));
}

#[test]
fn build_common_tool_view_handles_search_filter_meta_and_tail_omission() {
    let app = test_app();
    let view_ctx = ctx(&app, "grep", "completed", "");
    let mut render_state = state(12, true);
    render_state.display_count = 5;
    render_state.filter_query = "main".to_string();
    render_state.tail_omitted = 7;
    let list_column = iced::widget::column![];

    keep_element(build_common_tool_view(&view_ctx, &render_state, list_column));
}

#[test]
fn build_common_tool_view_handles_non_error_output_fallback() {
    let app = test_app();
    let view_ctx = ctx(&app, "bash", "completed", "plain output\nsecond line");
    let list_column = iced::widget::column![];

    keep_element(build_common_tool_view(&view_ctx, &state(0, false), list_column));
}
