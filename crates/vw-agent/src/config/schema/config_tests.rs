use super::{Config, ConfigExt};

#[test]
fn config_ext_validate_accepts_default_config() {
    Config::default().validate().unwrap();
}
