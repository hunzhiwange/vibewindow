use super::web_view::{
    web_metadata_bool, web_metadata_field, web_metadata_number, web_metadata_text, web_string_field,
};
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

#[test]
fn web_metadata_prefers_render_hint_and_search_specific_fields() {
    let value = json!({
        "renderHint": {
            "metadata": {
                "provider": "bing",
                "result_count": 12,
                "truncated": true
            }
        },
        "input": "{\"urls\":[\"https://example.test/a\"],\"query\":\"latest docs\"}"
    });

    let search_text = web_metadata_text("web_search", &value);
    assert!(search_text.contains("bing"));
    assert!(search_text.contains("12 条结果"));

    let fetch_text = web_metadata_text("web_fetch", &value);
    assert!(fetch_text.contains("https://example.test/a"));
    assert!(fetch_text.contains("bing"));
    assert!(fetch_text.contains("已截断"));
}
