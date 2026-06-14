use super::*;
use crate::app::App;
use vw_config_types::channels::{LarkReceiveMode, QQReceiveMode};

fn app() -> App {
    App::new().0
}

#[test]
fn channel_toggle_enabled_creates_defaults_and_expands_panel() {
    let mut app = app();

    toggle_enabled(&mut app, "telegram", true);
    toggle_enabled(&mut app, "feishu", true);
    toggle_enabled(&mut app, "qq", true);

    assert!(app.channels_settings.telegram.is_some());
    assert_eq!(
        app.channels_settings.feishu.as_ref().expect("feishu").receive_mode,
        LarkReceiveMode::Websocket
    );
    assert_eq!(app.channels_settings.qq.as_ref().expect("qq").receive_mode, QQReceiveMode::Webhook);
    assert!(app.channels_settings.expanded_panels.contains("telegram"));

    toggle_enabled(&mut app, "telegram", false);
    assert!(app.channels_settings.telegram.is_none());
}
