use super::*;
use crate::app::agent::config::schema::{
    DiscordConfig, GroupReplyConfig, GroupReplyMode, IMessageConfig, IrcConfig, LinqConfig,
    QQConfig, QQReceiveMode, StreamMode, TelegramConfig,
};

#[test]
fn collect_configured_channels_returns_empty_for_default_config() {
    let config = Config::default();

    assert!(collect_configured_channels(&config, "test").is_empty());
}

#[test]
fn collect_configured_channels_builds_common_native_channels() {
    let mut config = Config::default();
    config.channels_config.telegram = Some(TelegramConfig {
        bot_token: "tg-token".to_string(),
        allowed_users: vec!["alice".to_string()],
        stream_mode: StreamMode::Partial,
        draft_update_interval_ms: 500,
        interrupt_on_new_message: true,
        mention_only: true,
        group_reply: Some(GroupReplyConfig {
            mode: Some(GroupReplyMode::MentionOnly),
            allowed_sender_ids: vec!["admin".to_string()],
        }),
        base_url: Some("https://api.telegram.test".to_string()),
    });
    config.channels_config.discord = Some(DiscordConfig {
        bot_token: "discord-token".to_string(),
        guild_id: Some("guild".to_string()),
        allowed_users: vec!["bob".to_string()],
        listen_to_bots: true,
        mention_only: true,
        group_reply: Some(GroupReplyConfig {
            mode: Some(GroupReplyMode::MentionOnly),
            allowed_sender_ids: vec!["mod".to_string()],
        }),
    });
    config.channels_config.imessage =
        Some(IMessageConfig { allowed_contacts: vec!["+15551234567".to_string()] });
    config.channels_config.linq = Some(LinqConfig {
        api_token: "linq-token".to_string(),
        from_phone: "+15550000000".to_string(),
        signing_secret: None,
        allowed_senders: vec!["+15551111111".to_string()],
    });
    config.channels_config.irc = Some(IrcConfig {
        server: "irc.example.test".to_string(),
        port: 6697,
        nickname: "vibe".to_string(),
        username: None,
        channels: vec!["#general".to_string()],
        allowed_users: vec!["alice".to_string()],
        server_password: None,
        nickserv_password: None,
        sasl_password: None,
        verify_tls: Some(true),
    });

    let channels = collect_configured_channels(&config, "test");
    let names = channels.iter().map(|channel| channel.display_name).collect::<Vec<_>>();

    assert!(names.contains(&"Telegram"));
    assert!(names.contains(&"Discord"));
    assert!(names.contains(&"iMessage"));
    assert!(names.contains(&"Linq"));
    assert!(names.contains(&"IRC"));
}

#[test]
fn collect_configured_channels_skips_qq_webhook_mode_and_keeps_websocket_mode() {
    let mut config = Config::default();
    config.channels_config.qq = Some(QQConfig {
        app_id: "app".to_string(),
        app_secret: "secret".to_string(),
        allowed_users: vec!["user".to_string()],
        receive_mode: QQReceiveMode::Webhook,
    });
    assert!(collect_configured_channels(&config, "test").is_empty());

    config.channels_config.qq = Some(QQConfig {
        app_id: "app".to_string(),
        app_secret: "secret".to_string(),
        allowed_users: vec!["user".to_string()],
        receive_mode: QQReceiveMode::Websocket,
    });
    let channels = collect_configured_channels(&config, "test");

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].display_name, "QQ");
}

#[tokio::test]
async fn append_nostr_channel_if_available_is_noop_when_not_configured() {
    let config = Config::default();
    let mut channels = Vec::new();

    assert_eq!(append_nostr_channel_if_available(&config, &mut channels, "test").await, None);
    assert!(channels.is_empty());
}
