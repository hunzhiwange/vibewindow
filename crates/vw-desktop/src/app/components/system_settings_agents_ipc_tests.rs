use super::*;
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn system_settings_agents_ipc_tests_are_wired() {
    assert!(module_path!().contains("system_settings_agents_ipc_tests"));
}

#[test]
fn view_builds_default_enabled_error_and_boundary_states() {
    let mut app = test_app();
    keep_element(view(&app));

    app.agents_ipc_settings.enabled = true;
    app.agents_ipc_settings.db_path_input = "/tmp/vw-agents.db".to_string();
    app.agents_ipc_settings.staleness_secs = 1;
    keep_element(view(&app));

    app.agents_ipc_settings.staleness_secs = 86_400;
    app.agents_ipc_settings.save_error = Some("save failed".to_string());
    keep_element(view(&app));
}

#[test]
fn overlays_return_dialog_or_help_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.agents_ipc_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}
