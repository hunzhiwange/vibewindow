//! 处理渠道设置子模块的状态变更、字段转换和持久化。

use crate::app::App;
use vw_config_types::channels::ClawdTalkConfig;
#[cfg(not(target_arch = "wasm32"))]
use vw_config_types::channels::EmailConfig;
use vw_config_types::channels::{
    DingTalkConfig, DiscordConfig, IMessageConfig, IrcConfig, LarkConfig, LarkReceiveMode,
    LinqConfig, MatrixConfig, MattermostConfig, NextcloudTalkConfig, NostrConfig, QQConfig,
    QQReceiveMode, SignalConfig, SlackConfig, TelegramConfig, WatiConfig, WebhookConfig,
    WhatsAppConfig,
};

use super::helpers::default_feishu_config;

/// 处理 `toggle_enabled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn toggle_enabled(app: &mut App, channel: &str, enabled: bool) {
    match channel {
        "telegram" => {
            app.channels_settings.telegram = enabled.then(|| TelegramConfig {
                bot_token: String::new(),
                allowed_users: Vec::new(),
                stream_mode: Default::default(),
                draft_update_interval_ms: 1000,
                interrupt_on_new_message: false,
                mention_only: false,
                group_reply: None,
                base_url: None,
            })
        }
        "discord" => {
            app.channels_settings.discord = enabled.then(|| DiscordConfig {
                bot_token: String::new(),
                guild_id: None,
                allowed_users: Vec::new(),
                listen_to_bots: false,
                mention_only: false,
                group_reply: None,
            })
        }
        "slack" => {
            app.channels_settings.slack = enabled.then(|| SlackConfig {
                bot_token: String::new(),
                app_token: None,
                channel_id: None,
                allowed_users: Vec::new(),
                group_reply: None,
            })
        }
        "mattermost" => {
            app.channels_settings.mattermost = enabled.then(|| MattermostConfig {
                url: String::new(),
                bot_token: String::new(),
                channel_id: None,
                allowed_users: Vec::new(),
                thread_replies: Some(true),
                mention_only: Some(false),
                group_reply: None,
            })
        }
        "webhook" => {
            app.channels_settings.webhook =
                enabled.then_some(WebhookConfig { port: 8787, secret: None })
        }
        "imessage" => {
            app.channels_settings.imessage =
                enabled.then(|| IMessageConfig { allowed_contacts: Vec::new() })
        }
        "matrix" => {
            app.channels_settings.matrix = enabled.then(|| MatrixConfig {
                homeserver: String::new(),
                access_token: String::new(),
                user_id: None,
                device_id: None,
                room_id: String::new(),
                allowed_users: Vec::new(),
                mention_only: false,
            })
        }
        "signal" => {
            app.channels_settings.signal = enabled.then(|| SignalConfig {
                http_url: String::new(),
                account: String::new(),
                group_id: None,
                allowed_from: Vec::new(),
                ignore_attachments: false,
                ignore_stories: false,
            })
        }
        "whatsapp" => {
            app.channels_settings.whatsapp = enabled.then(|| WhatsAppConfig {
                access_token: None,
                phone_number_id: None,
                verify_token: None,
                app_secret: None,
                session_path: None,
                pair_phone: None,
                pair_code: None,
                allowed_numbers: Vec::new(),
            })
        }
        "linq" => {
            app.channels_settings.linq = enabled.then(|| LinqConfig {
                api_token: String::new(),
                from_phone: String::new(),
                signing_secret: None,
                allowed_senders: Vec::new(),
            })
        }
        "wati" => {
            app.channels_settings.wati = enabled.then(|| WatiConfig {
                api_token: String::new(),
                api_url: "https://live-mt-server.wati.io".to_string(),
                tenant_id: None,
                allowed_numbers: Vec::new(),
            })
        }
        "nextcloud_talk" => {
            app.channels_settings.nextcloud_talk = enabled.then(|| NextcloudTalkConfig {
                base_url: String::new(),
                app_token: String::new(),
                webhook_secret: None,
                allowed_users: Vec::new(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        "email" => app.channels_settings.email = enabled.then(EmailConfig::default),
        "irc" => {
            app.channels_settings.irc = enabled.then(|| IrcConfig {
                server: String::new(),
                port: 6697,
                nickname: String::new(),
                username: None,
                channels: Vec::new(),
                allowed_users: Vec::new(),
                server_password: None,
                nickserv_password: None,
                sasl_password: None,
                verify_tls: Some(true),
            })
        }
        "lark" => {
            app.channels_settings.lark = enabled.then(|| LarkConfig {
                app_id: String::new(),
                app_secret: String::new(),
                encrypt_key: None,
                verification_token: None,
                allowed_users: Vec::new(),
                mention_only: false,
                group_reply: None,
                use_feishu: false,
                receive_mode: LarkReceiveMode::Websocket,
                port: None,
                draft_update_interval_ms: 3000,
                max_draft_edits: 20,
            })
        }
        "feishu" => app.channels_settings.feishu = enabled.then(default_feishu_config),
        "dingtalk" => {
            app.channels_settings.dingtalk = enabled.then(|| DingTalkConfig {
                client_id: String::new(),
                client_secret: String::new(),
                allowed_users: Vec::new(),
            })
        }
        "qq" => {
            app.channels_settings.qq = enabled.then(|| QQConfig {
                app_id: String::new(),
                app_secret: String::new(),
                allowed_users: Vec::new(),
                receive_mode: QQReceiveMode::Webhook,
            })
        }
        "nostr" => {
            app.channels_settings.nostr = enabled.then(|| NostrConfig {
                private_key: String::new(),
                relays: vec![
                    "wss://relay.damus.io".to_string(),
                    "wss://nos.lol".to_string(),
                    "wss://relay.primal.net".to_string(),
                    "wss://relay.snort.social".to_string(),
                ],
                allowed_pubkeys: Vec::new(),
            })
        }
        "clawdtalk" => {
            app.channels_settings.clawdtalk = enabled.then(|| ClawdTalkConfig {
                api_key: String::new(),
                connection_id: String::new(),
                from_number: String::new(),
                allowed_destinations: Vec::new(),
                webhook_secret: None,
            })
        }
        _ => {}
    }

    if enabled {
        app.channels_settings
            .expanded_panels
            .insert(channel.to_string());
    }
}

#[cfg(test)]
#[path = "toggles_tests.rs"]
mod toggles_tests;
