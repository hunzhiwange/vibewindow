use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn hooks_update_toggles_fields_and_clears_error() {
    let mut app = app();
    app.hooks_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::Hooks(HooksMessage::EnabledToggled(true)));
    let _ = update(&mut app, SettingsMessage::Hooks(HooksMessage::CommandLoggerToggled(true)));

    assert!(app.hooks_settings.enabled);
    assert!(app.hooks_settings.command_logger);
    assert!(app.hooks_settings.save_error.is_none());

    let _ = update(&mut app, SettingsMessage::HeartbeatHelpOpen);
    assert!(app.hooks_settings.command_logger);
}
