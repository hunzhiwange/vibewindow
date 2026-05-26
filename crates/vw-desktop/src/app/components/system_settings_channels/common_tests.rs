use super::common::{is_multiline_list_key, localized_label, multiline_placeholder};

#[test]
fn multiline_list_keys_identify_collection_fields() {
    assert!(is_multiline_list_key("telegram.allowed_users"));
    assert!(!is_multiline_list_key("telegram.enabled"));
}

#[test]
fn localized_label_falls_back_to_original_key() {
    assert_eq!(localized_label("unknown.key"), "unknown.key");
}

#[test]
fn multiline_placeholder_falls_back_to_original_text() {
    assert_eq!(multiline_placeholder("unknown.key"), "unknown.key");
}
