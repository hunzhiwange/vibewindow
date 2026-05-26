use super::proxy::{
    clear_proxy_env, parse_proxy_enabled, parse_proxy_scope, runtime_proxy_config,
    set_runtime_proxy_config, validate_proxy_config, ProxyConfig, ProxyScope,
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
