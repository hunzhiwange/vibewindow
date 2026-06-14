use super::*;
use iced::Element;
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn field_row_wraps_controls() {
    keep_element(field_row("编辑器主题", "说明", text("control")));
}

#[test]
fn view_uses_app_theme_when_following_system_theme() {
    let mut app = test_app();
    app.editor_follow_system_theme = true;
    app.app_theme = iced::Theme::Dark;
    app.editor_theme = iced::Theme::Light;
    app.current_font_size = 15.0;
    app.current_line_height = 24.0;
    app.auto_adjust_line_height = true;

    keep_element(view(&app));
}

#[test]
fn view_uses_editor_theme_when_not_following_system_theme() {
    let mut app = test_app();
    app.editor_follow_system_theme = false;
    app.app_theme = iced::Theme::Light;
    app.editor_theme = iced::Theme::Dark;
    app.current_font_size = 30.0;
    app.current_line_height = 60.0;
    app.auto_adjust_line_height = false;

    keep_element(view(&app));
}
