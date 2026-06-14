use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn composio_update_changes_fields_and_ignores_unrelated_messages() {
    let mut app = app();
    app.composio_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::ComposioEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::ComposioApiKeyChanged(" secret ".to_string()));
    let _ = update(&mut app, SettingsMessage::ComposioEntityIdChanged(" workspace ".to_string()));

    assert!(app.composio_settings.enabled);
    assert_eq!(app.composio_settings.api_key_input, " secret ");
    assert_eq!(app.composio_settings.entity_id_input, " workspace ");
    assert!(app.composio_settings.save_error.is_none());

    let _ = update(&mut app, SettingsMessage::HeartbeatHelpOpen);
    assert_eq!(app.composio_settings.entity_id_input, " workspace ");
}
