use super::*;
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_heartbeat_tests_are_wired() {
    assert!(module_path!().contains("system_settings_heartbeat_tests"));
}

#[test]
fn view_builds_disabled_enabled_and_error_heartbeat_states() {
    let app = test_app();
    let _ = view(&app);

    let mut enabled = test_app();
    enabled.heartbeat_settings.enabled = true;
    enabled.heartbeat_settings.interval_minutes = 1440;
    enabled.heartbeat_settings.message_input = "巡检服务状态".to_string();
    enabled.heartbeat_settings.target_input = "telegram".to_string();
    enabled.heartbeat_settings.to_input = "123456".to_string();
    enabled.heartbeat_settings.save_error = Some("heartbeat save failed".to_string());
    let _ = view(&enabled);
}

#[test]
fn overlays_return_base_or_help_modal_for_heartbeat() {
    let app = test_app();
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&app, base);

    let mut with_help = test_app();
    with_help.heartbeat_settings.show_help_modal = true;
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = view_overlays(&with_help, base);
}
