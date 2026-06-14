use super::*;
use crate::app::{App, PreviewAutoSaveMode, Shell};

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_general_tests_are_wired() {
    assert!(module_path!().contains("system_settings_general_tests"));
}

#[test]
fn view_builds_default_and_custom_general_settings() {
    let app = test_app();
    let _ = view(&app);

    let mut custom = test_app();
    custom.app_theme = iced::Theme::Dark;
    custom.terminal.shell = Shell::Zsh;
    custom.terminal.font_family = "Monaco".to_string();
    custom.terminal.font_size = 18.0;
    custom.preview_auto_save_mode = PreviewAutoSaveMode::Off;
    let _ = view(&custom);
}
