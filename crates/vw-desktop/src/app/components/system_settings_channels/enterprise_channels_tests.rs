use crate::app::{App, Message};
use vw_config_types::channels::{
    ClawdTalkConfig, DingTalkConfig, FeishuConfig, GroupReplyConfig, GroupReplyMode, LarkConfig,
    LarkReceiveMode, NostrConfig, QQConfig, QQReceiveMode,
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
        allowed_sender_ids: vec!["alice".to_string()],
    }
}

fn configure_enterprise_channels(app: &mut App) {
    app.channels_settings.lark = Some(LarkConfig {
        app_id: "lark-app".to_string(),
        app_secret: "lark-secret".to_string(),
        encrypt_key: Some("encrypt".to_string()),
        verification_token: Some("verify".to_string()),
        allowed_users: vec!["alice".to_string()],
        mention_only: true,
        group_reply: Some(group_reply()),
        use_feishu: true,
        receive_mode: LarkReceiveMode::Webhook,
        port: Some(3001),
        draft_update_interval_ms: 500,
        max_draft_edits: 2,
    });
    app.channels_settings.feishu = Some(FeishuConfig {
        app_id: "feishu-app".to_string(),
        app_secret: "feishu-secret".to_string(),
        encrypt_key: Some("encrypt".to_string()),
        verification_token: Some("verify".to_string()),
        allowed_users: vec!["alice".to_string()],
        group_reply: Some(group_reply()),
        receive_mode: LarkReceiveMode::Websocket,
        port: None,
        draft_update_interval_ms: 1000,
        max_draft_edits: 3,
    });
    app.channels_settings.dingtalk = Some(DingTalkConfig {
        client_id: "client".to_string(),
        client_secret: "secret".to_string(),
        allowed_users: vec!["alice".to_string()],
    });
    app.channels_settings.qq = Some(QQConfig {
        app_id: "qq-app".to_string(),
        app_secret: "qq-secret".to_string(),
        allowed_users: vec!["alice".to_string()],
        receive_mode: QQReceiveMode::Websocket,
    });
    app.channels_settings.nostr = Some(NostrConfig {
        private_key: "nsec".to_string(),
        relays: vec!["wss://relay.example".to_string()],
        allowed_pubkeys: vec!["npub".to_string()],
    });
    app.channels_settings.clawdtalk = Some(ClawdTalkConfig {
        api_key: "api-key".to_string(),
        connection_id: "conn".to_string(),
        from_number: "+1000".to_string(),
        allowed_destinations: vec!["*".to_string()],
        webhook_secret: Some("secret".to_string()),
    });
    app.channels_settings.expanded_panels.extend(
        ["lark", "feishu", "dingtalk", "qq", "nostr", "clawdtalk"].into_iter().map(str::to_string),
    );
    app.channels_settings.refresh_text_inputs();
}

#[test]
fn enterprise_channels_tests_are_wired() {
    assert!(module_path!().contains("enterprise_channels_tests"));
}

#[test]
fn enterprise_panels_build_disabled_and_expanded_enabled_states() {
    let mut app = test_app();
    keep_element(super::enterprise_channels::lark_panel(&app));
    keep_element(super::enterprise_channels::feishu_panel(&app));
    keep_element(super::enterprise_channels::dingtalk_panel(&app));
    keep_element(super::enterprise_channels::qq_panel(&app));
    keep_element(super::enterprise_channels::nostr_panel(&app));
    keep_element(super::enterprise_channels::clawdtalk_panel(&app));

    configure_enterprise_channels(&mut app);
    keep_element(super::enterprise_channels::lark_panel(&app));
    keep_element(super::enterprise_channels::feishu_panel(&app));
    keep_element(super::enterprise_channels::dingtalk_panel(&app));
    keep_element(super::enterprise_channels::qq_panel(&app));
    keep_element(super::enterprise_channels::nostr_panel(&app));
    keep_element(super::enterprise_channels::clawdtalk_panel(&app));
}
