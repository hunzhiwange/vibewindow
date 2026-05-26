use super::Config;

#[test]
fn config_save_test_uses_default_config_shape() {
    let config = Config::default();

    assert!(!config.gateway.host.trim().is_empty());
}
