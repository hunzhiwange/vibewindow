use super::config_validate::{normalize_wire_api, validate_config};
use super::Config;

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
