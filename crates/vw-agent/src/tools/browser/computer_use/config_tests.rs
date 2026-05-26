use super::*;

#[test]
fn default_is_local_and_bounded() {
    let config = ComputerUseConfig::default();
    assert_eq!(config.endpoint, "http://127.0.0.1:8787/v1/actions");
    assert_eq!(config.timeout_ms, 15_000);
    assert!(!config.allow_remote_endpoint);
    assert!(config.window_allowlist.is_empty());
}

#[test]
fn debug_does_not_expose_api_key() {
    let config = ComputerUseConfig { api_key: Some("secret-token".into()), ..Default::default() };
    let rendered = format!("{config:?}");
    assert!(rendered.contains("ComputerUseConfig"));
    assert!(!rendered.contains("secret-token"));
    assert!(!rendered.contains("api_key"));
}
