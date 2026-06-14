use super::*;

#[tokio::test]
async fn start_channels_returns_provider_initialization_error_before_listener_startup() {
    let mut config = Config::default();
    config.default_provider = Some("__missing_provider_for_start_test__".to_string());
    config.default_model = Some("model".to_string());
    config.channels_config.telegram = None;
    config.channels_config.discord = None;
    config.channels_config.slack = None;
    config.channels_config.matrix = None;

    let err = start_channels(config).await.expect_err("missing provider should fail");

    assert!(!err.to_string().is_empty());
}

#[test]
fn start_module_exports_start_channels_symbol() {
    let _ = start_channels;
}
