#[test]
fn default_adapter_matches_openai_compatible_wire_value() {
    assert_eq!(super::default_adapter(), "openai-compatible");
}

#[test]
fn api_info_deserialization_fills_default_adapter() {
    let api: super::ApiInfo =
        serde_json::from_str(r#"{"id":"main","url":"https://example.test"}"#).unwrap();

    assert_eq!(api.adapter, "openai-compatible");
}

#[test]
fn parse_model_preserves_model_slashes() {
    let parsed = super::parse_model("provider/family/model");

    assert_eq!(parsed.provider_id, "provider");
    assert_eq!(parsed.model_id, "family/model");
}

#[test]
fn model_not_found_display_includes_suggestions_when_available() {
    let error = super::ModelNotFoundError {
        provider_id: "openai".to_string(),
        model_id: "missing".to_string(),
        suggestions: vec!["gpt-5".to_string(), "gpt-5-mini".to_string()],
    };

    let message = error.to_string();

    assert!(message.contains("openai/missing"));
    assert!(message.contains("gpt-5, gpt-5-mini"));
}
