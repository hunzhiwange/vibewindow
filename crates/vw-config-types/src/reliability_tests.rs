#[test]
fn reliability_defaults_match_documented_values() {
    let config = super::ReliabilityConfig::default();
    assert_eq!(config.provider_retries, 3);
    assert_eq!(config.provider_backoff_ms, 500);
    assert_eq!(config.channel_initial_backoff_secs, 1);
    assert_eq!(config.channel_max_backoff_secs, 60);
    assert_eq!(config.scheduler_poll_secs, 60);
    assert_eq!(config.scheduler_retries, 3);
}

#[test]
fn reliability_deserializes_fallback_maps() {
    let parsed: super::ReliabilityConfig = serde_json::from_value(serde_json::json!({
        "fallback_providers": ["a", "b"],
        "fallback_api_keys": {"a": "key-a"},
        "api_keys": ["k1"]
    }))
    .unwrap();

    assert_eq!(parsed.fallback_providers, vec!["a", "b"]);
    assert_eq!(parsed.fallback_api_keys["a"], "key-a");
    assert_eq!(parsed.api_keys, vec!["k1"]);
}
