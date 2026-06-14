use super::*;
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn system_settings_browser_tests_are_wired() {
    assert!(module_path!().contains("system_settings_browser_tests"));
}

#[test]
fn view_builds_default_native_unknown_and_error_states() {
    let mut app = test_app();
    keep_element(view(&app));

    app.browser_settings.enabled = true;
    app.browser_settings.allowed_domains_editor =
        iced::widget::text_editor::Content::with_text("example.com\n*.example.org");
    app.browser_settings.browser_open = "new_window".to_string();
    app.browser_settings.session_name_input = "isolated".to_string();
    app.browser_settings.backend = "native".to_string();
    app.browser_settings.native_headless = false;
    app.browser_settings.native_webdriver_url = "http://127.0.0.1:4444".to_string();
    app.browser_settings.native_chrome_path_input = "/Applications/Chrome.app".to_string();
    keep_element(view(&app));

    app.browser_settings.backend = "computer_use".to_string();
    app.browser_settings.browser_open = "new_tab".to_string();
    app.browser_settings.computer_use_endpoint = "http://127.0.0.1:8787/v1/actions".to_string();
    app.browser_settings.computer_use_api_key_input = "secret".to_string();
    app.browser_settings.computer_use_timeout_ms_input = "1".to_string();
    app.browser_settings.computer_use_allow_remote_endpoint = true;
    app.browser_settings.computer_use_window_allowlist_input = "Chrome\nVibeWindow".to_string();
    app.browser_settings.computer_use_max_coordinate_x_input = "1920".to_string();
    app.browser_settings.computer_use_max_coordinate_y_input = "1080".to_string();
    app.browser_settings.save_error = Some("save failed".to_string());
    keep_element(view(&app));

    app.browser_settings.backend = "unknown".to_string();
    app.browser_settings.browser_open = "unknown".to_string();
    keep_element(view(&app));
}
