use super::todo_views::{
    todo_tool_card_padding, todo_tool_expanded, tool_todos_view, tool_todowrite_compact_view,
};
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn todo_views_test_module_is_linked() {
    assert_eq!("todo_views", "todo_views");
}

#[test]
fn todo_tool_card_padding_keeps_content_away_from_border() {
    let padding = todo_tool_card_padding();

    assert_eq!(padding.top, 10.0);
    assert_eq!(padding.right, 12.0);
    assert_eq!(padding.bottom, 10.0);
    assert_eq!(padding.left, 12.0);
}

#[test]
fn todo_tool_expanded_ignores_running_until_user_expands() {
    assert!(!todo_tool_expanded(false, true));
    assert!(todo_tool_expanded(true, true));
}

#[test]
fn todowrite_view_handles_collapsed_expanded_and_error_states() {
    let mut app = app();
    let key = (2_u64 << 32) | 3;
    let visible = r#"tool todowrite
{"status":"completed","input":"{\"merge\":true,\"todos\":[{\"status\":\"completed\",\"content\":\"done\"},{\"status\":\"in_progress\",\"content\":\"doing\"},{\"status\":\"pending\",\"content\":\"todo\"}]}"}"#;
    let error = r#"tool todowrite
{"status":"error","input":"{\"todos\":[]}","error":"write failed"}"#;

    assert!(tool_todowrite_compact_view(&app, 2, 3, visible).is_some());
    app.chat_tool_expanded.insert(key);
    assert!(tool_todowrite_compact_view(&app, 2, 3, visible).is_some());
    assert!(tool_todowrite_compact_view(&app, 2, 4, error).is_some());
    assert!(tool_todowrite_compact_view(&app, 2, 5, "tool bash\n{}").is_none());
    assert!(tool_todowrite_compact_view(&app, 2, 6, "tool todowrite\nnot-json").is_none());
}

#[test]
fn todoread_view_sorts_and_renders_parsed_todos() {
    let mut app = app();
    app.chat_tool_expanded.insert((4_u64 << 32) | 5);
    let visible = r#"tool todoread
{"status":"completed","output":"[{\"id\":\"10\",\"content\":\"later\",\"status\":\"pending\"},{\"id\":\"2\",\"content\":\"done\",\"status\":\"completed\"},{\"id\":\"x\",\"content\":\"doing\",\"status\":\"in_progress\"}]"}"#;
    let empty = r#"tool todoread
{"status":"completed","output":"[]"}"#;

    assert!(tool_todos_view(&app, 4, 5, visible).is_some());
    assert!(tool_todos_view(&app, 4, 6, empty).is_some());
    assert!(tool_todos_view(&app, 4, 7, "tool todowrite\n{}").is_none());
    assert!(tool_todos_view(&app, 4, 8, "tool todoread\n{}").is_none());
}
