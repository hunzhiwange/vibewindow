#[test]
fn config_defaults_include_expected_paths_and_models() {
    let config = super::Config::default();
    assert!(
        config.workspace_dir.ends_with(
            std::path::PathBuf::from(crate::paths::HOME_CONFIG_DIR_NAME).join("workspace")
        )
    );
    assert!(config.config_path.ends_with(
        std::path::PathBuf::from(crate::paths::HOME_CONFIG_DIR_NAME).join("vibewindow.json")
    ));
    assert_eq!(config.default_provider.as_deref(), Some("openrouter"));
    assert_eq!(config.default_model.as_deref(), Some("zhipuai-coding-plan/glm-5"));
}

#[test]
fn reasoning_level_normalization_accepts_known_values() {
    assert_eq!(
        super::Config::normalize_reasoning_level_override(Some("X-High"), "test"),
        Some("xhigh".to_string())
    );
    assert_eq!(super::Config::normalize_reasoning_level_override(Some(""), "test"), None);
    assert_eq!(super::Config::normalize_reasoning_level_override(Some("invalid"), "test"), None);
}

#[test]
fn provider_reasoning_level_takes_precedence_over_runtime_alias() {
    let mut config = super::Config::default();
    config.provider.reasoning_level = Some("medium".into());
    config.runtime.reasoning_level = Some("high".into());
    assert_eq!(config.effective_provider_reasoning_level(), Some("medium".to_string()));

    config.provider.reasoning_level = None;
    assert_eq!(config.effective_provider_reasoning_level(), Some("high".to_string()));
}
