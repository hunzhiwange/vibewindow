use super::config_secrets::{
    decrypt_config_secrets, decrypt_map_secrets, decrypt_vec_secrets, encrypt_config_secrets,
    encrypt_map_secrets, encrypt_vec_secrets,
};
use super::{Config, TelegramConfig};
use crate::app::agent::security::SecretStore;
use std::collections::HashMap;

#[test]
fn vec_and_map_secrets_noop_when_encryption_disabled() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SecretStore::new(tmp.path(), false);
    let mut vec_values = vec!["alpha".to_string()];
    let mut map_values = HashMap::from([("one".to_string(), "bravo".to_string())]);

    encrypt_vec_secrets(&store, &mut vec_values, "vec").unwrap();
    encrypt_map_secrets(&store, &mut map_values, "map").unwrap();
    decrypt_vec_secrets(&store, &mut vec_values[..], "vec").unwrap();
    decrypt_map_secrets(&store, &mut map_values, "map").unwrap();

    assert_eq!(vec_values, vec!["alpha".to_string()]);
    assert_eq!(map_values.get("one").map(String::as_str), Some("bravo"));
}

#[test]
fn config_secrets_encrypt_and_decrypt_representative_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.config_path = tmp.path().join("vibewindow.json");
    config.api_key = Some("api-key".to_string());
    config.composio.api_key = Some("composio-key".to_string());
    config.proxy.http_proxy = Some("http://user:pass@proxy.example:8080".to_string());
    config.web_search.brave_api_key = Some("brave-key".to_string());
    config.reliability.api_keys = vec!["fallback-key".to_string()];
    config.reliability.fallback_api_keys.insert("openai".to_string(), "openai-key".to_string());
    config.gateway.paired_tokens = vec!["paired-token".to_string()];
    config.channels_config.telegram = Some(TelegramConfig {
        bot_token: "telegram-token".to_string(),
        allowed_users: Vec::new(),
        stream_mode: Default::default(),
        draft_update_interval_ms: 1000,
        interrupt_on_new_message: false,
        mention_only: false,
        group_reply: None,
        base_url: None,
    });

    encrypt_config_secrets(&mut config).unwrap();

    assert!(crate::app::agent::security::SecretStore::is_encrypted(
        config.api_key.as_deref().unwrap()
    ));
    assert!(crate::app::agent::security::SecretStore::is_encrypted(
        &config.channels_config.telegram.as_ref().unwrap().bot_token
    ));

    decrypt_config_secrets(&mut config, tmp.path()).unwrap();

    assert_eq!(config.api_key.as_deref(), Some("api-key"));
    assert_eq!(config.channels_config.telegram.as_ref().unwrap().bot_token, "telegram-token");
}
