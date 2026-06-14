use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn settings_apply_agent_config_saved_routes_errors() {
    let mut app = app();

    apply_agent_config_saved(&mut app, "browser", Err("boom".to_string()));
    assert!(app.browser_settings.save_error.is_some());

    apply_agent_config_saved(&mut app, "memory", Ok(()));
    assert!(app.memory_settings.save_error.is_none());

    apply_agent_config_saved(&mut app, "unknown", Err("ignored".to_string()));
}

#[test]
fn settings_update_dispatches_to_child_modules() {
    let mut app = app();

    let _ = update(&mut app, SettingsMessage::HeartbeatHelpOpen);
    assert!(app.heartbeat_settings.show_help_modal);

    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::PanelToggled("telegram".to_string())),
    );
    assert!(app.channels_settings.expanded_panels.contains("telegram"));

    let _ = update(
        &mut app,
        SettingsMessage::AgentConfigSaved { tag: "hooks", result: Err("x".to_string()) },
    );
    assert!(app.hooks_settings.save_error.is_some());
}
