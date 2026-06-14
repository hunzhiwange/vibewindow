use super::Config;
use super::config_validate::{normalize_wire_api, validate_config};

#[test]
fn normalize_wire_api_accepts_common_spellings() {
    assert_eq!(normalize_wire_api("responses"), Some("responses"));
    assert_eq!(normalize_wire_api("chat-completions"), Some("chat_completions"));
    assert_eq!(normalize_wire_api("unknown"), None);
}

#[test]
fn default_config_validates() {
    validate_config(&Config::default()).unwrap();
}

#[test]
fn validate_config_rejects_blank_gateway_host_and_zero_limits() {
    let mut config = Config::default();
    config.gateway.host = " ".to_string();
    assert!(validate_config(&config).unwrap_err().to_string().contains("gateway.host"));

    let mut config = Config::default();
    config.scheduler.max_concurrent = 0;
    assert!(validate_config(&config).unwrap_err().to_string().contains("scheduler.max_concurrent"));

    let mut config = Config::default();
    config.autonomy.max_actions_per_hour = 0;
    assert!(validate_config(&config).unwrap_err().to_string().contains("max_actions_per_hour"));
}

#[test]
fn validate_config_rejects_invalid_fallback_and_tool_entries() {
    let mut config = Config::default();
    config.reliability.fallback_api_keys.insert("missing-provider".to_string(), "key".to_string());
    assert!(validate_config(&config).unwrap_err().to_string().contains("no matching entry"));

    let mut config = Config::default();
    config.autonomy.non_cli_excluded_tools.push("bad tool".to_string());
    assert!(
        validate_config(&config).unwrap_err().to_string().contains("contains invalid characters")
    );
}

#[test]
fn validate_config_rejects_invalid_model_provider_profiles() {
    let mut config = Config::default();
    config.model_providers.insert("blank".to_string(), Default::default());
    assert!(validate_config(&config).unwrap_err().to_string().contains("must define at least one"));

    let mut config = Config::default();
    config.model_providers.insert(
        "bad-url".to_string(),
        super::ModelProviderConfig {
            base_url: Some("ftp://example".to_string()),
            ..Default::default()
        },
    );
    assert!(validate_config(&config).unwrap_err().to_string().contains("http/https"));
}

#[test]
fn validate_config_rejects_ollama_cloud_without_remote_endpoint_or_key() {
    let mut config = Config::default();
    config.default_provider = Some("ollama".to_string());
    config.default_model = Some("llama3:cloud".to_string());
    config.api_url = Some("http://localhost:11434".to_string());

    assert!(validate_config(&config).unwrap_err().to_string().contains("api_url is local"));

    config.api_url = Some("https://ollama.com".to_string());
    unsafe {
        std::env::remove_var("OLLAMA_API_KEY");
        std::env::remove_var("VIBEWINDOW_API_KEY");
        std::env::remove_var("API_KEY");
    }
    assert!(validate_config(&config).unwrap_err().to_string().contains("no API key"));

    config.api_key = Some("key".to_string());
    validate_config(&config).unwrap();
}
