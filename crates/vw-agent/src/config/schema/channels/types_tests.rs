use super::types::{
    DiscordConfig, GroupReplyMode, IMessageConfig, MattermostConfig, SlackConfig, TelegramConfig,
    WebhookConfig,
};
use crate::app::agent::config::traits::ChannelConfig;

#[test]
fn core_channel_metadata_is_stable() {
    assert_eq!(TelegramConfig::name(), "Telegram");
    assert_eq!(DiscordConfig::name(), "Discord");
    assert_eq!(SlackConfig::name(), "Slack");
    assert_eq!(MattermostConfig::desc(), "connect to your bot");
    assert_eq!(WebhookConfig::desc(), "HTTP endpoint");
}

#[test]
fn group_reply_mode_and_core_metadata_cover_remaining_variants() {
    assert!(GroupReplyMode::MentionOnly.requires_mention());
    assert!(!GroupReplyMode::AllMessages.requires_mention());
    assert_eq!(IMessageConfig::name(), "iMessage");
    assert_eq!(IMessageConfig::desc(), "macOS only");
}
