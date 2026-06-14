use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn channel_updates_refresh_toggle_and_mutate_values() {
    let mut app = app();
    app.channels_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::Channels(ChannelsMessage::Refresh));
    assert!(app.channels_settings.save_error.is_none());

    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::PanelToggled("telegram".to_string())),
    );
    assert!(app.channels_settings.expanded_panels.contains("telegram"));

    let _ = update(&mut app, SettingsMessage::Channels(ChannelsMessage::CliToggled(true)));
    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::ProjectDirChanged("/tmp/work".to_string())),
    );
    let _ =
        update(&mut app, SettingsMessage::Channels(ChannelsMessage::MessageTimeoutSecsChanged(0)));
    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::EnabledToggled("telegram".to_string(), true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::TextChanged(
            "telegram.bot_token".to_string(),
            "token".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::BoolToggled(
            "telegram.mention_only".to_string(),
            true,
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::NumberChanged(
            "telegram.draft_update_interval_ms".to_string(),
            20,
        )),
    );

    assert!(app.channels_settings.cli);
    assert_eq!(app.channels_settings.project_dir_input, "/tmp/work");
    assert_eq!(app.channels_settings.message_timeout_secs, 1);
    let telegram = app.channels_settings.telegram.as_ref().expect("telegram enabled");
    assert_eq!(telegram.bot_token, "token");
    assert!(telegram.mention_only);
    assert_eq!(telegram.draft_update_interval_ms, 100);
}
