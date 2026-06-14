#![allow(unused_must_use)]
//! 处理聊天输入区的局部消息。
//! 本模块将编辑器操作、文件检索和工具细节限制在输入面板边界内。

use super::file_search::{
    DroppedPath, build_ranked_file_search_entries, format_drop_mentions,
    handle_file_search_input_changed, handle_file_search_navigate_down,
    handle_file_search_navigate_up, handle_file_search_select, handle_file_search_select_current,
    handle_input_area_drag_drop, handle_input_editor_action, handle_input_editor_backspace,
    handle_input_editor_commit, handle_input_editor_delete, handle_input_editor_motion,
    handle_remove_file_reference, ranked_file_search_entries, ranked_file_search_results,
};
use crate::app::{App, FocusArea};
use iced::widget::text_editor;
use std::sync::Arc;

fn test_app() -> App {
    App::new().0
}

fn set_input_text(app: &mut App, text: &str) {
    let runtime = app.current_session_runtime_mut();
    runtime.input_editor = text_editor::Content::with_text(text);
    runtime.input_editor.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
}

#[test]
fn format_drop_mentions_appends_trailing_slash_for_directories() {
    let mentions = format_drop_mentions(
        Some("/workspace"),
        &[DroppedPath { path: "/workspace/src/components".to_string(), is_dir: true }],
        None,
    );

    assert_eq!(mentions, vec!["src/components/".to_string()]);
}

#[test]
fn ranked_file_search_entries_include_deduped_parent_directories() {
    let entries = build_ranked_file_search_entries(
        &["src/main.rs".to_string(), "src/bin/tool.rs".to_string(), "README.md".to_string()],
        "",
    );

    assert!(entries.iter().any(|entry| entry.path == "src/" && entry.is_dir));
    assert!(entries.iter().any(|entry| entry.path == "src/bin/" && entry.is_dir));
    assert_eq!(entries.iter().filter(|entry| entry.path == "src/").count(), 1);
    assert!(entries.iter().any(|entry| entry.path == "README.md" && !entry.is_dir));
}

#[test]
fn ranked_file_search_entries_prioritize_filename_matches() {
    let entries = build_ranked_file_search_entries(
        &[
            "docs/readme.md".to_string(),
            "src/search_panel.rs".to_string(),
            "src/file_search.rs".to_string(),
        ],
        "file_search",
    );

    assert_eq!(entries.first().map(|entry| entry.path.as_str()), Some("src/file_search.rs"));
}

#[test]
fn ranked_file_search_entries_filter_non_matching_queries() {
    let entries =
        build_ranked_file_search_entries(&["src/main.rs".to_string()], "zzzz-not-present");

    assert!(entries.is_empty());
}

#[test]
fn app_ranked_file_search_results_use_refreshed_cache() {
    let mut app = test_app();
    app.project_path = Some("/workspace".to_string());
    app.file_search_query = "main".to_string();
    app.set_file_index("/workspace", vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]);

    assert_eq!(ranked_file_search_results(&app), ["src/main.rs"]);
    assert_eq!(ranked_file_search_entries(&app)[0].path, "src/main.rs");
}

#[test]
fn file_search_input_changed_toggles_visibility_and_resets_selection() {
    let mut app = test_app();
    app.file_search_selected_index = 4;

    handle_file_search_input_changed(&mut app, " src ".to_string());

    assert!(app.show_file_search);
    assert_eq!(app.file_search_query, " src ");
    assert_eq!(app.file_search_selected_index, 0);

    app.file_search_selected_index = 3;
    handle_file_search_input_changed(&mut app, "   ".to_string());

    assert!(!app.show_file_search);
    assert_eq!(app.file_search_selected_index, 0);
}

#[test]
fn input_editor_actions_open_and_clear_file_search() {
    let mut app = test_app();
    app.project_path = Some("/workspace".to_string());
    app.set_file_index("/workspace", vec!["src/main.rs".to_string(), "tests/input.rs".to_string()]);

    handle_input_editor_commit(&mut app, "@src".to_string());

    assert!(app.show_file_search);
    assert_eq!(app.file_search_query, "src");
    assert_eq!(app.focus_area, FocusArea::None);

    handle_input_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(" ".to_string()))),
    );

    assert!(!app.show_file_search);
    assert!(app.file_search_query.is_empty());

    handle_input_editor_commit(&mut app, String::new());
    handle_input_editor_backspace(&mut app);
    handle_input_editor_delete(&mut app);
    handle_input_editor_motion(&mut app, text_editor::Motion::DocumentEnd, false);
    handle_input_editor_motion(&mut app, text_editor::Motion::DocumentStart, true);
}

