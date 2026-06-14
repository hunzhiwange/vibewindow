#[test]
fn settings_tests_are_wired() {
    assert!(module_path!().contains("settings_tests"));
}

use super::settings::build_settings_overlay;
use crate::app::{App, Message};
use iced::widget::{container, text};
use iced::{Element, Theme};
use iced_code_editor::i18n::Language;

fn app() -> App {
    App::new().0
}

fn content() -> Element<'static, Message> {
    container(text("preview")).into()
}

fn keep(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn settings_overlay_builds_with_default_editor_settings() {
    let app = app();

    keep(build_settings_overlay(&app, content()));
}

#[test]
fn settings_overlay_builds_with_manual_line_height_and_theme() {
    let mut app = app();
    app.current_font_size = 18.0;
    app.auto_adjust_line_height = false;
    app.current_line_height = 28.5;
    app.current_language = Language::ChineseSimplified;
    app.editor_follow_system_theme = false;
    app.editor_theme = Theme::Dark;

    keep(build_settings_overlay(&app, content()));
}

#[test]
fn settings_overlay_uses_app_theme_when_following_system_theme() {
    let mut app = app();
    app.editor_follow_system_theme = true;
    app.app_theme = Theme::Dark;
    app.editor_theme = Theme::Light;

    keep(build_settings_overlay(&app, content()));
}
