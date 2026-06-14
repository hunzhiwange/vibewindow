use super::*;
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_http_request_tests_are_wired() {
    assert!(module_path!().contains("system_settings_http_request_tests"));
}

#[test]
fn view_builds_empty_and_populated_http_request_states() {
    let app = test_app();
    let _ = view(&app);

    let mut populated = test_app();
    populated.http_request_settings.enabled = true;
    populated.http_request_settings.allowed_domains =
        vec!["example.com".to_string(), "*.example.org".to_string(), "*".to_string()];
    populated.http_request_settings.new_allowed_domain_input = "api.example.com".to_string();
    populated.http_request_settings.max_response_size = 0;
    populated.http_request_settings.timeout_secs = 0;
    populated.http_request_settings.user_agent = "VibeWindow-Test/1.0".to_string();
    populated.http_request_settings.save_error = Some("http request save failed".to_string());
    let _ = view(&populated);
}
