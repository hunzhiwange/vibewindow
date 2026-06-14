use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn heartbeat_update_clamps_interval_and_toggles_help() {
    let mut app = app();

    let _ = update(&mut app, SettingsMessage::HeartbeatEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::HeartbeatIntervalChanged(0));
    let _ = update(&mut app, SettingsMessage::HeartbeatMessageChanged(" ping ".to_string()));
    let _ = update(&mut app, SettingsMessage::HeartbeatTargetChanged(" room ".to_string()));
    let _ = update(&mut app, SettingsMessage::HeartbeatToChanged(" user ".to_string()));

    assert!(app.heartbeat_settings.enabled);
    assert_eq!(app.heartbeat_settings.interval_minutes, 1);
    assert_eq!(app.heartbeat_settings.message_input, " ping ");
    assert_eq!(app.heartbeat_settings.target_input, " room ");
    assert_eq!(app.heartbeat_settings.to_input, " user ");

    let _ = update(&mut app, SettingsMessage::HeartbeatHelpOpen);
    assert!(app.heartbeat_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::HeartbeatHelpClose);
    assert!(!app.heartbeat_settings.show_help_modal);
}
