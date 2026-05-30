#[test]
fn task_631_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("skills_tests.rs"));
}

#[test]
fn skills_config_defaults_to_vibewindow_directory_provider() {
    let config = super::SkillsConfig::default();
    assert_eq!(config.directory_provider, super::SkillsDirectoryProvider::Vibewindow);

    let parsed: super::SkillsConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(parsed.directory_provider, super::SkillsDirectoryProvider::Vibewindow);
}

#[test]
fn skills_directory_provider_round_trips_as_snake_case() {
    let config = super::SkillsConfig {
        directory_provider: super::SkillsDirectoryProvider::Codex,
        ..Default::default()
    };

    let serialized = serde_json::to_value(config).unwrap();
    assert_eq!(serialized["directory_provider"], "codex");

    let parsed: super::SkillsConfig =
        serde_json::from_value(serde_json::json!({ "directory_provider": "claude" })).unwrap();
    assert_eq!(parsed.directory_provider, super::SkillsDirectoryProvider::Claude);
}
