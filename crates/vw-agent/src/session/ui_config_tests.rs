use super::*;

#[test]
fn load_app_config_returns_json_object_when_missing_or_unreadable() {
    let cfg = load_app_config();
    assert!(cfg.is_object());
}
