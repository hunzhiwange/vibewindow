use super::*;
use serde_json::json;

#[test]
fn custom_provider_draft_maps_models_headers_and_errors() {
    let raw = json!({
        "name": "Custom",
        "api": "https://api.example.test",
        "models": {
            "z-model": {"name": "Zed", "headers": {"X-Z": "zed", "A": "a"}},
            "a-model": {"name": "Alpha"}
        }
    });
    let draft = custom_provider_draft_from_value("custom", &raw).unwrap();
    assert_eq!(draft.provider_id, "custom");
    assert_eq!(
        draft.models.iter().map(|m| m.model_id.as_str()).collect::<Vec<_>>(),
        vec!["a-model", "z-model"]
    );
    assert_eq!(draft.headers.iter().map(|h| h.key.as_str()).collect::<Vec<_>>(), vec!["A", "X-Z"]);

    let defaulted = custom_provider_draft_from_value("empty", &json!({"name": "Empty"})).unwrap();
    assert_eq!(defaulted.models.len(), 1);
    assert!(custom_provider_draft_from_value("bad", &json!(null)).unwrap_err().contains("bad"));
}
