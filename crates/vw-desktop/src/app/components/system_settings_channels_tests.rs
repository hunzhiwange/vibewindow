use super::*;
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn system_settings_channels_tests_are_wired() {
    assert!(module_path!().contains("system_settings_channels_tests"));
}

#[test]
fn view_builds_empty_enabled_and_error_summaries() {
    let mut app = test_app();
    app.channels_settings.cli = false;
    keep_element(view(&app));

    app.channels_settings.cli = true;
    app.channels_settings.message_timeout_secs = 0;
    app.channels_settings.project_dir_input = "/tmp/project".to_string();
    app.channels_settings.save_error = Some("save failed".to_string());
    keep_element(view(&app));
}
