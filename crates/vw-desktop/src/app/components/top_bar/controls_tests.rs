use super::controls;
use crate::app::{App, Message, Screen};
use iced::Element;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn settings_button_builds_element() {
    keep_element(controls::settings_button());
}

#[test]
fn project_view_tools_returns_empty_space_outside_project_screen() {
    let mut app = test_app();
    app.screen = Screen::Home;

    keep_element(controls::project_view_tools(&app));
}

#[test]
fn project_view_tools_builds_project_controls_when_panels_are_hidden() {
    let mut app = test_app();
    app.screen = Screen::Project;
    app.show_settings = false;
    app.terminal.is_visible = false;
    app.show_diff = false;
    app.show_file_manager = false;

    keep_element(controls::project_view_tools(&app));
}

#[test]
fn project_view_tools_builds_project_controls_when_panels_are_visible() {
    let mut app = test_app();
    app.screen = Screen::Project;
    app.show_settings = true;
    app.terminal.is_visible = true;
    app.show_diff = true;
    app.show_file_manager = true;

    keep_element(controls::project_view_tools(&app));
}
