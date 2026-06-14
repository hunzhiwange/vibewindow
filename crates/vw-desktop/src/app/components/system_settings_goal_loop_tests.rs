use super::*;
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_goal_loop_tests_are_wired() {
    assert!(module_path!().contains("system_settings_goal_loop_tests"));
}

#[test]
fn view_builds_disabled_enabled_and_error_goal_loop_states() {
    let app = test_app();
    let _ = view(&app);

    let mut enabled = test_app();
    enabled.goal_loop_settings.enabled = true;
    enabled.goal_loop_settings.interval_minutes_input = "60".to_string();
    enabled.goal_loop_settings.step_timeout_secs_input = "300".to_string();
    enabled.goal_loop_settings.max_steps_per_cycle_input = "10".to_string();
    enabled.goal_loop_settings.channel_input = "telegram".to_string();
    enabled.goal_loop_settings.target_input = "chat-id".to_string();
    enabled.goal_loop_settings.save_error = Some("goal loop save failed".to_string());
    let _ = view(&enabled);
}
