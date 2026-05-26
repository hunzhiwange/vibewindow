use super::types::{DiscordConfig, MattermostConfig, SlackConfig, TelegramConfig, WebhookConfig};
use crate::app::agent::config::traits::ChannelConfig;

#[test]
fn core_channel_metadata_is_stable() {
    assert_eq!(TelegramConfig::name(), "Telegram");
    assert_eq!(DiscordConfig::name(), "Discord");
    assert_eq!(SlackConfig::name(), "Slack");
    assert_eq!(MattermostConfig::desc(), "connect to your bot");
    assert_eq!(WebhookConfig::desc(), "HTTP endpoint");
}
