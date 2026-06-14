use super::config::{apply_named_model_provider_profile, apply_workspace_override};
use super::{Config, ConfigExt, ModelProviderConfig};

#[test]
fn config_ext_validate_accepts_default_config() {
    Config::default().validate().unwrap();
}

#[test]
fn named_model_provider_profile_sets_custom_provider_from_base_url() {
    let mut config = Config::default();
    config.default_provider = Some("local-profile".to_string());
    config.model_providers.insert(
        "local-profile".to_string(),
        ModelProviderConfig {
            base_url: Some("https://models.example/v1".to_string()),
            ..Default::default()
        },
    );

    apply_named_model_provider_profile(&mut config);

    assert_eq!(config.api_url.as_deref(), Some("https://models.example/v1"));
    assert_eq!(config.default_provider.as_deref(), Some("custom:https://models.example/v1"));
}

#[test]
fn named_model_provider_profile_uses_responses_provider_and_profile_name() {
    let mut responses = Config::default();
    responses.default_provider = Some("codex".to_string());
    responses.model_providers.insert(
        "codex".to_string(),
        ModelProviderConfig {
            name: Some("ignored-name".to_string()),
            wire_api: Some("responses".to_string()),
            ..Default::default()
        },
    );

    apply_named_model_provider_profile(&mut responses);
    assert_eq!(responses.default_provider.as_deref(), Some("openai-codex"));

    let mut named = Config::default();
    named.default_provider = Some("alias".to_string());
    named.model_providers.insert(
        "alias".to_string(),
        ModelProviderConfig { name: Some("openrouter".to_string()), ..Default::default() },
    );

    apply_named_model_provider_profile(&mut named);
    assert_eq!(named.default_provider.as_deref(), Some("openrouter"));
}

#[test]
fn workspace_override_ignores_blank_and_resolves_non_blank_paths() {
    let mut config = Config::default();
    let original = config.workspace_dir.clone();

    apply_workspace_override(&mut config, "   ");
    assert_eq!(config.workspace_dir, original);

    apply_workspace_override(&mut config, "/tmp/vw-workspace-override");
    assert_eq!(
        config.workspace_dir,
        std::path::Path::new("/tmp/vw-workspace-override").join("workspace")
    );
}
