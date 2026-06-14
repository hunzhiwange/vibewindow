#![allow(unused_must_use)]
#[test]
fn context_tests_module_is_wired() {
    assert!(module_path!().ends_with("context_tests"));
}

use super::ChatMessage;

#[test]
fn append_text_adds_text_to_current_runtime_editor() {
    let (mut app, _) = crate::app::App::new();

    super::update(&mut app, ChatMessage::AppendText("first".to_string()));
    super::update(&mut app, ChatMessage::AppendText("second".to_string()));

    assert_eq!(app.current_session_runtime().input_editor.text(), "first\nsecond");
    assert_eq!(app.input_editor.text(), "first\nsecond");
}

#[test]
fn append_text_ignores_empty_text() {
    let (mut app, _) = crate::app::App::new();
    app.input_editor = iced::widget::text_editor::Content::with_text("keep");
    app.current_session_runtime_mut().input_editor =
        iced::widget::text_editor::Content::with_text("keep");

    super::update(&mut app, ChatMessage::AppendText(String::new()));

    assert_eq!(app.current_session_runtime().input_editor.text(), "keep");
}

#[test]
fn insert_position_uses_project_relative_path_and_closes_context_menu() {
    let (mut app, _) = crate::app::App::new();
    app.project_path = Some("/workspace".to_string());
    app.show_preview_context_menu = true;

    super::update(
        &mut app,
        ChatMessage::InsertPosition("/workspace/src/main.rs".to_string(), 12, 4),
    );

    assert!(
        app.current_session_runtime()
            .input_editor
            .text()
            .contains("文件:src/main.rs 行:12 列:4")
    );
    assert!(!app.show_preview_context_menu);
}

#[test]
fn insert_selection_positions_uses_context_target_when_present() {
    let (mut app, _) = crate::app::App::new();
    app.project_path = Some("/workspace".to_string());
    app.preview_context_target = Some(("/workspace/src/lib.rs".to_string(), 1, 2, 3, 4));
    app.show_preview_context_menu = true;

    super::update(&mut app, ChatMessage::InsertSelectionPositions);

    let text = app.current_session_runtime().input_editor.text().to_string();
    assert!(text.contains("src/lib.rs"));
    assert!(text.contains("1:2"));
    assert!(text.contains("3:4"));
    assert!(!app.show_preview_context_menu);
}

#[test]
fn insert_selection_positions_without_target_only_closes_menu() {
    let (mut app, _) = crate::app::App::new();
    app.show_preview_context_menu = true;

    super::update(&mut app, ChatMessage::InsertSelectionPositions);

    assert_eq!(app.current_session_runtime().input_editor.text(), "");
    assert!(!app.show_preview_context_menu);
}
