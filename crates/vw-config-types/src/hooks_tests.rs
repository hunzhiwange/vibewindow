#[test]
fn hooks_defaults_enable_runtime_hooks() {
    let config = super::HooksConfig::default();
    assert!(config.enabled);
    assert!(!config.builtin.command_logger);

    let parsed: super::HooksConfig = serde_json::from_str("{\"enabled\":false}").unwrap();
    assert!(!parsed.enabled);
}
