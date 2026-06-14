use super::*;
use crate::app::App;
use crate::app::message::settings::channels::toggles;
use vw_config_types::channels::{LarkReceiveMode, QQReceiveMode};

fn app() -> App {
    App::new().0
}

#[test]
fn channel_changes_apply_text_bool_number_and_modes() {
    let mut app = app();
    toggles::toggle_enabled(&mut app, "telegram", true);
    toggles::toggle_enabled(&mut app, "lark", true);
    toggles::toggle_enabled(&mut app, "qq", true);
    toggles::toggle_enabled(&mut app, "webhook", true);

    apply_text_change(&mut app, "telegram.bot_token", "token".to_string());
    apply_text_change(&mut app, "telegram.group_reply.allowed_sender_ids", "u1,u2".to_string());
    apply_bool_change(&mut app, "telegram.mention_only", true);
    apply_number_change(&mut app, "telegram.draft_update_interval_ms", 10);
    apply_number_change(&mut app, "webhook.port", 0);
    apply_receive_mode_change(&mut app, "lark.receive_mode", "webhook".to_string());
    apply_receive_mode_change(&mut app, "qq.receive_mode", "websocket".to_string());

    let telegram = app.channels_settings.telegram.as_ref().expect("telegram enabled");
    assert_eq!(telegram.bot_token, "token");
    assert_eq!(
        telegram.group_reply.as_ref().expect("group reply").allowed_sender_ids,
        vec!["u1".to_string(), "u2".to_string()]
    );
    assert!(telegram.mention_only);
    assert_eq!(telegram.draft_update_interval_ms, 100);
    assert_eq!(app.channels_settings.webhook.as_ref().expect("webhook").port, 1);
    assert_eq!(
        app.channels_settings.lark.as_ref().expect("lark").receive_mode,
        LarkReceiveMode::Webhook
    );
    assert_eq!(
        app.channels_settings.qq.as_ref().expect("qq").receive_mode,
        QQReceiveMode::Websocket
    );
}