#[test]
fn file_search_navigation_clamps_to_available_results() {
    let mut app = test_app();
    app.project_path = Some("/workspace".to_string());
    app.file_search_query = "src".to_string();
    app.set_file_index(
        "/workspace",
        vec!["src/a.rs".to_string(), "src/b.rs".to_string(), "src/c.rs".to_string()],
    );

    handle_file_search_navigate_down(&mut app);
    handle_file_search_navigate_down(&mut app);
    assert_eq!(app.file_search_selected_index, 2);

    handle_file_search_navigate_down(&mut app);
    assert_eq!(app.file_search_selected_index, 3);

    handle_file_search_navigate_up(&mut app);
    assert_eq!(app.file_search_selected_index, 2);

    app.file_search_query = "missing".to_string();
    app.refresh_file_search_cache();
    app.file_search_selected_index = 0;
    handle_file_search_navigate_down(&mut app);
    handle_file_search_navigate_up(&mut app);
    assert_eq!(app.file_search_selected_index, 0);
}

#[test]
fn selecting_file_search_path_replaces_active_mention_or_appends() {
    let mut app = test_app();
    app.project_path = Some("/workspace".to_string());

    set_input_text(&mut app, "open @sr");
    handle_file_search_select(&mut app, "/workspace/src/main.rs".to_string());

    assert_eq!(app.current_session_runtime().input_editor.text(), "open @src/main.rs ");
    assert!(!app.show_file_search);
    assert!(app.file_search_query.is_empty());

    set_input_text(&mut app, "plain text");
    handle_file_search_select(&mut app, "tests/input.rs".to_string());

    assert_eq!(app.current_session_runtime().input_editor.text(), "plain text @tests/input.rs ");
}

#[test]
fn selecting_current_file_search_result_uses_selected_index() {
    let mut app = test_app();
    app.project_path = Some("/workspace".to_string());
    app.file_search_query = "src".to_string();
    app.set_file_index("/workspace", vec!["src/a.rs".to_string(), "src/b.rs".to_string()]);
    set_input_text(&mut app, "@s");
    app.file_search_selected_index = 1;

    handle_file_search_select_current(&mut app);

    assert!(app.current_session_runtime().input_editor.text().starts_with("@src/"));

    app.file_search_selected_index = 999;
    let previous = app.current_session_runtime().input_editor.text();
    handle_file_search_select_current(&mut app);
    assert_eq!(app.current_session_runtime().input_editor.text(), previous);
}

#[test]
fn remove_file_reference_deletes_mention_and_compacts_whitespace() {
    let mut app = test_app();
    set_input_text(&mut app, "read @src/main.rs and @tests/input.rs");
    app.file_ref_hovered_index = Some(2);

    handle_remove_file_reference(&mut app, "src/main.rs".to_string());

    assert_eq!(app.current_session_runtime().input_editor.text(), "read and @tests/input.rs");
    assert_eq!(app.file_ref_hovered_index, None);
}

#[test]
fn input_area_drag_drop_inserts_mentions_and_clears_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("main.rs");
    std::fs::write(&file_path, "fn main() {}").expect("write temp file");

    let mut app = test_app();
    app.project_path = Some(temp.path().to_string_lossy().to_string());
    app.pending_drop_file_paths = vec![file_path.to_string_lossy().to_string()];
    app.pending_drop_file_position = Some((9, 2));
    app.input_drop_hovered = true;

    handle_input_area_drag_drop(&mut app);

    assert_eq!(app.current_session_runtime().input_editor.text(), "@main.rs:9:2 ");
    assert!(app.pending_drop_file_paths.is_empty());
    assert_eq!(app.pending_drop_file_position, None);
    assert!(!app.input_drop_hovered);
    assert_eq!(app.focus_area, FocusArea::None);
}

#[test]
fn input_area_drag_drop_clears_empty_pending_drop_without_inserting() {
    let mut app = test_app();
    app.pending_drop_file_paths.clear();
    app.pending_drop_file_position = Some((1, 1));
    app.input_drop_hovered = true;

    handle_input_area_drag_drop(&mut app);

    assert_eq!(app.current_session_runtime().input_editor.text(), "");
    assert_eq!(app.pending_drop_file_position, None);
    assert!(!app.input_drop_hovered);
}

#[test]
fn format_drop_mentions_keeps_single_file_position() {
    let mentions = format_drop_mentions(
        Some("/workspace"),
        &[DroppedPath { path: "/workspace/src/main.rs".to_string(), is_dir: false }],
        Some((12, 4)),
    );

    assert_eq!(mentions, vec!["src/main.rs:12:4".to_string()]);
}

#[test]
fn format_drop_mentions_ignores_position_for_multiple_paths() {
    let mentions = format_drop_mentions(
        Some("/workspace"),
        &[
            DroppedPath { path: "/workspace/src/main.rs".to_string(), is_dir: false },
            DroppedPath { path: "/workspace/src/lib.rs".to_string(), is_dir: false },
        ],
        Some((12, 4)),
    );

    assert_eq!(mentions, vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]);
}
