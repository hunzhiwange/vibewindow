#[test]
fn proxy_normalization_and_scope_matching_work() {
    let config = super::ProxyConfig {
        enabled: true,
        http_proxy: Some(" http://proxy ".into()),
        services: vec!["provider.*".into(), "tool.browser".into(), "provider.*".into()],
        no_proxy: vec![" localhost, 127.0.0.1 ".into()],
        scope: super::ProxyScope::Services,
        ..Default::default()
    };

    assert!(config.has_any_proxy_url());
    assert_eq!(config.normalized_services(), vec!["provider.*", "tool.browser"]);
    assert_eq!(config.normalized_no_proxy(), vec!["127.0.0.1", "localhost"]);
    assert!(config.should_apply_to_service("provider.openai"));
    assert!(config.should_apply_to_service("tool.browser"));
    assert!(!config.should_apply_to_service("channel.slack"));
}

#[test]
fn proxy_selector_helpers_cover_keys_and_wildcards() {
    assert!(super::is_supported_proxy_service_selector("provider.openai"));
    assert!(super::is_supported_proxy_service_selector("tool.*"));
    assert!(super::service_selector_matches("provider.*", "provider.openrouter"));
    assert!(!super::service_selector_matches("provider.*", "provider"));
    assert_eq!(super::normalize_proxy_url_option(Some("   ")), None);
}
