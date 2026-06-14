use super::*;

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("channels_tests"));
}

fn values(prefix: &str) -> Vec<String> {
    vec![format!("{prefix}-1"), format!("{prefix}-2")]
}

fn group(prefix: &str) -> vw_config_types::channels::GroupReplyConfig {
    vw_config_types::channels::GroupReplyConfig { mode: None, allowed_sender_ids: values(prefix) }
}

#[test]
fn default_channels_state_matches_ui_defaults() {
    let state = ChannelsSettingsState::default();

    assert_eq!(state.project_dir(), None);
    assert!(state.cli);
    assert_eq!(state.message_timeout_secs, 300);
    assert!(state.expanded_panels.contains("feishu"));
    assert!(state.save_error.is_none());
    assert!(state.text_inputs.is_empty());
    assert!(state.text_editors.is_empty());
}

#[test]
fn project_dir_trims_blank_input_and_preserves_non_blank_path() {
    let mut state = ChannelsSettingsState {
        project_dir_input: "   ".to_string(),
        ..ChannelsSettingsState::default()
    };

    assert_eq!(state.project_dir(), None);

    state.project_dir_input = "  /tmp/vibe-window  ".to_string();
    assert_eq!(state.project_dir(), Some(PathBuf::from("/tmp/vibe-window")));
}

