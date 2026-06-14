use super::editor::{EditorMessage, update};
use crate::app::App;
use iced::Theme;
use iced_code_editor::i18n::Language;

fn test_app() -> App {
    App::new().0
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("editor_tests"));
}

#[test]
fn toggle_settings_flips_preview_settings_panel() {
    let mut app = test_app();
    let original = app.show_preview_settings;

    let _ = update(&mut app, EditorMessage::ToggleSettings);
    assert_eq!(app.show_preview_settings, !original);

    let _ = update(&mut app, EditorMessage::ToggleSettings);
    assert_eq!(app.show_preview_settings, original);
}

#[test]
fn font_size_changed_updates_auto_line_height() {
    let mut app = test_app();
    app.auto_adjust_line_height = true;

    let _ = update(&mut app, EditorMessage::FontSizeChanged(20.0));

    assert_eq!(app.current_font_size, 20.0);
    assert_eq!(app.current_line_height, 28.0);
}

#[test]
fn line_height_changed_disables_auto_adjustment() {
    let mut app = test_app();
    app.auto_adjust_line_height = true;

    let _ = update(&mut app, EditorMessage::LineHeightChanged(32.0));

    assert_eq!(app.current_line_height, 32.0);
    assert!(!app.auto_adjust_line_height);
}

#[test]
fn toggle_auto_line_height_recomputes_when_enabled() {
    let mut app = test_app();
    app.current_font_size = 15.0;
    app.current_line_height = 19.0;
    app.auto_adjust_line_height = false;

    let _ = update(&mut app, EditorMessage::ToggleAutoLineHeight(true));

    assert!(app.auto_adjust_line_height);
    assert_eq!(app.current_line_height, 21.0);
}

#[test]
fn toggle_auto_line_height_off_keeps_current_height() {
    let mut app = test_app();
    app.current_font_size = 18.0;
    app.current_line_height = 25.0;

    let _ = update(&mut app, EditorMessage::ToggleAutoLineHeight(false));

    assert!(!app.auto_adjust_line_height);
    assert_eq!(app.current_line_height, 25.0);
}

#[test]
fn language_changed_updates_current_language() {
    let mut app = test_app();

    let _ = update(&mut app, EditorMessage::LanguageChanged(Language::English));

    assert_eq!(app.current_language, Language::English);
}

#[test]
fn theme_changed_disables_follow_system_theme() {
    let mut app = test_app();
    app.editor_follow_system_theme = true;

    let _ = update(&mut app, EditorMessage::ThemeChanged(Theme::Dark));

    assert_eq!(app.editor_theme, Theme::Dark);
    assert!(!app.editor_follow_system_theme);
}

#[test]
fn toggle_follow_system_theme_updates_flag() {
    let mut app = test_app();
    app.editor_follow_system_theme = false;

    let _ = update(&mut app, EditorMessage::ToggleFollowSystemTheme(true));

    assert!(app.editor_follow_system_theme);
}

#[test]
fn editor_actions_without_active_preview_are_noops() {
    let mut app = test_app();
    app.active_preview_path = None;

    for message in [
        EditorMessage::OpenSearch,
        EditorMessage::OpenReplace,
        EditorMessage::CloseSearch,
        EditorMessage::Undo,
        EditorMessage::Redo,
        EditorMessage::Copy,
        EditorMessage::Cut,
        EditorMessage::ClipboardContentReceived(Some("text".to_string())),
        EditorMessage::ClipboardContentReceived(None),
        EditorMessage::Delete,
        EditorMessage::FontChanged("Monospace".to_string()),
    ] {
        let _ = update(&mut app, message);
    }

    assert_eq!(app.active_preview_path, None);
}
