use super::other::{
    DingTalkConfig, IrcConfig, LinqConfig, MatrixConfig, NextcloudTalkConfig, NostrConfig,
    QQConfig, SignalConfig, WatiConfig, WhatsAppConfig,
};
use crate::app::agent::config::traits::ChannelConfig;

#[test]
fn other_channel_metadata_is_user_facing() {
    assert_eq!(MatrixConfig::name(), "Matrix");
    assert_eq!(DingTalkConfig::desc(), "DingTalk Stream Mode");
    assert_eq!(QQConfig::name(), "QQ Official");
    assert_eq!(NextcloudTalkConfig::name(), "NextCloud Talk");
    assert_eq!(NostrConfig::desc(), "Nostr DMs");
}

#[test]
fn remaining_other_channel_metadata_is_stable() {
    assert_eq!(SignalConfig::name(), "Signal");
    assert_eq!(SignalConfig::desc(), "An open-source, encrypted messaging service");
    assert_eq!(WhatsAppConfig::name(), "WhatsApp");
    assert_eq!(LinqConfig::desc(), "iMessage/RCS/SMS via Linq API");
    assert_eq!(WatiConfig::name(), "WATI");
    assert_eq!(IrcConfig::desc(), "IRC over TLS");
}

#[test]
fn whatsapp_backend_helpers_distinguish_cloud_web_and_ambiguous_configs() {
    let cloud = WhatsAppConfig {
        access_token: Some("token".to_string()),
        phone_number_id: Some("phone".to_string()),
        verify_token: Some("verify".to_string()),
        app_secret: None,
        session_path: None,
        pair_phone: None,
        pair_code: None,
        allowed_numbers: Vec::new(),
    };
    let web = WhatsAppConfig { session_path: Some("/tmp/session".to_string()), ..cloud.clone() };

    assert_eq!(cloud.backend_type(), "cloud");
    assert!(cloud.is_cloud_config());
    assert!(!cloud.is_web_config());
    assert!(web.is_web_config());
    assert!(web.is_ambiguous_config());
}
