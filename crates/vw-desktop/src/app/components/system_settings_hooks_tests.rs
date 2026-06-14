use super::*;
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_hooks_tests_are_wired() {
    assert!(module_path!().contains("system_settings_hooks_tests"));
}

#[test]
fn view_builds_enabled_disabled_and_error_hooks_states() {
    let app = test_app();
    let _ = view(&app);

    let mut enabled = test_app();
    enabled.hooks_settings.enabled = true;
    enabled.hooks_settings.command_logger = true;
    enabled.hooks_settings.save_error = Some("hooks save failed".to_string());
    let _ = view(&enabled);
}
