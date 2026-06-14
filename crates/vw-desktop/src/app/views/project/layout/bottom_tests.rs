use crate::app::{App, Message};

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("bottom_tests"));
}

#[test]
fn bottom_panel_builds_empty_space_when_terminal_hidden() {
    let mut app = test_app();
    app.terminal.is_visible = false;

    keep_element(super::bottom_panel(&app));
}

#[test]
fn bottom_panel_builds_terminal_when_visible() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.terminal.height = 240.0;

    keep_element(super::bottom_panel(&app));
}

#[test]
fn bottom_panel_clamps_negative_visible_terminal_height() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.terminal.height = -24.0;

    keep_element(super::bottom_panel(&app));
}

#[test]
fn terminal_content_height_subtracts_resize_handle() {
    assert_eq!(super::terminal_content_height(240.0, 8.0), 232.0);
}

#[test]
fn terminal_content_height_never_goes_negative() {
    assert_eq!(super::terminal_content_height(4.0, 8.0), 0.0);
    assert_eq!(super::terminal_content_height(-4.0, 8.0), 0.0);
    assert_eq!(super::terminal_content_height(10.0, -8.0), 10.0);
    assert_eq!(super::terminal_content_height(-10.0, -8.0), 0.0);
}
