use super::*;

#[test]
fn provider_alias_matches_is_case_insensitive() {
    let spec = DashboardAiIntegrationSpec {
        id: "openai",
        integration_name: "OpenAI",
        provider_id: "openai",
        requires_api_key: true,
        supports_api_url: false,
        model_options: &[],
    };

    assert!(provider_alias_matches(&spec, "OpenAI"));
    assert!(!provider_alias_matches(&spec, "anthropic"));
}

#[test]
fn has_non_empty_rejects_blank_values() {
    assert!(!has_non_empty(None));
    assert!(!has_non_empty(Some("  ")));
    assert!(has_non_empty(Some("token")));
}
