#[test]
fn provider_api_mode_serializes_as_kebab_case() {
    let serialized = serde_json::to_string(&super::ProviderApiMode::OpenAiResponses).unwrap();
    assert_eq!(serialized, "\"open-ai-responses\"");
}

#[test]
fn provider_api_mode_maps_to_compatible_mode() {
    assert_eq!(
        super::ProviderApiMode::OpenAiChatCompletions.as_compatible_mode(),
        super::CompatibleApiMode::OpenAiChatCompletions
    );
    assert_eq!(
        super::ProviderApiMode::OpenAiResponses.as_compatible_mode(),
        super::CompatibleApiMode::OpenAiResponses
    );
}
