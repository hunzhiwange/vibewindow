use super::{ChannelsConfig, decrypt_channel_secrets, encrypt_channel_secrets};
use crate::app::agent::security::SecretStore;

#[test]
fn channel_secret_round_trip_is_noop_when_encryption_disabled() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SecretStore::new(tmp.path(), false);
    let mut channels = ChannelsConfig::default();

    encrypt_channel_secrets(&store, &mut channels).unwrap();
    decrypt_channel_secrets(&store, &mut channels).unwrap();

    assert!(channels.telegram.is_none());
}

#[test]
fn channel_secret_round_trip_encrypts_representative_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SecretStore::new(tmp.path(), true);
    let mut channels: ChannelsConfig = serde_json::from_value(serde_json::json!({
        "cli": true,
        "telegram": {"bot_token": "telegram-token", "allowed_users": []},
        "slack": {"bot_token": "slack-token", "app_token": "slack-app"},
        "webhook": {"port": 8080, "secret": "webhook-secret"},
        "matrix": {
            "homeserver": "https://matrix.example",
            "access_token": "matrix-token",
            "room_id": "!room:example",
            "allowed_users": []
        },
        "whatsapp": {
            "access_token": "wa-token",
            "app_secret": "wa-secret",
            "verify_token": "wa-verify"
        },
        "irc": {
            "server": "irc.example",
            "nickname": "bot",
            "username": "bot",
            "server_password": "server-pass",
            "nickserv_password": "nick-pass",
            "sasl_password": "sasl-pass"
        },
        "lark": {
            "app_id": "app",
            "app_secret": "lark-secret",
            "encrypt_key": "lark-encrypt",
            "verification_token": "lark-verify"
        },
        "dingtalk": {"client_id": "client", "client_secret": "ding-secret"},
        "qq": {"app_id": "qq", "app_secret": "qq-secret"},
        "nostr": {"private_key": "nostr-secret"},
        "clawdtalk": {
            "api_key": "clawd-key",
            "connection_id": "conn",
            "from_number": "+10000000000",
            "webhook_secret": "clawd-webhook"
        }
    }))
    .unwrap();

    encrypt_channel_secrets(&store, &mut channels).unwrap();

    assert!(SecretStore::is_encrypted(&channels.telegram.as_ref().unwrap().bot_token));
    assert!(SecretStore::is_encrypted(
        channels.slack.as_ref().unwrap().app_token.as_ref().unwrap()
    ));
    assert!(SecretStore::is_encrypted(channels.webhook.as_ref().unwrap().secret.as_ref().unwrap()));
    assert!(SecretStore::is_encrypted(&channels.matrix.as_ref().unwrap().access_token));
    assert!(SecretStore::is_encrypted(
        channels.irc.as_ref().unwrap().sasl_password.as_ref().unwrap()
    ));
    assert!(SecretStore::is_encrypted(&channels.lark.as_ref().unwrap().app_secret));
    assert!(SecretStore::is_encrypted(&channels.dingtalk.as_ref().unwrap().client_secret));
    assert!(SecretStore::is_encrypted(&channels.qq.as_ref().unwrap().app_secret));
    assert!(SecretStore::is_encrypted(&channels.nostr.as_ref().unwrap().private_key));

    decrypt_channel_secrets(&store, &mut channels).unwrap();

    assert_eq!(channels.telegram.unwrap().bot_token, "telegram-token");
    assert_eq!(channels.slack.unwrap().app_token.as_deref(), Some("slack-app"));
    assert_eq!(channels.irc.unwrap().sasl_password.as_deref(), Some("sasl-pass"));
    assert_eq!(channels.clawdtalk.unwrap().webhook_secret.as_deref(), Some("clawd-webhook"));
}
