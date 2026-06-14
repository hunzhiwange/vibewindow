use crate::app::{App, Message};
use vw_config_types::channels::{
    DiscordConfig, GroupReplyConfig, GroupReplyMode, IMessageConfig, MatrixConfig,
    MattermostConfig, SignalConfig, SlackConfig, TelegramConfig, WebhookConfig,
};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn group_reply() -> GroupReplyConfig {
    GroupReplyConfig {
        mode: Some(GroupReplyMode::MentionOnly),
        allowed_sender_ids: vec!["alice".to_string(), "bob".to_string()],
    }
}

fn configure_basic_channels(app: &mut App) {
    app.channels_settings.telegram = Some(TelegramConfig {
        bot_token: "telegram-token".to_string(),
        allowed_users: vec!["telegram-user".to_string()],
        stream_mode: Default::default(),
        draft_update_interval_ms: 500,
        interrupt_on_new_message: true,
        mention_only: true,
        group_reply: Some(group_reply()),
        base_url: Some("https://telegram.example".to_string()),
    });
    app.channels_settings.discord = Some(DiscordConfig {
        bot_token: "discord-token".to_string(),
        guild_id: Some("guild".to_string()),
        allowed_users: vec!["discord-user".to_string()],
        listen_to_bots: true,
        mention_only: true,
        group_reply: Some(group_reply()),
    });
    app.channels_settings.slack = Some(SlackConfig {
        bot_token: "slack-token".to_string(),
        app_token: Some("xapp".to_string()),
        channel_id: Some("channel".to_string()),
        allowed_users: vec!["slack-user".to_string()],
        group_reply: Some(group_reply()),
    });
    app.channels_settings.mattermost = Some(MattermostConfig {
        url: "https://mattermost.example".to_string(),
        bot_token: "mattermost-token".to_string(),
        channel_id: Some("channel".to_string()),
        allowed_users: vec!["mattermost-user".to_string()],
        thread_replies: Some(false),
        mention_only: Some(true),
        group_reply: Some(group_reply()),
    });
    app.channels_settings.webhook =
        Some(WebhookConfig { port: 8787, secret: Some("secret".to_string()) });
    app.channels_settings.imessage =
        Some(IMessageConfig { allowed_contacts: vec!["user@example.com".to_string()] });
    app.channels_settings.matrix = Some(MatrixConfig {
        homeserver: "https://matrix.example".to_string(),
        access_token: "matrix-token".to_string(),
        user_id: Some("@bot:matrix.example".to_string()),
        device_id: Some("DEVICE".to_string()),
        room_id: "!room:matrix.example".to_string(),
        allowed_users: vec!["@alice:matrix.example".to_string()],
        mention_only: true,
    });
    app.channels_settings.signal = Some(SignalConfig {
        http_url: "http://127.0.0.1:8686".to_string(),
        account: "+10000000000".to_string(),
        group_id: Some("group".to_string()),
        allowed_from: vec!["+10000000001".to_string()],
        ignore_attachments: true,
        ignore_stories: true,
    });
    app.channels_settings.expanded_panels.extend(
        ["telegram", "discord", "slack", "mattermost", "webhook", "imessage", "matrix", "signal"]
            .into_iter()
            .map(str::to_string),
    );
    app.channels_settings.refresh_text_inputs();
}

#[test]
fn basic_channels_tests_are_wired() {
    assert!(module_path!().contains("basic_channels_tests"));
}

#[test]
fn basic_panels_build_disabled_and_expanded_enabled_states() {
    let mut app = test_app();
    keep_element(super::basic_channels::telegram_panel(&app));
    keep_element(super::basic_channels::discord_panel(&app));
    keep_element(super::basic_channels::slack_panel(&app));
    keep_element(super::basic_channels::mattermost_panel(&app));
    keep_element(super::basic_channels::webhook_panel(&app));
    keep_element(super::basic_channels::imessage_panel(&app));
    keep_element(super::basic_channels::matrix_panel(&app));
    keep_element(super::basic_channels::signal_panel(&app));

    configure_basic_channels(&mut app);
    keep_element(super::basic_channels::telegram_panel(&app));
    keep_element(super::basic_channels::discord_panel(&app));
    keep_element(super::basic_channels::slack_panel(&app));
    keep_element(super::basic_channels::mattermost_panel(&app));
    keep_element(super::basic_channels::webhook_panel(&app));
    keep_element(super::basic_channels::imessage_panel(&app));
    keep_element(super::basic_channels::matrix_panel(&app));
    keep_element(super::basic_channels::signal_panel(&app));
}
