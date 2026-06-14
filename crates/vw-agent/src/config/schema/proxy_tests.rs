use super::proxy::{
    ProxyConfig, ProxyScope, apply_proxy_to_process_env, build_runtime_proxy_client,
    build_runtime_proxy_client_with_timeouts, clear_proxy_env, parse_proxy_enabled,
    parse_proxy_scope, runtime_proxy_config, set_runtime_proxy_config, validate_proxy_config,
};

#[test]
fn proxy_parser_accepts_stable_aliases() {
    assert_eq!(parse_proxy_enabled("true"), Some(true));
    assert_eq!(parse_proxy_enabled("0"), Some(false));
    assert_eq!(parse_proxy_scope("services"), Some(ProxyScope::Services));
    assert_eq!(parse_proxy_scope("bad"), None);
}

#[test]
fn enabled_proxy_requires_url() {
    let config = ProxyConfig { enabled: true, ..Default::default() };

    assert!(validate_proxy_config(&config).is_err());
}

#[test]
fn proxy_validation_rejects_bad_urls_and_service_scope_without_services() {
    let bad_scheme =
        ProxyConfig { http_proxy: Some("ftp://proxy.example".to_string()), ..Default::default() };
    assert!(validate_proxy_config(&bad_scheme).unwrap_err().to_string().contains("Allowed"));

    let missing_host =
        ProxyConfig { http_proxy: Some("http://".to_string()), ..Default::default() };
    assert!(validate_proxy_config(&missing_host).is_err());

    let empty_services = ProxyConfig {
        enabled: true,
        scope: ProxyScope::Services,
        http_proxy: Some("http://127.0.0.1:8080".to_string()),
        services: Vec::new(),
        ..Default::default()
    };
    assert!(validate_proxy_config(&empty_services).unwrap_err().to_string().contains("services"));
}

#[test]
fn process_env_proxy_application_sets_and_clears_upper_and_lowercase_keys() {
    clear_proxy_env();
    let config = ProxyConfig {
        http_proxy: Some(" http://127.0.0.1:8080 ".to_string()),
        https_proxy: Some("https://127.0.0.1:8443".to_string()),
        all_proxy: None,
        no_proxy: vec![" localhost ".to_string(), "127.0.0.1".to_string()],
        ..Default::default()
    };

    apply_proxy_to_process_env(&config);

    assert_eq!(std::env::var("HTTP_PROXY").unwrap(), "http://127.0.0.1:8080");
    assert_eq!(std::env::var("http_proxy").unwrap(), "http://127.0.0.1:8080");
    assert_eq!(std::env::var("NO_PROXY").unwrap(), "127.0.0.1,localhost");

    clear_proxy_env();
    assert!(std::env::var("HTTP_PROXY").is_err());
    assert!(std::env::var("http_proxy").is_err());
}

#[test]
fn runtime_proxy_config_round_trip_preserves_state() {
    let config = ProxyConfig {
        enabled: true,
        http_proxy: Some("http://127.0.0.1:8080".to_string()),
        ..Default::default()
    };

    set_runtime_proxy_config(config.clone());

    assert_eq!(runtime_proxy_config().enabled, config.enabled);
    clear_proxy_env();
}

#[test]
fn runtime_proxy_clients_are_cached_by_service_and_timeout_key() {
    set_runtime_proxy_config(ProxyConfig::default());

    let first = build_runtime_proxy_client("openai");
    let second = build_runtime_proxy_client("OPENAI");
    assert_eq!(format!("{first:?}"), format!("{second:?}"));

    let timeout_client = build_runtime_proxy_client_with_timeouts("openai", 5, 2);
    let timeout_client_again = build_runtime_proxy_client_with_timeouts("openai", 5, 2);
    assert_eq!(format!("{timeout_client:?}"), format!("{timeout_client_again:?}"));
}
