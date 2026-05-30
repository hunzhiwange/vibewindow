use super::*;
use serde_json::json;

#[test]
fn remap_provider_options_moves_matching_key() {
    let remapped = remap_provider_options(
        &json!({"openai": {"temperature": 0.2}, "keep": true}),
        "openai",
        "azure",
    );
    assert_eq!(remapped["azure"]["temperature"], json!(0.2));
    assert!(remapped.get("openai").is_none());
}

#[test]
fn provider_options_normalizes_adapter_aliases() {
    let options =
        provider_options("openai", "openai-compatible", json!({"openai": {"maxTokens": 12}}));
    assert_eq!(options["openai"]["maxTokens"], json!(12));
}
