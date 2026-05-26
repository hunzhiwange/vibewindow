use super::other::{DingTalkConfig, MatrixConfig, NextcloudTalkConfig, NostrConfig, QQConfig};
use crate::app::agent::config::traits::ChannelConfig;

#[test]
fn other_channel_metadata_is_user_facing() {
    assert_eq!(MatrixConfig::name(), "Matrix");
    assert_eq!(DingTalkConfig::desc(), "DingTalk Stream Mode");
    assert_eq!(QQConfig::name(), "QQ Official");
    assert_eq!(NextcloudTalkConfig::name(), "NextCloud Talk");
    assert_eq!(NostrConfig::desc(), "Nostr DMs");
}
