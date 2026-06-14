use super::*;

#[test]
fn load_app_config_returns_json_object_when_missing_or_unreadable() {
    let cfg = load_app_config();
    assert!(cfg.is_object());
}

#[test]
fn set_config_field_merges_into_existing_object() {
    let key = format!("ui_config_test_{}", std::process::id());

    set_config_field(&key, serde_json::json!({"enabled": true}));
    let cfg = load_app_config();

    assert_eq!(cfg[&key]["enabled"], true);
}
