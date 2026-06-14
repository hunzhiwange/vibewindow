use super::{GitMessage, update};
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("modal_tests"));
}

#[test]
fn custom_diff_modal_open_close_and_title_change_update_state() {
    let mut app = test_app();
    app.git_custom_diff_title.clear();

    let _ = update(&mut app, GitMessage::OpenCustomDiffModal);
    assert!(app.show_git_custom_diff_modal);
    assert!(!app.git_custom_diff_hide_inputs);
    assert_eq!(app.git_custom_diff_title, "自定义对比");

    let _ = update(&mut app, GitMessage::CustomDiffTitleChanged("Review".to_string()));
    assert_eq!(app.git_custom_diff_title, "Review");

    let _ = update(&mut app, GitMessage::CloseCustomDiffModal);
    assert!(!app.show_git_custom_diff_modal);
    assert!(!app.git_custom_diff_hide_inputs);
}

#[test]
fn custom_diff_result_hides_inputs_and_shows_diff_view() {
    let mut app = test_app();
    app.active_preview_path = Some("/tmp/a.rs".to_string());
    app.show_diff = false;

    let _ = update(
        &mut app,
        GitMessage::OpenCustomDiffResult {
            title: "Patch".to_string(),
            before: "old".to_string(),
            after: "new".to_string(),
        },
    );

    assert!(app.show_git_custom_diff_modal);
    assert!(app.git_custom_diff_hide_inputs);
    assert_eq!(app.git_custom_diff_title, "Patch");
    assert_eq!(app.git_custom_diff_before_editor.text(), "old");
    assert_eq!(app.git_custom_diff_after_editor.text(), "new");
    assert_eq!(app.active_preview_path, None);
    assert!(app.show_diff);
}

#[test]
fn chat_text_diff_open_and_close_update_panel_state() {
    let mut app = test_app();

    let _ = update(
        &mut app,
        GitMessage::OpenChatTextDiff {
            title: "Chat patch".to_string(),
            file: "main.rs".to_string(),
            before: "a".to_string(),
            after: "b".to_string(),
        },
    );

    let diff = app.chat_text_diff.as_ref().expect("chat text diff");
    assert_eq!(diff.title, "Chat patch");
    assert_eq!(diff.file, "main.rs");
    assert!(app.show_diff);
    assert!(app.file_manager_show_changes);
    assert!(!app.show_git_custom_diff_modal);

    let _ = update(&mut app, GitMessage::CloseChatTextDiff);
    assert!(app.chat_text_diff.is_none());
}

#[test]
fn copy_modal_with_text_uses_plain_editor_for_large_text() {
    let mut app = test_app();
    let large = "x".repeat(260_000);

    let _ = update(&mut app, GitMessage::OpenCopyModalWithText(large.clone()));

    assert!(app.show_git_copy_modal);
    assert!(!app.git_copy_modal_use_color);
    assert_eq!(app.git_copy_modal_editor.text(), large);
}

#[test]
fn copy_modal_color_toggle_falls_back_for_large_text() {
    let mut app = test_app();
    let large = "x".repeat(260_000);
    let _ = update(&mut app, GitMessage::OpenCopyModalWithText(large));

    let _ = update(&mut app, GitMessage::ToggleCopyModalColored(true));

    assert!(!app.git_copy_modal_use_color);
}

#[test]
fn copy_modal_close_and_insert_text_hide_modal() {
    let mut app = test_app();
    app.show_git_copy_modal = true;

    let _ = update(&mut app, GitMessage::CloseCopyModal);
    assert!(!app.show_git_copy_modal);

    app.show_git_copy_modal = true;
    let _ = update(&mut app, GitMessage::InsertCopyModalToChat("hello".to_string()));
    assert!(!app.show_git_copy_modal);
}
