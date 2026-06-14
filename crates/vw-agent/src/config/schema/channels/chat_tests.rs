use super::chat::ChannelsConfigExt;
use super::other::{QQConfig, QQReceiveMode};
use super::{ChannelsConfig, TelegramConfig, WebhookConfig};

#[test]
fn channels_list_adds_webhook_after_regular_channels() {
    let config = ChannelsConfig::default();
    let regular_len = config.channels_except_webhook().len();
    let all = config.channels();

    assert_eq!(all.len(), regular_len + 1);
    assert_eq!(all.last().unwrap().0.name(), "Webhook");
    assert!(!all.last().unwrap().1);
}

#[test]
fn channels_list_reports_enabled_state_for_configured_channels() {
    let mut config = ChannelsConfig {
        telegram: Some(TelegramConfig {
            bot_token: "token".to_string(),
            allowed_users: Vec::new(),
            stream_mode: Default::default(),
            draft_update_interval_ms: 1000,
            interrupt_on_new_message: false,
            mention_only: false,
            group_reply: None,
            base_url: None,
        }),
        webhook: Some(WebhookConfig { port: 8787, secret: None }),
        ..Default::default()
    };

    let channels = config.channels();
    assert!(channels.iter().any(|(handle, enabled)| handle.name() == "Telegram" && *enabled));
    assert!(channels.iter().any(|(handle, enabled)| handle.name() == "Webhook" && *enabled));

    config.qq = Some(QQConfig {
        app_id: "app".to_string(),
        app_secret: "secret".to_string(),
        allowed_users: Vec::new(),
        receive_mode: QQReceiveMode::Webhook,
    });
    assert!(
        config
            .channels_except_webhook()
            .iter()
            .any(|(handle, enabled)| handle.name() == "QQ Official" && !*enabled)
    );

    config.qq.as_mut().unwrap().receive_mode = QQReceiveMode::Websocket;
    assert!(
        config
            .channels_except_webhook()
            .iter()
            .any(|(handle, enabled)| handle.name() == "QQ Official" && *enabled)
    );
}
