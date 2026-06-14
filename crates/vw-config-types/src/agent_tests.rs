use std::path::Path;

#[test]
fn workspaces_config_resolves_default_and_relative_roots() {
    let config_dir = Path::new("/tmp/vw");

    let default_root = super::WorkspacesConfig::default().resolve_root(config_dir);
    assert_eq!(default_root, config_dir.join("workspaces"));

    let relative = super::WorkspacesConfig { enabled: true, root: Some("custom".into()) }
        .resolve_root(config_dir);
    assert_eq!(relative, config_dir.join("custom"));
}

#[test]
fn agent_definition_defaults_and_builtin_specs_are_stable() {
    let definition = super::AgentDefinitionConfig::default();
    assert_eq!(definition.mode, "all");
    assert!(definition.enabled);
    assert_eq!(definition.max_depth, 3);
    assert_eq!(definition.max_iterations, 10);

    assert_eq!(super::BUILTIN_AGENT_SPECS.len(), 11);
    assert_eq!(super::BUILTIN_AGENT_SPECS[0].key, "main");
}
