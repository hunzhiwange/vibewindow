use super::*;
use crate::app::App;
use crate::app::message::settings::ChannelsMessage;

#[test]
fn channels_top_level_update_delegates_to_updates_module() {
    let mut app = App::new().0;

    let _ = update(
        &mut app,
        SettingsMessage::Channels(ChannelsMessage::PanelToggled("telegram".to_string())),
    );

    assert!(app.channels_settings.expanded_panels.contains("telegram"));
}
