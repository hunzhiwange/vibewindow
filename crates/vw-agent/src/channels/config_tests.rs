use crate::app::agent::config::Config;

use super::{
    channel_message_timeout_budget_secs, effective_channel_message_timeout_secs,
    resolved_default_model, resolved_default_provider, runtime_autonomy_policy_from_config,
    runtime_config_store, runtime_defaults_from_config,
};

#[test]
fn timeout_helpers_apply_floor_scale_and_saturating_math() {
    assert_eq!(effective_channel_message_timeout_secs(1), 30);
    assert_eq!(effective_channel_message_timeout_secs(45), 45);
    assert_eq!(channel_message_timeout_budget_secs(10, 0), 10);
    assert_eq!(channel_message_timeout_budget_secs(10, 99), 40);
}

#[test]
fn runtime_defaults_preserve_configured_values() {
    let config = Config {
        default_provider: Some("provider-a".to_string()),
        default_model: Some("model-a".to_string()),
        default_temperature: 0.25,
        api_key: Some("key".to_string()),
        api_url: Some("https://api.example.test".to_string()),
        ..Config::default()
    };

    assert_eq!(resolved_default_provider(&config), "provider-a");
    assert_eq!(resolved_default_model(&config), "model-a");

    let defaults = runtime_defaults_from_config(&config);
    assert_eq!(defaults.default_provider, "provider-a");
    assert_eq!(defaults.model, "model-a");
    assert_eq!(defaults.temperature, 0.25);
    assert_eq!(defaults.api_key.as_deref(), Some("key"));

    let policy = runtime_autonomy_policy_from_config(&config);
    assert_eq!(policy.auto_approve, config.autonomy.auto_approve);
    let _guard = runtime_config_store().lock().unwrap_or_else(|e| e.into_inner());
}
