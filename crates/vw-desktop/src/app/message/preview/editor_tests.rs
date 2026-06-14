//! 覆盖预览编辑器消息处理的本地编辑器、菜单和定位行为。

use super::{
    cursor_from_point, editor_message_may_change_content, normalize_pos,
    update_context_target_from_editor_event,
};
use crate::app::components::editor::Editor;
use crate::app::message::preview::{PreviewMessage, update};
use crate::app::preview::PreviewTab;
use crate::app::{App, FocusArea, PreviewAutoSaveMode};
use iced::widget::Id;

fn test_app() -> App {
    App::new().0
}

fn tab(path: &str, content: &str) -> PreviewTab {
    PreviewTab {
        path: path.to_string(),
        title: path.rsplit('/').next().unwrap_or(path).to_string(),
        content: content.to_string(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: Editor::new(content, "rust"),
        scroll_id: Id::unique(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_server_key: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_uri: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_language_id: None,
    }
}

#[test]
fn content_change_classifier_matches_editing_events() {
    assert!(editor_message_may_change_content(&iced_code_editor::Message::CharacterInput('x')));
    assert!(editor_message_may_change_content(&iced_code_editor::Message::Backspace));
    assert!(editor_message_may_change_content(&iced_code_editor::Message::Paste("x".to_string())));
    assert!(editor_message_may_change_content(&iced_code_editor::Message::Undo));
    assert!(!editor_message_may_change_content(&iced_code_editor::Message::MouseRelease));
    assert!(!editor_message_may_change_content(&iced_code_editor::Message::MouseHover(
        iced::Point::new(1.0, 1.0)
    )));
}

#[test]
fn normalize_pos_orders_line_and_column_pairs() {
    assert_eq!(normalize_pos((1, 1), (2, 1)), ((1, 1), (2, 1)));
    assert_eq!(normalize_pos((2, 1), (1, 9)), ((1, 9), (2, 1)));
    assert_eq!(normalize_pos((3, 8), (3, 2)), ((3, 2), (3, 8)));
}

#[test]
fn cursor_from_point_respects_gutter_lines_and_wide_chars() {
    let editor = Editor::new("abc\n中文", "text");

    assert_eq!(cursor_from_point(&editor.inner, iced::Point::new(1.0, 1.0)), None);

    let first = cursor_from_point(&editor.inner, iced::Point::new(55.0, 1.0)).expect("first line");
    assert_eq!(first.0, 1);
    assert!(first.1 >= 1);

    let second =
        cursor_from_point(&editor.inner, iced::Point::new(55.0, editor.inner.line_height() + 1.0))
            .expect("second line");
    assert_eq!(second.0, 2);
    assert!(second.1 >= 1);
}

#[test]
fn editor_events_update_context_target_for_click_and_drag() {
    let mut app = test_app();
    let path = "/tmp/main.rs".to_string();
    app.active_preview_path = Some(path.clone());
    app.preview_tabs = vec![tab(&path, "abc\n中文")];

    update_context_target_from_editor_event(
        &mut app,
        &iced_code_editor::Message::MouseClick(iced::Point::new(55.0, 1.0)),
    );
    let Some((target_path, start_line, start_col, end_line, end_col)) =
        app.preview_context_target.clone()
    else {
        panic!("expected context target");
    };
    assert_eq!(target_path, path);
    assert_eq!((start_line, start_col), (end_line, end_col));

    let second_line_y = app.preview_tabs[0].editor.inner.line_height() + 1.0;
    update_context_target_from_editor_event(
        &mut app,
        &iced_code_editor::Message::MouseDrag(iced::Point::new(85.0, second_line_y)),
    );
    let Some((_, drag_start_line, _, drag_end_line, _)) = app.preview_context_target.clone() else {
        panic!("expected drag target");
    };
    assert!(drag_start_line <= drag_end_line);

    app.active_preview_path = Some("/tmp/other.rs".to_string());
    update_context_target_from_editor_event(
        &mut app,
        &iced_code_editor::Message::MouseClick(iced::Point::new(55.0, 1.0)),
    );
    assert_eq!(
        app.preview_context_target.as_ref().map(|item| item.0.as_str()),
        Some(path.as_str())
    );
}

#[test]
fn update_context_menu_and_fullscreen_flags_are_local() {
    let mut app = test_app();
    let path = "/tmp/main.rs".to_string();
    app.active_preview_path = Some(path.clone());
    app.preview_tabs = vec![tab(&path, "fn main() {}")];

    let _ = update(&mut app, PreviewMessage::ContextMenuOpenForActiveEditor(10.0, 20.0));
    assert!(app.show_preview_context_menu);
    assert_eq!(app.preview_context_menu_pos, Some((10.0, 20.0)));
    assert_eq!(app.preview_context_target, Some((path.clone(), 1, 1, 1, 1)));

    app.cursor_position = iced::Point::new(30.0, 40.0);
    let _ = update(&mut app, PreviewMessage::ContextMenuOpenForActiveEditor(10.0, 20.0));
    assert_eq!(app.preview_context_menu_pos, Some((30.0, 40.0)));

    let _ = update(&mut app, PreviewMessage::ContextMenuClose);
    assert!(!app.show_preview_context_menu);
    assert_eq!(app.preview_context_menu_pos, None);
    assert_eq!(app.preview_context_target, None);

    let _ = update(&mut app, PreviewMessage::FullscreenOverlayEntered);
    assert!(app.show_preview_fullscreen_overlay);
    let _ = update(&mut app, PreviewMessage::FullscreenOverlayExited);
    assert!(!app.show_preview_fullscreen_overlay);
}

#[test]
fn update_editor_event_sets_focus_dirty_and_autosave_revision() {
    let mut app = test_app();
    let path = "/tmp/main.rs".to_string();
    app.active_preview_path = Some(path.clone());
    app.preview_tabs = vec![tab(&path, "")];
    app.preview_auto_save_mode = PreviewAutoSaveMode::AfterDelay;

    let _ = update(
        &mut app,
        PreviewMessage::EditorEvent(iced_code_editor::Message::CharacterInput('x')),
    );

    assert_eq!(app.focus_area, FocusArea::Preview);
    assert_eq!(app.preview_tabs[0].editor.content(), "");
    assert!(!app.preview_tabs[0].is_dirty);
    assert_eq!(app.preview_tabs[0].auto_save_revision, 0);

    let _ = update(&mut app, PreviewMessage::EditorEvent(iced_code_editor::Message::MouseRelease));
    assert_eq!(app.preview_tabs[0].auto_save_revision, 0);
}
