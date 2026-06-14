use crate::app::terminal::{Shell, TerminalState, TerminalTheme};
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn blank_terminal(is_visible: bool) -> TerminalState {
    TerminalState::blank_with_settings(
        is_visible,
        Shell::Bash,
        TerminalTheme::System,
        "JetBrains Mono".to_string(),
        13.0,
        200.0,
    )
}

#[test]
fn view_builds_empty_container_when_terminal_is_hidden() {
    let mut app = test_app();
    app.terminal.is_visible = false;

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_empty_terminal_when_no_tabs_exist() {
    let mut app = test_app();
    app.terminal = blank_terminal(true);

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_visible_terminal_with_default_tab() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.window_size = (1200.0, 800.0);
    app.show_settings = false;

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_falls_back_to_first_tab_when_active_id_is_missing() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.terminal.active_id = None;
    app.window_size = (0.0, 800.0);

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_settings_layout_with_non_finite_width() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.show_settings = true;
    app.settings_panel_width = f32::NAN;
    app.window_size = (420.0, 700.0);

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_settings_layout_with_clamped_width() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.show_settings = true;
    app.settings_panel_width = 1200.0;
    app.window_size = (980.0, 700.0);

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_settings_layout_with_minimum_clamped_width_and_tiny_window() {
    let mut app = test_app();
    app.terminal.is_visible = true;
    app.show_settings = true;
    app.settings_panel_width = 20.0;
    app.window_size = (24.0, 700.0);

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_long_title_tabs_and_invalid_active_fallback() {
    let mut app = test_app();
    app.terminal.is_visible = true;

    let first_id = app.terminal.tabs.first().expect("first terminal tab").id;
    let tab = app.terminal.tabs.iter_mut().find(|tab| tab.id == first_id).expect("first tab");
    tab.title = "a very long terminal tab title that should clamp to the maximum width".to_string();
    app.terminal.active_id = Some(u64::MAX);

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_multi_tab_context_menu() {
    let mut app = test_app();
    app.terminal.is_visible = true;

    let added = app.terminal.add_terminal(None);
    assert!(added, "expected test terminal tab to be created");

    let first_id = app.terminal.tabs.first().expect("first terminal tab").id;
    app.terminal.active_id = Some(first_id);
    app.terminal.tab_context_menu_id = Some(first_id);
    app.terminal.tab_context_menu_pos = Some((48.0, 32.0));

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_single_tab_context_menu_with_disabled_close() {
    let mut app = test_app();
    app.terminal.is_visible = true;

    let first_id = app.terminal.tabs.first().expect("first terminal tab").id;
    app.terminal.tab_context_menu_id = Some(first_id);
    app.terminal.tab_context_menu_pos = None;

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_builds_rename_modal_for_active_tab() {
    let mut app = test_app();
    app.terminal.is_visible = true;

    let active_id = app.terminal.active_id.expect("active terminal tab");
    let tab =
        app.terminal.tabs.iter_mut().find(|tab| tab.id == active_id).expect("active terminal tab");
    tab.edit_title = Some("新的终端名称".to_string());

    keep_element(super::terminal_panel::view(&app));
}

#[test]
fn view_ignores_rename_modal_for_inactive_tab() {
    let mut app = test_app();
    app.terminal.is_visible = true;

    let added = app.terminal.add_terminal(None);
    assert!(added, "expected test terminal tab to be created");

    let inactive_id = app.terminal.tabs.first().expect("first terminal tab").id;
    let active_id = app.terminal.tabs.last().expect("last terminal tab").id;
    app.terminal.active_id = Some(active_id);
    app.terminal
        .tabs
        .iter_mut()
        .find(|tab| tab.id == inactive_id)
        .expect("inactive terminal tab")
        .edit_title = Some("未激活名称".to_string());

    keep_element(super::terminal_panel::view(&app));
}
