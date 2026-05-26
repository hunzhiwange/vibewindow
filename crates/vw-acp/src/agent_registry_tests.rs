use std::collections::HashMap;

use crate::types::AcpAgentConfig;

use super::*;

#[test]
fn command_line_joins_command_and_args_without_mutating_env() {
    let mut env = HashMap::new();
    env.insert("KEY".to_string(), "VALUE".to_string());
    let spec = AgentCommandSpec {
        display_name: "Demo".to_string(),
        command: "demo".to_string(),
        args: vec!["--one".to_string(), "two".to_string()],
        env: env.clone(),
    };

    assert_eq!(spec.command_line(), "demo --one two");
    assert_eq!(AcpAgentConfig::from(&spec).env, env);
}

#[test]
fn normalize_agent_name_trims_and_lowercases() {
    assert_eq!(normalize_agent_name("  Codex CLI  "), "codex cli");
}

#[test]
fn built_in_specs_include_stable_user_keys() {
    let specs = built_in_agent_specs();

    assert!(specs.contains_key(DEFAULT_AGENT_NAME));
    assert!(specs.contains_key("claude"));
    assert!(
        built_in_agent_definitions()
            .iter()
            .all(|definition| definition.name == normalize_agent_name(definition.name))
    );
}

#[test]
fn merge_specs_ignores_empty_overrides_and_normalizes_keys() {
    let mut overrides = HashMap::new();
    overrides.insert(
        " Custom ".to_string(),
        AgentCommandSpec {
            display_name: " Custom Agent ".to_string(),
            command: " custom-bin ".to_string(),
            args: vec!["--flag".to_string()],
            env: HashMap::new(),
        },
    );
    overrides.insert(
        "empty".to_string(),
        AgentCommandSpec {
            display_name: "Empty".to_string(),
            command: "   ".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
        },
    );

    let merged = merge_agent_specs(Some(&overrides));

    assert_eq!(merged["custom"].display_name, "Custom Agent");
    assert_eq!(merged["custom"].command, "custom-bin");
    assert!(!merged.contains_key("empty"));
}

#[test]
fn resolve_agent_command_uses_aliases_and_preserves_unknown_input() {
    assert_eq!(resolve_agent_command("codex cli", None), built_in_agent_registry()["codex"]);
    assert_eq!(resolve_agent_command("./custom-agent", None), "./custom-agent");
}