#[test]
fn refresh_text_inputs_rebuilds_all_channel_list_fields() {
    let mut state = ChannelsSettingsState::default();
    state.text_inputs.insert("telegram.allowed_users".to_string(), "stale".to_string());
    state.text_inputs.insert("unrelated".to_string(), "keep".to_string());
    state.text_editors.insert("telegram.allowed_users".to_string(), text_editor::Content::new());
    state.text_editors.insert("unrelated".to_string(), text_editor::Content::new());

    state.telegram = Some(vw_config_types::channels::TelegramConfig {
        bot_token: "telegram-token".to_string(),
        allowed_users: values("telegram-user"),
        stream_mode: vw_config_types::channels::StreamMode::Partial,
        draft_update_interval_ms: 250,
        interrupt_on_new_message: true,
        mention_only: true,
        group_reply: Some(group("telegram-sender")),
        base_url: Some("https://telegram.example".to_string()),
    });
    state.discord = Some(vw_config_types::channels::DiscordConfig {
        bot_token: "discord-token".to_string(),
        guild_id: Some("guild".to_string()),
        allowed_users: values("discord-user"),
        listen_to_bots: true,
        mention_only: true,
        group_reply: Some(group("discord-sender")),
    });
    state.slack = Some(vw_config_types::channels::SlackConfig {
        bot_token: "slack-token".to_string(),
        app_token: Some("app".to_string()),
        channel_id: Some("channel".to_string()),
        allowed_users: values("slack-user"),
        group_reply: Some(group("slack-sender")),
    });
    state.mattermost = Some(vw_config_types::channels::MattermostConfig {
        url: "https://mattermost.example".to_string(),
        bot_token: "mattermost-token".to_string(),
        channel_id: Some("channel".to_string()),
        allowed_users: values("mattermost-user"),
        thread_replies: Some(true),
        mention_only: Some(true),
        group_reply: Some(group("mattermost-sender")),
    });
    state.imessage = Some(vw_config_types::channels::IMessageConfig {
        allowed_contacts: values("imessage-contact"),
    });
    state.matrix = Some(vw_config_types::channels::MatrixConfig {
        homeserver: "https://matrix.example".to_string(),
        access_token: "matrix-token".to_string(),
        user_id: Some("@user:example".to_string()),
        device_id: Some("device".to_string()),
        room_id: "!room:example".to_string(),
        allowed_users: values("matrix-user"),
        mention_only: true,
    });
    state.signal = Some(vw_config_types::channels::SignalConfig {
        http_url: "http://signal.example".to_string(),
        account: "+1000".to_string(),
        group_id: Some("group".to_string()),
        allowed_from: values("signal-from"),
        ignore_attachments: true,
        ignore_stories: true,
    });
    state.whatsapp = Some(vw_config_types::channels::WhatsAppConfig {
        access_token: Some("wa-token".to_string()),
        phone_number_id: Some("phone-id".to_string()),
        verify_token: Some("verify".to_string()),
        app_secret: Some("secret".to_string()),
        session_path: None,
        pair_phone: None,
        pair_code: None,
        allowed_numbers: values("whatsapp-number"),
    });
    state.linq = Some(vw_config_types::channels::LinqConfig {
        api_token: "linq-token".to_string(),
        from_phone: "+1001".to_string(),
        signing_secret: Some("secret".to_string()),
        allowed_senders: values("linq-sender"),
    });
    state.wati = Some(vw_config_types::channels::WatiConfig {
        api_token: "wati-token".to_string(),
        api_url: "https://wati.example".to_string(),
        tenant_id: Some("tenant".to_string()),
        allowed_numbers: values("wati-number"),
    });
    state.nextcloud_talk = Some(vw_config_types::channels::NextcloudTalkConfig {
        base_url: "https://nextcloud.example".to_string(),
        app_token: "nextcloud-token".to_string(),
        webhook_secret: Some("secret".to_string()),
        allowed_users: values("nextcloud-user"),
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        state.email = Some(vw_config_types::channels::EmailConfig {
            allowed_senders: values("email-sender"),
            ..vw_config_types::channels::EmailConfig::default()
        });
    }
    state.irc = Some(vw_config_types::channels::IrcConfig {
        server: "irc.example".to_string(),
        port: 6697,
        nickname: "bot".to_string(),
        username: Some("user".to_string()),
        channels: values("irc-channel"),
        allowed_users: values("irc-user"),
        server_password: Some("server-pass".to_string()),
        nickserv_password: Some("nick-pass".to_string()),
        sasl_password: Some("sasl-pass".to_string()),
        verify_tls: Some(true),
    });
    state.lark = Some(vw_config_types::channels::LarkConfig {
        app_id: "lark-app".to_string(),
        app_secret: "lark-secret".to_string(),
        encrypt_key: Some("encrypt".to_string()),
        verification_token: Some("verify".to_string()),
        allowed_users: values("lark-user"),
        mention_only: true,
        group_reply: Some(group("lark-sender")),
        use_feishu: false,
        receive_mode: vw_config_types::channels::LarkReceiveMode::Webhook,
        port: Some(80),
        draft_update_interval_ms: 3000,
        max_draft_edits: 20,
    });
    state.feishu = Some(vw_config_types::channels::FeishuConfig {
        app_id: "feishu-app".to_string(),
        app_secret: "feishu-secret".to_string(),
        encrypt_key: Some("encrypt".to_string()),
        verification_token: Some("verify".to_string()),
        allowed_users: values("feishu-user"),
        group_reply: Some(group("feishu-sender")),
        receive_mode: vw_config_types::channels::LarkReceiveMode::Webhook,
        port: Some(81),
        draft_update_interval_ms: 3000,
        max_draft_edits: 20,
    });
    state.dingtalk = Some(vw_config_types::channels::DingTalkConfig {
        client_id: "dingtalk-client".to_string(),
        client_secret: "dingtalk-secret".to_string(),
        allowed_users: values("dingtalk-user"),
    });
    state.qq = Some(vw_config_types::channels::QQConfig {
        app_id: "qq-app".to_string(),
        app_secret: "qq-secret".to_string(),
        allowed_users: values("qq-user"),
        receive_mode: vw_config_types::channels::QQReceiveMode::Websocket,
    });
    state.nostr = Some(vw_config_types::channels::NostrConfig {
        private_key: "nostr-key".to_string(),
        relays: values("nostr-relay"),
        allowed_pubkeys: values("nostr-pubkey"),
    });
    state.clawdtalk = Some(vw_config_types::channels::ClawdTalkConfig {
        api_key: "clawd-key".to_string(),
        connection_id: "conn".to_string(),
        from_number: "+1002".to_string(),
        allowed_destinations: values("clawd-destination"),
        webhook_secret: Some("secret".to_string()),
    });

    state.refresh_text_inputs();

    assert_eq!(state.text_inputs["unrelated"], "keep");
    assert!(!state.text_editors.contains_key("unrelated"));
    assert_eq!(state.text_inputs["telegram.allowed_users"], "telegram-user-1, telegram-user-2");
    assert_eq!(
        state.text_editors["telegram.allowed_users"].text(),
        "telegram-user-1\ntelegram-user-2"
    );
    assert_eq!(
        state.text_inputs["telegram.group_reply.allowed_sender_ids"],
        "telegram-sender-1, telegram-sender-2"
    );
    assert_eq!(state.text_inputs["discord.allowed_users"], "discord-user-1, discord-user-2");
    assert_eq!(
        state.text_inputs["slack.group_reply.allowed_sender_ids"],
        "slack-sender-1, slack-sender-2"
    );
    assert_eq!(
        state.text_inputs["mattermost.group_reply.allowed_sender_ids"],
        "mattermost-sender-1, mattermost-sender-2"
    );
    assert_eq!(
        state.text_inputs["imessage.allowed_contacts"],
        "imessage-contact-1, imessage-contact-2"
    );
    assert_eq!(state.text_inputs["matrix.allowed_users"], "matrix-user-1, matrix-user-2");
    assert_eq!(state.text_inputs["signal.allowed_from"], "signal-from-1, signal-from-2");
    assert_eq!(
        state.text_inputs["whatsapp.allowed_numbers"],
        "whatsapp-number-1, whatsapp-number-2"
    );
    assert_eq!(state.text_inputs["linq.allowed_senders"], "linq-sender-1, linq-sender-2");
    assert_eq!(state.text_inputs["wati.allowed_numbers"], "wati-number-1, wati-number-2");
    assert_eq!(
        state.text_inputs["nextcloud_talk.allowed_users"],
        "nextcloud-user-1, nextcloud-user-2"
    );
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(state.text_inputs["email.allowed_senders"], "email-sender-1, email-sender-2");
    assert_eq!(state.text_inputs["irc.channels"], "irc-channel-1, irc-channel-2");
    assert_eq!(state.text_inputs["irc.allowed_users"], "irc-user-1, irc-user-2");
    assert_eq!(state.text_inputs["lark.allowed_users"], "lark-user-1, lark-user-2");
    assert_eq!(
        state.text_inputs["lark.group_reply.allowed_sender_ids"],
        "lark-sender-1, lark-sender-2"
    );
    assert_eq!(state.text_inputs["feishu.allowed_users"], "feishu-user-1, feishu-user-2");
    assert_eq!(
        state.text_inputs["feishu.group_reply.allowed_sender_ids"],
        "feishu-sender-1, feishu-sender-2"
    );
    assert_eq!(state.text_inputs["dingtalk.allowed_users"], "dingtalk-user-1, dingtalk-user-2");
    assert_eq!(state.text_inputs["qq.allowed_users"], "qq-user-1, qq-user-2");
    assert_eq!(state.text_inputs["nostr.relays"], "nostr-relay-1, nostr-relay-2");
    assert_eq!(state.text_inputs["nostr.allowed_pubkeys"], "nostr-pubkey-1, nostr-pubkey-2");
    assert_eq!(
        state.text_inputs["clawdtalk.allowed_destinations"],
        "clawd-destination-1, clawd-destination-2"
    );
}

#[test]
fn refresh_text_inputs_inserts_empty_group_reply_when_config_omits_group_reply() {
    let mut state = ChannelsSettingsState {
        telegram: Some(vw_config_types::channels::TelegramConfig {
            bot_token: "token".to_string(),
            allowed_users: Vec::new(),
            stream_mode: vw_config_types::channels::StreamMode::Off,
            draft_update_interval_ms: 1000,
            interrupt_on_new_message: false,
            mention_only: false,
            group_reply: None,
            base_url: None,
        }),
        ..ChannelsSettingsState::default()
    };

    state.refresh_text_inputs();

    assert_eq!(state.text_inputs["telegram.allowed_users"], "");
    assert_eq!(state.text_inputs["telegram.group_reply.allowed_sender_ids"], "");
    assert_eq!(state.text_editors["telegram.group_reply.allowed_sender_ids"].text(), "");
}
