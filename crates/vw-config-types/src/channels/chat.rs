use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::clawdtalk::ClawdTalkConfig;
#[cfg(not(target_arch = "wasm32"))]
use super::email::EmailConfig;
use super::lark::{FeishuConfig, LarkConfig};
use super::{
    other::{
        DingTalkConfig, IrcConfig, LinqConfig, MatrixConfig, NextcloudTalkConfig, NostrConfig,
        QQConfig, SignalConfig, WatiConfig, WhatsAppConfig,
    },
    types::{
        DiscordConfig, IMessageConfig, MattermostConfig, SlackConfig, TelegramConfig, WebhookConfig,
    },
};

/// 默认的单条通道消息处理超时时间，单位为秒。
pub fn default_channel_message_timeout_secs() -> u64 {
    300
}

/// 通道配置总表。
///
/// 该结构把所有可选消息通道集中到一个位置，调用方可以按需启用其中任意子通道。
/// 某个字段为 `Some` 通常表示对应通道已配置并可初始化。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChannelsConfig {
    /// 是否启用 CLI 通道。
    pub cli: bool,
    /// Telegram 通道配置。
    pub telegram: Option<TelegramConfig>,
    /// Discord 通道配置。
    pub discord: Option<DiscordConfig>,
    /// Slack 通道配置。
    pub slack: Option<SlackConfig>,
    /// Mattermost 通道配置。
    pub mattermost: Option<MattermostConfig>,
    /// Webhook 通道配置。
    pub webhook: Option<WebhookConfig>,
    /// iMessage 通道配置。
    pub imessage: Option<IMessageConfig>,
    /// Matrix 通道配置。
    pub matrix: Option<MatrixConfig>,
    /// Signal 通道配置。
    pub signal: Option<SignalConfig>,
    /// WhatsApp 通道配置。
    pub whatsapp: Option<WhatsAppConfig>,
    /// Linq 通道配置。
    pub linq: Option<LinqConfig>,
    /// Wati 通道配置。
    pub wati: Option<WatiConfig>,
    /// Nextcloud Talk 通道配置。
    pub nextcloud_talk: Option<NextcloudTalkConfig>,
    /// 邮件通道配置，仅在非 WASM 平台可用。
    #[cfg(not(target_arch = "wasm32"))]
    pub email: Option<EmailConfig>,
    /// IRC 通道配置。
    pub irc: Option<IrcConfig>,
    /// Lark 通道配置。
    pub lark: Option<LarkConfig>,
    /// Feishu 通道配置。
    pub feishu: Option<FeishuConfig>,
    /// DingTalk 通道配置。
    pub dingtalk: Option<DingTalkConfig>,
    /// QQ 通道配置。
    pub qq: Option<QQConfig>,
    /// Nostr 通道配置。
    pub nostr: Option<NostrConfig>,
    /// ClawdTalk 通道配置。
    pub clawdtalk: Option<ClawdTalkConfig>,
    /// 单条消息允许的处理超时，单位为秒。
    #[serde(default = "default_channel_message_timeout_secs")]
    pub message_timeout_secs: u64,
    /// 通道运行时可选的项目目录覆盖值。
    pub project_dir: Option<PathBuf>,
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            cli: true,
            telegram: None,
            discord: None,
            slack: None,
            mattermost: None,
            webhook: None,
            imessage: None,
            matrix: None,
            signal: None,
            whatsapp: None,
            linq: None,
            wati: None,
            nextcloud_talk: None,
            #[cfg(not(target_arch = "wasm32"))]
            email: None,
            irc: None,
            lark: None,
            feishu: None,
            dingtalk: None,
            qq: None,
            nostr: None,
            clawdtalk: None,
            message_timeout_secs: default_channel_message_timeout_secs(),
            project_dir: None,
        }
    }
}
#[cfg(test)]
#[path = "chat_tests.rs"]
mod chat_tests;
