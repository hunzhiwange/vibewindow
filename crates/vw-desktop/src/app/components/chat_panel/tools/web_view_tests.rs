use super::web_view::{web_metadata_bool, web_metadata_field, web_metadata_number, web_metadata_text, web_string_field};
use serde_json::json;

#[test]
fn web_metadata_helpers_read_typed_fields() {
    let value = json!({"data":{"url":"https://example.test","status":200,"cached":true}});

    assert_eq!(web_metadata_field(&value, "url"), Some("https://example.test".to_string()));
    assert_eq!(web_metadata_number(&value, "status"), Some(200));
    assert!(web_metadata_bool(&value, "cached"));
    assert_eq!(web_string_field(&value, &["url"]), Some("https://example.test".to_string()));
}

#[test]
fn web_metadata_text_includes_url() {
    let value = json!({"data":{"url":"https://example.test"}});

    assert!(web_metadata_text("web_fetch", &value).contains("https://example.test"));
}
